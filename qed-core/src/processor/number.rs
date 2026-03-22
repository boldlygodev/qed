//! The `qed:number()` processor — prefixes each line with its line number.

use super::{Processor, ProcessorError};

/// Prefixes each line with its line number.
///
/// `start` controls the first number (default 1).
/// `width` right-aligns numbers to at least N digits.
/// Format: `<number>: <line>`.
#[derive(Debug)]
pub(crate) struct NumberProcessor {
    pub(crate) start: i64,
    pub(crate) width: usize,
}

impl Processor for NumberProcessor {
    fn execute(&self, input: &str) -> Result<String, ProcessorError> {
        let has_trailing_newline = input.ends_with('\n');
        let content = if has_trailing_newline {
            &input[..input.len() - 1]
        } else {
            input
        };

        let lines: Vec<&str> = content.split('\n').collect();
        let max_num = self.start + lines.len() as i64 - 1;
        let actual_width = self.width.max(max_num.to_string().len());

        let mut result: String = lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let num = self.start + i as i64;
                format!("{:>width$}: {}", num, line, width = actual_width)
            })
            .collect::<Vec<_>>()
            .join("\n");

        if has_trailing_newline {
            result.push('\n');
        }
        Ok(result)
    }
}
