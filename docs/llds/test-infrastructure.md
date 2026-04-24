# Test Infrastructure

## Context and Design Philosophy

A two-layer integration test harness: a Rust layer manages trial lifecycle and delegates to a bash layer for execution. The split is intentional — bash is the natural layer for testing a CLI tool that interacts with the shell (PATH, stdin/stdout, environment variables, subprocesses). The Rust layer handles the parts bash cannot do well: typed manifest parsing, temp directory management, trial naming, and libtest-mimic integration.

Tests treat `qed` as a black box. The `qed-tests` crate has no dependency on `qed-core`; it discovers and runs the `qed` binary via the target directory.

## Two-Layer Architecture

```
Rust layer (qed-tests/)          Bash layer (tests/harness/)
─────────────────────────────    ──────────────────────────────────
manifest.rs  — discovery         run-scenario.sh   — orchestration
scenario.rs  — variable gen      generate-mock.sh  — mock creation
runner.rs    — trial exec        compare-golden.sh — output assert
main.rs      — libtest-mimic reg
```

The Rust layer produces one `Trial` per scenario invocation index. Each trial:
1. Generates a `scenario.sh` variable file
2. Creates an isolated temp directory
3. Executes `bash run-scenario.sh <tmpdir>`
4. Reports pass/fail from the script's exit code

## Manifest and Trial Discovery

`manifest.rs` walks `tests/` to depth 2 via `read_dir`, looking for `manifest.toml` files. The depth-2 cap supports nested suites like `tests/usecases/code-editing/` but would silently skip anything deeper.

Each `manifest.toml` is deserialized via serde/toml into `Manifest { scenario: Vec<Scenario> }`. `Scenario` fields:

| Field | Type | Notes |
|---|---|---|
| `id` | `String` | Used in trial name and `SCENARIO_ID` |
| `description` | `String` | Informational |
| `script` | `Option<String>` | Filename under `scripts/` |
| `input` | `String` | Filename under `inputs/` |
| `stdout` / `stderr` / `output` | `Option<String>` | Golden filenames (extension determines comparison mode) |
| `exit_code` | `u32` (default 0) | Expected process exit code |
| `invoke` | `Vec<String>` | One string per invocation; each produces one Trial |
| `env` | `BTreeMap<String,String>` | Injected into the scenario environment |
| `mock` | `Vec<MockDecl>` | Zero or more mock command declarations |

`discover_manifests` accumulates errors rather than failing fast; all discovery errors are reported together.

Suite names are relative paths with backslash-to-forward-slash normalization (cross-platform).

## Scenario Generation

`scenario.rs::generate()` produces a bash variable file sourced by `run-scenario.sh`. Key variables:

- `SCENARIO_ID`, `SCENARIO_DESC`, `SUITE_DIR`
- `SCRIPT`, `INPUT_SRC`, `STDOUT_GOLDEN`, `STDERR_GOLDEN`, `OUTPUT_GOLDEN`
- `EXPECTED_EXIT_CODE`
- `INVOCATION` — single-quoted with internal single-quotes escaped via `'\''`; selected by `invocation_index`
- `MOCK_COUNT` and per-mock variables: `MOCK_{i}_COMMAND`, `MOCK_{i}_INPUT`, `MOCK_{i}_STDOUT`, `MOCK_{i}_STDERR`, `MOCK_{i}_EXIT_CODE`, `MOCK_{i}_EXPECTED_ARGS_COUNT`, `MOCK_{i}_EXPECTED_ARG_{j}`

`MOCK_{i}_EXPECTED_ARG_{j}` values are single-quoted to preserve literal `${QED_FILE}` references — expansion occurs at mock validation time, not generation time.

## Trial Lifecycle

`runner.rs::run_trial()`:

1. Derives temp dir path: `<system_temp>/<suite>-<scenario_id>-<invocation_index>-<pid>`
2. Creates temp dir tree: `scenario.sh`, `bin/`, `mock-state/`, `input`, `stdout`, `stderr`, `output`
3. On Unix: symlinks qed binary into `bin/qed`; on non-Unix: copies it
4. Writes `scenario.sh` to `<tmpdir>/scenario.sh`
5. Runs `bash <harness_script> <tmpdir>`; captures combined stdout+stderr for failure messages
6. Removes temp dir unconditionally (pass or fail)

