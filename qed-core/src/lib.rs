// TODO: remove once modules have consumers
#![allow(dead_code)]

pub mod span;
pub mod error;
pub(crate) mod parse;
pub(crate) mod compile;
mod exec;
pub(crate) mod processor;

/// Uniquely identifies a statement within a compiled script.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct StatementId(usize);

impl StatementId {
    pub(crate) fn new(id: usize) -> Self {
        Self(id)
    }

    pub(crate) fn value(self) -> usize {
        self.0
    }
}

/// Uniquely identifies a selector within a compiled script.
/// Global scope — every selector has a unique ID regardless of its parent statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct SelectorId(usize);

impl SelectorId {
    pub(crate) fn new(id: usize) -> Self {
        Self(id)
    }

    pub(crate) fn value(self) -> usize {
        self.0
    }
}
