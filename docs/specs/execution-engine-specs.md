# EARS Specs — Execution Engine

ID prefix: `EXEC`

## Buffer Model

- [x] EXEC-001: The engine SHALL load the full input string into a `Buffer` before execution; streaming line-by-line is not supported.
- [x] EXEC-002: `Buffer` SHALL pre-compute `line_offsets` at construction time, enabling O(1) slicing of any line or range.
- [x] EXEC-003: The original `Buffer` content SHALL NOT be mutated during execution; all changes SHALL be expressed as fragment replacements assembled into a new output string.
- [x] EXEC-004: A trailing `'\n'` in the buffer SHALL be treated as a line terminator, not as the start of a new empty line.

## Fragment Model

- [x] EXEC-010: `FragmentContent` SHALL be either `Borrowed(LineRange)` (zero-copy reference into the buffer) or `Owned(String)` (processor output).
- [x] EXEC-011: A `Fragment` SHALL be either `Passthrough(FragmentContent)` (emitted unchanged) or `Selected { content, tags }` (targeted by one or more statements).
- [x] EXEC-012: The fragmentation algorithm SHALL collect selector matches using `rayon::par_iter` for parallel evaluation.
- [x] EXEC-013: WHEN the buffer is empty, only an `at()` with an empty literal match SHALL produce a zero-width `Selected` fragment; all other selectors SHALL produce no match.
- [x] EXEC-014: Boundary events at the same line SHALL be processed End-before-Start, allowing adjacent ranges from the same selector to hand off without losing their tag.
- [x] EXEC-015: The active tag set SHALL use `BTreeSet` to guarantee deterministic statement ordering in output regardless of match evaluation order.
- [x] EXEC-016: `pair_from_to` SHALL use nearest-next pairing for compound `from > to` selectors, preventing a delimiter line from acting as both open and close of a region.
- [x] EXEC-017: `merge_adjacent_lines` SHALL collapse contiguous single-line matches into one `LineRange` so processors receive them in a single `execute()` call.

## Execution Loop

- [x] EXEC-020: Before the fragment walk, the engine SHALL perform a pre-check pass that detects statements with no matches and, WHEN `on_error` is `Fail`, SHALL attempt their fallback and return early if the fallback produces output.
- [x] EXEC-021: WHEN a `Selected` fragment maps to a `StatementAction::Process`, the engine SHALL call `processor.execute(content)` and splice the result back into the fragment list.
- [x] EXEC-022: WHEN a `Selected` fragment maps to `StatementAction::CopyTo` or `MoveTo`, the engine SHALL record a `PendingRelocation` and apply it after the full fragment walk.
- [x] EXEC-023: WHEN a `Selected` fragment maps to `StatementAction::Warn`, the engine SHALL push content to `stderr_lines` and pass content through to output unchanged.
- [x] EXEC-024: WHEN a `Selected` fragment maps to `StatementAction::Fail`, the engine SHALL push content to `stderr_lines`, set `halted_by_fail`, and stop processing further fragments.
- [x] EXEC-025: WHEN a `Selected` fragment maps to `StatementAction::DebugCount`, the engine SHALL accumulate a hit count per `StatementId` and emit a diagnostic after the walk.
- [x] EXEC-026: WHEN a `Selected` fragment maps to `StatementAction::DebugPrint`, the engine SHALL echo content to `stderr_lines` verbatim.
- [x] EXEC-027: After a `Process` action transforms a fragment, the engine SHALL apply a post-hoc `selector_still_matches` guard to check whether the next statement's selector still applies to the new text.

## Processor Contract

- [x] EXEC-030: The `Processor` trait SHALL define `execute(&self, input: &str) -> Result<String, ProcessorError>` and a defaulted `is_file_marker() -> bool` that returns `false`.
- [x] EXEC-031: `ProcessorError` SHALL carry variants: `NoMatch`, `ProcessorFailed`, `ExternalFailed`, and `FileEmptyRegion`.
- [x] EXEC-032: `ChainProcessor` SHALL short-circuit and return `Ok("")` (the deletion signal) if any step in the chain returns an empty string.
- [x] EXEC-033: `map_lines` SHALL strip trailing `'\n'` before applying a per-line function and re-append it if the original had one.

## Error and Fallback Semantics

- [x] EXEC-040: WHEN a processor error occurs and a `CompiledFallback::Chain` exists, the engine SHALL execute the fallback chain and push a `Diagnostic { recovered: true }`.
- [x] EXEC-041: WHEN a processor error occurs and no fallback produces output, the engine SHALL set `has_processor_error = true`.
- [x] EXEC-042: `FileEmptyRegion` SHALL be handled inline in the execution loop: the engine SHALL emit a `Warning` diagnostic and continue without halting.
- [x] EXEC-043: WHEN `halted_by_fail` is set, the output accumulated before the halting fragment SHALL be preserved and returned.
- [x] EXEC-044: WHEN `has_processor_error` is set (unrecovered processor failure), the output SHALL be cleared before return.

## Stream Control

- [x] EXEC-050: `qed:warn()` SHALL push selected content to stderr and pass the content through to output unchanged.
- [x] EXEC-051: `qed:fail()` SHALL push selected content to stderr, set `halted_by_fail`, and stop execution; it SHALL NOT emit selected content to output.
- [x] EXEC-052: `qed:skip()` SHALL pass content through to output unchanged; it is an identity operation useful with `--extract`.
- [x] EXEC-053: `qed:debug:count()` SHALL accumulate a match count per statement and emit an `"N match"` diagnostic after the walk.
- [x] EXEC-054: `qed:debug:print()` SHALL echo selected content to stderr verbatim.

## Public Run API

- [x] EXEC-060: `run(script_source, input, options) -> Result<RunResult, String>` SHALL wire `parse_program → compile → Buffer::new → execute` and translate internal types to public types.
- [x] EXEC-061: `RunOptions` SHALL carry `no_env: bool`, `on_error: OnError`, and `extract: bool`.
- [x] EXEC-062: `RunResult` SHALL carry `output: String`, `diagnostics: Vec<RunDiagnostic>`, `has_errors: bool`, and `stderr_lines: Vec<String>`.
- [x] EXEC-063: `RunResult.has_errors` SHALL be `true` if any diagnostic has level `Error` OR if `ExecuteResult.halted_by_fail` is set.

## Non-Features

- [D] EXEC-070: The execution engine SHALL NOT stream input line-by-line; full-buffer loading is required for correctness of range selectors.
- [D] EXEC-071: The engine SHALL NOT emit an end-of-run summary line (e.g. `"2 warnings, 1 error"`) unless explicitly designed and tested in a future phase.

## References

- `qed-core/src/lib.rs`
- `qed-core/src/exec/mod.rs`
- `qed-core/src/exec/engine.rs`
- `qed-core/src/exec/fragment.rs`
- `qed-core/src/processor/mod.rs`
- `qed-core/src/processor/chain.rs`
- `docs/llds/execution-engine.md`
