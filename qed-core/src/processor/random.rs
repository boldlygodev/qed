//! Random string generation processor.
//!
//! `qed:random(length)` generates a random string of the specified length
//! from a configurable alphabet. The input is ignored entirely — output is
//! produced purely from parameters.

use rand::RngExt;

use super::{Processor, ProcessorError};

/// Generates a random string of `length` characters drawn from `alphabet`.
#[derive(Debug)]
pub(crate) struct RandomProcessor {
    pub(crate) length: usize,
    pub(crate) alphabet: String,
}

impl Processor for RandomProcessor {
    fn execute(&self, _input: &str) -> Result<String, ProcessorError> {
        let chars: Vec<char> = self.alphabet.chars().collect();
        if chars.is_empty() {
            return Err(ProcessorError::ProcessorFailed {
                processor: "qed:random".into(),
                reason: "alphabet is empty".into(),
            });
        }
        let mut rng = rand::rng();
        let result: String = (0..self.length)
            .map(|_| chars[rng.random_range(0..chars.len())])
            .collect();
        Ok(result)
    }
}
