# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

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

[0.1.0]: https://github.com/boldlygodev/qed/releases/tag/v0.1.0
