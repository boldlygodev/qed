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

use std::collections::{HashMap, HashSet};

use crate::StatementId;
use crate::compile::{
    CompiledFallback, CompiledPattern, Destination, DestinationKind, OnError, PatternMatcher,
    RegistryEntry, Script, SelectorOp, StatementAction,
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
    Debug,
}

/// Result of executing a script.
pub(crate) struct ExecuteResult {
    pub(crate) output: String,
    pub(crate) diagnostics: Vec<Diagnostic>,
    /// Raw lines to emit to stderr (from `qed:warn()`, `qed:fail()`,
    /// `qed:debug:print()`).
    pub(crate) stderr_lines: Vec<String>,
    /// Whether execution was halted by `qed:fail()`.
    pub(crate) halted_by_fail: bool,
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
    let mut stderr_lines: Vec<String> = Vec::new();
    let mut has_unrecovered_error = false;
    let mut has_processor_error = false;
    let mut halted_by_fail = false;
    let mut pending_relocations: Vec<PendingRelocation> = Vec::new();
    let mut debug_counts: HashMap<StatementId, usize> = HashMap::new();

    // ── Pre-check: handle no-match before the fragment walk ─────────
    //
    // For each unmatched statement with on_error:fail, try its fallback.
    // If fallback succeeds, return immediately with the fallback output.
    // If no fallback or fallback exhausted, record the error and the ID
    // of the first failed statement for halting the fragment walk.
    let mut first_failed_stmt: Option<StatementId> = None;

