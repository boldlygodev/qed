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

mod env;

use std::collections::HashMap;

use crate::SelectorId;
use crate::StatementId;
use crate::error::{CompileError, CompileWarning};
use crate::parse::ast::{
    self, NthTerm, Param, ParamValue, PatternRefValue, PatternValue, Program,
    SelectorOp as AstSelectorOp,
};
use crate::processor::Processor;
use crate::processor::chain::ChainProcessor;
use crate::processor::dedent::DedentProcessor;
use crate::processor::delete::DeleteProcessor;
use crate::processor::duplicate::DuplicateProcessor;
use crate::processor::external::ExternalCommandProcessor;
use crate::processor::indent::IndentProcessor;
use crate::processor::lower::LowerProcessor;
use crate::processor::number::NumberProcessor;
use crate::processor::prefix::PrefixProcessor;
use crate::processor::replace;
use crate::processor::skip::SkipProcessor;
use crate::processor::suffix::SuffixProcessor;
use crate::processor::trim::TrimProcessor;
use crate::processor::upper::UpperProcessor;
use crate::processor::wrap::WrapProcessor;
use crate::span::Spanned;

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
    /// Source span of the fallback processor chain (for diagnostics).
    pub(crate) fallback_span: Option<crate::span::Span>,
    /// Original source text of the fallback processor chain (for diagnostics).
    pub(crate) fallback_text: Option<String>,
    /// Source span of the selector expression (for diagnostics).
    pub(crate) selector_span: crate::span::Span,
    /// Original source text of the selector expression (for diagnostics).
    pub(crate) selector_text: String,
    /// Source span of the processor chain (for processor error diagnostics).
    pub(crate) processor_span: crate::span::Span,
    /// Original source text of the processor chain (for processor error diagnostics).
    pub(crate) processor_text: String,
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
    After { pattern: CompiledPattern },
    /// Select the zero-width insertion point before each matching line.
    Before { pattern: CompiledPattern },
    /// Select from the matching line to end of input (inclusivity
    /// controlled by `pattern.inclusive`).
    From { pattern: CompiledPattern },
    /// Select from start of input to the matching line (inclusivity
    /// controlled by `pattern.inclusive`).
    To { pattern: CompiledPattern },
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
///
/// Returns `Ok((script, warnings))` on success, where `warnings` contains
/// non-fatal diagnostics such as unset environment variable references.
/// Returns `Err(errors)` if any hard compilation errors are encountered.
pub(crate) fn compile(
    program: &Program,
    source: &str,
    no_env: bool,
) -> Result<(Script, Vec<CompileWarning>), Vec<CompileError>> {
    let mut errors = Vec::new();
    let mut warnings: Vec<CompileWarning> = Vec::new();

    // ── Pass 1: collect definitions ──────────────────────────────────
    let mut pattern_defs: HashMap<&str, &PatternValue> = HashMap::new();
    let mut alias_defs: HashMap<&str, &ast::ProcessorChain> = HashMap::new();

    for spanned_stmt in &program.statements {
        match &spanned_stmt.node {
            ast::Statement::PatternDef { name, value } => {
                if pattern_defs.insert(&name.node, &value.node).is_some() {
                    warnings.push(CompileWarning::DuplicateName {
                        name: name.node.clone(),
                        kind: crate::error::SymbolKind::Pattern,
                        span: spanned_stmt.span,
                    });
                }
            }
            ast::Statement::AliasDef { name, chain } => {
                if alias_defs.insert(&name.node, &chain.node).is_some() {
                    warnings.push(CompileWarning::DuplicateName {
                        name: name.node.clone(),
                        kind: crate::error::SymbolKind::Alias,
                        span: spanned_stmt.span,
                    });
                }
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
        let sel_id = match compile_selector(
            node,
            &mut selectors,
            &pattern_defs,
            &alias_defs,
            no_env,
            &mut warnings,
        ) {
            Ok(id) => id,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };

        // Compile the processor
        let processor: Box<dyn Processor> = match &node.chain {
            Some(chain) => {
                match compile_processor_chain(
                    &chain.node,
                    &pattern_defs,
                    &alias_defs,
                    no_env,
                    &mut warnings,
                ) {
                    Ok(p) => p,
                    Err(e) => {
                        errors.push(e);
                        continue;
                    }
                }
            }
            None => {
                errors.push(CompileError::InvalidParam {
                    processor: "(none)".into(),
                    param: "missing processor chain".into(),
                    span: spanned_stmt.span,
                });
                continue;
            }
        };

        // Compile optional fallback, tracking its processor span for diagnostics.
        let (fallback, fallback_chain_span): (
            Option<Box<dyn Processor>>,
            Option<crate::span::Span>,
        ) = match &node.fallback {
            Some(fb) => match &fb.node {
                ast::Fallback::Chain(chain) => {
                    match compile_processor_chain(
                        chain,
                        &pattern_defs,
                        &alias_defs,
                        no_env,
                        &mut warnings,
                    ) {
                        Ok(p) => (Some(p), Some(fb.span)),
                        Err(e) => {
                            errors.push(e);
                            (None, None)
                        }
                    }
                }
                ast::Fallback::SelectAction(sa) => match &sa.chain {
                    Some(chain) => {
                        match compile_processor_chain(
                            &chain.node,
                            &pattern_defs,
                            &alias_defs,
                            no_env,
                            &mut warnings,
                        ) {
                            Ok(p) => (Some(p), Some(chain.span)),
                            Err(e) => {
                                errors.push(e);
                                (None, None)
                            }
                        }
                    }
                    None => (None, None),
                },
            },
            None => (None, None),
        };

        let sel_span = node.selector.span;
        let sel_text = source[sel_span.start..sel_span.end].to_owned();

        let proc_span = node
            .chain
            .as_ref()
            .map(|c| c.span)
            .unwrap_or(spanned_stmt.span);
        let proc_text = source[proc_span.start..proc_span.end].to_owned();

        let fb_span = fallback_chain_span;
        let fb_text = fb_span.map(|s| source[s.start..s.end].to_owned());

        statements.push(Statement {
            id: stmt_id,
            selector: sel_id,
            processor,
            fallback,
            fallback_span: fb_span,
            fallback_text: fb_text,
            selector_span: sel_span,
            selector_text: sel_text,
            processor_span: proc_span,
            processor_text: proc_text,
        });
    }

    if errors.is_empty() {
        Ok((
            Script {
                statements,
                selectors,
            },
            warnings,
        ))
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
    alias_defs: &HashMap<&str, &ast::ProcessorChain>,
    no_env: bool,
    warnings: &mut Vec<CompileWarning>,
) -> Result<SelectorId, CompileError> {
    let selector_ast = &node.selector.node;

    if selector_ast.steps.len() == 1 {
        // Single-step selector
        let step = &selector_ast.steps[0].node;
        let sel_id = SelectorId::new(registry.len());
        let entry = compile_simple_selector(
            step,
            sel_id,
            node.selector.span,
            pattern_defs,
            alias_defs,
            no_env,
            warnings,
        )?;
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
                alias_defs,
                no_env,
                warnings,
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
    alias_defs: &HashMap<&str, &ast::ProcessorChain>,
    no_env: bool,
    warnings: &mut Vec<CompileWarning>,
) -> Result<RegistryEntry, CompileError> {
    let mut compiled_pattern = match &step.pattern {
        Some(pat_ref) => compile_pattern(
            &pat_ref.node,
            pat_ref.span,
            pattern_defs,
            alias_defs,
            no_env,
            warnings,
        )?,
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

    // Warn if `+` (inclusive) is used on a non-boundary selector.
    let non_boundary_op = match step.op {
        AstSelectorOp::At => Some("at"),
        AstSelectorOp::After => Some("after"),
        AstSelectorOp::Before => Some("before"),
        AstSelectorOp::From | AstSelectorOp::To => None,
    };
    if compiled_pattern.inclusive
        && let Some(op_name) = non_boundary_op
    {
        warnings.push(CompileWarning::InclusiveIgnored {
            selector_op: op_name,
            span,
        });
        compiled_pattern.inclusive = false;
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
    alias_defs: &HashMap<&str, &ast::ProcessorChain>,
    no_env: bool,
    warnings: &mut Vec<CompileWarning>,
) -> Result<CompiledPattern, CompileError> {
    let matcher = match &pat_ref.value {
        PatternRefValue::Inline(PatternValue::String(s)) => {
            let expanded = expand_and_warn(s, no_env, span, warnings);
            PatternMatcher::Literal(expanded)
        }
        PatternRefValue::Inline(PatternValue::Regex(r)) => {
            let expanded = expand_and_warn(r, no_env, span, warnings);
            compile_regex_matcher(&expanded, span)?
        }
        PatternRefValue::Named(name) => match pattern_defs.get(name.as_str()) {
            Some(PatternValue::String(s)) => {
                let expanded = expand_and_warn(s, no_env, span, warnings);
                PatternMatcher::Literal(expanded)
            }
            Some(PatternValue::Regex(r)) => {
                let expanded = expand_and_warn(r, no_env, span, warnings);
                compile_regex_matcher(&expanded, span)?
            }
            None => {
                if alias_defs.contains_key(name.as_str()) {
                    return Err(CompileError::WrongSymbolKind {
                        name: name.clone(),
                        expected: crate::error::SymbolKind::Pattern,
                        found: crate::error::SymbolKind::Alias,
                        span,
                    });
                }
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
    pattern_defs: &HashMap<&str, &PatternValue>,
    alias_defs: &HashMap<&str, &ast::ProcessorChain>,
    no_env: bool,
    warnings: &mut Vec<CompileWarning>,
) -> Result<Box<dyn Processor>, CompileError> {
    // Alias refs may expand to multi-step chains; flatten into a single list.
    let mut steps: Vec<Box<dyn Processor>> = Vec::new();
    for proc_spanned in &chain.processors {
        compile_single_processor_into(
            proc_spanned,
            pattern_defs,
            alias_defs,
            &mut steps,
            no_env,
            warnings,
        )?;
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
    pattern_defs: &HashMap<&str, &PatternValue>,
    alias_defs: &HashMap<&str, &ast::ProcessorChain>,
    out: &mut Vec<Box<dyn Processor>>,
    no_env: bool,
    warnings: &mut Vec<CompileWarning>,
) -> Result<(), CompileError> {
    match &proc_spanned.node {
        ast::Processor::Qed(qed_proc) => {
            out.push(compile_qed_processor(
                qed_proc,
                pattern_defs,
                alias_defs,
                no_env,
                warnings,
            )?);
            Ok(())
        }
        ast::Processor::External(ext) => {
            let command = expand_and_warn(&ext.command.node, no_env, proc_spanned.span, warnings);
            let args: Vec<String> = ext
                .args
                .iter()
                .map(|a| {
                    let raw = match &a.node {
                        ast::ExternalArg::Quoted(s) | ast::ExternalArg::Unquoted(s) => s.as_str(),
                    };
                    expand_and_warn(raw, no_env, a.span, warnings)
                })
                .collect();
            out.push(Box::new(ExternalCommandProcessor { command, args }));
            Ok(())
        }
        ast::Processor::AliasRef(name) => match alias_defs.get(name.as_str()) {
            Some(chain) => {
                for p in &chain.processors {
                    compile_single_processor_into(
                        p,
                        pattern_defs,
                        alias_defs,
                        out,
                        no_env,
                        warnings,
                    )?;
                }
                Ok(())
            }
            None => {
                if pattern_defs.contains_key(name.as_str()) {
                    Err(CompileError::WrongSymbolKind {
                        name: name.clone(),
                        expected: crate::error::SymbolKind::Alias,
                        found: crate::error::SymbolKind::Pattern,
                        span: proc_spanned.span,
                    })
                } else {
                    // No alias or pattern with this name — treat as an
                    // external command. Bare words in processor position
                    // that don't resolve to a defined alias fall through
                    // to PATH lookup at runtime.
                    let command = expand_and_warn(name, no_env, proc_spanned.span, warnings);
                    out.push(Box::new(ExternalCommandProcessor {
                        command,
                        args: Vec::new(),
                    }));
                    Ok(())
                }
            }
        },
    }
}

/// Compile a `qed:*` processor invocation.
fn compile_qed_processor(
    qed_proc: &ast::QedProcessor,
    pattern_defs: &HashMap<&str, &PatternValue>,
    alias_defs: &HashMap<&str, &ast::ProcessorChain>,
    no_env: bool,
    warnings: &mut Vec<CompileWarning>,
) -> Result<Box<dyn Processor>, CompileError> {
    match qed_proc.name.node.as_str() {
        "delete" | "upper" | "lower" | "duplicate" | "skip" | "trim" => {
            reject_unknown_params(&qed_proc.params, &[], &qed_proc.name.node)?;
            match qed_proc.name.node.as_str() {
                "delete" => Ok(Box::new(DeleteProcessor)),
                "upper" => Ok(Box::new(UpperProcessor)),
                "lower" => Ok(Box::new(LowerProcessor)),
                "duplicate" => Ok(Box::new(DuplicateProcessor)),
                "skip" => Ok(Box::new(SkipProcessor)),
                "trim" => Ok(Box::new(TrimProcessor)),
                _ => unreachable!(),
            }
        }
        "prefix" | "suffix" => {
            reject_unknown_params(&qed_proc.params, &["text"], &qed_proc.name.node)?;
            let raw = extract_string_param(&qed_proc.params, "text").ok_or_else(|| {
                CompileError::InvalidParam {
                    processor: format!("qed:{}", qed_proc.name.node),
                    param: "missing required parameter 'text'".into(),
                    span: qed_proc.name.span,
                }
            })?;
            let text = expand_and_warn(&raw, no_env, qed_proc.name.span, warnings);
            match qed_proc.name.node.as_str() {
                "prefix" => Ok(Box::new(PrefixProcessor { text })),
                "suffix" => Ok(Box::new(SuffixProcessor { text })),
                _ => unreachable!(),
            }
        }
        "replace" => {
            reject_unknown_params(&qed_proc.params, &[], &qed_proc.name.node)?;
            compile_replace_processor(qed_proc, pattern_defs, alias_defs, no_env, warnings)
        }
        "number" => {
            reject_unknown_params(&qed_proc.params, &["start", "width"], &qed_proc.name.node)?;
            let start = extract_int_param(&qed_proc.params, "start").unwrap_or(1);
            let width = extract_int_param(&qed_proc.params, "width").unwrap_or(0) as usize;
            Ok(Box::new(NumberProcessor { start, width }))
        }
        "indent" => {
            reject_unknown_params(&qed_proc.params, &["width", "char"], &qed_proc.name.node)?;
            let width = extract_int_param(&qed_proc.params, "width").ok_or_else(|| {
                CompileError::InvalidParam {
                    processor: "qed:indent".into(),
                    param: "missing required parameter 'width'".into(),
                    span: qed_proc.name.span,
                }
            })? as usize;
            let indent_char =
                extract_string_param(&qed_proc.params, "char").unwrap_or_else(|| " ".to_owned());
            Ok(Box::new(IndentProcessor { width, indent_char }))
        }
        "dedent" => {
            reject_unknown_params(&qed_proc.params, &[], &qed_proc.name.node)?;
            Ok(Box::new(DedentProcessor))
        }
        "wrap" => {
            reject_unknown_params(&qed_proc.params, &["width"], &qed_proc.name.node)?;
            let width = extract_int_param(&qed_proc.params, "width").ok_or_else(|| {
                CompileError::InvalidParam {
                    processor: "qed:wrap".into(),
                    param: "missing required parameter 'width'".into(),
                    span: qed_proc.name.span,
                }
            })? as usize;
            Ok(Box::new(WrapProcessor { width }))
        }
        other => Err(CompileError::UndefinedName {
            name: format!("qed:{other}"),
            span: qed_proc.name.span,
        }),
    }
}

/// Compile `qed:replace(search, replacement)`.
fn compile_replace_processor(
    qed_proc: &ast::QedProcessor,
    pattern_defs: &HashMap<&str, &PatternValue>,
    alias_defs: &HashMap<&str, &ast::ProcessorChain>,
    no_env: bool,
    warnings: &mut Vec<CompileWarning>,
) -> Result<Box<dyn Processor>, CompileError> {
    if qed_proc.args.len() != 2 {
        return Err(CompileError::InvalidParam {
            processor: "qed:replace".into(),
            param: format!(
                "expected 2 arguments (search, replacement), got {}",
                qed_proc.args.len()
            ),
            span: qed_proc.name.span,
        });
    }

    // First arg: search pattern (string or regex).
    let search = match &qed_proc.args[0].node {
        ast::QedArg::String(s) => {
            let expanded = expand_and_warn(s, no_env, qed_proc.args[0].span, warnings);
            replace::ReplaceSearch::Literal(expanded)
        }
        ast::QedArg::Regex(r) => {
            let expanded = expand_and_warn(r, no_env, qed_proc.args[0].span, warnings);
            let re = regex::Regex::new(&expanded).map_err(|e| CompileError::InvalidRegex {
                pattern: expanded,
                reason: e.to_string(),
                span: qed_proc.args[0].span,
            })?;
            replace::ReplaceSearch::Regex(re)
        }
        _ => {
            return Err(CompileError::InvalidParam {
                processor: "qed:replace".into(),
                param: "first argument must be a string or regex pattern".into(),
                span: qed_proc.args[0].span,
            });
        }
    };

    // Second arg: replacement (string, regex template, or processor chain).
    let is_literal_search = matches!(search, replace::ReplaceSearch::Literal(_));

    let replacement = match &qed_proc.args[1].node {
        ast::QedArg::String(s) => {
            let expanded = expand_and_warn(s, no_env, qed_proc.args[1].span, warnings);
            replace::ReplaceWith::Literal(expanded)
        }
        ast::QedArg::Regex(r) => {
            if is_literal_search {
                return Err(CompileError::InvalidParam {
                    processor: "qed:replace".into(),
                    param: "regex template replacement requires a regex search pattern".into(),
                    span: qed_proc.args[1].span,
                });
            }
            let expanded = expand_and_warn(r, no_env, qed_proc.args[1].span, warnings);
            replace::ReplaceWith::Template(expanded)
        }
        ast::QedArg::ProcessorChain(chain) => {
            let proc = compile_processor_chain(chain, pattern_defs, alias_defs, no_env, warnings)?;
            replace::ReplaceWith::Pipeline(proc)
        }
        _ => {
            return Err(CompileError::InvalidParam {
                processor: "qed:replace".into(),
                param: "second argument must be a string, regex template, or processor chain"
                    .into(),
                span: qed_proc.args[1].span,
            });
        }
    };

    Ok(Box::new(replace::ReplaceProcessor {
        search,
        replacement,
    }))
}

/// Reject any parameter whose name is not in `known`.
fn reject_unknown_params(
    params: &[Spanned<Param>],
    known: &[&str],
    proc_name: &str,
) -> Result<(), CompileError> {
    for param in params {
        if !known.contains(&param.node.name.node.as_str()) {
            return Err(CompileError::InvalidParam {
                processor: format!("qed:{proc_name}"),
                param: format!("unknown parameter: {}", param.node.name.node),
                span: param.span,
            });
        }
    }
    Ok(())
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

fn extract_int_param(params: &[Spanned<Param>], name: &str) -> Option<i64> {
    params.iter().find_map(|p| {
        if p.node.name.node != name {
            return None;
        }
        match &p.node.value.node {
            ParamValue::Integer(n) => Some(*n),
            // The parser produces NthExpr for bare integers like `width:4`.
            // Extract the integer when it's a single NthTerm::Integer term.
            ParamValue::NthExpr(nth) if nth.terms.len() == 1 => {
                if let NthTerm::Integer(n) = &nth.terms[0].node {
                    Some(*n)
                } else {
                    None
                }
            }
            _ => None,
        }
    })
}

// ── Env expansion helper ────────────────────────────────────────────

/// Expand environment variables in a string and push any unset-var
/// warnings into the warnings accumulator.
fn expand_and_warn(
    input: &str,
    no_env: bool,
    span: crate::span::Span,
    warnings: &mut Vec<CompileWarning>,
) -> String {
    let (expanded, unset) = env::expand_env_vars(input, no_env);
    for var in unset {
        warnings.push(CompileWarning::UnsetEnvVar {
            name: var.name,
            span,
        });
    }
    expanded
}
