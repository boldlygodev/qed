//! The `qed:substring()` processor — narrows each line to the matched span.

use super::{Processor, ProcessorError, map_lines};

/// How to find the substring to extract.
#[derive(Debug)]
pub(crate) enum SubstringSearch {
    /// Match a literal substring.
    Literal(String),
    /// Match a regex pattern.
    Regex(regex::Regex),
}

/// Narrows each line to the first span matching the search pattern.
///
/// Lines that do not contain a match are passed through unchanged.
#[derive(Debug)]
pub(crate) struct SubstringProcessor {
    pub(crate) search: SubstringSearch,
}

impl Processor for SubstringProcessor {
    fn execute(&self, input: &str) -> Result<String, ProcessorError> {
        Ok(map_lines(input, |line| match &self.search {
            SubstringSearch::Literal(s) => {
                if let Some(pos) = line.find(s.as_str()) {
                    line[pos..pos + s.len()].to_owned()
                } else {
                    line.to_owned()
                }
            }
            SubstringSearch::Regex(re) => {
                if let Some(m) = re.find(line) {
                    m.as_str().to_owned()
                } else {
                    line.to_owned()
                }
            }
        }))
    }
}
