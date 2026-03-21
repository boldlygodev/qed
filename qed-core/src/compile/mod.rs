//! Compilation pass — AST → executable IR.
//!
//! Transforms a [`Program`] (AST) into a [`Script`] (compiled IR) in two
//! passes:
//!
//! 1. **Symbol collection** — walk `PatternDef` and `AliasDef` statements
//!    to populate symbol tables for named patterns and aliases.
//! 2. **Compilation** — walk `SelectAction` statements, compiling each
//!    selector into a [`CompiledSelector`] (or [`CompoundSelector`]) and
//!    each processor chain into a `Box<dyn Processor>`, resolving named
//!    references through the symbol tables.
//!
//! Errors are accumulated into a `Vec<CompileError>` so the compiler can
//! report all problems in a single pass.

use std::collections::HashMap;

use crate::error::CompileError;
use crate::parse::ast::{
    self, NthTerm, Param, ParamValue, PatternRefValue, PatternValue, Program,
    SelectorOp as AstSelectorOp,
};
use crate::processor::chain::ChainProcessor;
use crate::processor::delete::DeleteProcessor;
use crate::processor::external::ExternalCommandProcessor;
use crate::processor::lower::LowerProcessor;
use crate::processor::prefix::PrefixProcessor;
use crate::processor::upper::UpperProcessor;
use crate::processor::Processor;
use crate::span::Spanned;
use crate::SelectorId;
use crate::StatementId;

// ── Script (top-level IR) ───────────────────────────────────────────

/// Compiled script — the output of the compilation pass and input
/// to the execution engine.
#[derive(Debug)]
pub(crate) struct Script {
    pub(crate) statements: Vec<Statement>,
    pub(crate) selectors: Vec<RegistryEntry>,
}

/// A compiled statement: selector + processor + optional fallback.
/// Cannot derive Clone because it holds `Box<dyn Processor>`.
#[derive(Debug)]
pub(crate) struct Statement {
    pub(crate) id: StatementId,
    pub(crate) selector: SelectorId,
    pub(crate) processor: Box<dyn Processor>,
    pub(crate) fallback: Option<Box<dyn Processor>>,
    /// Source span of the selector expression (for diagnostics).
    pub(crate) selector_span: crate::span::Span,
    /// Original source text of the selector expression (for diagnostics).
    pub(crate) selector_text: String,
}

// ── Selector registry ───────────────────────────────────────────────

/// A registry entry is either a simple (single-step) or compound
/// (multi-step) compiled selector.
#[derive(Debug, Clone)]
pub(crate) enum RegistryEntry {
    Simple(CompiledSelector),
    Compound(CompoundSelector),
}

/// A single compiled selector with its operation and error behavior.
#[derive(Debug, Clone)]
pub(crate) struct CompiledSelector {
    pub(crate) id: SelectorId,
    pub(crate) op: SelectorOp,
    pub(crate) on_error: OnError,
}

/// A compound selector composed of multiple selector steps.
#[derive(Debug, Clone)]
pub(crate) struct CompoundSelector {
    pub(crate) id: SelectorId,
    pub(crate) steps: Vec<SelectorId>,
}

// ── Selector operations ─────────────────────────────────────────────

/// The concrete operation a compiled selector performs.
///
/// Each variant holds its compiled pattern. `nth` on `At` uses
/// `Option<Vec<NthTerm>>` — `None` means no filtering (all matches).
#[derive(Debug, Clone)]
pub(crate) enum SelectorOp {
    /// Select each line matching the pattern, optionally filtered by `nth`.
    At {
        pattern: CompiledPattern,
        nth: Option<Vec<NthTerm>>,
    },
    /// Select the zero-width insertion point after each matching line.
    After {
        pattern: CompiledPattern,
    },
    /// Select the zero-width insertion point before each matching line.
    Before {
        pattern: CompiledPattern,
    },
    /// Select from the matching line to end of input (inclusivity
    /// controlled by `pattern.inclusive`).
    From {
        pattern: CompiledPattern,
    },
    /// Select from start of input to the matching line (inclusivity
    /// controlled by `pattern.inclusive`).
    To {
        pattern: CompiledPattern,
    },
}

// ── Pattern matching ────────────────────────────────────────────────

/// A compiled pattern with its negation and inclusivity flags resolved.
#[derive(Debug, Clone)]
pub(crate) struct CompiledPattern {
    pub(crate) matcher: PatternMatcher,
    pub(crate) negated: bool,
    pub(crate) inclusive: bool,
}

