# Execution Engine

## Context and Design Philosophy

Takes a compiled `Script` and an input string and produces a `RunResult`. This segment is the runtime heart of qed: it implements the buffer model, the fragment partitioning algorithm, the execution loop, and the stream-control behaviors. It also owns the `Processor` trait contract (the interface all processor implementations satisfy) and the public `run()` API that the CLI and any library consumers call.

The central design commitment is **immutability of the input buffer**: the original `Buffer` is never mutated. All changes are expressed as fragment replacements that are assembled into a new output string. This eliminates a class of ordering bugs that arise when editors mutate state in place.

## Buffer Model

`exec/mod.rs` defines:

**`Buffer { content: String, line_offsets: Vec<usize> }`** — holds the full input. `line_offsets` is pre-computed at construction time, enabling O(1) slicing of any line or range. A trailing `'\n'` is treated as a line terminator, not as the start of a new empty line.

**`FragmentContent`** — either `Borrowed(LineRange)` (a zero-copy reference into `Buffer.content`) or `Owned(String)` (processor output). Borrowed variants avoid allocation for passthrough regions.

**`Fragment`** — either `Passthrough(FragmentContent)` (lines emitted unchanged) or `Selected { content, tags: Vec<(StatementId, SelectorId)> }` (lines targeted by one or more statements).

**`FragmentList`** — `Vec<Fragment>`; the partitioned view of the buffer for one execution pass.

## Fragment Model and Fragmentation

`exec/fragment.rs` implements `fragment(buffer, requests, registry) -> FragmentList`:

1. **Collect matches** — for each requested selector, evaluate it against the buffer to produce `MatchResult { range: LineRange, statement_id, selector_id }`. Uses `rayon::par_iter` for parallel evaluation — the only parallelism site in the codebase.
2. **Empty buffer special case** — only a universally-matching `at()` with an empty literal produces a zero-width `Selected` fragment; all other selectors return empty.
3. **Boundary decomposition** — convert each `MatchResult` range into `BoundaryEvent { line, kind: Start|End, ... }`. Sort events: End before Start at the same line (allows adjacent-range handoff from the same selector without losing the tag).
4. **Sweep** — walk events in order, maintaining an active tag set (`BTreeSet` for deterministic statement ordering). Zero-width Start+End at the same line produce zero-width insertion-point fragments.
5. **`pair_from_to`** — nearest-next pairing for compound `from > to` selectors; prevents the same delimiter line from acting as both open and close of a region.
6. **`merge_adjacent_lines`** — collapses contiguous single-line matches into one `LineRange` so processors receive them in a single `execute()` call.

`pattern_matches(pattern, line) -> bool` is a `pub(crate)` utility used both in fragmentation and in the engine's `selector_still_matches` post-hoc guard.

## Execution Loop

`exec/engine.rs` implements `execute(script, buffer, extract) -> ExecuteResult`:

**Pre-check pass** (`engine.rs:94–150`) — before the fragment walk, check each statement for no-match. If a statement has no matches and `on_error` is `Fail`, attempt its fallback. If the fallback produces output, return early without a full fragment walk.

**Fragment walk** — for each `Selected` fragment, find the owning statement and dispatch:
- `StatementAction::Process` — call `processor.execute(content)`, handle error/fallback, splice result back into fragment list
- `StatementAction::CopyTo` / `MoveTo` — record a `PendingRelocation`; apply after the walk
- `StatementAction::Warn` / `DebugPrint` — push content to `stderr_lines`; pass content through to output
- `StatementAction::Fail` — push to `stderr_lines`; set `halted_by_fail = true`; stop processing
- `StatementAction::DebugCount` — accumulate hit count per `StatementId`; emit diagnostic after walk

**Post-hoc selector guard** — after one `Process` action transforms a fragment, `selector_still_matches` re-checks whether the next statement's selector still applies to the new text. This is not a full re-fragmentation; it is a per-fragment guard.

**Error and fallback dispatch** — `handle_processor_error` tries `CompiledFallback::Chain` or `CompiledFallback::SelectAction`. Recovered errors push a `Diagnostic { recovered: true }`; unrecovered errors set `has_processor_error = true`. `FileEmptyRegion` is the only error handled inline in the loop (emits Warning, does not halt).

**Relocation** — `apply_relocations` scans output lines for destination pattern matches, inserts copy/move payloads in reverse order for stable indices.

**Failure semantics** — `halted_by_fail` (from `qed:fail()`) does not clear output; `has_processor_error` (from an unrecovered processor failure) does clear output. These are two distinct failure modes with different output contracts.

## Processor Contract

`processor/mod.rs` defines:

```rust
pub(crate) trait Processor: Debug {
    fn execute(&self, input: &str) -> Result<String, ProcessorError>;
    fn is_file_marker(&self) -> bool { false }
}
```

