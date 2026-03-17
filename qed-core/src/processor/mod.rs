pub(crate) mod delete;

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
    NoMatch {
        selector_id: SelectorId,
    },
    ProcessorFailed {
        processor: String,
        reason: String,
    },
    ExternalFailed {
        command: String,
        exit_code: Option<i32>,
        stderr: String,
    },
}
