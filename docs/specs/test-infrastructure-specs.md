# EARS Specs — Test Infrastructure

ID prefix: `TINFRA`

## Two-Layer Architecture

- [x] TINFRA-001: The integration test harness SHALL implement a two-layer split: a Rust layer (`qed-tests/`) manages trial lifecycle, manifest parsing, and libtest-mimic registration; a bash layer (`tests/harness/`) handles execution, mock generation, and golden comparison.
- [x] TINFRA-002: `qed-tests` SHALL have no dependency on `qed-core`; it SHALL treat `qed` as a black box, discovering it via the target directory.
- [x] TINFRA-003: Trial names SHALL follow the pattern `<suite>::<scenario-id>::<invocation-index>`, enabling `cargo test "pattern"` filtering via libtest-mimic.

## Manifest Discovery

- [x] TINFRA-010: `discover_manifests` SHALL walk `tests/` to a maximum depth of 2 using `read_dir`, looking for `manifest.toml` files.
- [x] TINFRA-011: `discover_manifests` SHALL accumulate all discovery errors rather than failing on the first, so all problems are reported together.
- [x] TINFRA-012: Suite names SHALL be relative paths with backslash-to-forward-slash normalization for cross-platform consistency.
- [ ] TINFRA-013: WHEN test suites are nested more than 2 levels deep, the manifest walker SHALL discover them; the current depth-2 cap silently ignores deeper nesting.

## Scenario Fields

- [x] TINFRA-020: Each scenario in a `manifest.toml` SHALL support: `id`, `description`, `script` (optional), `input`, `stdout`/`stderr`/`output` golden filenames (optional), `exit_code` (default 0), `invoke` (one string per invocation), `env` (key-value map), and `mock` (zero or more mock declarations).
- [x] TINFRA-021: Each string in `invoke` SHALL produce exactly one `Trial`; multiple invocations of one scenario produce multiple independent trials.
- [x] TINFRA-022: `scenario.rs::generate()` SHALL produce a bash variable file (`scenario.sh`) that is sourced by `run-scenario.sh`.
- [x] TINFRA-023: The `INVOCATION` variable in `scenario.sh` SHALL be single-quoted with internal single-quotes escaped via `'\''`.
- [x] TINFRA-024: `MOCK_{i}_EXPECTED_ARG_{j}` values SHALL be single-quoted to preserve literal `${QED_FILE}` references for expansion at mock validation time.

## Trial Lifecycle

- [x] TINFRA-030: Each trial SHALL create a fully isolated temp directory at `<system_temp>/<suite>-<scenario_id>-<invocation_index>-<pid>`.
- [x] TINFRA-031: The temp directory tree SHALL contain: `scenario.sh`, `bin/`, `mock-state/`, `input`, `stdout`, `stderr`, `output`.
- [x] TINFRA-032: On Unix systems, the `qed` binary SHALL be symlinked into `<tmpdir>/bin/qed`; on non-Unix systems it SHALL be copied.
- [x] TINFRA-033: Each trial SHALL execute `bash <harness_script> <tmpdir>` and report pass/fail from the script's exit code.
- [x] TINFRA-034: The temp directory SHALL be removed unconditionally after each trial, regardless of pass or fail.
- [x] TINFRA-035: `find_qed_binary()` SHALL check both debug and release target directories and SHALL panic with an actionable message if the binary is not built.

## Mock System

- [x] TINFRA-040: `generate-mock.sh` SHALL generate a self-contained executable bash script for each distinct command name declared in a scenario.
- [x] TINFRA-041: Generated mocks SHALL be placed in `<tmpdir>/bin/` which is prepended to `PATH`, shadowing any system commands with the same name.
- [x] TINFRA-042: Each mock script SHALL maintain a call counter in `$MOCK_STATE_DIR/<command>.count` using atomic tmp-file + rename update.
- [x] TINFRA-043: On the Nth call, the mock SHALL use `DECL_{N-1}_*` variables (stdout, stderr, exit code, expected args) baked in at generation time.
- [x] TINFRA-044: WHEN a mock is called more times than there are declarations, it SHALL exit 127 with a diagnostic message.
- [x] TINFRA-045: WHEN `$QED_FILE` is set, the mock SHALL read input from the file path; otherwise it SHALL read from stdin.
- [x] TINFRA-046: WHEN `EXPECTED_ARGS_CT > 0`, the mock SHALL validate that actual args match expected args; `${QED_FILE}` references in expected args SHALL be expanded via `eval` at validation time.
- [ ] TINFRA-047: WHEN a trial completes, the harness SHALL verify that all declared mock calls were consumed; scenarios that under-call declared mocks SHALL fail rather than passing silently.

## Golden Comparison

- [x] TINFRA-050: `compare-golden.sh` SHALL dispatch on the golden file extension to determine comparison mode.
- [x] TINFRA-051: WHEN the golden file has extension `.pattern`, the harness SHALL compare using a full-string anchored regex via bash `[[ =~ ]]`; literal `\n` in the pattern SHALL be resolved to real newlines; RE2 syntax SHALL be used.
- [x] TINFRA-052: WHEN the golden filename contains a glob character (`*`), the harness SHALL match it against one or more files and compare each independently; WHEN the glob resolves to zero files, it SHALL be an error.
- [x] TINFRA-053: WHEN the golden file has any other extension, the harness SHALL use exact text diff via `diff`.

## Non-Features

- [D] TINFRA-060: The `qed-tests` crate SHALL NOT import or depend on `qed-core` types; black-box testing via the binary interface is intentional.
- [D] TINFRA-061: Trial temp directories SHALL NOT be preserved on pass; cleanup is unconditional.

## References

- `qed-tests/src/main.rs`
- `qed-tests/src/manifest.rs`
- `qed-tests/src/runner.rs`
- `qed-tests/src/scenario.rs`
- `tests/harness/run-scenario.sh`
- `tests/harness/generate-mock.sh`
- `tests/harness/compare-golden.sh`
- `docs/llds/test-infrastructure.md`
