//! The `qed:lower()` processor — converts selected text to lowercase.

use super::{Processor, ProcessorError};

// @spec TXFM-021
/// Converts all characters in the selected text to lowercase.
#[derive(Debug)]
pub(crate) struct LowerProcessor;

impl Processor for LowerProcessor {
    fn execute(&self, input: &str) -> Result<String, ProcessorError> {
        Ok(input.to_lowercase())
    }
}
