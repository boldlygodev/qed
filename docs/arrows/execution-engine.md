# Arrow: execution-engine

Runs compiled IR against input — fragment model, execution loop, processor dispatch, stream-control behaviors, and public `run()` API.

## Status

**MAPPED** — last audited 2026-04-24 (git SHA `null`).
Brownfield mapping pass; no code annotations yet.

## References

### HLD
- `docs/high-level-design.md` — System Design section (fragment model), Key Design Decisions (buffer model, immutable fragment list)

### LLD
- `docs/llds/execution-engine.md`

### EARS
- `docs/specs/execution-engine-specs.md`

### Tests
- `qed-core/src/exec/mod.rs` — 12 inline unit tests (Buffer construction, line slicing)
- `qed-core/src/exec/fragment.rs` — 10 inline unit tests (single match, overlapping selectors, nth, from/to, negated, empty buffer)
- `tests/selectors/`, `tests/selectors-edge-cases/`, `tests/error-handling/`, `tests/stream-control/` — integration coverage

### Code
- `qed-core/src/lib.rs`
- `qed-core/src/exec/mod.rs`
- `qed-core/src/exec/engine.rs`
- `qed-core/src/exec/fragment.rs`
- `qed-core/src/processor/mod.rs`
- `qed-core/src/processor/chain.rs`

## Architecture

**Purpose:** Takes a compiled `Script` and an input string, partitions the input into `Passthrough`/`Selected` fragments, dispatches each selected fragment through its statement's action, handles errors and fallbacks, applies copy/move relocations, and returns `RunResult`. Also owns the `Processor` trait contract and `ChainProcessor` composition.

**Key Components:**
1. `Buffer` — immutable input; O(1) line slicing via pre-computed `line_offsets` (`exec/mod.rs`)
2. `FragmentList` / `Fragment` / `FragmentContent` — zero-copy partitioning (`Borrowed(LineRange)` vs `Owned(String)`) (`exec/mod.rs`)
3. `fragment()` — boundary-sweep fragmentation algorithm with rayon parallel match collection (`exec/fragment.rs`)
4. `execute()` — pre-check pass + fragment walk + relocation application; returns `ExecuteResult` (`exec/engine.rs`)
5. `Processor` trait — `execute(&self, &str) -> Result<String, ProcessorError>` + `is_file_marker()` (`processor/mod.rs`)
6. `ChainProcessor` — composes ordered `Vec<Box<dyn Processor>>`; short-circuits on empty output (deletion signal) (`processor/chain.rs`)
7. `run()` — public facade in `lib.rs`; wires parse→compile→execute; translates internal types to `RunResult`/`RunDiagnostic`
8. Stream-control dispatch — `StatementAction::{Warn, Fail, DebugCount, DebugPrint}` handled in `engine.rs`

## Spec Coverage

| Category | Spec IDs | Implemented | Deferred | Gaps |
|---|---|---|---|---|
| Buffer model | EXEC-001–EXEC-005 | *(to be filled)* | 0 | *(to be filled)* |
| Fragmentation | EXEC-006–EXEC-020 | *(to be filled)* | 0 | *(to be filled)* |
| Execution loop | EXEC-021–EXEC-040 | *(to be filled)* | 0 | *(to be filled)* |
| Error/fallback dispatch | EXEC-041–EXEC-055 | *(to be filled)* | 0 | *(to be filled)* |
| Stream control | EXEC-056–EXEC-065 | *(to be filled)* | 0 | *(to be filled)* |
| Public API | EXEC-066–EXEC-070 | *(to be filled)* | 0 | *(to be filled)* |

**Summary:** Spec coverage to be populated during EARS authoring session.

## Key Findings

1. **Rayon parallelism site** — `collect_all_matches` uses `rayon::par_iter` for parallel selector evaluation (`fragment.rs:99–115`). Only parallelism site in the codebase; controlled by `rayon` crate in `qed-core/Cargo.toml`.
2. **Pre-check pass before fragment walk** — Engine runs a no-match pre-check pass at `engine.rs:94–150` before the fragment walk. If a fallback produces output during pre-check, the function returns early without a full fragment walk.
3. **`halted_by_fail` vs `has_unrecovered_error` asymmetry** — `halted_by_fail` does not clear output; `has_processor_error` does (`engine.rs:321–323`). Two distinct failure modes with different output semantics.
4. **`FileEmptyRegion` special-cased inline** — This is the only `ProcessorError` variant handled directly in the fragment walk loop (emits Warning, does not halt); all other errors go through `handle_processor_error` (`engine.rs:208–217`).
5. **`ChainProcessor` empty-output short-circuit** — An empty string return from any step halts the chain; this is the deletion signal used by `DeleteProcessor`. Processors that intentionally return empty output cannot be composed in a chain (`chain.rs:17–19`).
6. **`pair_from_to` nearest-next pairing** — Prevents the same delimiter line from acting as both open and close of a region; credits a Phase 11C fix (`fragment.rs:228–288`).
7. **`union_ranges` potentially dead** — Defined in `fragment.rs` but the general compound path uses a per-line bitmask rather than calling it directly. May be dead code (`fragment.rs:336`).
8. **Stream control is not a `Processor`** — `warn`, `fail`, `skip`, `debug:count`, `debug:print` compile to `StatementAction` variants, not `Box<dyn Processor>`. Their behavioral spec lives here, not in `text-transformation`.
9. **`#![allow(dead_code)]` in `lib.rs`** — Temporary suppression at `lib.rs:47`; comment says "remove once modules have consumers." Still present at v1.0.

## Work Required

### Must Fix
*(none identified — core execution model is stable)*

### Should Fix
1. Remove `#![allow(dead_code)]` suppression from `lib.rs:47`; surface and resolve any remaining dead code.
2. Verify or remove `union_ranges` in `fragment.rs` — either make it part of the compound path or delete it.

### Nice to Have
1. Document `halted_by_fail` vs `has_unrecovered_error` asymmetry explicitly in the LLD (currently only traceable from code).
