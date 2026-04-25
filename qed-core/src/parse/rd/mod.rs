//! Recursive descent parser for the qed language.
//!
//! Hand-written for precise error messages and straightforward recovery.
//! The parser operates on a byte-offset [`cursor::Cursor`] over the source
//! string, producing [`ast::Program`] on success or a list of
//! [`ParseError`]s on failure.

mod cursor;
mod parser;

use super::ast::Program;
use super::error::ParseError;

/// Parse a complete qed program from source text using recursive descent.
pub(super) fn parse_program(source: &str) -> Result<Program, Vec<ParseError>> {
    parser::parse_program(source)
}
