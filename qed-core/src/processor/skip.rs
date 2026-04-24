//! The `qed:skip()` processor — no-op passthrough.

use super::{Processor, ProcessorError};

// @spec TXFM-012
/// Returns the input unchanged. Useful with `--extract` to pass through
/// selected regions without transformation.
#[derive(Debug)]
pub(crate) struct SkipProcessor;

impl Processor for SkipProcessor {
    fn execute(&self, input: &str) -> Result<String, ProcessorError> {
        Ok(input.to_owned())
    }
}
