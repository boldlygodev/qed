//! Source-location spans for error reporting and diagnostics.
//!
//! Every AST node is wrapped in [`Spanned<T>`], pairing it with the byte
//! range in the original source text where it was parsed. This allows the
//! compiler and diagnostic layer to point back to exact source positions
//! without retaining the source string itself.

/// A byte-offset range into source text. `start` is inclusive, `end` is exclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

/// An AST node paired with its source location.
#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}
