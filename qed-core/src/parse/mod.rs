pub(crate) mod ast;
pub(crate) mod error;
mod rd;

use error::{ParseError, ParseResult};

/// Parse an nth expression string into an `NthExpr` AST node.
///
/// Returns `Ok(ParseResult)` on success (possibly with warnings),
/// or `Err(Vec<ParseError>)` on hard error(s).
pub(crate) fn parse_nth_expr(source: &str) -> Result<ParseResult, Vec<ParseError>> {
    rd::parse_nth_expr(source)
}
