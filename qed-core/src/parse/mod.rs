//! Parse phase — source text → AST.
//!
//! Transforms a raw qed script string into a [`ast::Program`] (the abstract
//! syntax tree). Two parser backends exist behind feature flags:
//!
//! - **`parser-rd`** (default) — hand-written recursive descent in the `rd`
//!   module. Chosen for precise error recovery and zero external dependencies.
//! - **`parser-chumsky`** — combinator parser under evaluation.
//!
//! Feature-flag switching is isolated to this module; no `#[cfg(feature)]`
//! appears anywhere else in the crate.

pub(crate) mod ast;
pub(crate) mod error;
mod rd;

use ast::Program;
use error::{ParseError, ParseResult};

// @spec PCOMP-008
/// Parse an nth expression string into an `NthExpr` AST node.
///
/// Returns `Ok(ParseResult)` on success (possibly with warnings),
/// or `Err(Vec<ParseError>)` on hard error(s).
pub(crate) fn parse_nth_expr(source: &str) -> Result<ParseResult, Vec<ParseError>> {
    rd::parse_nth_expr(source)
}

// @spec PCOMP-008
/// Parse a complete qed program from source text.
pub(crate) fn parse_program(source: &str) -> Result<Program, Vec<ParseError>> {
    rd::parse_program(source)
}
