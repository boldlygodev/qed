//! Compilation pass — AST → executable IR.
//!
//! Transforms a [`Program`] (AST) into a [`Script`] (compiled IR) in two
//! conceptual phases:
//!
//! 1. **Symbol collection** — walk `PatternDef` and `AliasDef` statements
//!    to populate the symbol table (not yet implemented).
//! 2. **Compilation** — walk `SelectAction` statements, compiling each
//!    selector into a [`CompiledSelector`] (or [`CompoundSelector`]) and
//!    each processor chain into a `Box<dyn Processor>`.
//!
//! Errors are accumulated into a `Vec<CompileError>` so the compiler can
//! report all problems in a single pass.

use crate::error::CompileError;
use crate::parse::ast::{
    self, NthTerm, ParamValue, PatternRefValue, PatternValue, Program,
    SelectorOp as AstSelectorOp,
};
use crate::processor::delete::DeleteProcessor;
use crate::processor::lower::LowerProcessor;
use crate::processor::upper::UpperProcessor;
use crate::processor::Processor;
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
    let mut statements = Vec::new();
    let mut selectors: Vec<RegistryEntry> = Vec::new();
    let mut errors = Vec::new();

    for (i, spanned_stmt) in program.statements.iter().enumerate() {
        let stmt_id = StatementId::new(i);

        match &spanned_stmt.node {
            ast::Statement::SelectAction(node) => {
                // Compile the selector — may push multiple entries for compound selectors
                let sel_id = match compile_selector(node, &mut selectors) {
                    Ok(id) => id,
                    Err(e) => {
                        errors.push(e);
                        continue;
                    }
                };

                // Compile the processor
                let processor: Box<dyn Processor> = match &node.chain {
                    Some(chain) => match compile_processor_chain(&chain.node) {
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

                let sel_span = node.selector.span;
                let sel_text = source[sel_span.start..sel_span.end].to_owned();

                statements.push(Statement {
                    id: stmt_id,
                    selector: sel_id,
                    processor,
                    fallback: None,
                    selector_span: sel_span,
                    selector_text: sel_text,
                });
            }
            ast::Statement::PatternDef { .. } | ast::Statement::AliasDef { .. } => {
                // Phase 5D: symbol table
            }
        }
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
) -> Result<SelectorId, CompileError> {
    let selector_ast = &node.selector.node;

    if selector_ast.steps.len() == 1 {
        // Single-step selector
        let step = &selector_ast.steps[0].node;
        let sel_id = SelectorId::new(registry.len());
        let entry = compile_simple_selector(step, sel_id, node.selector.span)?;
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
) -> Result<RegistryEntry, CompileError> {
    let compiled_pattern = match &step.pattern {
        Some(pat_ref) => compile_pattern(&pat_ref.node, pat_ref.span)?,
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
) -> Result<CompiledPattern, CompileError> {
    let matcher = match &pat_ref.value {
        PatternRefValue::Inline(PatternValue::String(s)) => PatternMatcher::Literal(s.clone()),
        PatternRefValue::Inline(PatternValue::Regex(r)) => {
            let re = regex::Regex::new(r).map_err(|e| CompileError::InvalidRegex {
                pattern: r.clone(),
                reason: e.to_string(),
                span,
            })?;
            PatternMatcher::Regex(re)
        }
        PatternRefValue::Named(_) => {
            return Err(CompileError::UndefinedName {
                name: "named patterns not yet supported".into(),
                span,
            });
        }
    };

    Ok(CompiledPattern {
        matcher,
        negated: pat_ref.negated,
        inclusive: pat_ref.inclusive,
    })
}

/// Compile a processor chain into a single `Box<dyn Processor>`.
///
/// Currently supports single-processor chains only. Multi-processor
/// pipelines will be handled in a later phase.
fn compile_processor_chain(
    chain: &ast::ProcessorChain,
) -> Result<Box<dyn Processor>, CompileError> {
    // For the skeleton, only single-processor chains
    if chain.processors.len() != 1 {
        return Err(CompileError::InvalidParam {
            processor: "chain".into(),
            param: "multi-processor chains not yet supported".into(),
            span: chain.processors[0].span,
        });
    }

    let proc_ast = &chain.processors[0].node;
    match proc_ast {
        ast::Processor::Qed(qed_proc) => match qed_proc.name.node.as_str() {
            "delete" => Ok(Box::new(DeleteProcessor)),
            "upper" => Ok(Box::new(UpperProcessor)),
            "lower" => Ok(Box::new(LowerProcessor)),
            other => Err(CompileError::UndefinedName {
                name: format!("qed:{other}"),
                span: qed_proc.name.span,
            }),
        },
        ast::Processor::External(_) => Err(CompileError::InvalidParam {
            processor: "external".into(),
            param: "external processors not yet supported".into(),
            span: chain.processors[0].span,
        }),
    }
}
