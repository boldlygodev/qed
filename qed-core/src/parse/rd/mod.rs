mod cursor;
mod parser;

use super::error::{ParseError, ParseResult};

/// Parse an nth expression from source text using recursive descent.
pub(super) fn parse_nth_expr(source: &str) -> Result<ParseResult, Vec<ParseError>> {
    parser::parse_nth_expr(source)
}
