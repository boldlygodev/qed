//! The `qed:upper()` processor — converts selected text to uppercase.

use super::{Processor, ProcessorError};

// @spec TXFM-020
/// Converts all characters in the selected text to uppercase.
#[derive(Debug)]
pub(crate) struct UpperProcessor;

impl Processor for UpperProcessor {
    fn execute(&self, input: &str) -> Result<String, ProcessorError> {
        Ok(input.to_uppercase())
    }
}
