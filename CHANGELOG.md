# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

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

[0.3.0]: https://github.com/boldlygodev/qed/releases/tag/v0.3.0
[0.2.0]: https://github.com/boldlygodev/qed/releases/tag/v0.2.0
[0.1.0]: https://github.com/boldlygodev/qed/releases/tag/v0.1.0
