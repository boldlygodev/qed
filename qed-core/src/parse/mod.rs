//! Parse phase — source text → AST.
//!
//! Transforms a raw qed script string into a [`ast::Program`] (the abstract
//! syntax tree) using the hand-written recursive descent parser in the `rd`
//! module.
//! Chosen for precise error recovery and zero external dependencies.

pub(crate) mod ast;
pub(crate) mod error;
mod rd;

use ast::Program;
use error::ParseError;

// @spec PCOMP-008
/// Parse a complete qed program from source text.
pub(crate) fn parse_program(source: &str) -> Result<Program, Vec<ParseError>> {
    rd::parse_program(source)
}
