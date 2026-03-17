pub(crate) mod ast;
pub(crate) mod error;
mod rd;

use ast::Program;
use error::{ParseError, ParseResult};

/// Parse an nth expression string into an `NthExpr` AST node.
///
/// Returns `Ok(ParseResult)` on success (possibly with warnings),
/// or `Err(Vec<ParseError>)` on hard error(s).
pub(crate) fn parse_nth_expr(source: &str) -> Result<ParseResult, Vec<ParseError>> {
    rd::parse_nth_expr(source)
}

/// Parse a complete qed program from source text.
pub(crate) fn parse_program(source: &str) -> Result<Program, Vec<ParseError>> {
    rd::parse_program(source)
}
