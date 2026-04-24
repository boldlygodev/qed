# External Integration

## Context and Design Philosophy

Bridges qed and external shell tools. Two processors implement the `Processor` trait for operations that delegate to subprocesses: `ExternalCommandProcessor` (stdin/stdout pipe) and `FileHandoffProcessor` (tempfile materialization). These are the only processors in qed-core with filesystem and subprocess side effects, and they carry distinct error semantics, resource management concerns, and behavioral contracts that do not belong in `text-transformation`.

The design philosophy is pragmatic delegation: qed selects and routes content; the external tool does the work. qed does not attempt to manage or constrain the external tool beyond capturing its output.

## ExternalCommandProcessor

`ExternalCommandProcessor { command: String, args: Vec<String> }` — spawns a subprocess, writes the selected text to its stdin, and reads stdout as the replacement.

**Execution flow:**
1. Spawn `Command::new(&self.command).args(&self.args).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped())`
2. Write selected text to stdin via `child.stdin.take()`; suppress write errors silently (`let _ = ...`) — the process may have already exited
3. `child.wait_with_output()` — collect stdout, stderr, and exit status
4. On success (exit 0): emit child stderr directly via `eprint!` (bypasses qed's diagnostic system); return stdout as replacement with trailing-newline normalization
5. On failure (non-zero exit): return `ProcessorError::ExternalFailed { command, exit_code: status.code(), stderr }` — `exit_code` is `None` on signal termination

**Trailing-newline normalization:** if the input ended with `'\n'` and stdout doesn't, append `'\n'`. Zero-width insertion points (empty input) use command output verbatim.

## FileHandoffProcessor and qed:file() Fusion

`qed:file()` is not a standalone processor at runtime — it is a compile-time sentinel that triggers fusion with the next external command in the chain.

**`FileMarker { span: Span }`** — the compile-time representation of `qed:file()`. `is_file_marker()` returns `true`, allowing the compiler (`compile/mod.rs:1021–1056`) to detect and fuse it with the immediately following external command into a `FileHandoffProcessor`.

**`FileHandoffProcessor { command: String, raw_args: Vec<String> }`** — the fused runtime processor. `raw_args` contain unexpanded `${QED_FILE}` tokens:

**Execution flow:**
1. Write selected text to a `NamedTempFile`
2. Convert to `TempPath` (disables auto-delete on drop)
3. For each arg in `raw_args`: substitute `${QED_FILE}` with the tempfile path
4. Set `QED_FILE` environment variable in the child process (supports tools that read from env rather than positional args)
5. Spawn subprocess with substituted args; capture stdout/stderr/exit
6. `drop(tmp_path)` — explicit cleanup after subprocess completes
7. Same success/failure and newline-normalization logic as `ExternalCommandProcessor`

**Scoping:** `${QED_FILE}` is scoped to the immediately downstream command only. In a pipeline `qed:file() | cmd1 | cmd2`, only `cmd1` receives `${QED_FILE}`; `cmd2` receives stdin normally.

**Insertion point warning:** if `qed:file()` is used on a zero-width insertion point, the engine emits `ProcessorError::FileEmptyRegion` — the empty region cannot be meaningfully written to a tempfile.

## Error Handling

Both processors produce `ProcessorError::ExternalFailed` on any subprocess failure. This conflates two distinct failure modes:
- **Spawn failure** (qed could not exec the command — command not found, permission denied)
- **Command failure** (command ran but returned non-zero)

A `ProcessorFailed` variant exists but is not used for subprocess spawn failures; this is a known gap.

Child stderr behavior:
- **On success:** emitted directly to the qed process stderr via `eprint!`; not captured as a `RunDiagnostic`
- **On failure:** captured into `ExternalFailed.stderr`; emitted as part of the diagnostic message

**No timeout:** neither processor sets a timeout or resource limit on spawned processes.

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| Stdin write errors suppressed | `let _ = stdin.write_all(...)` | Propagate as error | Process may exit before consuming all stdin (e.g. `head -n 1`); treating this as a fatal error would break legitimate use cases. [inferred] |
| Child stderr emitted via `eprint!` on success | Bypass diagnostic system | Capture into `RunDiagnostic` | External tools may emit progress/status to stderr as a normal part of operation; routing through qed's diagnostic system would add noise. [inferred] |
| Explicit `drop(tmp_path)` for tempfile cleanup | `drop()` after subprocess | Rely on `NamedTempFile::Drop` | `TempPath` conversion was already done to pass the path to the subprocess; explicit drop makes the intent clear and avoids subtle Drop-timing questions. [inferred] |
| `qed:file()` fusion at compile time | `FileMarker` + compiler fusion | Runtime detection | Compile-time fusion allows the compiler to validate chain position and emit `FileEmptyRegion` warnings statically. [inferred] |
| `${QED_FILE}` scoped to next command only | Single-command scope | Global scope for the whole chain | Prevents accidental file reuse by downstream commands; the scope is documented and tested. [inferred] |

## Open Questions & Future Decisions

### Resolved
*(none yet)*

### Deferred
1. **No subprocess timeout** — Should a configurable timeout (off by default) be added to prevent infinite hangs on misbehaving external tools?
2. **Spawn vs command failure distinction** — Should spawn failures produce `ProcessorFailed` (qed plumbing error) rather than `ExternalFailed` (command behavior error)?
3. **Child stderr routing** — Should successful-subprocess stderr be routable through `RunDiagnostic` for callers who need structured output?

## References

- `qed-core/src/processor/external.rs`
- `qed-core/src/processor/file.rs`
- `docs/qed-design.md` — external processor and qed:file() specifications
- `docs/arrows/external-integration.md`
- `docs/specs/external-integration-specs.md`
