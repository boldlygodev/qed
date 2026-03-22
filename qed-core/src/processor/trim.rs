//! The `qed:trim()` processor — strips leading/trailing whitespace per line.

use super::{Processor, ProcessorError, map_lines};

/// Strips leading and trailing whitespace from each line.
#[derive(Debug)]
pub(crate) struct TrimProcessor;

impl Processor for TrimProcessor {
    fn execute(&self, input: &str) -> Result<String, ProcessorError> {
        Ok(map_lines(input, |line| line.trim().to_owned()))
    }
}
