//! Zero-copy byte-offset scanner for the recursive descent parser.
//!
//! [`Cursor`] tracks a position within a borrowed source string and provides
//! primitive operations (peek, advance, eat) that the parser builds on.
//! It never allocates or copies the source — all slices are `&'src str`
//! references back into the original input.

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

    /// Set the cursor position (for backtracking).
    pub(super) fn set_pos(&mut self, pos: usize) {
        self.pos = pos;
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

    /// Peek at a byte at the given offset from the current position without consuming.
    pub(super) fn peek_at(&self, offset: usize) -> Option<u8> {
        self.source.as_bytes().get(self.pos + offset).copied()
    }

    /// Parse a single-quoted string literal, consuming the opening and closing `'`.
    /// Returns the inner content with basic escape handling (`\'`, `\\`).
    /// Assumes the cursor is positioned at the opening `'`.
    pub(super) fn eat_single_quoted_string_literal(&mut self) -> Option<String> {
        if self.peek() != Some(b'\'') {
            return None;
        }
        self.advance(); // consume opening '

        let mut value = String::new();
        loop {
            match self.advance() {
                None => return None, // unterminated string
                Some(b'\'') => return Some(value),
                Some(b'\\') => match self.advance() {
                    Some(b'\'') => value.push('\''),
                    Some(b'\\') => value.push('\\'),
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

    /// Parse a regex literal `/pattern/`, consuming the opening and closing `/`.
    /// Returns the inner content with `\/` escape handling.
    /// Assumes the cursor is positioned at the opening `/`.
    pub(super) fn eat_regex_literal(&mut self) -> Option<String> {
        if self.peek() != Some(b'/') {
            return None;
        }
        self.advance(); // consume opening /

        let mut value = String::new();
        loop {
            match self.advance() {
                None => return None, // unterminated regex
                Some(b'/') => return Some(value),
                Some(b'\\') => match self.peek() {
                    Some(b'/') => {
                        self.advance();
                        value.push('/');
                    }
                    _ => {
                        // Keep the backslash — it's a regex escape
                        value.push('\\');
                    }
                },
                Some(ch) => value.push(ch as char),
            }
        }
    }

    /// Parse an identifier: `[a-zA-Z_][a-zA-Z0-9_]*`.
    /// Returns `None` if the next byte is not a valid identifier start.
    pub(super) fn eat_identifier(&mut self) -> Option<String> {
        let start = self.pos;
        match self.peek() {
            Some(b) if b.is_ascii_alphabetic() || b == b'_' => {
                self.advance();
            }
            _ => return None,
        }
        while let Some(b) = self.peek() {
            if b.is_ascii_alphanumeric() || b == b'_' {
                self.advance();
            } else {
                break;
            }
        }
        Some(self.slice_from(start).to_owned())
    }

    /// Parse an unquoted external processor argument.
    /// Consumes `[^ \t\n|\\;'"()]+` — stops at whitespace, pipe, backslash,
    /// semicolon, quotes, or parens.
    pub(super) fn eat_unquoted_arg(&mut self) -> Option<String> {
        let start = self.pos;
        while let Some(b) = self.peek() {
            match b {
                b' ' | b'\t' | b'\n' | b'|' | b'\\' | b';' | b'\'' | b'"' | b'(' | b')' => {
                    break
                }
                _ => {
                    self.advance();
                }
            }
        }
        if self.pos == start {
            None
        } else {
            Some(self.slice_from(start).to_owned())
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
