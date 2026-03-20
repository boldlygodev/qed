//! Processor chain — composes multiple processors by piping output left to right.

use super::{Processor, ProcessorError};

/// Chains multiple processors, feeding each one's output into the next.
#[derive(Debug)]
pub(crate) struct ChainProcessor {
    pub(crate) steps: Vec<Box<dyn Processor>>,
}

impl Processor for ChainProcessor {
    fn execute(&self, input: &str) -> Result<String, ProcessorError> {
        let mut current = input.to_owned();
        for step in &self.steps {
            current = step.execute(&current)?;
            // Short-circuit: empty output means deletion — nothing left
            // to transform.
            if current.is_empty() {
                return Ok(current);
            }
        }
        Ok(current)
    }
}
