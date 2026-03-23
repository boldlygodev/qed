//! Top-level execution loop.
//!
//! Takes a compiled [`Script`] and an immutable [`Buffer`], fragments the
//! buffer according to the script's selectors, routes each `Selected`
//! fragment through its statement's processor, and concatenates the results
//! into the final output string.
//!
//! Copy/move statements are handled as a post-processing pass: the source
//! text is collected during the fragment walk, then inserted at the
//! destination after the main output is assembled.

use std::collections::HashSet;

use crate::StatementId;
use crate::compile::{
    CompiledPattern, Destination, DestinationKind, OnError, PatternMatcher, Script, StatementAction,
};
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

/// A pending copy/move insertion collected during the fragment walk.
struct PendingRelocation {
    source_text: String,
    destination: Destination,
}

/// Execute a compiled script against a buffer, producing output and diagnostics.
///
/// When `extract` is true, passthrough fragments are suppressed — only selected
/// (and processed) regions appear in the output.
pub(crate) fn execute(script: &Script, buffer: &Buffer, extract: bool) -> ExecuteResult {
    // Build requests: (StatementId, SelectorId) for fragmentation
    let requests: Vec<_> = script
        .statements
        .iter()
        .map(|s| (s.id, s.selector))
        .collect();

    let fragments = fragment::fragment(buffer, &requests, &script.selectors);

    // Collect which statements were matched
    let mut matched_statements: HashSet<StatementId> = HashSet::new();
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
    let mut pending_relocations: Vec<PendingRelocation> = Vec::new();

    for frag in &fragments {
        match frag {
            Fragment::Passthrough(content) => {
                if !extract {
                    output.push_str(&resolve_content(content, buffer));
                }
            }
            Fragment::Selected { content, tags } => {
                let text = resolve_content(content, buffer);

                let mut handled = false;
                for (stmt_id, _sel_id) in tags {
                    let Some(stmt) = script.statements.iter().find(|s| s.id == *stmt_id) else {
                        continue;
                    };

                    match &stmt.action {
                        StatementAction::Process(processor) => match processor.execute(&text) {
                            Ok(result) => {
                                output.push_str(&result);
                                handled = true;
                                break;
                            }
                            Err(e) => {
                                handle_processor_error(
                                    e,
                                    stmt,
                                    &text,
                                    &mut output,
                                    &mut diagnostics,
                                    &mut has_unrecovered_error,
                                    &mut handled,
                                );
                                if handled {
                                    break;
                                }
                            }
                        },
                        StatementAction::CopyTo(dest) => {
                            // Copy: emit source text in place AND record for insertion.
                            output.push_str(&text);
                            pending_relocations.push(PendingRelocation {
                                source_text: text.clone(),
                                destination: Destination {
                                    kind: dest.kind,
                                    pattern: dest.pattern.clone(),
                                },
                            });
                            handled = true;
                            break;
                        }
                        StatementAction::MoveTo(dest) => {
                            // Move: do NOT emit source text, only record for insertion.
                            pending_relocations.push(PendingRelocation {
                                source_text: text.clone(),
                                destination: Destination {
                                    kind: dest.kind,
                                    pattern: dest.pattern.clone(),
                                },
                            });
                            handled = true;
                            break;
                        }
                    }
                }

                if !handled {
                    output.push_str(&text);
                }
            }
        }
    }

    // Apply pending copy/move relocations to the output.
    if !pending_relocations.is_empty() && !has_unrecovered_error {
        output = apply_relocations(&output, &pending_relocations);
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

/// Handle a processor error, attempting fallback recovery.
fn handle_processor_error(
    e: ProcessorError,
    stmt: &crate::compile::Statement,
    text: &str,
    output: &mut String,
    diagnostics: &mut Vec<Diagnostic>,
    has_unrecovered_error: &mut bool,
    handled: &mut bool,
) {
    let mut recovered = false;
    let mut diag_span = stmt.processor_span;
    let mut diag_text = stmt.processor_text.clone();
    let mut diag_msg = format_processor_error(&e);

    if let Some(ref fb) = stmt.fallback {
        match fb.execute(text) {
            Ok(result) => {
                output.push_str(&result);
                recovered = true;
            }
            Err(fb_err) => {
                diag_span = stmt.fallback_span.unwrap_or(stmt.processor_span);
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
        *handled = true;
    } else {
        *has_unrecovered_error = true;
    }
}

/// Apply pending copy/move relocations to the assembled output.
///
/// For each relocation, scans the output lines for the destination pattern
/// and inserts/replaces at the matched position.
fn apply_relocations(output: &str, relocations: &[PendingRelocation]) -> String {
    let has_trailing_newline = output.ends_with('\n');
    let content = if has_trailing_newline {
        &output[..output.len() - 1]
    } else {
        output
    };

    let mut lines: Vec<String> = content.split('\n').map(String::from).collect();

    for reloc in relocations {
        // Strip trailing newline from source text for clean insertion.
        let source = if reloc.source_text.ends_with('\n') {
            &reloc.source_text[..reloc.source_text.len() - 1]
        } else {
            &reloc.source_text
        };
        let source_lines: Vec<&str> = source.split('\n').collect();

        // Find destination line indices (scan in reverse for stable insertion).
        let dest_indices: Vec<usize> = lines
            .iter()
            .enumerate()
            .filter(|(_, line)| pattern_matches(&reloc.destination.pattern, line))
            .map(|(i, _)| i)
            .collect();

        match reloc.destination.kind {
            DestinationKind::After => {
                // Insert after each matching line (process in reverse for stable indices).
                for &idx in dest_indices.iter().rev() {
                    let insert_pos = idx + 1;
                    for (j, src_line) in source_lines.iter().enumerate() {
                        lines.insert(insert_pos + j, (*src_line).to_owned());
                    }
                }
            }
            DestinationKind::Before => {
                for &idx in dest_indices.iter().rev() {
                    for (j, src_line) in source_lines.iter().enumerate() {
                        lines.insert(idx + j, (*src_line).to_owned());
                    }
                }
            }
            DestinationKind::At => {
                // Replace each matching line with the source text.
                for &idx in dest_indices.iter().rev() {
                    lines.remove(idx);
                    for (j, src_line) in source_lines.iter().enumerate() {
                        lines.insert(idx + j, (*src_line).to_owned());
                    }
                }
            }
        }
    }

    let mut result = lines.join("\n");
    if has_trailing_newline {
        result.push('\n');
    }
    result
}

/// Check if a line matches a compiled pattern.
fn pattern_matches(pattern: &CompiledPattern, line: &str) -> bool {
    let matched = match &pattern.matcher {
        PatternMatcher::Literal(s) => line.contains(s.as_str()),
        PatternMatcher::Regex(re) => re.is_match(line),
    };
    if pattern.negated { !matched } else { matched }
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
