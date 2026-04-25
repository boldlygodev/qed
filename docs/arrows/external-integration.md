# Arrow: external-integration

Subprocess delegation and file materialization — the bridge between qed and external shell tools.

## Status

**PARTIAL** — last audited 2026-04-25 (git SHA `ae1b9ec`).
20 of 21 behavioral specs implemented. One active gap: XINT-031 (spawn failure error classification). Note: XINT-030 and XINT-031 describe contradictory behaviors; resolution needed.

## References

### HLD
- `docs/high-level-design.md` — Non-Goals section (not a replacement for awk field processing; external delegation is explicit scope)

### LLD
- `docs/llds/external-integration.md`

### EARS
- `docs/specs/external-integration-specs.md`

### Tests
- `tests/external-processors/` — 6 scenarios (stdin handoff, file materialization, args, alias bypass, pipeline, replace-pipeline)
- `tests/external-processors-edge-cases/` — 11 scenarios (empty stdin/stdout, non-zero exit, file scoping, stderr passthrough, insertion-point warning, backslash continuation)

### Code
- `qed-core/src/processor/external.rs`
- `qed-core/src/processor/file.rs`

## Architecture

**Purpose:** Implements the `Processor` trait for operations that delegate to external shell commands. `ExternalCommandProcessor` pipes selected text through a subprocess via stdin/stdout. `FileHandoffProcessor` (fused with `qed:file()` at compile time) materializes the selected text as a named tempfile and injects `${QED_FILE}` into the command's argument list.

**Key Components:**
1. `ExternalCommandProcessor` — spawns subprocess; writes selected text to stdin; reads stdout as replacement (`external.rs`)
2. `FileMarker` — compile-time sentinel for `qed:file()`; `is_file_marker()` returns `true` so the compiler can detect and fuse it with the next external command (`file.rs:31–33`)
3. `FileHandoffProcessor` — fused processor; writes input to `NamedTempFile`; substitutes `${QED_FILE}` in args; also sets `QED_FILE` env var (`file.rs`)

## Spec Coverage

| Category | Spec IDs | Implemented | Deferred | Gaps |
|---|---|---|---|---|
| ExternalCommandProcessor | XINT-001–XINT-010 | 10 | 0 | 0 |
| FileHandoffProcessor and `qed:file()` fusion | XINT-020–XINT-028 | 9 | 0 | 0 |
| Error handling | XINT-030–XINT-031 | 1 | 0 | 1 (XINT-031) |
| Non-features | XINT-040–XINT-042 | 0 | 3 | 0 |
| **Total** | | **20** | **3** | **1** |

**Summary:** 20 of 21 behavioral specs implemented. XINT-031 (spawn vs command failure distinction) is an active gap; note that XINT-030 and XINT-031 currently describe contradictory behaviors.

## Key Findings

1. **No subprocess timeout** — Neither `ExternalCommandProcessor` nor `FileHandoffProcessor` sets a timeout or resource limit on spawned processes. Long-running commands block the qed process indefinitely (`external.rs`, `file.rs`).
2. **Subprocess stderr bypasses diagnostic system** — On successful subprocess exit, child stderr is emitted directly via `eprint!` (`external.rs:60–62`, `file.rs` equivalent). This bypasses qed's `RunDiagnostic` pipeline; child stderr cannot be captured or filtered by callers of `run()`.
3. **Stdin write errors suppressed** — `let _ = stdin.write_all(...)` in both processors. The comment says "process may have exited early." Intentional but means stdin delivery failures are invisible.
4. **`FileHandoffProcessor::spawn` error maps to `ExternalFailed`** — A failure to spawn the process (e.g. command not found) uses the same error variant as a successful-spawn-but-nonzero-exit, obscuring whether the error is in qed's plumbing or the external command itself.
5. **`${QED_FILE}` scoped to immediately downstream command** — In a pipeline `qed:file() | cmd1 | cmd2`, only `cmd1` receives `${QED_FILE}`; `cmd2` receives stdin normally. Tested explicitly in `external-processors-edge-cases`.
6. **Tempfile cleanup is explicit** — `FileHandoffProcessor` converts to `TempPath` then calls `drop(tmp_path)` explicitly after the subprocess completes (`file.rs:99`), rather than relying on `NamedTempFile`'s `Drop`.
7. **`qed:file()` on insertion point** — Using `qed:file()` on a zero-width insertion point (e.g. `after("x")`) emits a warning and ignores the file materialization. The empty region cannot be written to a tempfile meaningfully.

## Work Required

### Must Fix
*(none — current behavior is intentional and tested)*

### Should Fix
1. Add optional subprocess timeout (configurable, off by default) to prevent infinite hangs on misbehaving external tools (XINT specs TBD).

### Nice to Have
1. Route subprocess stderr through qed's diagnostic system when `--quiet` or structured output is added in a future CLI revision.
2. Distinguish spawn failures from command failures in `ProcessorError` (separate variant from `ExternalFailed`).