`ProcessorError` — `NoMatch { selector_id }`, `ProcessorFailed { processor, reason }`, `ExternalFailed { command, exit_code, stderr }`, `FileEmptyRegion { span }`. Derives `Clone` and `PartialEq` (unusual for error types; required for fallback comparison in the engine).

`ChainProcessor` (`processor/chain.rs`) — composes `Vec<Box<dyn Processor>>`; short-circuits and returns `Ok("")` (the deletion signal) if any step returns empty. First error halts the chain with `?`.

`map_lines(input, fn) -> String` — strips trailing `'\n'`, applies `fn` to each line, re-appends `'\n'` if the original had one. Used by 5 processors for consistent newline handling.

## Stream Control

Stream-control operations compile to `StatementAction` variants, not `Box<dyn Processor>`. Their behaviors:

- **`qed:warn()`** → `StatementAction::Warn` — pushes selected content to `stderr_lines`; content passes through to output unchanged
- **`qed:fail()`** → `StatementAction::Fail` — pushes content to `stderr_lines`; sets `halted_by_fail`; does not emit to output; lines before the selection are already in output
- **`qed:skip()`** → `StatementAction::Process(SkipProcessor)` — identity passthrough; useful with `--extract`
- **`qed:debug:count()`** → `StatementAction::DebugCount` — accumulates match count; emits `"N match"` diagnostic after walk
- **`qed:debug:print()`** → `StatementAction::DebugPrint` — echoes selected content to `stderr_lines` verbatim

## Public Run API

`lib.rs` exposes:

```rust
pub fn run(script_source: &str, input: &str, options: RunOptions) -> Result<RunResult, String>
```

`RunOptions { no_env: bool, on_error: OnError, extract: bool }` — maps to `CompileOptions` and the `extract` flag passed to `execute()`.

`RunResult { output: String, diagnostics: Vec<RunDiagnostic>, has_errors: bool, stderr_lines: Vec<String> }` — public translation of `ExecuteResult`.

`run()` wires `parse_program → compile → Buffer::new → execute` and translates internal types. `has_errors` is true if any diagnostic has level `Error` OR if `ExecuteResult.halted_by_fail` is set — two distinct conditions mapped to a single boolean.

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| Buffer model | Load full input into `Buffer` | Streaming line-by-line processing | Selectors like `from > to` require look-ahead across the full input; streaming would require two-pass buffering anyway. [confirmed — design doc] |
| Immutable buffer | Never mutate `Buffer`; express changes as fragment replacements | Mutate a working copy | Eliminates ordering bugs; each statement sees a consistent view of "original input". [confirmed — design doc] |
| Rayon parallel match collection | `par_iter` in `collect_all_matches` | Sequential evaluation | Selector evaluation is embarrassingly parallel; rayon provides wall-clock speedup for scripts with many selectors. [inferred] |
| `BTreeSet` for active tag set | Deterministic `BTreeSet` | `HashSet` | Guarantees deterministic statement-ordering in output regardless of evaluation order. [inferred] |
| `ChainProcessor` empty-string as deletion signal | Return `Ok("")` to signal deletion | Separate `ProcessorResult::Delete` variant | Simplest contract; `DeleteProcessor` is a zero-line processor that happens to return empty. [inferred] |
| Stream control as `StatementAction` not `Processor` | `StatementAction` enum variants | `Processor` implementations | `warn`/`fail`/`debug` affect execution flow (stderr routing, halting), not just text content. Conflating them with text processors would require the `Processor` trait to carry execution-engine state. [inferred] |

## Open Questions & Future Decisions

### Resolved
1. ✅ Buffer model over streaming — chosen for correctness with range selectors.

### Deferred
1. **`#![allow(dead_code)]` in `lib.rs`** — Still present at v1.0. What dead code remains once this is removed?
2. **`union_ranges` in `fragment.rs`** — Potentially unused in the general compound path. Remove or integrate?
3. **`halted_by_fail` vs `has_processor_error` output semantics** — Should both clear output, or is the current asymmetry intentional and worth documenting in the public API contract?
4. **End-of-run diagnostic summary** — Whether to emit a summary line (e.g. `2 warnings, 1 error`) is an open design question deferred from Phase 10.

## References

- `qed-core/src/lib.rs`
- `qed-core/src/exec/mod.rs`
- `qed-core/src/exec/engine.rs`
- `qed-core/src/exec/fragment.rs`
- `qed-core/src/processor/mod.rs`
- `qed-core/src/processor/chain.rs`
- `docs/qed-implementation-design.md` — pipeline architecture and fragment model (authoritative)
- `docs/arrows/execution-engine.md`
- `docs/specs/execution-engine-specs.md`
