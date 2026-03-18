//! Top-level execution loop.
//!
//! Takes a compiled [`Script`] and an immutable [`Buffer`], fragments the
//! buffer according to the script's selectors, routes each `Selected`
//! fragment through its statement's processor, and concatenates the results
//! into the final output string.

use std::collections::HashSet;

use crate::compile::{OnError, Script};
use crate::span::Span;

use super::{Buffer, Fragment, FragmentContent, fragment};

/// A diagnostic produced during execution.
#[derive(Debug, Clone)]
pub(crate) struct Diagnostic {
    pub(crate) level: DiagnosticLevel,
    pub(crate) message: String,
    pub(crate) span: Span,
    pub(crate) selector_text: String,
}

/// Severity level for diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiagnosticLevel {
    Error,
    Warning,
}

/// Result of executing a script.
pub(crate) struct ExecuteResult {
    pub(crate) output: String,
    pub(crate) diagnostics: Vec<Diagnostic>,
}

/// Execute a compiled script against a buffer, producing output and diagnostics.
pub(crate) fn execute(script: &Script, buffer: &Buffer) -> ExecuteResult {
    // Build requests: (StatementId, SelectorId) for fragmentation
    let requests: Vec<_> = script
        .statements
        .iter()
        .map(|s| (s.id, s.selector))
        .collect();

    let fragments = fragment::fragment(buffer, &requests, &script.selectors);

    // Collect which statements were matched
    let mut matched_statements: HashSet<crate::StatementId> = HashSet::new();
    for frag in &fragments {
        if let Fragment::Selected { tags, .. } = frag {
            for (stmt_id, _) in tags {
                matched_statements.insert(*stmt_id);
            }
        }
    }

    let mut output = String::new();

    for frag in &fragments {
        match frag {
            Fragment::Passthrough(content) => {
                output.push_str(&resolve_content(content, buffer));
            }
            Fragment::Selected { content, tags } => {
                let text = resolve_content(content, buffer);

                // Find the first matching statement's processor
                let processed = tags.iter().find_map(|(stmt_id, _sel_id)| {
                    script
                        .statements
                        .iter()
                        .find(|s| s.id == *stmt_id)
                        .and_then(|stmt| stmt.processor.execute(&text).ok())
                });

                match processed {
                    Some(result) => output.push_str(&result),
                    None => output.push_str(&text),
                }
            }
        }
    }

    // Check for no-match diagnostics
    let mut diagnostics = Vec::new();
    for stmt in &script.statements {
        if !matched_statements.contains(&stmt.id) {
            let on_error = script
                .selectors
                .get(stmt.selector.value())
                .map(|entry| match entry {
                    crate::compile::RegistryEntry::Simple(s) => s.on_error,
                    crate::compile::RegistryEntry::Compound(_) => OnError::Fail,
                })
                .unwrap_or(OnError::Fail);

            match on_error {
                OnError::Fail => {
                    diagnostics.push(Diagnostic {
                        level: DiagnosticLevel::Error,
                        message: "no lines matched".into(),
                        span: stmt.selector_span,
                        selector_text: stmt.selector_text.clone(),
                    });
                }
                OnError::Warn => {
                    diagnostics.push(Diagnostic {
                        level: DiagnosticLevel::Warning,
                        message: "no lines matched".into(),
                        span: stmt.selector_span,
                        selector_text: stmt.selector_text.clone(),
                    });
                }
                OnError::Skip => {
                    // Silently skip
                }
            }
        }
    }

    ExecuteResult {
        output,
        diagnostics,
    }
}

/// Materialize a fragment's content into an owned `String`.
///
/// `Borrowed` fragments slice into the buffer (zero-copy until this point);
/// `Owned` fragments already carry their text (produced by a prior processor).
fn resolve_content(content: &FragmentContent, buffer: &Buffer) -> String {
    match content {
        FragmentContent::Borrowed(range) => buffer.slice(*range).to_owned(),
        FragmentContent::Owned(s) => s.clone(),
    }
}
