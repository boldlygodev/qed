# CLI

## Context and Design Philosophy

A thin shell over `qed_core::run()`. The CLI crate (`qed/`) handles everything that is inherently process-boundary work: argument parsing, I/O routing, file operations, diagnostic formatting, exit codes, and shell completions. It contains no domain logic; all qed semantics live in `qed-core`. Any function in this crate that does more than I/O wiring or formatting is a candidate for moving to `qed-core`.

## Argument Parsing and Input Routing

`Cli` is a clap `Parser` derive struct. Positional `args` interpretation depends on whether `-f`/`--file` is set:

- **With `-f`:** `args` is zero or one file path (the input file); script is read from the `-f` path
- **Without `-f`:** `args[0]` is the inline script string; `args[1]` (optional) is the input file path

Input is read entirely into memory before calling `qed_core::run()` — no streaming.

Conflicts enforced by clap: `--in-place` cannot be combined with `--output` or `--dry-run`.

## Output Routing

Four output modes, mutually exclusive by construction:

1. **Default (stdout)** — `RunResult.output` written to stdout
2. **`--output <file>`** — `RunResult.output` written to the specified file; stdout receives nothing
3. **`--in-place`** — atomic replacement of the input file (see below)
4. **`--dry-run`** — `unified_diff(original, modified)` written to stdout; no file modified

**`--extract`** is orthogonal to output routing: when set, `RunOptions.extract = true` is passed to `qed_core::run()`, which causes the engine to emit only selected lines.

## In-Place Editing

Atomic write via a sibling temp file:

1. Write `RunResult.output` to `<input_path>.qed-tmp`
2. `fs::rename("<input_path>.qed-tmp", input_path)` — atomic on POSIX filesystems
3. On rename failure: remove the temp file; return error (original file untouched)

No backup is kept. If qed-core produces an error (`has_errors = true`), the in-place write is skipped entirely — the original file is left unmodified.

## Dry-Run Diff

`diff.rs` exports one function:

```rust
pub(crate) fn unified_diff(original: &str, modified: &str) -> String
```

Wraps `similar::TextDiff`. Returns empty string when `original == modified` (no diff header emitted). Uses fixed `a`/`b` diff headers for reproducibility. `missing_newline_hint(false)` suppresses the `"\ No newline at end of file"` annotation.

## Diagnostic Formatting

Diagnostics from `RunResult.diagnostics` are formatted to stderr as:

```
qed: {level:<9}{loc}: [{selector}: ]{message}
```

where `level` is padded to 9 characters with a colon (e.g. `"error:   "`, `"warning: "`). The `selector` field is omitted when empty. `RunResult.stderr_lines` (from `qed:warn()`, `qed:fail()`, subprocess stderr) are emitted raw to stderr without additional formatting.

## Exit Code Semantics

| Code | Meaning |
|---|---|
| `0` | Success — script ran, output produced |
| `1` | Script execution error — qed ran but `RunResult.has_errors` is true |
| `2` | Usage or I/O error — bad arguments, file not found, write failure |

## Shell Completions

`--completions <shell>` (hidden from `--help`) generates shell completions via `clap_complete` and writes to stdout. Supported shells: bash, zsh, fish, elvish, powershell. The flag is hidden because completions are typically installed once during setup, not used in daily operation.

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| Atomic in-place via `.qed-tmp` + rename | Sibling temp file + `rename` | Write directly to original; `tempfile` crate in-place | `rename` is atomic on POSIX; sibling file ensures same filesystem (avoids cross-device rename failure). [inferred] |
| No backup on in-place | No `.bak` file | Keep `.bak` sibling | Avoids polluting the working directory; users are expected to use version control. [inferred] |
| Two-level exit codes (1 = script, 2 = I/O) | `1` and `2` | Single `1` for all errors | Allows callers to distinguish script failures (recoverable by fixing the script) from I/O failures (recoverable by fixing the invocation). [inferred] |
| Fixed `a`/`b` diff headers | Hard-coded `"a"` / `"b"` | Actual file path labels | Makes dry-run output reproducible and testable via exact golden files. [inferred] |
| `--completions` hidden from `--help` | `#[clap(hide = true)]` | Visible in `--help` | Completions are a setup-time feature; surfacing them in `--help` adds noise for daily use. [inferred] |
| Load entire input into memory | `read_to_string` | Line-by-line streaming | `qed-core` requires the full input for range selectors; streaming is not possible. [inferred] |

## Open Questions & Future Decisions

### Resolved
*(none)*

### Deferred
1. **`--completions` discoverability** — Should completions be a visible subcommand (e.g. `qed completions bash`) rather than a hidden flag? More discoverable but adds a subcommand to the interface.
2. **No-backup in-place** — Should `--backup` be offered as an opt-in flag for users who want a `.bak` sibling without relying on version control?

## References

- `qed/src/main.rs`
- `qed/src/diff.rs`
- `docs/qed-dev-workflow.md` — build and invocation commands
- `docs/arrows/cli.md`
- `docs/specs/cli-specs.md`
