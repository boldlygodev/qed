//! The `qed:prefix()` processor — prepends text to selected content.

use super::{Processor, ProcessorError};

/// Prepends the configured text to the beginning of the selected content.
#[derive(Debug)]
pub(crate) struct PrefixProcessor {
    pub(crate) text: String,
}

impl Processor for PrefixProcessor {
    fn execute(&self, input: &str) -> Result<String, ProcessorError> {
        Ok(format!("{}{}", self.text, input))
    }
}
