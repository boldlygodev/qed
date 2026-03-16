# qed — Implementation TODOs

## Status

- [x] Phase 0 — Workspace Scaffold
- [x] Phase 1 — Test Harness Infrastructure
- [x] Phase 2 — Core Types and Fragmentation Algorithm
- [ ] Phase 3 — Parser POC Evaluation
- [ ] Phase 4 — Walking Skeleton
- [ ] Phase 5 — Full Parser
- [ ] Phase 6 — Full Compiler
- [ ] Phase 7 — Processor Coverage
- [ ] Phase 8 — Generation Processors
- [ ] Phase 9 — Invocation Features
- [ ] Phase 10 — Diagnostics
- [ ] Phase 11 — Edge Cases and Use Cases
- [ ] Phase 12 — Release Polish

---

## Phase 2 — Core Types and Fragmentation Algorithm

### 2a — Foundation types ✓

- [x] `span.rs`: define `Span { start: usize, end: usize }` and `Spanned<T> { node: T, span: Span }`
- [x] Identity newtypes: `StatementId(usize)`, `SelectorId(usize)` with accessor methods
- [x] `error.rs`: `CompileError` enum (all variants from implementation design), `SymbolKind` enum
- [x] Wire module declarations in `lib.rs` (`mod span`, `mod error`, `mod parse`, `mod compile`, `mod exec`, `mod processor`)
- [x] Checkpoint: `cargo build --workspace` and `cargo clippy --workspace` clean

### 2b — AST types ✓

- [x] `parse/ast.rs`: `Program`, `Statement`, `SelectActionNode`
- [x] `parse/ast.rs`: `Selector`, `SimpleSelector`, `SelectorOp`
- [x] `parse/ast.rs`: `PatternValue`, `PatternRef`, `PatternRefValue`
- [x] `parse/ast.rs`: `ProcessorChain`, `Processor`, `QedProcessor`, `ExternalProcessor`
- [x] `parse/ast.rs`: `QedArg`, `ExternalArg`
- [x] `parse/ast.rs`: `Fallback`
- [x] `parse/ast.rs`: `Param`, `ParamValue`
- [x] `parse/ast.rs`: `NthExpr`, `NthTerm`
- [x] Checkpoint: `cargo build --workspace` and `cargo clippy --workspace` clean

### 2c — Exec and IR types + Processor trait ✓

- [x] `exec/`: `LineRange { start, end }`, `FragmentContent` (`Borrowed(LineRange)` / `Owned(String)`), `Fragment` (`Passthrough(FragmentContent)` / `Selected { content, tags }`), `FragmentList` type alias
- [x] `exec/`: `Buffer { content: String, line_offsets: Vec<usize> }` with constructor and `slice(LineRange) -> &str`
- [x] `compile/`: `Script { statements, selectors }`, `Statement { id, selector, processor, fallback }`
- [x] `compile/`: `RegistryEntry` (`Simple(CompiledSelector)` / `Compound(CompoundSelector)`), `CompiledSelector`, `CompoundSelector`
- [x] `compile/`: `SelectorOp` with per-variant fields, `CompiledPattern { matcher, negated, inclusive }`, `PatternMatcher` (`Literal(String)` / `Regex(regex::Regex)`)
- [x] `compile/`: `OnError` enum
- [x] `processor/`: `Processor` trait (`fn execute(&self, input: &str) -> Result<String, ProcessorError>`), `ProcessorError` enum (`NoMatch`, `ProcessorFailed`, `ExternalFailed`)
- [x] Unit tests for `Buffer::new()` (line offset construction) and `Buffer::slice()` (correct line extraction)
- [x] Checkpoint: buffer unit tests pass, `cargo build --workspace` clean

### 2d — Fragmentation algorithm ✓

- [x] Implement parallel match collection using `rayon`
- [x] Boundary event decomposition (`Start` / `End` events per match)
- [x] Sort events (line ascending, `Start` before `End`, `StatementId` ascending)
- [x] Sweep with `BTreeSet` active tag set producing the `FragmentList`
- [x] `inclusive` boundary logic per `CompiledPattern`
- [x] `nth` filtering on match results
- [x] Unit test: single selector, single match → one `Selected` fragment flanked by `Passthrough`
- [x] Unit test: single selector, no match → all `Passthrough`
- [x] Unit test: two overlapping selectors → multi-tagged `Selected` fragment
- [x] Unit test: `nth:2` → only second match selected
- [x] Unit test: `from > to` compound → correct inclusive/exclusive boundary variants
- [x] Unit test: negated pattern → lines not matching are selected
- [x] Checkpoint: all fragmentation unit tests pass

---

## Phase 3 — Parser POC Evaluation

See `docs/qed-roadmap.md` for full details.

---

## Phase 4 — Walking Skeleton

See `docs/qed-roadmap.md` for full details.

---

## Phases 5–12

See `docs/qed-roadmap.md` for full details.

---

## Deferred

- [ ] Switch `collect_all_matches` in `exec/fragment.rs` to `rayon` parallel iteration (dependency already present)
