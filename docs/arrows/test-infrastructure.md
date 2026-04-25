# Arrow: test-infrastructure

Integration test harness — manifest discovery, trial lifecycle, scenario generation, and golden comparison.

## Status

**PARTIAL** — last audited 2026-04-25 (git SHA `ae1b9ec`).
28 of 30 behavioral specs implemented. Two active gaps: TINFRA-013 (depth cap) and TINFRA-047 (unconsumed mock check). Note: TINFRA-047 has a premature `@spec` annotation in `run-scenario.sh` — the check is only a placeholder.

## References

### HLD
- `docs/high-level-design.md` — Success Metrics section (all tests pass with zero warnings)

### LLD
- `docs/llds/test-infrastructure.md`

### EARS
- `docs/specs/test-infrastructure-specs.md`

### Tests
- This segment IS the test infrastructure; it is meta-tested only via the CI `mise ci` task.

### Code
- `qed-tests/src/main.rs`
- `qed-tests/src/manifest.rs`
- `qed-tests/src/runner.rs`
- `qed-tests/src/scenario.rs`
- `tests/harness/run-scenario.sh`
- `tests/harness/generate-mock.sh`
- `tests/harness/compare-golden.sh`

## Architecture

**Purpose:** Implements a two-layer integration test harness. The Rust layer (libtest-mimic) discovers manifests, registers one `Trial` per invocation index, generates `scenario.sh` variable files, and delegates to bash for execution. The bash layer sets up a temp directory, generates stateful mock scripts, evaluates the invocation string, and compares outputs against golden files.

**Key Components:**
1. `manifest.rs` — walks `tests/` to depth 2; deserializes `manifest.toml` files via serde/toml; supports `[[scenario]]` arrays with `invoke`, `env`, `mock` fields
2. `scenario.rs` — generates `scenario.sh` shell variable file from a `Scenario` + invocation index; emits `MOCK_{i}_*` variables; single-quotes invocation strings with internal-quote escaping
3. `runner.rs` — creates isolated temp dir per trial (name includes PID); symlinks/copies qed binary into `bin/`; execs `bash run-scenario.sh <tmpdir>`; removes temp dir unconditionally
4. `run-scenario.sh` — sources `scenario.sh`; copies input; generates mocks; evals `INVOCATION` in subshell; asserts exit code; calls `compare-golden.sh` for stdout/stderr/output
5. `generate-mock.sh` — bakes all `DECL_*` vars into a self-contained executable; call counter via `$MOCK_STATE_DIR/<cmd>.count`; dispatches on Nth call; validates args at call time
6. `compare-golden.sh` — dispatches on `.txt` (exact diff), `.pattern` (anchored full-string regex with `\n` resolution), `.*` (glob)

## Spec Coverage

| Category | Spec IDs | Implemented | Deferred | Gaps |
|---|---|---|---|---|
| Two-layer architecture | TINFRA-001–TINFRA-003 | 3 | 0 | 0 |
| Manifest discovery | TINFRA-010–TINFRA-013 | 3 | 0 | 1 (TINFRA-013) |
| Scenario fields | TINFRA-020–TINFRA-024 | 5 | 0 | 0 |
| Trial lifecycle | TINFRA-030–TINFRA-035 | 6 | 0 | 0 |
| Mock generation and validation | TINFRA-040–TINFRA-047 | 7 | 0 | 1 (TINFRA-047) |
| Golden comparison | TINFRA-050–TINFRA-053 | 4 | 0 | 0 |
| Non-features | TINFRA-060–TINFRA-061 | 0 | 2 | 0 |
| **Total** | | **28** | **2** | **2** |

**Summary:** 28 of 30 behavioral specs implemented. TINFRA-047 `@spec` annotation in `run-scenario.sh` is premature — the unconsumed-mock check is a placeholder comment only.

## Key Findings

1. **Unconsumed mock check unimplemented** — `run-scenario.sh:57` contains a placeholder comment ("Phase 7") for checking that all declared mock calls were consumed. The spec in `.claude/tests/harness.md` describes this behavior but it is not yet enforced. Mocks can be under-called silently.
2. **Manifest walk depth 2** — `manifest.rs:58` caps directory depth at 2, which supports nested suites like `tests/usecases/code-editing/`. Suites nested deeper than 2 levels would be silently skipped.
3. **Total isolation per trial** — Each trial gets its own temp dir (with PID in name for uniqueness); the dir is removed unconditionally on completion, pass or fail (`runner.rs`). No shared state between trials.
4. **`.pattern` golden comparison** — Reads actual output as a single string; resolves literal `\n` sequences to real newlines before applying an anchored bash `[[ =~ ]]` regex (`compare-golden.sh:29–30`). Uses RE2 syntax.
5. **`.*` glob golden** — Matches zero or more golden files; errors if the glob resolves to zero matches (`compare-golden.sh:56–70`). Used for non-deterministic outputs like uuid-v5 (deterministic but not pinned).
6. **Catch-all exact match** — Golden files with any extension other than `.pattern` or `.*` are treated as exact-text comparisons, not just `.txt` files. This is wider than the spec implies.
7. **`find_qed_binary()` panics with actionable message** — If the qed binary is not built, the test harness panics with a message directing the user to run `cargo build --bin qed` (`runner.rs:7–29`). Not a silent failure.

## Work Required

### Must Fix
*(none — harness is fully functional for current test suite)*

### Should Fix
1. Implement unconsumed mock check in `run-scenario.sh` (TINFRA specs TBD). Currently specified but not enforced; mocks can be under-called without test failure.

### Nice to Have
1. Consider making manifest walk depth configurable rather than hard-coded at 2.
