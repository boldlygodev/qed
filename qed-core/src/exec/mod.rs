pub(crate) mod engine;
mod fragment;

use crate::{SelectorId, StatementId};

// ── Line ranges ─────────────────────────────────────────────────────

/// Half-open range of line indices within a `Buffer`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LineRange {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

// ── Fragment model ──────────────────────────────────────────────────

/// The content backing a fragment — either a reference into the original
/// buffer (via line range) or owned text produced by a processor.
#[derive(Debug, Clone)]
pub(crate) enum FragmentContent {
    Borrowed(LineRange),
    Owned(String),
}

/// A fragment is either passthrough (unselected) or selected by one or
/// more statement/selector pairs.
#[derive(Debug, Clone)]
pub(crate) enum Fragment {
    Passthrough(FragmentContent),
    Selected {
        content: FragmentContent,
        tags: Vec<(StatementId, SelectorId)>,
    },
}

/// An ordered sequence of fragments representing the current state of
/// the buffer during execution.
pub(crate) type FragmentList = Vec<Fragment>;

// ── Buffer ──────────────────────────────────────────────────────────

/// Immutable source buffer with precomputed line offsets for O(1) slicing.
#[derive(Debug, Clone)]
pub(crate) struct Buffer {
    content: String,
    line_offsets: Vec<usize>,
}

impl Buffer {
    /// Build a buffer from raw content, scanning for line boundaries.
    ///
    /// A trailing `\n` is a line terminator, not a new empty line.
    pub(crate) fn new(content: String) -> Self {
        let mut line_offsets = Vec::new();

        if !content.is_empty() {
            line_offsets.push(0);
            for (i, byte) in content.bytes().enumerate() {
                if byte == b'\n' && i + 1 < content.len() {
                    line_offsets.push(i + 1);
                }
            }
        }

        Self {
            content,
            line_offsets,
        }
    }

    /// Number of lines in the buffer.
    pub(crate) fn line_count(&self) -> usize {
        self.line_offsets.len()
    }

    /// Text of a single line (including its trailing newline, if any).
    pub(crate) fn line(&self, idx: usize) -> &str {
        self.slice(LineRange { start: idx, end: idx + 1 })
    }

    /// Extract the text for a half-open line range.
    pub(crate) fn slice(&self, range: LineRange) -> &str {
        let start_byte = self.line_offsets[range.start];
        let end_byte = if range.end < self.line_offsets.len() {
            self.line_offsets[range.end]
        } else {
            self.content.len()
        };
        &self.content[start_byte..end_byte]
    }
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_empty() {
        let buf = Buffer::new(String::new());
        assert_eq!(buf.line_count(), 0);
        assert!(buf.line_offsets.is_empty());
    }

    #[test]
    fn new_single_line_no_newline() {
        let buf = Buffer::new("hello".into());
        assert_eq!(buf.line_count(), 1);
        assert_eq!(buf.line_offsets, vec![0]);
    }

    #[test]
    fn new_single_line_with_newline() {
        let buf = Buffer::new("hello\n".into());
        assert_eq!(buf.line_count(), 1);
        assert_eq!(buf.line_offsets, vec![0]);
    }

    #[test]
    fn new_multiple_lines() {
        let buf = Buffer::new("aaa\nbb\nccccc\n".into());
        assert_eq!(buf.line_count(), 3);
        assert_eq!(buf.line_offsets, vec![0, 4, 7]);
    }

    #[test]
    fn new_no_trailing_newline() {
        let buf = Buffer::new("aaa\nbb".into());
        assert_eq!(buf.line_count(), 2);
        assert_eq!(buf.line_offsets, vec![0, 4]);
    }

    #[test]
    fn slice_single_line() {
        let buf = Buffer::new("aaa\nbb\nccccc\n".into());
        assert_eq!(buf.slice(LineRange { start: 0, end: 1 }), "aaa\n");
    }

    #[test]
    fn slice_middle_line() {
        let buf = Buffer::new("aaa\nbb\nccccc\n".into());
        assert_eq!(buf.slice(LineRange { start: 1, end: 2 }), "bb\n");
    }

    #[test]
    fn slice_last_line_no_trailing() {
        let buf = Buffer::new("aaa\nbb".into());
        assert_eq!(buf.slice(LineRange { start: 1, end: 2 }), "bb");
    }

    #[test]
    fn slice_multiple_lines() {
        let buf = Buffer::new("aaa\nbb\nccccc\n".into());
        assert_eq!(buf.slice(LineRange { start: 0, end: 2 }), "aaa\nbb\n");
    }

    #[test]
    fn slice_full_range() {
        let content = "aaa\nbb\nccccc\n";
        let buf = Buffer::new(content.into());
        assert_eq!(buf.slice(LineRange { start: 0, end: buf.line_count() }), content);
    }
}
