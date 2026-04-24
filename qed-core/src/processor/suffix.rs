//! The `qed:suffix()` processor — appends text to each selected line.

use super::{Processor, ProcessorError, map_lines};

// @spec TXFM-043
/// Appends the configured text to the end of each line.
#[derive(Debug)]
pub(crate) struct SuffixProcessor {
    pub(crate) text: String,
}

impl Processor for SuffixProcessor {
    fn execute(&self, input: &str) -> Result<String, ProcessorError> {
        Ok(map_lines(input, |line| format!("{}{}", line, self.text)))
    }
}
