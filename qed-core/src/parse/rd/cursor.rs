use crate::span::Span;

/// A byte-offset cursor over a `&str` for hand-written parsing.
pub(super) struct Cursor<'src> {
    source: &'src str,
    pos: usize,
}

impl<'src> Cursor<'src> {
    pub(super) fn new(source: &'src str) -> Self {
        Self { source, pos: 0 }
    }

    /// Current byte offset.
    pub(super) fn pos(&self) -> usize {
        self.pos
    }

    /// True when all input has been consumed.
    pub(super) fn is_eof(&self) -> bool {
        self.pos >= self.source.len()
    }

    /// Peek at the next byte without consuming it.
    pub(super) fn peek(&self) -> Option<u8> {
        self.source.as_bytes().get(self.pos).copied()
    }

    /// Advance the cursor by one byte and return it.
    pub(super) fn advance(&mut self) -> Option<u8> {
        let byte = self.peek()?;
        self.pos += 1;
        Some(byte)
    }

    /// Consume a specific byte. Returns `true` if matched and consumed.
    pub(super) fn eat_char(&mut self, expected: u8) -> bool {
        if self.peek() == Some(expected) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    /// Consume whitespace characters (space and tab).
    pub(super) fn eat_whitespace(&mut self) {
        while let Some(b' ' | b'\t') = self.peek() {
            self.pos += 1;
        }
    }

    /// Build a `Span` from a saved start position to the current position.
    pub(super) fn span_from(&self, start: usize) -> Span {
        Span {
            start,
            end: self.pos,
        }
    }

    /// Slice of source from `start` to the current position.
    pub(super) fn slice_from(&self, start: usize) -> &'src str {
        &self.source[start..self.pos]
    }

    /// The full remaining unparsed source (for error messages).
    pub(super) fn remaining(&self) -> &'src str {
        &self.source[self.pos..]
    }
}
