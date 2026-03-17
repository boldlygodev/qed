use crate::span::Span;

use super::ast::NthExpr;

/// Errors and warnings produced during parsing.
#[derive(Debug, Clone)]
pub(crate) enum ParseError {
    UnexpectedToken {
        expected: String,
        found: String,
        span: Span,
    },
    UnexpectedEof {
        expected: String,
        span: Span,
    },
    InvalidNthExpr {
        reason: String,
        span: Span,
    },
    NthWarning {
        reason: String,
        span: Span,
    },
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
