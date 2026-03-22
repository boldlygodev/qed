//! Parse-time error and warning types.
//!
//! The parser collects errors as it goes rather than aborting on the first
//! one, enabling multi-error reporting. All variants carry a [`Span`] for
//! source-location diagnostics. Warnings (like [`NthWarning`]) are
//! non-fatal and attached to the successful [`ParseResult`].
//!
//! [`NthWarning`]: ParseError::NthWarning

use crate::span::Span;

use super::ast::NthExpr;

/// Errors and warnings produced during parsing.
#[derive(Debug, Clone)]
pub(crate) enum ParseError {
    /// The parser encountered a token that does not fit the expected grammar
    /// at this position.
    UnexpectedToken {
        expected: String,
        found: String,
        span: Span,
    },
    /// Input ended before the parser found the expected construct (e.g.,
    /// a closing `"` or `)` was never reached).
    UnexpectedEof { expected: String, span: Span },
    /// An nth expression is syntactically parseable but semantically invalid
    /// (e.g., `0n`, cross-sign range bounds, zero range endpoint).
    InvalidNthExpr { reason: String, span: Span },
    /// A non-fatal advisory about an nth expression (e.g., leading `+`
    /// ignored, zero term ignored). Attached to `ParseResult::warnings`.
    NthWarning { reason: String, span: Span },
}

impl ParseError {
    pub(crate) fn span(&self) -> Span {
        match self {
            ParseError::UnexpectedToken { span, .. }
            | ParseError::UnexpectedEof { span, .. }
            | ParseError::InvalidNthExpr { span, .. }
            | ParseError::NthWarning { span, .. } => *span,
        }
    }

    pub(crate) fn is_warning(&self) -> bool {
        matches!(self, ParseError::NthWarning { .. })
    }
}

/// Successful parse result — the parsed expression plus any warnings emitted.
#[derive(Debug, Clone)]
pub(crate) struct ParseResult {
    pub(crate) expr: NthExpr,
    pub(crate) warnings: Vec<ParseError>,
}
