# EARS Specs — External Integration

ID prefix: `XINT`

## ExternalCommandProcessor

- [x] XINT-001: `ExternalCommandProcessor.execute()` SHALL spawn the command with stdin, stdout, and stderr all piped.
- [x] XINT-002: The processor SHALL write the selected text to the child process's stdin.
- [x] XINT-003: WHEN a stdin write error occurs (e.g., because the process exited early), the error SHALL be silently suppressed; it SHALL NOT be treated as a fatal failure.
- [x] XINT-004: The processor SHALL collect child stdout, stderr, and exit status via `child.wait_with_output()`.
- [x] XINT-005: WHEN the child process exits with code 0, the processor SHALL return child stdout as the replacement string.
- [x] XINT-006: WHEN the child process exits with a non-zero code or is killed by a signal, the processor SHALL return `ProcessorError::ExternalFailed { command, exit_code, stderr }`.
- [x] XINT-007: WHEN the child process exits successfully, any child stderr output SHALL be emitted directly to the qed process stderr via `eprint!`, bypassing the diagnostic system.
- [x] XINT-008: WHEN the child process fails, child stderr SHALL be captured into the `ExternalFailed.stderr` field and emitted as part of the diagnostic message.
- [x] XINT-009: WHEN the input ended with `'\n'` and child stdout does not, the processor SHALL append `'\n'` to normalize the trailing newline.
- [x] XINT-010: WHEN the input is a zero-width insertion point (empty string), the processor SHALL return child stdout verbatim without newline normalization.

## FileHandoffProcessor and qed:file() Fusion

- [x] XINT-020: `qed:file()` at the AST level SHALL be represented as a `FileMarker` sentinel; it SHALL NOT exist as a standalone runtime processor.
- [x] XINT-021: WHEN the compiler detects a `FileMarker` immediately followed by an external command in a processor chain, it SHALL fuse them into a `FileHandoffProcessor`.
- [x] XINT-022: `FileHandoffProcessor.execute()` SHALL write selected text to a `NamedTempFile` before spawning the child process.
- [x] XINT-023: `FileHandoffProcessor.execute()` SHALL substitute `${QED_FILE}` tokens in `raw_args` with the tempfile path before spawning.
- [x] XINT-024: `FileHandoffProcessor.execute()` SHALL set the `QED_FILE` environment variable in the child process to the tempfile path.
- [x] XINT-025: After the child process completes, `FileHandoffProcessor.execute()` SHALL explicitly `drop(tmp_path)` to clean up the tempfile.
- [x] XINT-026: `${QED_FILE}` substitution SHALL be scoped to the immediately downstream command only; further downstream commands in the same chain SHALL receive stdin normally.
- [x] XINT-027: WHEN `qed:file()` is used on a zero-width insertion point, the engine SHALL emit `ProcessorError::FileEmptyRegion` and treat it as a warning.
- [x] XINT-028: The same success/failure and trailing-newline normalization logic used in `ExternalCommandProcessor` SHALL also apply in `FileHandoffProcessor`.

## Error Handling

- [x] XINT-030: Both processors SHALL return `ProcessorError::ExternalFailed` for any subprocess failure, including spawn failure and non-zero exit.
- [ ] XINT-031: WHEN the subprocess cannot be spawned (command not found, permission denied), the processor SHOULD return `ProcessorError::ProcessorFailed` rather than `ProcessorError::ExternalFailed` to distinguish qed plumbing errors from command behavior errors.

## Non-Features

- [D] XINT-040: Neither processor SHALL set a timeout or resource limit on spawned subprocesses; hanging processes are the responsibility of the caller.
- [D] XINT-041: Child stderr from a successful subprocess SHALL NOT be routed through the `RunDiagnostic` system; direct `eprint!` emission is intentional.
- [D] XINT-042: `${QED_FILE}` SHALL NOT be made globally available to all commands in a pipeline chain; single-command scoping is intentional.

## References

- `qed-core/src/processor/external.rs`
- `qed-core/src/processor/file.rs`
- `docs/llds/external-integration.md`