/// The underlying matching strategy for a compiled pattern.
#[derive(Debug, Clone)]
pub(crate) enum PatternMatcher {
    Literal(String),
    Regex(regex::Regex),
}

// ── Error behavior ──────────────────────────────────────────────────

/// How the engine handles a selector that matches nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OnError {
    /// Abort execution and report an error (default).
    Fail,
    /// Emit a diagnostic warning but continue execution.
    Warn,
    /// Silently skip the statement as if it were not present.
    Skip,
}

// ── Compilation ────────────────────────────────────────────────────

/// Compile a parsed AST `Program` into an executable `Script`.
pub(crate) fn compile(program: &Program, source: &str) -> Result<Script, Vec<CompileError>> {
    let mut errors = Vec::new();

    // ── Pass 1: collect definitions ──────────────────────────────────
    let mut pattern_defs: HashMap<&str, &PatternValue> = HashMap::new();
    let mut alias_defs: HashMap<&str, &ast::ProcessorChain> = HashMap::new();

    for spanned_stmt in &program.statements {
        match &spanned_stmt.node {
            ast::Statement::PatternDef { name, value } => {
                pattern_defs.insert(&name.node, &value.node);
            }
            ast::Statement::AliasDef { name, chain } => {
                alias_defs.insert(&name.node, &chain.node);
            }
            ast::Statement::SelectAction(_) => {}
        }
    }

    // ── Pass 2: compile select-actions ───────────────────────────────
    let mut statements = Vec::new();
    let mut selectors: Vec<RegistryEntry> = Vec::new();
    let mut stmt_index = 0;

    for spanned_stmt in &program.statements {
        let ast::Statement::SelectAction(node) = &spanned_stmt.node else {
            continue;
        };

        let stmt_id = StatementId::new(stmt_index);
        stmt_index += 1;

        // Compile the selector
        let sel_id = match compile_selector(node, &mut selectors, &pattern_defs) {
            Ok(id) => id,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };

        // Compile the processor
        let processor: Box<dyn Processor> = match &node.chain {
            Some(chain) => match compile_processor_chain(&chain.node, &alias_defs) {
                Ok(p) => p,
                Err(e) => {
                    errors.push(e);
                    continue;
                }
            },
            None => {
                errors.push(CompileError::InvalidParam {
                    processor: "(none)".into(),
                    param: "missing processor chain".into(),
                    span: spanned_stmt.span,
                });
                continue;
            }
        };

        // Compile optional fallback
        let fallback = match &node.fallback {
            Some(fb) => match &fb.node {
                ast::Fallback::Chain(chain) => match compile_processor_chain(chain, &alias_defs) {
                    Ok(p) => Some(p),
                    Err(e) => {
                        errors.push(e);
                        None
                    }
                },
                ast::Fallback::SelectAction(_) => None, // deferred
            },
            None => None,
        };

        let sel_span = node.selector.span;
        let sel_text = source[sel_span.start..sel_span.end].to_owned();

        statements.push(Statement {
            id: stmt_id,
            selector: sel_id,
            processor,
            fallback,
            selector_span: sel_span,
            selector_text: sel_text,
        });
    }

    if errors.is_empty() {
        Ok(Script {
            statements,
            selectors,
        })
    } else {
        Err(errors)
    }
}

/// Compile a `SelectActionNode`'s selector into registry entries.
///
/// For single-step selectors, pushes one `Simple` entry.
/// For compound selectors (multi-step with `>`), pushes one `Simple`
/// entry per step plus a `Compound` entry referencing them.
/// Returns the `SelectorId` of the top-level entry.
fn compile_selector(
    node: &ast::SelectActionNode,
    registry: &mut Vec<RegistryEntry>,
    pattern_defs: &HashMap<&str, &PatternValue>,
) -> Result<SelectorId, CompileError> {
    let selector_ast = &node.selector.node;

    if selector_ast.steps.len() == 1 {
        // Single-step selector
        let step = &selector_ast.steps[0].node;
        let sel_id = SelectorId::new(registry.len());
        let entry = compile_simple_selector(step, sel_id, node.selector.span, pattern_defs)?;
        registry.push(entry);
        Ok(sel_id)
    } else {
        // Compound selector: compile each step, then create a compound entry
        let mut step_ids = Vec::new();
        for step_spanned in &selector_ast.steps {
            let step_id = SelectorId::new(registry.len());
            let entry = compile_simple_selector(
                &step_spanned.node,
                step_id,
                step_spanned.span,
                pattern_defs,
            )?;
            registry.push(entry);
            step_ids.push(step_id);
        }

        let compound_id = SelectorId::new(registry.len());
        registry.push(RegistryEntry::Compound(CompoundSelector {
            id: compound_id,
            steps: step_ids,
        }));
        Ok(compound_id)
    }
}

