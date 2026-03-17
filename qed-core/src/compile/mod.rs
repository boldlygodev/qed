use crate::error::CompileError;
use crate::parse::ast::{
    self, NthTerm, PatternRefValue, PatternValue, Program, SelectorOp as AstSelectorOp,
};
use crate::processor::delete::DeleteProcessor;
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
/// `nth` uses `Option<Vec<NthTerm>>` — `None` means no filtering (all matches).
#[derive(Debug, Clone)]
pub(crate) enum SelectorOp {
    At {
        pattern: CompiledPattern,
        nth: Option<Vec<NthTerm>>,
    },
    After {
        pattern: CompiledPattern,
    },
    Before {
        pattern: CompiledPattern,
    },
    From {
        pattern: CompiledPattern,
    },
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
    Fail,
    Warn,
    Skip,
}

// ── Compilation ────────────────────────────────────────────────────

/// Compile a parsed AST `Program` into an executable `Script`.
pub(crate) fn compile(program: &Program) -> Result<Script, Vec<CompileError>> {
    let mut statements = Vec::new();
    let mut selectors = Vec::new();
    let mut errors = Vec::new();

    for (i, spanned_stmt) in program.statements.iter().enumerate() {
        let stmt_id = StatementId::new(i);
        let sel_id = SelectorId::new(i);

        match &spanned_stmt.node {
            ast::Statement::SelectAction(node) => {
                // Compile the selector
                match compile_selector(node, sel_id) {
                    Ok(entry) => selectors.push(entry),
                    Err(e) => {
                        errors.push(e);
                        continue;
                    }
                }

                // Compile the processor
                let processor: Box<dyn Processor> = match &node.chain {
                    Some(chain) => {
                        match compile_processor_chain(&chain.node) {
                            Ok(p) => p,
                            Err(e) => {
                                errors.push(e);
                                continue;
                            }
                        }
                    }
                    None => {
                        // No processor chain — identity (passthrough)
                        // For the skeleton, this is an error
                        errors.push(CompileError::InvalidParam {
                            processor: "(none)".into(),
                            param: "missing processor chain".into(),
                            span: spanned_stmt.span,
                        });
                        continue;
                    }
                };

                statements.push(Statement {
                    id: stmt_id,
                    selector: sel_id,
                    processor,
                    fallback: None,
                });
            }
            ast::Statement::PatternDef { .. } | ast::Statement::AliasDef { .. } => {
                // Not yet supported in the skeleton
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

fn compile_selector(
    node: &ast::SelectActionNode,
    sel_id: SelectorId,
) -> Result<RegistryEntry, CompileError> {
    let selector_ast = &node.selector.node;

    // For the skeleton, handle single-step selectors only
    if selector_ast.steps.len() != 1 {
        return Err(CompileError::InvalidParam {
            processor: "selector".into(),
            param: "compound selectors not yet supported".into(),
            span: node.selector.span,
        });
    }

    let step = &selector_ast.steps[0].node;

    let compiled_pattern = match &step.pattern {
        Some(pat_ref) => compile_pattern(&pat_ref.node, pat_ref.span)?,
        None => {
            // No pattern means select everything (at() with no args)
            CompiledPattern {
                matcher: PatternMatcher::Literal(String::new()),
                negated: false,
                inclusive: false,
            }
        }
    };

    let op = match step.op {
        AstSelectorOp::At => SelectorOp::At {
            pattern: compiled_pattern,
            nth: None,
        },
        AstSelectorOp::After => SelectorOp::After {
            pattern: compiled_pattern,
        },
        AstSelectorOp::Before => SelectorOp::Before {
            pattern: compiled_pattern,
        },
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
        on_error: OnError::Fail,
    }))
}

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
