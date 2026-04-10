# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [0.5.0] — 2026-04-10

Release polish (Alpha 5).

### Added

- **`--version`** flag — print `qed <version>`
- **`--completions <shell>`** hidden flag — generate shell completions for
  `bash`, `zsh`, and `fish` via `clap_complete`
- **Backslash line continuation** — trailing `\<newline>` joins lines in
  external processor args; trailing whitespace after `\` is a parse error

### Changed

- **Parallelized selector match collection** — `collect_all_matches` now uses
  `rayon::par_iter` for faster fragmentation on large inputs
- Parse errors are now formatted as structured diagnostics (consistent with
  compile/runtime diagnostics) instead of raw `Debug` output
- Single-char and zero-width diagnostic spans display as `1:17` instead of
  `1:17-17`
- README refreshed: Alpha messaging, installation status, corrected
  `--extract` example, fixed logo asset path

### Fixed

- Test harness `generate-mock.sh` — `printf '%s'` replaces `echo -e` to avoid
  arg validation issues on some shells

## [0.4.0] — 2026-03-30

Feature complete (Alpha 4) — all 396 integration tests passing.

### Added

- **`qed:file()`** processor — compile-time chain fusion with external
  processors for temp-file handoff via `${QED_FILE}` substitution
- **`qed:warn()`** — emit selected text to stderr, pass through
- **`qed:fail()`** — emit selected text to stderr, halt with non-zero exit
- **`qed:debug:count()`** — count matches, emit diagnostic
- **`qed:debug:print()`** — echo selected text to stderr, pass through
- Selector-level fallback dispatch — `|| selector | processor` retries against
  the full buffer when a statement's selector produces no match
- `CompileWarning::NthZeroTerm` — `nth:0` now emits a warning instead of a
  hard parse error
- `CompileWarning::NthDuplicate` — duplicate terms in `nth:1,1` or
  overlapping ranges emit a warning
- Cross-type name redefinition warning — reusing a pattern name for an alias
  (or vice versa) is now reported
- CI integration test job

### Changed

- **Compound selector pairing** — `from > to` now uses a nearest-next
  algorithm with same-pattern fence handling (skips `from`-matches inside
  previous ranges)
- **Re-fragmentation** — after a processor transforms text, remaining tagged
  processors only run if their selector still matches the transformed output
- Negative-step nth expressions (`nth:-2n`) now count from the end
- Diagnostic locations are padded to uniform width across a run
- `CompoundSelector` threads `on_error` through compilation
- Partial output is preserved when a statement errors mid-run
- Test harness treats non-`.pattern` golden extensions as text diffs
- Pre-commit task restored to the full test suite

### Fixed

- Alias-forward-ref script files no longer eat trailing newlines
- `from > to` pairing no longer interleaves overlapping matches
- Portable `sed` invocation in `generate-mock.sh` for Linux CI

## [0.3.1] — 2026-03-27

Generation processors and full CLI invocation features (Alpha 3).

### Added

- **`qed:random()`** — generate random integers, floats, or strings
  with configurable `type:`, `min:`, `max:`, `length:`, and `charset:`
- **`qed:uuid()`** — generate UUIDs (v4, v5, v7) with optional `version:`
  and `namespace:`/`name:` for deterministic v5
- **`qed:timestamp()`** — generate timestamps with configurable `format:`
  and `timezone:`
- **`--output`** (`-o`) flag — write output to a file
- **`--in-place`** (`-i`) flag — edit input files in place with atomic writes
- **`--dry-run`** (`-d`) flag — preview changes as a unified diff
- **`--no-env`** flag — disable environment variable expansion in patterns
- **`--on-error`** flag — control error behavior (`stop` or `continue`)
- **`--extract`** (`-x`) flag — emit only selected regions
- **`--file`** (`-f`) flag — read script from a file
- Input file positional argument — read input from a file instead of stdin

### Changed

- `NeverMatch` selector used for env var patterns referencing unset variables
  (instead of failing), allowing graceful degradation
- Test harness supports multiline `.pattern` golden matching

### Fixed

- Multiline `.pattern` golden files now match correctly across line boundaries

## [0.2.0] — 2026-03-22

Full processor suite (Alpha 2).

### Added

- **`qed:copy()`** — copy selected region to a destination (`after:`, `before:`, or `at:`)
- **`qed:move()`** — move selected region to a destination (removes original)
- **`qed:substring()`** — narrow each line to the first span matching a literal or regex
- **`qed:number()`** — prefix each line with its line number, with optional `start:` and `width:`
- **`qed:indent()`** — prepend indentation per line (`width:` required, `char:` optional)
- **`qed:dedent()`** — remove common leading whitespace from all lines
- **`qed:wrap()`** — word-wrap at a specified column `width:`
- **`qed:suffix()`** — append text to each line
- **`qed:duplicate()`** — emit selected region twice
- **`qed:skip()`** — no-op passthrough
- **`qed:trim()`** — strip leading and trailing whitespace from each line

### Changed

- `qed:prefix()` now operates per-line instead of on the whole region
- `StatementAction` enum distinguishes regular processors from copy/move operations

### Fixed

- Parser handles params-only selectors like `at(on_error:skip)` without a pattern
- Integer parameter values (`width:4`) now parse correctly for `qed:indent()`, `qed:wrap()`, etc.
- `qed:number()` returns empty output on empty input

## [0.1.0] — 2026-03-21

Initial alpha release.

### Added

- **Select-action model** — `selector | processor` as the core language primitive
- **Selectors** — `at`, `after`, `before`, `from`, `to`, and range composition with `>`
- **Pattern matching** — string literals, regex, named patterns, negation (`!`), inclusive boundaries (`+`)
- **Nth qualifiers** — numeric indexing, ranges, even/odd, negative indexing
- **Built-in processors** — `qed:delete()`, `qed:replace()`, `qed:upper()`, `qed:lower()`,
  `qed:duplicate()`, `qed:skip()`, `qed:substring()`, `qed:trim()`,
  `qed:indent()`, `qed:dedent()`, `qed:wrap()`, `qed:prefix()`, `qed:suffix()`,
  `qed:number()`, `qed:warn()`, `qed:fail()`
- **External processors** — pipe selected regions through any command on `PATH`
- **Named patterns** — `name=/regex/` pattern definitions for reuse across statements
- **Script files** — `--file` flag for multi-statement scripts
- **In-place editing** — `--in-place` with atomic writes
- **Extract mode** — `--extract` to emit only selected regions
- **Diagnostics** — structured warnings and errors to stderr with source locations
- **Recursive descent parser** — hand-written parser with clear error messages

[0.5.0]: https://github.com/boldlygodev/qed/releases/tag/v0.5.0
[0.4.0]: https://github.com/boldlygodev/qed/releases/tag/v0.4.0
[0.3.1]: https://github.com/boldlygodev/qed/releases/tag/v0.3.1
[0.2.0]: https://github.com/boldlygodev/qed/releases/tag/v0.2.0
[0.1.0]: https://github.com/boldlygodev/qed/releases/tag/v0.1.0
