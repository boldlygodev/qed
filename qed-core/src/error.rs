use crate::span::Span;

/// Discriminates named symbols in the symbol table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Pattern,
    Alias,
}

/// Errors produced by the compilation pass.
/// All errors are collected before being returned to the caller.
#[derive(Debug, Clone)]
pub enum CompileError {
    UndefinedName { name: String, span: Span },
    WrongSymbolKind { name: String, expected: SymbolKind, found: SymbolKind, span: Span },
    InvalidRegex { pattern: String, reason: String, span: Span },
    InvalidParam { processor: String, param: String, span: Span },
    ConflictingParams { processor: String, params: Vec<String>, span: Span },
    InvalidNthExpr { reason: String, span: Span },
    /// Warning only — compilation continues with an empty string substitution.
    UnsetEnvVar { name: String, span: Span },
}