    for stmt in &script.statements {
        if matched_statements.contains(&stmt.id) {
            continue;
        }

        let on_error = get_on_error(stmt, script);

        match on_error {
            OnError::Fail => {
                if let Some(ref fb) = stmt.fallback
                    && let Some(fb_output) =
                        execute_no_match_fallback(fb, buffer, script, extract, &mut diagnostics)
                {
                    return ExecuteResult {
                        output: fb_output,
                        diagnostics,
                        stderr_lines,
                        halted_by_fail: false,
                    };
                }
                // No fallback or fallback failed
                if stmt.fallback.is_none() {
                    diagnostics.push(Diagnostic {
                        level: DiagnosticLevel::Error,
                        message: "no lines matched".into(),
                        span: stmt.selector_span,
                        selector_text: stmt.selector_text.clone(),
                        recovered: false,
                    });
                }
                has_unrecovered_error = true;
                if first_failed_stmt.is_none() {
                    first_failed_stmt = Some(stmt.id);
                }
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

    // ── Fragment walk ────────────────────────────────────────────────
    let mut halted = false;

    for frag in &fragments {
        if halted {
            break;
        }
        match frag {
            Fragment::Passthrough(content) => {
                if !extract {
                    output.push_str(&resolve_content(content, buffer));
                }
            }
            Fragment::Selected { content, tags } => {
                // Halt if this fragment is tagged by a statement at or after
                // the first failed statement — those lines are blocked.
                if let Some(failed_id) = first_failed_stmt
                    && tags.iter().any(|(sid, _)| *sid >= failed_id)
                {
                    halted = true;
                    continue;
                }

                let mut text = resolve_content(content, buffer);

                // `finished` tracks whether a non-Process action already
                // wrote to `output`, so we know not to push `text` again
                // after the loop.
                let mut finished = false;
                let mut processed = false;
                for (stmt_id, sel_id) in tags {
                    let Some(stmt) = script.statements.iter().find(|s| s.id == *stmt_id) else {
                        continue;
                    };

                    // After a processor has transformed the text, only
                    // chain the next statement's processor if its selector
                    // still matches the new text (re-fragmentation).
                    if processed && !selector_still_matches(&script.selectors, *sel_id, &text) {
                        continue;
                    }

                    match &stmt.action {
                        StatementAction::Process(processor) => match processor.execute(&text) {
                            Ok(result) => {
                                // Chain: update text so the next tagged
                                // processor sees this processor's output.
                                // If the processor deleted the content (empty
                                // result), stop — the line is gone.
                                text = result;
                                processed = true;
                                if text.is_empty() {
                                    finished = true;
                                    break;
                                }
                            }
                            Err(ProcessorError::FileEmptyRegion { span }) => {
                                // qed:file() on an empty region (insertion
                                // point) — emit warning, pass text through.
                                diagnostics.push(Diagnostic {
                                    level: DiagnosticLevel::Warning,
                                    message: "qed:file() ignored for empty region".into(),
                                    span,
                                    selector_text: "qed:file()".into(),
                                    recovered: false,
                                });
                            }
                            Err(e) => {
                                let mut error_handled = false;
                                handle_processor_error(
                                    e,
                                    stmt,
                                    &text,
                                    &mut output,
                                    &mut diagnostics,
                                    &mut has_unrecovered_error,
                                    &mut error_handled,
                                );
                                if has_unrecovered_error {
                                    has_processor_error = true;
                                    halted = true;
                                }
                                if error_handled {
                                    // Fallback already pushed to output
                                    finished = true;
                                    break;
                                }
                            }
                        },
                        StatementAction::CopyTo(dest) => {
                            output.push_str(&text);
                            pending_relocations.push(PendingRelocation {
                                source_text: text.clone(),
                                destination: Destination {
                                    kind: dest.kind,
                                    pattern: dest.pattern.clone(),
                                },
                            });
                            finished = true;
                            break;
                        }
                        StatementAction::MoveTo(dest) => {
                            pending_relocations.push(PendingRelocation {
                                source_text: text.clone(),
                                destination: Destination {
                                    kind: dest.kind,
                                    pattern: dest.pattern.clone(),
                                },
                            });
                            finished = true;
                            break;
                        }
                        StatementAction::Warn => {
                            stderr_lines.push(text.clone());
                            output.push_str(&text);
                            finished = true;
                            break;
                        }
                        StatementAction::Fail => {
                            stderr_lines.push(text.clone());
                            halted_by_fail = true;
                            halted = true;
                            finished = true;
                            break;
                        }
                        StatementAction::DebugCount => {
                            *debug_counts.entry(*stmt_id).or_insert(0) += 1;
                            output.push_str(&text);
                            finished = true;
                            break;
                        }
                        StatementAction::DebugPrint => {
                            stderr_lines.push(text.clone());
                            output.push_str(&text);
                            finished = true;
                            break;
                        }
                    }
                }

                if !finished && !halted {
                    output.push_str(&text);
                }
            }
        }
    }

    // Apply pending copy/move relocations to the output.
    if !pending_relocations.is_empty() && !has_unrecovered_error {
        output = apply_relocations(&output, &pending_relocations);
    }

    // Emit debug:count diagnostics for each statement that used DebugCount.
    for stmt in &script.statements {
        if let Some(&count) = debug_counts.get(&stmt.id) {
            let noun = if count == 1 { "match" } else { "matches" };
            diagnostics.push(Diagnostic {
                level: DiagnosticLevel::Debug,
                message: format!("{count} {noun}"),
                span: stmt.selector_span,
                selector_text: stmt.selector_text.clone(),
                recovered: false,
            });
        }
    }

    // Discard output on unrecovered processor error — the script failed
    // mid-processing, so partial output should not be emitted. No-match
    // errors preserve output since the fragment walk completed normally.
    if has_processor_error {
        output.clear();
    }

    ExecuteResult {
        output,
        diagnostics,
        stderr_lines,
        halted_by_fail,
    }
}

/// Look up the `OnError` mode for a statement's selector.
fn get_on_error(stmt: &crate::compile::Statement, script: &Script) -> OnError {
    script
        .selectors
        .get(stmt.selector.value())
        .map(|entry| match entry {
            crate::compile::RegistryEntry::Simple(s) => s.on_error,
            crate::compile::RegistryEntry::Compound(c) => c.on_error,
        })
        .unwrap_or(OnError::Fail)
}

/// Check if a selector still matches the (possibly transformed) text.
///
/// Used for re-fragmentation: after one processor has transformed the
/// text, we only chain the next statement's processor if its selector
/// pattern still matches a line in the new text.
fn selector_still_matches(
    registry: &[RegistryEntry],
    sel_id: crate::SelectorId,
    text: &str,
) -> bool {
    let Some(entry) = registry.get(sel_id.value()) else {
        return false;
    };
    match entry {
        RegistryEntry::Simple(sel) => {
            let pattern = match &sel.op {
                SelectorOp::At { pattern, .. } => pattern,
                SelectorOp::After { pattern } => pattern,
                SelectorOp::Before { pattern } => pattern,
                SelectorOp::From { pattern } => pattern,
                SelectorOp::To { pattern } => pattern,
            };
            // Check if any line in the text still matches
            text.lines()
                .any(|line| fragment::pattern_matches(pattern, line))
        }
        RegistryEntry::Compound(compound) => {
            // All steps must still match for the compound to hold
            compound
                .steps
                .iter()
                .all(|step_id| selector_still_matches(registry, *step_id, text))
        }
    }
}

/// Execute a fallback when the primary selector matched nothing.
///
/// Returns `Some(output)` if the fallback succeeds, `None` if all
/// fallback branches are exhausted. Diagnostics for failed branches
/// are pushed into `diagnostics`.
fn execute_no_match_fallback(
    fallback: &CompiledFallback,
    buffer: &Buffer,
    script: &Script,
    extract: bool,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<String> {
    match fallback {
        CompiledFallback::Chain {
            processor,
            span,
            text,
        } => match processor.execute(buffer.content()) {
            Ok(result) => Some(result),
            Err(e) => {
                diagnostics.push(Diagnostic {
                    level: DiagnosticLevel::Error,
                    message: format_processor_error(&e),
                    span: *span,
                    selector_text: text.clone(),
                    recovered: false,
                });
                None
            }
        },
        CompiledFallback::SelectAction {
            selector,
            action,
            selector_span,
            selector_text,
            processor_span,
            processor_text,
            fallback: nested_fb,
        } => {
            // Re-fragment the buffer with the fallback's selector.
            let dummy_id = StatementId::new(0);
            let requests = vec![(dummy_id, *selector)];
            let fb_fragments = fragment::fragment(buffer, &requests, &script.selectors);

            let matched = fb_fragments
                .iter()
                .any(|f| matches!(f, Fragment::Selected { .. }));

            if matched {
                let mut fb_output = String::new();
                for frag in &fb_fragments {
                    match frag {
                        Fragment::Passthrough(content) => {
                            if !extract {
                                fb_output.push_str(&resolve_content(content, buffer));
                            }
                        }
                        Fragment::Selected { content, .. } => {
                            let frag_text = resolve_content(content, buffer);
                            match action {
                                StatementAction::Process(proc) => match proc.execute(&frag_text) {
                                    Ok(result) => fb_output.push_str(&result),
                                    Err(e) => {
                                        diagnostics.push(Diagnostic {
                                            level: DiagnosticLevel::Error,
                                            message: format_processor_error(&e),
                                            span: *processor_span,
                                            selector_text: processor_text.clone(),
                                            recovered: false,
                                        });
                                        return None;
                                    }
                                },
                                StatementAction::CopyTo(_) | StatementAction::MoveTo(_) => {
                                    fb_output.push_str(&frag_text);
                                }
                                StatementAction::Warn
                                | StatementAction::Fail
                                | StatementAction::DebugCount
                                | StatementAction::DebugPrint => {
                                    fb_output.push_str(&frag_text);
                                }
                            }
                        }
                    }
                }
                Some(fb_output)
            } else if let Some(nested) = nested_fb {
                execute_no_match_fallback(nested, buffer, script, extract, diagnostics)
            } else {
                diagnostics.push(Diagnostic {
                    level: DiagnosticLevel::Error,
                    message: "no lines matched".into(),
                    span: *selector_span,
                    selector_text: selector_text.clone(),
                    recovered: false,
                });
                None
            }
        }
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
        match fb {
            CompiledFallback::Chain {
                processor,
                span,
                text: fb_text_str,
            } => match processor.execute(text) {
                Ok(result) => {
                    output.push_str(&result);
                    recovered = true;
                }
                Err(fb_err) => {
                    diag_span = *span;
                    diag_text = fb_text_str.clone();
                    diag_msg = format_processor_error(&fb_err);
                }
            },
            CompiledFallback::SelectAction {
                action,
                processor_span,
                processor_text: fb_proc_text,
                ..
            } => match action {
                StatementAction::Process(proc) => match proc.execute(text) {
                    Ok(result) => {
                        output.push_str(&result);
                        recovered = true;
                    }
                    Err(fb_err) => {
                        diag_span = *processor_span;
                        diag_text = fb_proc_text.clone();
                        diag_msg = format_processor_error(&fb_err);
                    }
                },
                StatementAction::CopyTo(_) | StatementAction::MoveTo(_) => {
                    output.push_str(text);
                    recovered = true;
                }
                StatementAction::Warn
                | StatementAction::Fail
                | StatementAction::DebugCount
                | StatementAction::DebugPrint => {
                    output.push_str(text);
                    recovered = true;
                }
            },
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
        PatternMatcher::NeverMatch => false,
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
        ProcessorError::FileEmptyRegion { .. } => "qed:file() ignored for empty region".into(),
    }
}
