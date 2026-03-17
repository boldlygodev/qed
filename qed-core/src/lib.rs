// TODO: remove once modules have consumers
#![allow(dead_code)]

pub mod span;
pub mod error;
pub(crate) mod parse;
pub(crate) mod compile;
pub(crate) mod exec;
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
