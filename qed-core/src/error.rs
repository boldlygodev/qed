//! Compile-time error types.
//!
//! The compilation pass uses an accumulator pattern: errors are collected into
//! a `Vec<CompileError>` rather than aborting on the first failure. This lets
//! the compiler report every problem in a single run. All variants carry a
//! [`Span`] so diagnostics can point to the offending source location.

use std::fmt;

use crate::span::Span;

/// Discriminates named symbols in the symbol table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    /// A named pattern defined via `pattern name = "..."`.
    Pattern,
    /// A named processor alias defined via `alias name = qed:...(...)`.
    Alias,
}

impl fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymbolKind::Pattern => write!(f, "pattern"),
            SymbolKind::Alias => write!(f, "alias"),
        }
    }
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
    WrongSymbolKind {
        name: String,
        expected: SymbolKind,
        found: SymbolKind,
        span: Span,
    },
    /// A regex pattern failed to compile.
    InvalidRegex {
        pattern: String,
        reason: String,
        span: Span,
    },
    /// A processor or selector received a parameter it does not recognize.
    InvalidParam {
        processor: String,
        param: String,
        span: Span,
    },
    /// Two or more mutually exclusive parameters were specified together.
    ///
    /// Reserved — not yet emitted. Will be used by `qed:replace()` in Phase 6C.
    ConflictingParams {
        processor: String,
        params: Vec<String>,
        span: Span,
    },
    /// An nth expression is syntactically valid but semantically invalid
    /// (e.g., cross-sign range bounds).
    ///
    /// Reserved — nth semantic validation is handled at parse time. This
    /// variant exists for potential future compile-time validation.
    InvalidNthExpr { reason: String, span: Span },
}

/// Warnings produced by the compilation pass.
///
/// Warnings are non-fatal: compilation succeeds and execution proceeds
/// normally. They are accumulated alongside the compiled [`Script`] and
/// emitted to stderr.
///
/// [`Script`]: crate::compile::Script
#[derive(Debug, Clone)]
pub enum CompileWarning {
    /// An environment variable reference names a variable that is not set.
    /// Compilation continues with an empty string substitution.
    UnsetEnvVar { name: String, span: Span },
    /// A pattern or alias name was defined more than once.
    /// The last definition wins.
    DuplicateName {
        name: String,
        kind: SymbolKind,
        /// Span of the new (overwriting) definition.
        span: Span,
    },
    /// The `+` (inclusive) modifier was used on a selector where it has no
    /// effect (`at`, `after`, or `before`). The flag is silently cleared.
    InclusiveIgnored {
        selector_op: &'static str,
        span: Span,
    },
    /// A zero term appeared in an nth expression and was ignored.
    NthZeroTerm {
        /// The full nth expression source text (e.g. `"nth:0"`).
        nth_source: String,
        span: Span,
    },
    /// A duplicate occurrence was detected in an nth expression.
    NthDuplicate {
        /// The full nth expression source text.
        nth_source: String,
        /// The 1-based occurrence number that was duplicated.
        occurrence: i64,
        span: Span,
    },
}
