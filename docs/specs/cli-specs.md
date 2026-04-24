# EARS Specs — CLI

ID prefix: `CLI`

## Argument Parsing and Input Routing

- [x] CLI-001: WHEN `--file`/`-f` is provided, the CLI SHALL read the qed script from the file at the `-f` path; `args` SHALL be interpreted as zero or one input file path.
- [x] CLI-002: WHEN `--file`/`-f` is not provided, `args[0]` SHALL be the inline script string and `args[1]` (optional) SHALL be the input file path.
- [x] CLI-003: The CLI SHALL read the full input into memory before calling `qed_core::run()`; streaming input is not supported.
- [x] CLI-004: The CLI SHALL enforce via clap that `--in-place` cannot be combined with `--output` or `--dry-run`.

## Output Routing

- [x] CLI-010: WHEN no output flag is provided, the CLI SHALL write `RunResult.output` to stdout.
- [x] CLI-011: WHEN `--output <file>` is provided, the CLI SHALL write `RunResult.output` to the specified file; stdout SHALL receive nothing.
- [x] CLI-012: WHEN `--in-place` is provided, the CLI SHALL atomically replace the input file with `RunResult.output` (see In-Place Editing).
- [x] CLI-013: WHEN `--dry-run` is provided, the CLI SHALL write a unified diff of `(original, modified)` to stdout; no file SHALL be modified.
- [x] CLI-014: WHEN `--extract` is provided, the CLI SHALL set `RunOptions.extract = true`, causing `qed-core` to emit only selected lines.

## In-Place Editing

- [x] CLI-020: The CLI SHALL implement in-place editing as an atomic write: first write `RunResult.output` to `<input_path>.qed-tmp`, then rename to the original path.
- [x] CLI-021: `fs::rename` SHALL be used for the atomic replacement, which is atomic on POSIX filesystems and ensures the temp file is on the same filesystem as the original.
- [x] CLI-022: WHEN the rename fails, the CLI SHALL remove the temp file and return an error; the original file SHALL be left unmodified.
- [x] CLI-023: WHEN `RunResult.has_errors` is true, the CLI SHALL skip the in-place write entirely; the original file SHALL be left unmodified.
- [D] CLI-024: The CLI SHALL NOT keep a backup of the original file during in-place edits; users are expected to use version control.

## Dry-Run Diff

- [x] CLI-030: `unified_diff(original, modified)` SHALL wrap `similar::TextDiff` and return a unified diff string.
- [x] CLI-031: WHEN `original == modified`, `unified_diff` SHALL return an empty string with no diff header.
- [x] CLI-032: Diff headers SHALL use fixed `"a"` / `"b"` labels rather than actual file paths to make dry-run output reproducible and testable via exact golden files.
- [x] CLI-033: `missing_newline_hint(false)` SHALL be set to suppress the `"\ No newline at end of file"` annotation.

## Diagnostic Formatting

- [x] CLI-040: Diagnostics from `RunResult.diagnostics` SHALL be formatted to stderr as: `qed: {level:<9}{loc}: [{selector}: ]{message}`.
- [x] CLI-041: The `level` field SHALL be padded to 9 characters with a colon suffix (e.g. `"error:   "`, `"warning: "`).
- [x] CLI-042: The `selector` field SHALL be omitted from the diagnostic line WHEN it is empty.
- [x] CLI-043: `RunResult.stderr_lines` SHALL be emitted raw to stderr without additional formatting.

## Exit Code Semantics

- [x] CLI-050: The CLI SHALL exit with code `0` WHEN the script ran successfully and output was produced.
- [x] CLI-051: The CLI SHALL exit with code `1` WHEN `RunResult.has_errors` is true (script execution error — qed ran but the script produced errors).
- [x] CLI-052: The CLI SHALL exit with code `2` WHEN a usage or I/O error occurs (bad arguments, file not found, write failure).

## Shell Completions

- [x] CLI-060: `--completions <shell>` SHALL generate shell completions via `clap_complete` and write them to stdout.
- [x] CLI-061: `--completions` SHALL support the shells: bash, zsh, fish, elvish, powershell.
- [x] CLI-062: `--completions` SHALL be hidden from `--help` output via `#[clap(hide = true)]`.

## Non-Features

- [D] CLI-070: The CLI SHALL NOT contain domain logic; all qed semantics SHALL live in `qed-core`.
- [D] CLI-071: `--completions` SHALL NOT be a visible subcommand; the hidden-flag form is the intended interface.

## References

- `qed/src/main.rs`
- `qed/src/diff.rs`
- `docs/llds/cli.md`
