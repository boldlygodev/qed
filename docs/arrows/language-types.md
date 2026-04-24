# Arrow: language-types

Shared type foundations — spans, AST nodes, and error/warning enums used across the full pipeline.

## Status

**MAPPED** — last audited 2026-04-24 (git SHA `null`).
Brownfield mapping pass; no code annotations yet.

## References

### HLD
- `docs/high-level-design.md` — System Design section (pipeline overview)

### LLD
- `docs/llds/language-types.md`

### EARS
- `docs/specs/language-types-specs.md`

### Tests
- `qed-core/src/parse/rd/parser.rs` (inline unit tests — structural field access since no PartialEq on AST)
- `tests/selectors/`, `tests/patterns/` — exercise AST paths indirectly via integration tests

### Code
- `qed-core/src/span.rs`
- `qed-core/src/error.rs`
- `qed-core/src/parse/ast.rs`
- `qed-core/src/parse/error.rs`

## Architecture

**Purpose:** Defines the data layer that all pipeline stages share — source-location tracking (Span, Spanned<T>), compile-time error/warning types, and the full AST node hierarchy produced by the parser and consumed by the compiler.

**Key Components:**
1. `Span` / `Spanned<T>` — byte-offset source location; Copy; exclusive end stored, inclusive displayed
2. `CompileError` / `CompileWarning` — accumulated (not abort-on-first); all variants carry `Span`
3. `SymbolKind` — discriminates Pattern vs Alias in error messages
4. `Program` / `Statement` / `SelectActionNode` — top-level AST
5. `Selector` / `SelectorOp` / `SimpleSelector` — selector hierarchy with `+` (inclusive) and `nth` modifiers
6. `ProcessorChain` / `Processor` (AST) / `QedProcessor` / `ExternalProcessor` — processor representation
7. `Fallback` / `NthExpr` / `NthTerm` — fallback operator and nth selector expression

## Spec Coverage

| Category | Spec IDs | Implemented | Deferred | Gaps |
|---|---|---|---|---|
| Span tracking | LTYP-001–LTYP-005 | *(to be filled)* | 0 | *(to be filled)* |
| Error/warning types | LTYP-006–LTYP-010 | *(to be filled)* | 0 | *(to be filled)* |
| AST completeness | LTYP-011–LTYP-020 | *(to be filled)* | 0 | *(to be filled)* |

**Summary:** Spec coverage to be populated during EARS authoring session.

## Key Findings

1. **Span end-convention duality** — `Span.end` is stored exclusive but displayed inclusive (`span.end - 1` in `format_span`). Every call site must track which convention it uses (`span.rs:44–54`).
2. **Dead error variants** — `CompileError::ConflictingParams` and `CompileError::InvalidNthExpr` are reserved but never emitted (`error.rs:62–76`). They add match-arm maintenance overhead with no current benefit.
3. **No `PartialEq` on AST nodes** — `Program`, `Statement`, and their children cannot be compared directly in tests; parser unit tests use structural field access (`parse/ast.rs`).
4. **`SymbolKind` visibility leak** — `SymbolKind` is `pub` but not re-exported from `lib.rs`, so external consumers cannot name the type even though it appears in error messages (`error.rs`).
5. **`ExternalProcessor.escaped` field** — The `!!` double-bang prefix is captured in the AST (`ast.rs:205`) but no corresponding compile-time handling is visible in `compile/mod.rs`; may be unused or handled at exec time.
6. **`PatternRef.inclusive` on non-range selectors** — The `+` flag is stored on every `PatternRef` regardless of selector type; the compiler emits `InclusiveIgnored` warnings for `at`/`after`/`before` (`ast.rs:92`).

## Work Required

### Must Fix
*(none identified — this is a stable type layer)*

### Should Fix
1. Remove `CompileError::ConflictingParams` and `CompileError::InvalidNthExpr` dead variants, or promote them to active use (affects `error.rs:62–76`; LTYP specs TBD).
2. Add `#[derive(PartialEq)]` to AST nodes to enable direct comparison in parser tests.

### Nice to Have
1. Restrict `SymbolKind` to `pub(crate)` and remove the silent public leak.
