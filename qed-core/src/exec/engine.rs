//! Top-level execution loop.
//!
//! Takes a compiled [`Script`] and an immutable [`Buffer`], fragments the
//! buffer according to the script's selectors, routes each `Selected`
//! fragment through its statement's processor, and concatenates the results
//! into the final output string.

use std::collections::HashSet;

use crate::compile::{OnError, Script};
use crate::processor::ProcessorError;
use crate::span::Span;

use super::{Buffer, Fragment, FragmentContent, fragment};

/// A diagnostic produced during execution.
#[derive(Debug, Clone)]
pub(crate) struct Diagnostic {
    pub(crate) level: DiagnosticLevel,
    pub(crate) message: String,
    pub(crate) span: Span,
    pub(crate) selector_text: String,
    /// Whether this error was recovered by a fallback processor.
    /// Recovered errors are still reported but do not cause a non-zero exit.
    pub(crate) recovered: bool,
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
    let mut diagnostics = Vec::new();
    let mut has_unrecovered_error = false;

    for frag in &fragments {
        match frag {
            Fragment::Passthrough(content) => {
                output.push_str(&resolve_content(content, buffer));
            }
            Fragment::Selected { content, tags } => {
                let text = resolve_content(content, buffer);

                let mut handled = false;
                for (stmt_id, _sel_id) in tags {
                    let Some(stmt) = script.statements.iter().find(|s| s.id == *stmt_id) else {
                        continue;
                    };

                    match stmt.processor.execute(&text) {
                        Ok(result) => {
                            output.push_str(&result);
                            handled = true;
                            break;
                        }
                        Err(e) => {
                            let mut recovered = false;
                            let mut diag_span = stmt.processor_span;
                            let mut diag_text = stmt.processor_text.clone();
                            let mut diag_msg = format_processor_error(&e);

                            if let Some(ref fb) = stmt.fallback {
                                match fb.execute(&text) {
                                    Ok(result) => {
                                        output.push_str(&result);
                                        recovered = true;
                                    }
                                    Err(fb_err) => {
                                        // Fallback failed — report fallback error
                                        diag_span =
                                            stmt.fallback_span.unwrap_or(stmt.processor_span);
                                        diag_text = stmt
                                            .fallback_text
                                            .clone()
                                            .unwrap_or(stmt.processor_text.clone());
                                        diag_msg = format_processor_error(&fb_err);
                                    }
                                }
                            }

                            diagnostics.push(Diagnostic {
                                level: DiagnosticLevel::Error,
                                message: diag_msg,
                                span: diag_span,
                                selector_text: diag_text,
                                recovered,
                            });
                            if recovered {
                                handled = true;
                                break;
                            }
                            has_unrecovered_error = true;
                        }
                    }
                }

                if !handled {
                    output.push_str(&text);
                }
            }
        }
    }

    // Discard output on unrecovered processor error — the script failed,
    // so partial output should not be emitted.
    if has_unrecovered_error {
        output.clear();
    }

    // Check for no-match diagnostics
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
                        recovered: false,
                    });
                }
                OnError::Warn => {
                    diagnostics.push(Diagnostic {
                        level: DiagnosticLevel::Warning,
                        message: "no lines matched".into(),
                        span: stmt.selector_span,
                        selector_text: stmt.selector_text.clone(),
                        recovered: false,
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

/// Format a processor error into a human-readable diagnostic message.
fn format_processor_error(e: &ProcessorError) -> String {
    match e {
        ProcessorError::ExternalFailed {
            exit_code: Some(code),
            ..
        } => format!("exit code {code}"),
        ProcessorError::ExternalFailed {
            exit_code: None, ..
        } => "command failed".into(),
        ProcessorError::ProcessorFailed { reason, .. } => reason.clone(),
        ProcessorError::NoMatch { .. } => "no lines matched".into(),
    }
}
