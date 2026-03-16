use crate::span::Spanned;

// ── Top-level ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub(crate) struct Program {
    pub(crate) shebang: Option<Spanned<String>>,
    pub(crate) statements: Vec<Spanned<Statement>>,
}

#[derive(Debug, Clone)]
pub(crate) enum Statement {
    PatternDef {
        name: Spanned<String>,
        value: Spanned<PatternValue>,
    },
    AliasDef {
        name: Spanned<String>,
        chain: Spanned<ProcessorChain>,
    },
    SelectAction(SelectActionNode),
}

#[derive(Debug, Clone)]
pub(crate) struct SelectActionNode {
    pub(crate) selector: Spanned<Selector>,
    pub(crate) chain: Option<Spanned<ProcessorChain>>,
    pub(crate) fallback: Option<Spanned<Fallback>>,
}

// ── Patterns ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PatternValue {
    String(String),
    Regex(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PatternRefValue {
    Named(String),
    Inline(PatternValue),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PatternRef {
    pub(crate) value: PatternRefValue,
    pub(crate) negated: bool,
    pub(crate) inclusive: bool,
}

// ── Selectors ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SelectorOp {
    At,
    After,
    Before,
    From,
    To,
}

#[derive(Debug, Clone)]
pub(crate) struct SimpleSelector {
    pub(crate) op: SelectorOp,
    pub(crate) pattern: Option<Spanned<PatternRef>>,
    pub(crate) params: Vec<Spanned<Param>>,
}

#[derive(Debug, Clone)]
pub(crate) struct Selector {
    pub(crate) steps: Vec<Spanned<SimpleSelector>>,
}

// ── Parameters ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub(crate) struct Param {
    pub(crate) name: Spanned<String>,
    pub(crate) value: Spanned<ParamValue>,
}

#[derive(Debug, Clone)]
pub(crate) enum ParamValue {
    Identifier(String),
    String(String),
    Integer(i64),
    NthExpr(NthExpr),
    PatternRef(PatternRef),
}

// ── Processors ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub(crate) struct ProcessorChain {
    pub(crate) processors: Vec<Spanned<Processor>>,
}

#[derive(Debug, Clone)]
pub(crate) enum Processor {
    Qed(QedProcessor),
    External(ExternalProcessor),
}

#[derive(Debug, Clone)]
pub(crate) struct QedProcessor {
    pub(crate) name: Spanned<String>,
    pub(crate) args: Vec<Spanned<QedArg>>,
    pub(crate) params: Vec<Spanned<Param>>,
}

#[derive(Debug, Clone)]
pub(crate) enum QedArg {
    PatternRef(PatternRef),
    String(String),
    Regex(String),
    Integer(i64),
    ProcessorChain(Box<ProcessorChain>),
}

#[derive(Debug, Clone)]
pub(crate) struct ExternalProcessor {
    pub(crate) command: Spanned<String>,
    pub(crate) escaped: bool,
    pub(crate) args: Vec<Spanned<ExternalArg>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ExternalArg {
    Quoted(String),
    Unquoted(String),
}

// ── Fallback ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub(crate) enum Fallback {
    SelectAction(Box<SelectActionNode>),
    Chain(ProcessorChain),
}

// ── Nth expressions ──────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NthTerm {
    Integer(i64),
    Range { start: i64, end: i64 },
    Step { coefficient: i64, offset: i64 },
}

#[derive(Debug, Clone)]
pub(crate) struct NthExpr {
    pub(crate) terms: Vec<Spanned<NthTerm>>,
}
