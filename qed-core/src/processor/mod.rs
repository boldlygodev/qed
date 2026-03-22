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
pub(crate) mod delete;
pub(crate) mod external;
pub(crate) mod lower;
pub(crate) mod prefix;
pub(crate) mod replace;
pub(crate) mod upper;

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
