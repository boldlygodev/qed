# Language Types

## Context and Design Philosophy

Provides the shared data layer used by every pipeline stage: source-location primitives, the complete AST produced by the parser, and the compile-time error/warning types accumulated by the compiler. This segment contains no pipeline logic — only type definitions and pure formatting utilities. All other segments depend on it; it depends on nothing inside `qed-core`.

## Span and Source Location

`span.rs` defines two types:

**`Span { start: usize, end: usize }`** — a byte-offset range into the source string. `end` is stored exclusive (half-open), consistent with Rust slice conventions. `format_span` converts to a human-readable `"line:col"` or `"line:col-end_col"` string using inclusive display (subtracts 1 from `end`). Every call site must be aware of which convention it is using.

**`Spanned<T> { node: T, span: Span }`** — generic wrapper that attaches a source location to any AST node. Every node in the AST is wrapped in `Spanned<T>`, ensuring the parser never produces unlocated nodes. `Spanned` is `Clone` but not `Copy` (the inner `T` may not be `Copy`).

`offset_to_line_col` iterates bytes (not chars or grapheme clusters); column numbers are byte-based. Scripts containing multi-byte UTF-8 characters will show byte-offset columns rather than character columns.

## AST Node Hierarchy

`parse/ast.rs` defines the complete grammar of the qed language as Rust types. All nodes are `Spanned<T>`.

**Top level:**
- `Program { statements: Vec<Spanned<Statement>> }`
- `Statement` — `PatternDef`, `AliasDef`, or `SelectAction`

**Selector hierarchy:**
- `Selector { steps: Vec<SimpleSelector> }` — compound selector is a `Vec`; single-element is simple
- `SimpleSelector { op: SelectorOp, on_error: OnError }` — pairs an operation with its error mode
- `SelectorOp` — `At`, `After`, `Before`, `From`, `To`; each carries a `PatternRef` and optional `NthExpr`
- `PatternRef { value: PatternRefValue, negated: bool, inclusive: bool }` — inline or named; `inclusive` meaningful only for `From`/`To`

**Processor hierarchy:**
- `ProcessorChain { steps: Vec<Spanned<Processor>> }`
- `Processor` — `Qed(QedProcessor)`, `External(ExternalProcessor)`, `AliasRef(String)`
- `QedArg::ProcessorChain` is `Box<ProcessorChain>` — the only boxing in the AST, breaking a recursive type cycle
- `Fallback` — `SelectAction(Box<SelectActionNode>)` or `Chain(ProcessorChain)`

**Nth expressions:**
- `NthExpr { terms: Vec<NthTerm> }` — list of `Integer`, `Range`, or `Step` terms

## Error and Warning Types

`error.rs` defines two accumulated-error enums (both `pub(crate)`) and one public helper:

**`CompileError`** — `UndefinedName`, `WrongSymbolKind`, `InvalidRegex`, `InvalidParam`, `ConflictingParams` *(reserved, never emitted)*, `InvalidNthExpr` *(reserved, handled at parse time)*. All carry `Span`.

**`CompileWarning`** — `UnsetEnvVar`, `DuplicateName`, `InclusiveIgnored`, `NthZeroTerm`, `NthDuplicate`. All carry `Span`.

**`SymbolKind`** — `Pattern` or `Alias`; used in `WrongSymbolKind` error messages. Declared `pub` but not re-exported from `lib.rs`.

`parse/error.rs` defines `ParseError` (4 variants) and `ParseResult` (success + warnings from nth parsing). Warnings are encoded as a variant of the same `ParseError` enum, distinguished by `is_warning()`.

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| Span end convention | Exclusive (stored), inclusive (displayed) | All-inclusive, all-exclusive | Exclusive storage aligns with Rust slice conventions; inclusive display matches user expectations for error messages. [inferred] |
| `Spanned<T>` wrapper vs flat span fields | Generic wrapper on every node | Inline `span: Span` fields per struct | Wrapper enforces that all nodes have locations; no node can be constructed without a span. [inferred] |
| Accumulate errors vs abort on first | Accumulate into `Vec<CompileError>` | Return first error immediately | Collecting all errors lets users see and fix multiple problems in one pass. [inferred] |
| Parse warnings in same enum as errors | `ParseError::NthWarning` variant + `is_warning()` | Separate `ParseWarning` enum | Simplifies return type for `parse_nth_expr`; avoids dual-return complexity. [inferred] |
| No `PartialEq` on AST nodes | Not derived | Derive `PartialEq` | [inferred] — reason unclear; may be an oversight rather than a deliberate decision. |

## Open Questions & Future Decisions

### Resolved
*(none yet — all decisions marked `[inferred]`)*

### Deferred
1. **Dead error variants** — `CompileError::ConflictingParams` and `CompileError::InvalidNthExpr` are never emitted. Remove them or promote to active use?
2. **`PartialEq` on AST** — Should AST nodes derive `PartialEq` to enable direct equality assertions in parser unit tests?
3. **`SymbolKind` visibility** — Should `SymbolKind` be `pub(crate)` instead of `pub`? Currently leaks through the public type surface without being usable by external consumers.
4. **`ExternalProcessor.escaped` field** — The `!!` double-bang prefix is captured in the AST but no compile-time handling is visible. Is this field used at execution time, or is it a planned feature stub?

## References

- `qed-core/src/span.rs`
- `qed-core/src/error.rs`
- `qed-core/src/parse/ast.rs`
- `qed-core/src/parse/error.rs`
- `docs/qed-design.md` — language specification (authoritative for AST shape)
- `docs/arrows/language-types.md`
- `docs/specs/language-types-specs.md`
