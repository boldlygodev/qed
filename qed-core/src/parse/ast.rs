//! Abstract syntax tree types for the qed language.
//!
//! Every node in the tree is wrapped in [`Spanned<T>`] to carry its source
//! location. The top-level structure is:
//!
//! ```text
//!  Program
//!    └── statements: Vec<Statement>
//!          ├── PatternDef       pattern name = "..." | /regex/
//!          ├── AliasDef         alias name = processor_chain
//!          └── SelectAction
//!                ├── Selector         (one or more SimpleSelector steps)
//!                │     └── SimpleSelector
//!                │           ├── SelectorOp   (at | after | before | from | to)
//!                │           ├── PatternRef?
//!                │           └── params: Vec<Param>
//!                ├── ProcessorChain?  (piped transformations)
//!                │     └── Processor
//!                │           ├── Qed(QedProcessor)      qed:name(args; params)
//!                │           └── External(ExternalProcessor)  !cmd args
//!                └── Fallback?        (else clause)
//! ```

use crate::span::Spanned;

// ── Top-level ────────────────────────────────────────────────────────

// @spec LTYP-010
/// Root AST node — a complete qed program parsed from source text.
#[derive(Debug, Clone)]
pub(crate) struct Program {
    /// The ordered list of statements that make up the program.
    pub(crate) statements: Vec<Spanned<Statement>>,
}

// @spec LTYP-011
/// A single top-level statement in a qed program.
#[derive(Debug, Clone)]
pub(crate) enum Statement {
    /// `pattern name = "..." | /regex/` — defines a reusable pattern.
    PatternDef {
        name: Spanned<String>,
        value: Spanned<PatternValue>,
    },
    /// `alias name = processor_chain` — defines a reusable processor chain.
    AliasDef {
        name: Spanned<String>,
        chain: Spanned<ProcessorChain>,
    },
    /// `selector | processor_chain` — the primary action statement.
    SelectAction(SelectActionNode),
}

/// The core action: select lines, optionally transform them, optionally
/// specify a fallback if the selector matches nothing.
#[derive(Debug, Clone)]
pub(crate) struct SelectActionNode {
    pub(crate) selector: Spanned<Selector>,
    pub(crate) chain: Option<Spanned<ProcessorChain>>,
    pub(crate) fallback: Option<Spanned<Fallback>>,
}

// ── Patterns ─────────────────────────────────────────────────────────

/// The concrete value of a pattern — either a literal string or a regex.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PatternValue {
    /// Double-quoted literal: `"hello"` — matched by substring containment.
    String(String),
    /// Slash-delimited regex: `/^hello/` — matched by regex search.
    Regex(String),
}

/// How a pattern is referenced — by name or inline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PatternRefValue {
    /// A reference to a named pattern defined earlier via `PatternDef`.
    Named(String),
    /// An inline literal or regex provided directly in the selector.
    Inline(PatternValue),
}

// @spec LTYP-015
/// A pattern reference with optional modifiers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PatternRef {
    pub(crate) value: PatternRefValue,
    /// `!pattern` — inverts the match (selects lines that do *not* match).
    pub(crate) negated: bool,
    /// For `from`/`to` selectors: whether the anchor line itself is
    /// included in the selection.
    pub(crate) inclusive: bool,
}

// ── Selectors ────────────────────────────────────────────────────────

// @spec LTYP-014
/// The selector operation keyword.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SelectorOp {
    /// `at(pattern)` — selects each line matching the pattern.
    At,
    /// `after(pattern)` — selects the insertion point after matching lines.
    After,
    /// `before(pattern)` — selects the insertion point before matching lines.
    Before,
    /// `from(pattern)` — selects from the matching line to end of input.
    From,
    /// `to(pattern)` — selects from start of input to the matching line.
    To,
}

// @spec LTYP-013
/// A single selector step: an operation, an optional pattern, and
/// optional parameters (e.g., `nth`).
#[derive(Debug, Clone)]
pub(crate) struct SimpleSelector {
    pub(crate) op: SelectorOp,
    pub(crate) pattern: Option<Spanned<PatternRef>>,
    pub(crate) params: Vec<Spanned<Param>>,
}

// @spec LTYP-012
/// A selector composed of one or more chained steps. Multi-step selectors
/// (e.g., `from("start") > to("end")`) intersect their ranges.
#[derive(Debug, Clone)]
pub(crate) struct Selector {
    pub(crate) steps: Vec<Spanned<SimpleSelector>>,
}

