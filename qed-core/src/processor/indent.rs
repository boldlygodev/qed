//! The `qed:indent()` processor — prepends indentation to each line.

use super::{Processor, ProcessorError, map_lines};

// @spec TXFM-031
/// Prepends `width` copies of `char` to each line.
#[derive(Debug)]
pub(crate) struct IndentProcessor {
    pub(crate) width: usize,
    pub(crate) indent_char: String,
}

impl Processor for IndentProcessor {
    fn execute(&self, input: &str) -> Result<String, ProcessorError> {
        let prefix = self.indent_char.repeat(self.width);
        Ok(map_lines(input, |line| format!("{}{}", prefix, line)))
    }
}
