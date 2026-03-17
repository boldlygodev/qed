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

    /// Parse a double-quoted string literal, consuming the opening and closing `"`.
    /// Returns the inner content with basic escape handling (`\"`, `\\`).
    /// Assumes the cursor is positioned at the opening `"`.
    pub(super) fn eat_string_literal(&mut self) -> Option<String> {
        if self.peek() != Some(b'"') {
            return None;
        }
        self.advance(); // consume opening "

        let mut value = String::new();
        loop {
            match self.advance() {
                None => return None, // unterminated string
                Some(b'"') => return Some(value),
                Some(b'\\') => match self.advance() {
                    Some(b'"') => value.push('"'),
                    Some(b'\\') => value.push('\\'),
                    Some(b'n') => value.push('\n'),
                    Some(b't') => value.push('\t'),
                    Some(ch) => {
                        value.push('\\');
                        value.push(ch as char);
                    }
                    None => return None,
                },
                Some(ch) => value.push(ch as char),
            }
        }
    }

    /// Try to consume a specific ASCII keyword. Returns true if matched.
    pub(super) fn eat_keyword(&mut self, keyword: &str) -> bool {
        let remaining = self.remaining();
        if remaining.starts_with(keyword) {
            // Ensure the keyword isn't a prefix of a longer identifier
            let next = remaining.as_bytes().get(keyword.len());
            if next.is_none() || !next.is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_') {
                self.pos += keyword.len();
                return true;
            }
        }
        false
    }
}
