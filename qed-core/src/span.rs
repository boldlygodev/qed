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

/// Convert a byte offset to a `(line, column)` pair (both 1-based).
pub fn offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (i, ch) in source.bytes().enumerate() {
        if i >= offset {
            break;
        }
        if ch == b'\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

/// Format a span as `line:col_start-col_end` for diagnostics.
/// The end column is inclusive (last character of the span).
pub fn format_span(source: &str, span: Span) -> String {
    let (line, col_start) = offset_to_line_col(source, span.start);
    if span.end <= span.start {
        // Zero-width span — single point
        return format!("{line}:{col_start}");
    }
    // End is exclusive in Span, so subtract 1 for inclusive display
    let (_, col_end) = offset_to_line_col(source, span.end - 1);
    if col_start == col_end {
        format!("{line}:{col_start}")
    } else {
        format!("{line}:{col_start}-{col_end}")
    }
}