`find_qed_binary()` checks debug and release target directories; panics with an actionable message if the binary is not built.

## Mock System

`generate-mock.sh` generates a self-contained executable bash script for each distinct command name declared in the scenario. Generated mocks are placed in `<tmpdir>/bin/`, which is prepended to `PATH` — system commands with the same name are shadowed.

**Mock script behavior:**
- Maintains a call counter in `$MOCK_STATE_DIR/<command>.count` (atomic update via tmp + rename)
- On Nth call, uses `DECL_{N-1}_*` variables for that call's expected input, stdout, stderr, exit code, and args
- Over-call detection: exits 127 with diagnostic if called more times than declarations
- Input validation: if `$QED_FILE` is set, reads from the file path; otherwise reads from stdin
- Arg validation: if `EXPECTED_ARGS_CT > 0`, checks actual args match expected; `${QED_FILE}` references in expected args are expanded at validation time via `eval`
- All `DECL_*` vars are baked in at generation time — the mock is self-contained; `scenario.sh` is not re-sourced at call time

**Unconsumed mock check:** `run-scenario.sh:57` has a placeholder comment for verifying that all declared mock calls were consumed by the end of the trial. **Not yet implemented.** Mocks can be under-called silently.

## Golden Comparison

`compare-golden.sh` dispatches on the golden file extension:

| Extension | Comparison mode |
|---|---|
| `.pattern` | Full-string anchored regex via bash `[[ =~ ]]`; literal `\n` in pattern resolved to real newlines; RE2 syntax |
| `.*` (glob) | Matches one or more golden files; each compared independently; error if glob resolves to zero files |
| anything else | Exact text diff via `diff` |

The "anything else" catch-all means `.yaml`, `.toml`, `.go` etc. golden files are all exact-match — not just `.txt`. This is wider than the spec implies but consistent with observed test suite usage.

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| Two-layer Rust + bash architecture | Rust for lifecycle; bash for execution | Pure Rust test runner | Bash is the natural medium for testing CLI + subprocess + PATH interactions; Rust handles typed manifest parsing and trial isolation cleanly. [inferred] |
| `libtest-mimic` over `#[test]` harness | Custom test runner | Standard `#[test]` with cargo test | `libtest-mimic` provides `<suite>::<scenario>::<invocation>` naming and `cargo test "pattern"` filtering; libtest cannot produce custom trial names. [inferred] |
| Total temp dir isolation per trial | Fresh dir per invocation | Shared dir per scenario | Prevents inter-invocation state leakage; makes failures reproducible without ordering dependencies. [inferred] |
| Self-contained mock scripts | Bake `DECL_*` vars at generation time | Re-source `scenario.sh` at call time | Mock scripts are called from within the `eval`'d `INVOCATION` subshell; re-sourcing would require careful path management. Self-contained is simpler. [inferred] |
| Depth-2 manifest walk | Hard-coded `depth <= 2` | Recursive walk | Sufficient for the current two-level nesting (`usecases/{sub}/`); deeper nesting would require changing this constant. [inferred] |

## Open Questions & Future Decisions

### Resolved
*(none yet)*

### Deferred
1. **Unconsumed mock check** — Implement the placeholder at `run-scenario.sh:57`. Scenarios that under-call their declared mocks currently pass silently.
2. **Manifest walk depth** — Should depth-2 be made configurable, or is a recursive walk preferable?
3. **Non-`.txt` catch-all in `compare-golden.sh`** — Should the exact-match catch-all be narrowed to `.txt` only? Currently any unrecognized extension is exact-matched, which may be surprising.

## References

- `qed-tests/src/main.rs`
- `qed-tests/src/manifest.rs`
- `qed-tests/src/runner.rs`
- `qed-tests/src/scenario.rs`
- `tests/harness/run-scenario.sh`
- `tests/harness/generate-mock.sh`
- `tests/harness/compare-golden.sh`
- `.claude/tests/harness.md` — authoritative test harness specification
- `docs/arrows/test-infrastructure.md`
- `docs/specs/test-infrastructure-specs.md`
