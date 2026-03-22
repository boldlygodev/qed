//! The `qed:dedent()` processor — removes common leading whitespace.

use super::{Processor, ProcessorError};

/// Removes the common leading whitespace prefix from all lines.
///
/// Computes the shortest run of leading whitespace across all non-empty
/// lines, then strips that many characters from the front of every line.
#[derive(Debug)]
pub(crate) struct DedentProcessor;

impl Processor for DedentProcessor {
    fn execute(&self, input: &str) -> Result<String, ProcessorError> {
        let has_trailing_newline = input.ends_with('\n');
        let content = if has_trailing_newline {
            &input[..input.len() - 1]
        } else {
            input
        };

        let lines: Vec<&str> = content.split('\n').collect();

        // Find the minimum leading whitespace among non-empty lines.
        let min_indent = lines
            .iter()
            .filter(|line| !line.is_empty())
            .map(|line| line.len() - line.trim_start().len())
            .min()
            .unwrap_or(0);

        let mut result: String = lines
            .iter()
            .map(|line| {
                if line.len() >= min_indent {
                    &line[min_indent..]
                } else {
                    line
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        if has_trailing_newline {
            result.push('\n');
        }
        Ok(result)
    }
}
