//! The `qed:replace()` processor — find-and-replace within selected content.
//!
//! Supports three replacement forms:
//!
//! - **Literal → Literal**: `qed:replace("old", "new")` — substring replacement.
//! - **Regex → Template**: `qed:replace(/pattern/, /template/)` — regex match
//!   with capture group expansion (`$1`, `$name`).
//! - **Any → Pipeline**: `qed:replace("match", processor)` — run a processor
//!   chain on each matched substring and splice its output back.
//!
//! All forms replace every occurrence within the selected text.

use super::{Processor, ProcessorError};

/// How to find the text to replace.
#[derive(Debug)]
pub(crate) enum ReplaceSearch {
    /// Match a literal substring.
    Literal(String),
    /// Match a regex pattern.
    Regex(regex::Regex),
}

/// What to replace matched text with.
#[derive(Debug)]
pub(crate) enum ReplaceWith {
    /// Literal string replacement — no special character interpretation.
    Literal(String),
    /// Regex template — `$1`, `$name` expand to capture groups.
    Template(String),
    /// Run a processor chain on each match, splice output back.
    Pipeline(Box<dyn Processor>),
}

/// Find-and-replace processor.
#[derive(Debug)]
pub(crate) struct ReplaceProcessor {
    pub(crate) search: ReplaceSearch,
    pub(crate) replacement: ReplaceWith,
}

impl Processor for ReplaceProcessor {
    fn execute(&self, input: &str) -> Result<String, ProcessorError> {
        match (&self.search, &self.replacement) {
            (ReplaceSearch::Literal(pat), ReplaceWith::Literal(rep)) => {
                Ok(input.replace(pat.as_str(), rep.as_str()))
            }

            (ReplaceSearch::Regex(re), ReplaceWith::Literal(rep)) => Ok(re
                .replace_all(input, regex::NoExpand(rep.as_str()))
                .into_owned()),

            (ReplaceSearch::Regex(re), ReplaceWith::Template(tmpl)) => {
                Ok(re.replace_all(input, tmpl.as_str()).into_owned())
            }

            (ReplaceSearch::Literal(pat), ReplaceWith::Pipeline(proc)) => {
                replace_with_pipeline_literal(input, pat, proc.as_ref())
            }

            (ReplaceSearch::Regex(re), ReplaceWith::Pipeline(proc)) => {
                replace_with_pipeline_regex(input, re, proc.as_ref())
            }

            // (Literal, Template) is rejected at compile time — cannot reach here.
            (ReplaceSearch::Literal(_), ReplaceWith::Template(_)) => {
                Err(ProcessorError::ProcessorFailed {
                    processor: "qed:replace".into(),
                    reason: "regex template replacement requires a regex search pattern".into(),
                })
            }
        }
    }
}

/// Replace all literal occurrences by running a pipeline processor on each match.
fn replace_with_pipeline_literal(
    input: &str,
    pattern: &str,
    processor: &dyn Processor,
) -> Result<String, ProcessorError> {
    let mut result = String::new();
    let mut last_end = 0;

    for (start, matched) in input.match_indices(pattern) {
        result.push_str(&input[last_end..start]);
        let mut replacement = processor.execute(matched)?;
        strip_trailing_newline_if_needed(matched, &mut replacement);
        result.push_str(&replacement);
        last_end = start + matched.len();
    }

    result.push_str(&input[last_end..]);
    Ok(result)
}

/// Replace all regex matches by running a pipeline processor on each match.
fn replace_with_pipeline_regex(
    input: &str,
    re: &regex::Regex,
    processor: &dyn Processor,
) -> Result<String, ProcessorError> {
    let mut result = String::new();
    let mut last_end = 0;

    for mat in re.find_iter(input) {
        result.push_str(&input[last_end..mat.start()]);
        let matched = mat.as_str();
        let mut replacement = processor.execute(matched)?;
        strip_trailing_newline_if_needed(matched, &mut replacement);
        result.push_str(&replacement);
        last_end = mat.end();
    }

    result.push_str(&input[last_end..]);
    Ok(result)
}

/// Strip a trailing newline from the pipeline output when the matched text
/// does not end with one. Prevents external commands (which typically append
/// `\n`) from injecting newlines mid-line.
fn strip_trailing_newline_if_needed(matched: &str, replacement: &mut String) {
    if !matched.ends_with('\n') && replacement.ends_with('\n') {
        replacement.pop();
    }
}