/// Compile a single `SimpleSelector` AST node into a `RegistryEntry::Simple`.
fn compile_simple_selector(
    step: &ast::SimpleSelector,
    sel_id: SelectorId,
    span: crate::span::Span,
    pattern_defs: &HashMap<&str, &PatternValue>,
) -> Result<RegistryEntry, CompileError> {
    let compiled_pattern = match &step.pattern {
        Some(pat_ref) => compile_pattern(&pat_ref.node, pat_ref.span, pattern_defs)?,
        None => CompiledPattern {
            matcher: PatternMatcher::Literal(String::new()),
            negated: false,
            inclusive: false,
        },
    };

    // Extract params
    let mut nth: Option<Vec<NthTerm>> = None;
    let mut on_error = OnError::Fail;

    for param in &step.params {
        match param.node.name.node.as_str() {
            "nth" => {
                if let ParamValue::NthExpr(expr) = &param.node.value.node {
                    nth = Some(expr.terms.iter().map(|t| t.node).collect());
                } else {
                    return Err(CompileError::InvalidParam {
                        processor: "selector".into(),
                        param: "nth requires an nth expression".into(),
                        span: param.span,
                    });
                }
            }
            "on_error" => {
                if let ParamValue::Identifier(ident) = &param.node.value.node {
                    on_error = match ident.as_str() {
                        "fail" => OnError::Fail,
                        "warn" => OnError::Warn,
                        "skip" => OnError::Skip,
                        other => {
                            return Err(CompileError::InvalidParam {
                                processor: "selector".into(),
                                param: format!("unknown on_error value: {other}"),
                                span: param.span,
                            });
                        }
                    };
                } else {
                    return Err(CompileError::InvalidParam {
                        processor: "selector".into(),
                        param: "on_error requires an identifier (fail, warn, skip)".into(),
                        span: param.span,
                    });
                }
            }
            other => {
                return Err(CompileError::InvalidParam {
                    processor: "selector".into(),
                    param: format!("unknown parameter: {other}"),
                    span: param.span,
                });
            }
        }
    }

    let op = match step.op {
        AstSelectorOp::At => SelectorOp::At {
            pattern: compiled_pattern,
            nth,
        },
        AstSelectorOp::After => {
            if nth.is_some() {
                return Err(CompileError::InvalidParam {
                    processor: "selector".into(),
                    param: "nth is not supported on after()".into(),
                    span,
                });
            }
            SelectorOp::After {
                pattern: compiled_pattern,
            }
        }
        AstSelectorOp::Before => {
            if nth.is_some() {
                return Err(CompileError::InvalidParam {
                    processor: "selector".into(),
                    param: "nth is not supported on before()".into(),
                    span,
                });
            }
            SelectorOp::Before {
                pattern: compiled_pattern,
            }
        }
        AstSelectorOp::From => SelectorOp::From {
            pattern: compiled_pattern,
        },
        AstSelectorOp::To => SelectorOp::To {
            pattern: compiled_pattern,
        },
    };

    Ok(RegistryEntry::Simple(CompiledSelector {
        id: sel_id,
        op,
        on_error,
    }))
}

/// Compile a `PatternRef` into a `CompiledPattern` with its matcher,
/// negation flag, and inclusivity flag resolved.
fn compile_pattern(
    pat_ref: &ast::PatternRef,
    span: crate::span::Span,
    pattern_defs: &HashMap<&str, &PatternValue>,
) -> Result<CompiledPattern, CompileError> {
    let matcher = match &pat_ref.value {
        PatternRefValue::Inline(PatternValue::String(s)) => PatternMatcher::Literal(s.clone()),
        PatternRefValue::Inline(PatternValue::Regex(r)) => {
            compile_regex_matcher(r, span)?
        }
        PatternRefValue::Named(name) => match pattern_defs.get(name.as_str()) {
            Some(PatternValue::String(s)) => PatternMatcher::Literal(s.clone()),
            Some(PatternValue::Regex(r)) => {
                compile_regex_matcher(r, span)?
            }
            None => {
                return Err(CompileError::UndefinedName {
                    name: name.clone(),
                    span,
                });
            }
        },
    };

    Ok(CompiledPattern {
        matcher,
        negated: pat_ref.negated,
        inclusive: pat_ref.inclusive,
    })
}

