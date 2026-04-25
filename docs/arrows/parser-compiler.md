# Arrow: parser-compiler

Transforms qed source text into executable IR — recursive descent parser, two-pass compiler, and env-var expansion.

## Status

**PARTIAL** — last audited 2026-04-25 (git SHA `ae1b9ec`).
Most specs annotated. Two active gaps: PCOMP-022, PCOMP-027. Five `[x]` specs lack `@spec` annotations (PCOMP-016, 020, 021, 025, 026).

## References

### HLD
- `docs/high-level-design.md` — Approach section (selector|processor primitive), Key Design Decisions (RD parser choice)

### LLD
- `docs/llds/parser-compiler.md`

### EARS
- `docs/specs/parser-compiler-specs.md`

### Tests
- `qed-core/src/parse/rd/parser.rs` — 60+ inline unit tests (nth forms, programs, selectors, shebang, continuation)
- `qed-core/src/compile/env.rs` — inline unit tests for env expansion
- `tests/selectors/`, `tests/patterns/`, `tests/script-files/`, `tests/invocation/` — integration coverage

### Code
- `qed-core/src/parse/mod.rs`
- `qed-core/src/parse/rd/mod.rs`
- `qed-core/src/parse/rd/cursor.rs`
- `qed-core/src/parse/rd/parser.rs`
- `qed-core/src/compile/mod.rs`
- `qed-core/src/compile/env.rs`

## Architecture

**Purpose:** Ingests a qed script string and produces a validated, executable `Script` IR (or an accumulated list of errors/warnings). Two-pass: (1) collect pattern/alias definitions into symbol tables; (2) compile `SelectAction` statements into `CompiledSelector` + `StatementAction` + optional `CompiledFallback`.

**Key Components:**
1. `Cursor` — zero-copy byte-offset scanner; backtracking via `set_pos()` (`rd/cursor.rs`)
2. `parser.rs` — full RD grammar; line-based error recovery (skip-to-newline on statement errors)
3. `parse/mod.rs` — feature-flag isolation layer (formerly toggled chumsky; now rd-only)
4. `compile/mod.rs` — two-pass compiler producing `Script` IR; accumulates errors/warnings
5. `compile/env.rs` — `${VAR}` expansion with `\${VAR}` escape and unset-var tracking

## Spec Coverage

| Category | Spec IDs | Implemented | Deferred | Gaps |
|---|---|---|---|---|
| Recursive descent parser | PCOMP-001–PCOMP-008 | 8 | 0 | 0 |
| Two-pass compiler | PCOMP-010–PCOMP-016 | 7 | 0 | 0 |
| `qed:file()` fusion | PCOMP-020–PCOMP-022 | 2 | 0 | 1 (PCOMP-022) |
| AliasRef resolution | PCOMP-025–PCOMP-027 | 2 | 0 | 1 (PCOMP-027) |
| Stream-control detection | PCOMP-030 | 1 | 0 | 0 |
| Environment variable expansion | PCOMP-035–PCOMP-038 | 4 | 0 | 0 |
| Non-features | PCOMP-040–PCOMP-041 | 0 | 2 | 0 |
| **Total** | | **24** | **2** | **2** |

**Summary:** 24 of 26 behavioral specs implemented. Active gaps: file fusion state machine (PCOMP-022) and alias-typo warning (PCOMP-027).

## Key Findings

1. **Stale chumsky comment** — `parse/mod.rs:8–11` references the chumsky alternative as "under evaluation"; it was removed in Phase 3. Minor doc drift.
2. **Non-ASCII byte handling** — `cursor.rs` casts bytes to `char` via `as char` at multiple sites (lines 108, 136, etc.), which only holds for Latin-1. Multi-byte UTF-8 sequences in patterns or args will be mis-assembled. Same issue in `compile/env.rs:79`.
3. **Silent alias-typo promotion** — An unresolved `AliasRef` falls through to an external PATH-lookup command with no diagnostic (`compile/mod.rs:1119–1152`). A typo in an alias name silently becomes a subprocess invocation.
4. **`file`-handoff fusion state machine** — `pending_file_span` flag in `compile_processor_chain` is implicit; could misbehave if `qed:file()` appears in non-standard chain positions (`compile/mod.rs:1021–1056`).
5. **Clippy suppression on wide functions** — Two functions carry `#[allow(clippy::too_many_arguments)]` (8 params each): `compile_fallback` and `compile_simple_selector`. Both would benefit from a context struct.
6. **`detect_nth_duplicates` partial coverage** — Negative indices and `Step` terms are not checked for duplicates; only positive integers and ranges are validated (`compile/mod.rs:1631`).
7. **`\$` at EOF in env expansion** — A trailing `\$` with nothing after it emits `\` without emitting `$`; untested edge case (`compile/env.rs:43–49`).
8. **Unsafe env mutation in unit tests** — `compile/env.rs` tests use `std::env::set_var`/`remove_var`, which are unsound in multi-threaded test contexts (mitigated by unique key names).

## Work Required

### Must Fix
1. Non-ASCII byte handling in `cursor.rs` and `env.rs` — `as char` byte cast breaks multi-byte UTF-8 (PCOMP specs TBD).

### Should Fix
1. Silent alias-typo → external command promotion should emit a diagnostic (PCOMP specs TBD).
2. Refactor `compile_fallback` and `compile_simple_selector` to use a context struct; remove `#[allow(clippy::too_many_arguments)]`.
3. Extend `detect_nth_duplicates` to cover negative indices and `Step` terms.

### Nice to Have
1. Remove stale chumsky comment from `parse/mod.rs`.
2. Cover `\$` at EOF in env expansion tests.
