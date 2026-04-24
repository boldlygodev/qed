//! The `qed:prefix()` processor — prepends text to each selected line.

use super::{Processor, ProcessorError, map_lines};

// @spec TXFM-042
/// Prepends the configured text to the beginning of each line.
#[derive(Debug)]
pub(crate) struct PrefixProcessor {
    pub(crate) text: String,
}

impl Processor for PrefixProcessor {
    fn execute(&self, input: &str) -> Result<String, ProcessorError> {
        Ok(map_lines(input, |line| format!("{}{}", self.text, line)))
    }
}
