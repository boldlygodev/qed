# EARS Specs — Language Types

ID prefix: `LTYP`

## Span and Source Location

- [x] LTYP-001: The `Span` type SHALL store its range as a half-open byte-offset interval `[start, end)` consistent with Rust slice conventions.
- [x] LTYP-002: WHEN `format_span` converts a `Span` to a string, it SHALL display column numbers using an inclusive end convention (subtracting 1 from `end`) to match user expectations in error messages.
- [x] LTYP-003: `offset_to_line_col` SHALL derive line and column numbers by iterating bytes, producing byte-based column offsets.
- [x] LTYP-004: The `Spanned<T>` generic wrapper SHALL attach a `Span` to any AST node type, ensuring no parser-produced node can exist without a source location.
- [x] LTYP-005: `Spanned<T>` SHALL derive `Clone`; it SHALL NOT derive `Copy`, because inner types may not be `Copy`.

## AST Node Hierarchy

- [x] LTYP-010: The root AST node SHALL be `Program { statements: Vec<Spanned<Statement>> }`.
- [x] LTYP-011: A `Statement` SHALL be one of: `PatternDef`, `AliasDef`, or `SelectAction`.
- [x] LTYP-012: A `Selector` SHALL be represented as `Vec<SimpleSelector>`, where a single-element vec is a simple selector and a multi-element vec is a compound selector.
- [x] LTYP-013: Each `SimpleSelector` SHALL pair a `SelectorOp` with an `OnError` mode.
- [x] LTYP-014: `SelectorOp` SHALL cover the variants: `At`, `After`, `Before`, `From`, `To`, each carrying a `PatternRef` and an optional `NthExpr`.
- [x] LTYP-015: A `PatternRef` SHALL carry `value`, `negated`, and `inclusive` fields; `inclusive` SHALL be meaningful only for `From` and `To` operations.
- [x] LTYP-016: A `ProcessorChain` SHALL be a `Vec<Spanned<Processor>>`.
- [x] LTYP-017: A `Processor` SHALL be one of: `Qed(QedProcessor)`, `External(ExternalProcessor)`, or `AliasRef(String)`.
- [x] LTYP-018: WHERE a `QedArg` contains a nested `ProcessorChain`, it SHALL be boxed (`Box<ProcessorChain>`) to break the recursive type cycle.
- [x] LTYP-019: An `NthExpr` SHALL be a `Vec<NthTerm>` where each term is `Integer`, `Range`, or `Step`.

## Error and Warning Types

- [x] LTYP-020: The compiler SHALL accumulate parse errors into a `Vec<CompileError>` rather than aborting on the first error.
- [x] LTYP-021: `CompileError` SHALL carry a `Span` on every variant.
- [x] LTYP-022: `CompileWarning` SHALL cover: `UnsetEnvVar`, `DuplicateName`, `InclusiveIgnored`, `NthZeroTerm`, `NthDuplicate` — each carrying a `Span`.
- [x] LTYP-023: `ParseError` SHALL encode both hard parse errors and nth-parse warnings as variants of the same enum, distinguished by `is_warning()`.
- [x] LTYP-024: `ParseResult` SHALL carry both a success value and any accumulated parse warnings.
- [D] LTYP-025: `SymbolKind` SHALL NOT be re-exported from `qed_core::lib.rs`; it is a compiler-internal type.
- [ ] LTYP-026: `CompileError::ConflictingParams` and `CompileError::InvalidNthExpr` SHALL either be actively emitted or removed; they SHALL NOT remain as permanently dead variants.

## Non-Features

- [D] LTYP-030: AST node types SHALL NOT derive `PartialEq` until a concrete use case (e.g., parser unit tests) justifies the overhead of maintaining equality semantics across the full type hierarchy.
- [D] LTYP-031: `ExternalProcessor.escaped` (the `!!` double-bang field) SHALL NOT be silently ignored at compile time if it captures a semantic distinction; either implement it or remove the field.

## References

- `qed-core/src/span.rs`
- `qed-core/src/error.rs`
- `qed-core/src/parse/ast.rs`
- `qed-core/src/parse/error.rs`
- `docs/llds/language-types.md`
