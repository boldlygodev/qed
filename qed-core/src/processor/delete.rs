//! The `qed:delete()` processor — removes selected text entirely.

use super::{Processor, ProcessorError};

/// Deletes the selected text by replacing it with an empty string.
#[derive(Debug)]
pub(crate) struct DeleteProcessor;

impl Processor for DeleteProcessor {
    fn execute(&self, _input: &str) -> Result<String, ProcessorError> {
        Ok(String::new())
    }
}
