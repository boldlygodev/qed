//! Compile-time error types.
//!
//! The compilation pass uses an accumulator pattern: errors are collected into
//! a `Vec<CompileError>` rather than aborting on the first failure. This lets
//! the compiler report every problem in a single run. All variants carry a
//! [`Span`] so diagnostics can point to the offending source location.

use crate::span::Span;

/// Discriminates named symbols in the symbol table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    /// A named pattern defined via `pattern name = "..."`.
    Pattern,
    /// A named processor alias defined via `alias name = qed:...(...)`.
    Alias,
}

/// Errors produced by the compilation pass.
///
/// All errors are collected before being returned to the caller. Most
/// variants are hard errors that prevent execution; [`UnsetEnvVar`] is a
/// warning that allows compilation to continue.
///
/// [`UnsetEnvVar`]: CompileError::UnsetEnvVar
#[derive(Debug, Clone)]
pub enum CompileError {
    /// A name was referenced (pattern or alias) that does not appear in the
    /// symbol table.
    UndefinedName { name: String, span: Span },
    /// A name exists in the symbol table but has the wrong kind — e.g., using
    /// a pattern name where an alias is expected.
    WrongSymbolKind { name: String, expected: SymbolKind, found: SymbolKind, span: Span },
    /// A regex pattern failed to compile.
    InvalidRegex { pattern: String, reason: String, span: Span },
    /// A processor or selector received a parameter it does not recognize.
    InvalidParam { processor: String, param: String, span: Span },
    /// Two or more mutually exclusive parameters were specified together.
    ConflictingParams { processor: String, params: Vec<String>, span: Span },
    /// An nth expression is syntactically valid but semantically invalid
    /// (e.g., cross-sign range bounds).
    InvalidNthExpr { reason: String, span: Span },
    /// Warning only — compilation continues with an empty string substitution.
    /// Emitted when a `$ENV_VAR` reference names a variable that is not set.
    UnsetEnvVar { name: String, span: Span },
}
