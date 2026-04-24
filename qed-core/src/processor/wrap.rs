//! The `qed:wrap()` processor — word-wraps lines at a column width.

use super::{Processor, ProcessorError};

// @spec TXFM-040, TXFM-041
/// Word-wraps each line at the specified column width.
///
/// Breaks at word boundaries (spaces). Words longer than `width` are
/// placed on their own line without breaking.
#[derive(Debug)]
pub(crate) struct WrapProcessor {
    pub(crate) width: usize,
}

impl Processor for WrapProcessor {
    fn execute(&self, input: &str) -> Result<String, ProcessorError> {
        let has_trailing_newline = input.ends_with('\n');
        let content = if has_trailing_newline {
            &input[..input.len() - 1]
        } else {
            input
        };

        let mut result_lines: Vec<String> = Vec::new();

        for line in content.split('\n') {
            if line.is_empty() {
                result_lines.push(String::new());
                continue;
            }
            wrap_line(line, self.width, &mut result_lines);
        }

        let mut result = result_lines.join("\n");
        if has_trailing_newline {
            result.push('\n');
        }
        Ok(result)
    }
}

fn wrap_line(line: &str, width: usize, out: &mut Vec<String>) {
    let words: Vec<&str> = line.split(' ').collect();
    let mut current = String::new();

    for word in words {
        if current.is_empty() {
            current.push_str(word);
        } else if current.len() + 1 + word.len() <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            out.push(current);
            current = word.to_owned();
        }
    }
    if !current.is_empty() {
        out.push(current);
    }
}
