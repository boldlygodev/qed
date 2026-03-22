//! The `qed:duplicate()` processor — emits the selected region twice.

use super::{Processor, ProcessorError};

/// Emits the selected region twice consecutively.
#[derive(Debug)]
pub(crate) struct DuplicateProcessor;

impl Processor for DuplicateProcessor {
    fn execute(&self, input: &str) -> Result<String, ProcessorError> {
        Ok(format!("{}{}", input, input))
    }
}
