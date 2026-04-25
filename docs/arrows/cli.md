# Arrow: cli

Command-line interface and diff display — the user-facing shell tool wrapping qed-core.

## Status

**OK** — last audited 2026-04-25 (git SHA `ae1b9ec`).
All 27 behavioral specs implemented and annotated. No active gaps.

## References

### HLD
- `docs/high-level-design.md` — Target Users section, Goals section (shell pipeline use)

### LLD
- `docs/llds/cli.md`

### EARS
- `docs/specs/cli-specs.md`

### Tests
- `tests/invocation/` — 8 scenarios (--output, --in-place, --extract, --dry-run, --on-error, --no-env)
- `tests/invocation-edge-cases/` — 8 scenarios (empty input, no-change, multiple hunks, per-selector override, unset env warn)

### Code
- `qed/src/main.rs`
- `qed/src/diff.rs`

## Architecture

**Purpose:** Thin CLI wrapper over `qed_core::run()`. Handles argument parsing (clap), input sourcing (stdin or file), output routing (stdout, `--output` file, `--in-place`), dry-run diff display, diagnostic formatting to stderr, shell completions, and process exit codes.

**Key Components:**
1. `Cli` struct — clap derive-based argument parser; flags: `-f`, `-i`, `-x`, `-o`, `-d`, `--on-error`, `--no-env`, `--completions`
2. Atomic in-place write — writes to `<input>.qed-tmp` sibling, then `rename`; removes temp on rename failure (`main.rs:138–149`)
3. `unified_diff()` — thin wrapper over `similar::TextDiff`; returns empty string when input equals output (`diff.rs`)
4. Diagnostic formatter — `"qed: {level:<9}{loc}: [{sel}: ]{msg}"` padded to 9 chars with colon (`main.rs:161–177`)
5. Exit codes — `0` success, `1` script execution error, `2` usage/I/O error (`main.rs:7–10`)

## Spec Coverage

| Category | Spec IDs | Implemented | Deferred | Gaps |
|---|---|---|---|---|
| Argument parsing and input routing | CLI-001–CLI-004 | 4 | 0 | 0 |
| Output routing | CLI-010–CLI-014 | 5 | 0 | 0 |
| In-place editing | CLI-020–CLI-024 | 4 | 1 (CLI-024) | 0 |
| Dry-run diff | CLI-030–CLI-033 | 4 | 0 | 0 |
| Diagnostic formatting | CLI-040–CLI-043 | 4 | 0 | 0 |
| Exit code semantics | CLI-050–CLI-052 | 3 | 0 | 0 |
| Shell completions | CLI-060–CLI-062 | 3 | 0 | 0 |
| Non-features | CLI-070–CLI-071 | 0 | 2 | 0 |
| **Total** | | **27** | **3** | **0** |

**Summary:** All 27 behavioral specs implemented and annotated. No active gaps.

## Key Findings

1. **Atomic in-place write** — Uses `.qed-tmp` sibling + rename for atomicity; no backup file is kept. On rename failure the temp file is cleaned up but the original is left unmodified (`main.rs:138–149`).
2. **Exit code semantics** — Exit 1 = script execution error (qed ran, something went wrong); exit 2 = usage/I/O error (bad args, file not found, etc.). This two-level scheme is intentional and testable.
3. **`INVOCATION` is `eval`'d** — The test harness evaluates invocation strings in a subshell (`run-scenario.sh:41`), allowing shell pipeline syntax. The CLI itself does not `eval` — this is a harness property.
4. **Diagnostic level padded to 9 chars** — `"error:   "`, `"warning: "`, `"debug:   "` (with trailing spaces) align message bodies across levels (`main.rs:161–177`).
5. **`--completions` is hidden** — The flag does not appear in `--help` output (`main.rs:55`); completions are generated at runtime via `clap_complete`.
6. **`--in-place` conflicts enforced by clap** — `--in-place` cannot be combined with `--output` or `--dry-run`; clap `conflicts_with` enforces this at argument-parse time.
7. **Dry-run uses fixed `a`/`b` diff headers** — Not filenames, for reproducibility (`diff.rs:15`). `missing_newline_hint(false)` suppresses the "\ No newline at end of file" annotation.

## Work Required

### Must Fix
*(none — CLI is complete and tested)*

### Should Fix
*(none identified)*

### Nice to Have
1. Expose `--completions` in `--help` output (or add a dedicated `qed completions <shell>` subcommand in a future revision).
