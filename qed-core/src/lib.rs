//! Core library for **qed**, a modern CLI stream editor.
//!
//! The fundamental primitive is `selector | processor` — select a region of
//! input lines, then pipe that region through a transformation.
//!
//! # Pipeline
//!
//! Every invocation flows through three stages:
//!
//! ```text
//!  source text
//!       │
//!       ▼
//!  ┌─────────┐
//!  │  parse  │   source text → Program (AST)
//!  └────┬────┘
//!       │
//!       ▼
//!  ┌─────────┐
//!  │ compile │   Program → Script (IR: compiled selectors + processors)
//!  └────┬────┘
//!       │
//!       ▼
//!  ┌─────────┐
//!  │ execute │   Script + Buffer → output string
//!  └─────────┘
//! ```
//!
//! # Crate organization
//!
//! | Module      | Responsibility                                          |
//! |-------------|---------------------------------------------------------|
//! | `parse`     | Source text → `Program` (AST) via recursive descent     |
//! | `compile`   | `Program` → `Script` (compiled IR with selector ops)    |
//! | `exec`      | `Script` + input `Buffer` → output string               |
//! | `processor` | Trait object interface and built-in processor impls      |
//! | [`span`]    | Byte-offset source spans for diagnostics                |
//! | [`error`]   | Compile-time error types (accumulator pattern)          |
//!
//! # Public API
//!
//! The only public entry point is [`run`], which takes a script string and
//! input text and returns the transformed output. All internal types use
//! `pub(crate)` visibility.

// TODO: remove once modules have consumers
#![allow(dead_code)]

pub mod span;
pub mod error;
pub(crate) mod parse;
pub(crate) mod compile;
pub(crate) mod exec;
pub(crate) mod processor;

/// Uniquely identifies a statement within a compiled `Script`.
///
/// Newtype over `usize` to prevent accidentally passing a raw index where a
/// typed ID is expected. Statements execute in definition order; the ID
/// reflects that order (0, 1, 2, ...).
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

/// Uniquely identifies a selector within a compiled `Script`.
///
/// Global scope — every selector receives a unique ID regardless of which
/// statement it belongs to. Compound selectors consume multiple IDs: one per
/// step plus one for the compound itself. Used as an index into
/// `Script::selectors`.
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

/// Run a qed script against input text, returning the transformed output.
///
/// This is the primary public API for the library.
pub fn run(script_source: &str, input: &str) -> Result<String, String> {
    let program = parse::parse_program(script_source).map_err(|errors| {
        errors
            .iter()
            .map(|e| format!("{e:?}"))
            .collect::<Vec<_>>()
            .join("\n")
    })?;

    let script = compile::compile(&program).map_err(|errors| {
        errors
            .iter()
            .map(|e| format!("{e:?}"))
            .collect::<Vec<_>>()
            .join("\n")
    })?;

    let buffer = exec::Buffer::new(input.to_owned());
    let output = exec::engine::execute(&script, &buffer);

    Ok(output)
}