// ── Parameters ───────────────────────────────────────────────────────

/// A named parameter on a selector or processor: `name: value`.
#[derive(Debug, Clone)]
pub(crate) struct Param {
    pub(crate) name: Spanned<String>,
    pub(crate) value: Spanned<ParamValue>,
}

/// The value side of a parameter.
#[derive(Debug, Clone)]
pub(crate) enum ParamValue {
    /// A bare identifier like `fail`, `warn`, `skip`.
    Identifier(String),
    /// A double-quoted string.
    String(String),
    /// An nth expression used with the `nth` parameter on `at()`.
    NthExpr(NthExpr),
}

// ── Processors ───────────────────────────────────────────────────────

// @spec LTYP-016
/// An ordered chain of processors separated by `|`. Each processor's
/// output feeds into the next.
#[derive(Debug, Clone)]
pub(crate) struct ProcessorChain {
    pub(crate) processors: Vec<Spanned<Processor>>,
}

// @spec LTYP-017, LTYP-018
/// A single processor — either a built-in qed processor, an external
/// shell command, or a reference to a named alias.
#[derive(Debug, Clone)]
pub(crate) enum Processor {
    /// Built-in processor: `qed:name(args; params)`.
    Qed(QedProcessor),
    /// External command: `!command args` or `!!command args` (escaped).
    External(ExternalProcessor),
    /// A bare identifier referencing a named alias defined via `AliasDef`.
    AliasRef(String),
}

/// A built-in qed processor invocation.
#[derive(Debug, Clone)]
pub(crate) struct QedProcessor {
    /// The processor name (e.g., `delete`, `replace`, `indent`).
    pub(crate) name: Spanned<String>,
    /// Positional arguments.
    pub(crate) args: Vec<Spanned<QedArg>>,
    /// Named parameters (after `;`).
    pub(crate) params: Vec<Spanned<Param>>,
}

/// A positional argument to a qed processor.
#[derive(Debug, Clone)]
pub(crate) enum QedArg {
    /// A double-quoted string literal.
    String(String),
    /// A slash-delimited regex literal.
    Regex(String),
    /// An integer literal.
    Integer(i64),
    /// A nested processor chain (for processors that accept sub-chains).
    ProcessorChain(Box<ProcessorChain>),
}

/// An external shell command processor.
#[derive(Debug, Clone)]
pub(crate) struct ExternalProcessor {
    /// The command name or path.
    pub(crate) command: Spanned<String>,
    /// Command-line arguments to the external command.
    pub(crate) args: Vec<Spanned<ExternalArg>>,
}

/// An argument to an external processor command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ExternalArg {
    /// A double-quoted argument (whitespace preserved).
    Quoted(String),
    /// A bare argument (split on whitespace at the shell level).
    Unquoted(String),
}

// ── Fallback ─────────────────────────────────────────────────────────

/// An else clause — used when the preceding selector matches nothing.
#[derive(Debug, Clone)]
pub(crate) enum Fallback {
    /// Chain another select-action as the fallback.
    SelectAction(Box<SelectActionNode>),
    /// Apply a processor chain directly (no further selector).
    Chain(ProcessorChain),
}

// ── Nth expressions ──────────────────────────────────────────────────

/// A single term in an nth expression. Terms are combined as a union
/// to select specific ordinal positions from a set of matches.
///
/// All positions are 1-based. Negative values count from the end
/// (`-1` = last, `-2` = second-to-last).
///
/// # Examples
///
/// - `1` → `Integer(1)` — first match
/// - `1...3` → `Range { start: 1, end: 3 }` — first through third
/// - `2n+1` → `Step { coefficient: 2, offset: 1 }` — 1st, 3rd, 5th, ...
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NthTerm {
    /// A single ordinal position (1-based, may be negative).
    Integer(i64),
    /// An inclusive range of ordinal positions. Both bounds must have
    /// the same sign.
    Range { start: i64, end: i64 },
    /// A repeating step: selects positions `coefficient * k + offset`
    /// for k = 0, 1, 2, ... (in 1-based indexing).
    Step { coefficient: i64, offset: i64 },
}

// @spec LTYP-019
/// A comma-separated list of [`NthTerm`]s, used as the value of the
/// `nth` parameter on `at()` selectors.
#[derive(Debug, Clone)]
pub(crate) struct NthExpr {
    pub(crate) terms: Vec<Spanned<NthTerm>>,
}