/// Compile a regex string into a `PatternMatcher::Regex`.
fn compile_regex_matcher(
    pattern: &str,
    span: crate::span::Span,
) -> Result<PatternMatcher, CompileError> {
    let re = regex::Regex::new(pattern).map_err(|e| CompileError::InvalidRegex {
        pattern: pattern.to_owned(),
        reason: e.to_string(),
        span,
    })?;
    Ok(PatternMatcher::Regex(re))
}

/// Compile a processor chain into a single `Box<dyn Processor>`.
///
/// Single-processor chains return the processor directly.
/// Multi-processor chains wrap in a `ChainProcessor`.
fn compile_processor_chain(
    chain: &ast::ProcessorChain,
    alias_defs: &HashMap<&str, &ast::ProcessorChain>,
) -> Result<Box<dyn Processor>, CompileError> {
    // Alias refs may expand to multi-step chains; flatten into a single list.
    let mut steps: Vec<Box<dyn Processor>> = Vec::new();
    for proc_spanned in &chain.processors {
        compile_single_processor_into(proc_spanned, alias_defs, &mut steps)?;
    }
    if steps.len() == 1 {
        Ok(steps.into_iter().next().expect("checked len"))
    } else {
        Ok(Box::new(ChainProcessor { steps }))
    }
}

/// Compile a single processor AST node, appending results to `out`.
///
/// Alias references are resolved and their chain steps appended directly,
/// so `alias | other` flattens correctly into a single chain.
fn compile_single_processor_into(
    proc_spanned: &Spanned<ast::Processor>,
    alias_defs: &HashMap<&str, &ast::ProcessorChain>,
    out: &mut Vec<Box<dyn Processor>>,
) -> Result<(), CompileError> {
    match &proc_spanned.node {
        ast::Processor::Qed(qed_proc) => {
            out.push(compile_qed_processor(qed_proc)?);
            Ok(())
        }
        ast::Processor::External(ext) => {
            let args: Vec<String> = ext
                .args
                .iter()
                .map(|a| match &a.node {
                    ast::ExternalArg::Quoted(s) => s.clone(),
                    ast::ExternalArg::Unquoted(s) => s.clone(),
                })
                .collect();
            out.push(Box::new(ExternalCommandProcessor {
                command: ext.command.node.clone(),
                args,
            }));
            Ok(())
        }
        ast::Processor::AliasRef(name) => match alias_defs.get(name.as_str()) {
            Some(chain) => {
                for p in &chain.processors {
                    compile_single_processor_into(p, alias_defs, out)?;
                }
                Ok(())
            }
            None => Err(CompileError::UndefinedName {
                name: name.clone(),
                span: proc_spanned.span,
            }),
        },
    }
}

/// Compile a `qed:*` processor invocation.
fn compile_qed_processor(
    qed_proc: &ast::QedProcessor,
) -> Result<Box<dyn Processor>, CompileError> {
    match qed_proc.name.node.as_str() {
        "delete" => Ok(Box::new(DeleteProcessor)),
        "upper" => Ok(Box::new(UpperProcessor)),
        "lower" => Ok(Box::new(LowerProcessor)),
        "prefix" => {
            let text = extract_string_param(&qed_proc.params, "text").ok_or_else(|| {
                CompileError::InvalidParam {
                    processor: "qed:prefix".into(),
                    param: "missing required parameter 'text'".into(),
                    span: qed_proc.name.span,
                }
            })?;
            Ok(Box::new(PrefixProcessor { text }))
        }
        other => Err(CompileError::UndefinedName {
            name: format!("qed:{other}"),
            span: qed_proc.name.span,
        }),
    }
}

/// Extract a string-valued named parameter from a parameter list.
fn extract_string_param(params: &[Spanned<Param>], name: &str) -> Option<String> {
    params.iter().find_map(|p| {
        if p.node.name.node == name
            && let ParamValue::String(s) = &p.node.value.node
        {
            Some(s.clone())
        } else {
            None
        }
    })
}
