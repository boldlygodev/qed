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

pub(crate) mod compile;
pub mod error;
pub(crate) mod exec;
pub(crate) mod parse;
pub(crate) mod processor;
pub mod span;

pub use compile::OnError;

/// Options that control how a qed script is executed.
pub struct RunOptions {
    /// Disable environment variable expansion in patterns and processor args.
    pub no_env: bool,
    /// Global on-error mode — sets the default for selectors that do not
    /// specify `on_error` explicitly.
    pub on_error: OnError,
    /// Suppress passthrough output; only selected (processed) regions are emitted.
    pub extract: bool,
}

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

/// Result of running a qed script.
pub struct RunResult {
    /// The transformed output text.
    pub output: String,
    /// Diagnostic messages (warnings and errors).
    pub diagnostics: Vec<RunDiagnostic>,
    /// Whether any diagnostic is an error (execution should be considered failed).
    pub has_errors: bool,
    /// Raw lines to emit to stderr (from `qed:warn()`, `qed:fail()`,
    /// `qed:debug:print()`).
    pub stderr_lines: Vec<String>,
}

/// A diagnostic message from script execution.
pub struct RunDiagnostic {
    /// "error" or "warning"
    pub level: &'static str,
    /// Formatted source location (e.g., "1:1-10")
    pub location: String,
    /// The source text label (selector text or processor text).
    pub selector_text: String,
    /// The diagnostic message (e.g., "no lines matched").
    pub message: String,
}

/// Run a qed script against input text, returning the result with diagnostics.
pub fn run(script_source: &str, input: &str, options: &RunOptions) -> Result<RunResult, String> {
    let program = parse::parse_program(script_source).map_err(|errors| {
        errors
            .iter()
            .map(|e| format!("{e:?}"))
            .collect::<Vec<_>>()
            .join("\n")
    })?;

    let compile_options = compile::CompileOptions {
        no_env: options.no_env,
        global_on_error: options.on_error,
    };

    let (script, compile_warnings) =
        match compile::compile(&program, script_source, &compile_options) {
            Ok(pair) => pair,
            Err(errors) => {
                let diagnostics = errors
                    .iter()
                    .map(|e| {
                        let (span, message) = compile_error_to_diagnostic(e);
                        RunDiagnostic {
                            level: "error",
                            location: span::format_span(script_source, span),
                            selector_text: script_source[span.start..span.end].to_owned(),
                            message,
                        }
                    })
                    .collect();
                return Ok(RunResult {
                    output: String::new(),
                    diagnostics,
                    has_errors: true,
                    stderr_lines: Vec::new(),
                });
            }
        };

    let buffer = exec::Buffer::new(input.to_owned());
    let result = exec::engine::execute(&script, &buffer, options.extract);

    let mut has_errors = false;
    let mut diagnostics: Vec<RunDiagnostic> = Vec::new();

    // Convert compile warnings into diagnostics.
    for w in &compile_warnings {
        let (span, source_text, message) = match w {
            error::CompileWarning::UnsetEnvVar { span, .. } => (
                *span,
                script_source[span.start..span.end].to_owned(),
                "environment variable not set, expanding to empty string".to_owned(),
            ),
            error::CompileWarning::DuplicateName { name, kind, span } => (
                *span,
                script_source[span.start..span.end].to_owned(),
                format!("{kind} {name} already defined, using last definition"),
            ),
            error::CompileWarning::InclusiveIgnored { selector_op, span } => (
                *span,
                script_source[span.start..span.end].to_owned(),
                format!("+ ignored on {selector_op}"),
            ),
        };
        diagnostics.push(RunDiagnostic {
            level: "warning",
            location: span::format_span(script_source, span),
            selector_text: source_text,
            message,
        });
    }

    // Convert execution diagnostics.
    for d in &result.diagnostics {
        let level = match d.level {
            exec::engine::DiagnosticLevel::Error => {
                if !d.recovered {
                    has_errors = true;
                }
                "error"
            }
            exec::engine::DiagnosticLevel::Warning => "warning",
            exec::engine::DiagnosticLevel::Debug => "debug",
        };
        diagnostics.push(RunDiagnostic {
            level,
            location: span::format_span(script_source, d.span),
            selector_text: d.selector_text.clone(),
            message: d.message.clone(),
        });
    }

    if result.halted_by_fail {
        has_errors = true;
    }

    // Pad location fields to the width of the widest location string.
    let max_loc_width = diagnostics
        .iter()
        .map(|d| d.location.len())
        .max()
        .unwrap_or(0);
    for d in &mut diagnostics {
        let pad = max_loc_width - d.location.len();
        if pad > 0 {
            d.location.push_str(&" ".repeat(pad));
        }
    }

    Ok(RunResult {
        output: result.output,
        diagnostics,
        has_errors,
        stderr_lines: result.stderr_lines,
    })
}

/// Extract the span and human-readable message from a compile error.
fn compile_error_to_diagnostic(e: &error::CompileError) -> (span::Span, String) {
    match e {
        error::CompileError::UndefinedName { name, span } => {
            (*span, format!("undefined name: {name}"))
        }
        error::CompileError::WrongSymbolKind {
            name,
            expected,
            found,
            span,
        } => (*span, format!("{name} is a {found}, not a {expected}")),
        error::CompileError::InvalidRegex {
            pattern,
            reason,
            span,
        } => (*span, format!("invalid regex /{pattern}/: {reason}")),
        error::CompileError::InvalidParam {
            processor,
            param,
            span,
        } => (*span, format!("{processor}: {param}")),
        error::CompileError::ConflictingParams {
            processor,
            params,
            span,
        } => (
            *span,
            format!("{processor}: conflicting parameters: {}", params.join(", ")),
        ),
        error::CompileError::InvalidNthExpr { reason, span } => (*span, reason.clone()),
    }
}
