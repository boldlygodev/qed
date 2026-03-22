//! Processor trait and error types.
//!
//! A processor is a transformation applied to selected text. All processors
//! implement the [`Processor`] trait, which takes a `&str` input and returns
//! an owned `String` (or a [`ProcessorError`]). Processors are stored as
//! `Box<dyn Processor>` in compiled statements, enabling open-ended
//! extension via both built-in (`qed:delete`, `qed:replace`, ...) and
//! external (`!sort`, `!jq`, ...) implementations.
//!
//! Processor chains (piped with `|`) feed each processor's output into
//! the next, left to right.

pub(crate) mod chain;
pub(crate) mod dedent;
pub(crate) mod delete;
pub(crate) mod duplicate;
pub(crate) mod external;
pub(crate) mod indent;
pub(crate) mod lower;
pub(crate) mod number;
pub(crate) mod prefix;
pub(crate) mod random;
pub(crate) mod replace;
pub(crate) mod skip;
pub(crate) mod substring;
pub(crate) mod suffix;
pub(crate) mod timestamp;
pub(crate) mod trim;
pub(crate) mod upper;
pub(crate) mod uuid;
pub(crate) mod wrap;

use crate::SelectorId;

/// A processor transforms selected text, producing new output or an error.
///
/// `Debug` supertrait enables `Box<dyn Processor>` to implement `Debug`,
/// which lets `Statement` derive `Debug`.
pub(crate) trait Processor: std::fmt::Debug {
    fn execute(&self, input: &str) -> Result<String, ProcessorError>;
}

/// Errors that can occur during processor execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProcessorError {
    /// The selector matched no lines and the statement's `on_error` mode
    /// requires an error.
    NoMatch { selector_id: SelectorId },
    /// A built-in qed processor encountered an error during execution
    /// (e.g., a regex replacement with an invalid capture reference).
    ProcessorFailed { processor: String, reason: String },
    /// An external command exited with a non-zero status or could not
    /// be spawned.
    ExternalFailed {
        command: String,
        exit_code: Option<i32>,
        stderr: String,
    },
}

/// Apply a function to each line of the input, preserving trailing newline.
///
/// Splits on `\n`, maps each line through `f`, and rejoins with `\n`.
/// A trailing newline in the input is preserved in the output.
pub(crate) fn map_lines(input: &str, f: impl Fn(&str) -> String) -> String {
    let has_trailing_newline = input.ends_with('\n');
    let content = if has_trailing_newline {
        &input[..input.len() - 1]
    } else {
        input
    };
    let mut result: String = content.split('\n').map(&f).collect::<Vec<_>>().join("\n");
    if has_trailing_newline {
        result.push('\n');
    }
    result
}
