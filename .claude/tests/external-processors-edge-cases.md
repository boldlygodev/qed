# External Processor Edge Case Scenarios

Additional scenarios covering boundary conditions in external processor behaviour.
These extend `tests/external-processors/` with new scenarios, inputs, scripts,
goldens, and mock declarations.

---

## New Inputs

### `inputs/three-lines.txt`

Already exists.

### `inputs/empty.txt`

Used by: `external-empty-input`

```
```

---

## New Scripts

### `scripts/external-empty-input.qed`

External command receives empty stdin — verifies the command is still called
and its stdout replaces the (empty) region.

```
at() | addline
```

### `scripts/external-empty-output.qed`

External command returns empty stdout — the selected region is replaced with nothing,
effectively deleting it.

```
at("bar") | devnull
```

### `scripts/external-nonzero-exit.qed`

External command exits non-zero — triggers `||` fallback.

```
at("bar") | failcmd || at("bar") | qed:delete()
```

### `scripts/external-nonzero-no-fallback.qed`

External command exits non-zero with no fallback — qed exits non-zero.

```
at("bar") | failcmd
```

### `scripts/file-then-stdin.qed`

`qed:file()` followed by a command that reads the file path, then a second command
in the chain that receives the first command's stdout via stdin.
Verifies that `${QED_FILE}` is scoped to the immediately downstream command only.

```
at("bar") | qed:file() | readfile "${QED_FILE}" | upcase
```

### `scripts/external-stderr-passthrough.qed`

External command writes to stderr — verifies qed passes it through to its own stderr.

```
at("bar") | noisycmd
```

### `scripts/file-on-insertion-point.qed`

`qed:file()` on an `after` insertion point — the region is always empty so materialization
is meaningless. `qed:file()` is warned and ignored; `qed:upper()` receives empty stdin and
produces empty output. Nothing is inserted, stream passes through.

```
after("bar") | qed:file() | qed:upper()
```

### `scripts/insertion-point-no-output.qed`

An external command fires at an `after` insertion point but writes nothing to stdout.
No line is inserted and the stream passes through unchanged.

```
after("bar") | sidefx
```

---

## New Manifest Scenarios

```toml
# Append to tests/external-processors/manifest.toml

# ── empty input / output edge cases ──────────────────────────────────────────

[[scenario]]
id = "external-empty-input"
description = "an external command receiving empty stdin is still called and its stdout replaces the region"
script = "external-empty-input.qed"
input = "empty.txt"
stdout = "inserted-line.txt"
stderr = "empty.txt"
output = "inserted-line.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario.mock]]
command = "addline"
input = "empty.txt"
stdout = "inserted-line.txt"

[[scenario]]
id = "external-empty-output"
description = "an external command returning empty stdout replaces the selected region with nothing"
script = "external-empty-output.qed"
input = "three-lines.txt"
stdout = "foo-baz.txt"
stderr = "empty.txt"
output = "foo-baz.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario.mock]]
command = "devnull"
input = "bar.txt"
stdout = "empty.txt"

# ── processor failure ─────────────────────────────────────────────────────────

[[scenario]]
id = "external-nonzero-triggers-fallback"
description = "an external command exiting non-zero triggers the || fallback"
script = "external-nonzero-exit.qed"
input = "three-lines.txt"
stdout = "foo-baz.txt"
stderr = "external-failed.txt"
output = "foo-baz.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario.mock]]
command = "failcmd"
input = "bar.txt"
exit_code = 1
stderr = "external-failed.txt"

[[scenario]]
id = "external-nonzero-no-fallback"
description = "an external command exiting non-zero with no fallback causes qed to exit non-zero"
script = "external-nonzero-no-fallback.qed"
input = "three-lines.txt"
stdout = "empty.txt"
stderr = "external-failed.txt"
output = "empty.txt"
exit_code = 1
invoke = [
  """
  qed "$(cat "$SCRIPT")" < "$INPUT" > "$STDOUT" 2> "$STDERR"
  QED_EXIT=$?
  cp "$STDOUT" "$OUTPUT"
  exit $QED_EXIT
  """,
  """
  qed -f "$SCRIPT" < "$INPUT" > "$STDOUT" 2> "$STDERR"
  QED_EXIT=$?
  cp "$STDOUT" "$OUTPUT"
  exit $QED_EXIT
  """,
]

[[scenario.mock]]
command = "failcmd"
input = "bar.txt"
exit_code = 1
stderr = "external-failed.txt"

# ── qed:file() scope ──────────────────────────────────────────────────────────

[[scenario]]
id = "file-then-stdin-pipeline"
description = "${QED_FILE} is scoped to the immediately downstream command; the next command in the chain receives stdout via stdin normally"
script = "file-then-stdin.qed"
input = "three-lines.txt"
stdout = "foo-bar-upper-baz.txt"
stderr = "empty.txt"
output = "foo-bar-upper-baz.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario.mock]]
command = "readfile"
expected_args = ["${QED_FILE}"]
input = "bar.txt"
stdout = "bar.txt"

[[scenario.mock]]
command = "upcase"
input = "bar.txt"
stdout = "bar-upper.txt"

# ── stderr passthrough ────────────────────────────────────────────────────────

[[scenario]]
id = "external-stderr-passthrough"
description = "stderr written by an external command is passed through to qed's own stderr"
script = "external-stderr-passthrough.qed"
input = "three-lines.txt"
stdout = "foo-baz.txt"
stderr = "noise.txt"
output = "foo-baz.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario.mock]]
command = "noisycmd"
input = "bar.txt"
stdout = "empty.txt"
stderr = "noise.txt"

# ── qed:file() warning ────────────────────────────────────────────────────────

[[scenario]]
id = "file-on-insertion-point"
description = "qed:file() on an after() insertion point is warned and ignored; the empty region passes through unchanged"
script = "file-on-insertion-point.qed"
input = "three-lines.txt"
stdout = "foo-bar-baz.txt"
stderr = "file-on-insertion-point.txt"
output = "foo-bar-baz.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── insertion-point side-effect-only ─────────────────────────────────────────

[[scenario]]
id = "insertion-point-no-output"
description = "an external command at an after() insertion point that writes nothing to stdout inserts nothing; stream passes through unchanged"
script = "insertion-point-no-output.qed"
input = "three-lines.txt"
stdout = "foo-bar-baz.txt"
stderr = "empty.txt"
output = "foo-bar-baz.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario.mock]]
command = "sidefx"
input = "empty.txt"
stdout = "empty.txt"
stderr = "empty.txt"
```

---

## New Mock Files

### `mocks/input/`

#### `empty.txt`

Used by: `addline` (external-empty-input), `sidefx` (insertion-point-no-output)

```
```

#### `bar.txt`

Used by: `devnull`, `failcmd`, `noisycmd`, `readfile`, `upcase`

```
bar
```

---

### `mocks/stdout/`

#### `inserted-line.txt`

Used by: `addline`

```
inserted
```

#### `bar-upper.txt`

Used by: `upcase` (file-then-stdin-pipeline)

```
BAR
```

---

### `mocks/stderr/`

#### `external-failed.txt`

Used by: `failcmd` (external-nonzero-triggers-fallback, external-nonzero-no-fallback)

This is the content `failcmd` writes to its own stderr before exiting non-zero.
Whether qed forwards child process stderr is an open concern —
the content here is what the mock emits, independent of qed's diagnostic.

```
failcmd: exiting with non-zero status
```

#### `noise.txt`

Used by: `noisycmd` (external-stderr-passthrough)

```
noise from noisycmd
```

---

## New Golden Files

### `goldens/stdout/` and `goldens/output/`

#### `foo-bar-baz.txt`

Already exists.
Reused by: `insertion-point-no-output`

Stream passes through entirely unchanged — `sidefx` writes nothing to stdout,
so nothing is inserted after `bar`.

#### `inserted-line.txt`

Used by: `external-empty-input`

`at()` on empty input selects the empty stream; `addline` mock output becomes
the entire stream content.

```
inserted
```

#### `foo-baz.txt`

Already exists.

#### `foo-bar-upper-baz.txt`

Used by: `file-then-stdin-pipeline`

`bar` uppercased via `readfile | upcase`; `foo` and `baz` pass through.

```
foo
BAR
baz
```

---

### `goldens/stderr/`

#### `external-failed.txt`

Used by: `external-nonzero-triggers-fallback`, `external-nonzero-no-fallback`

Script: `at("bar") | failcmd || …` — `failcmd` at 1:13-19, widest span `qed:delete()` at 1:36-47 (7 chars).
Note: whether qed forwards `failcmd`'s own stderr before this diagnostic is an open concern.

```
qed: error:   1:13-19: failcmd: exit code 1
```

#### `noise.txt`

Used by: `external-stderr-passthrough`

```
noise from noisycmd
```

#### `file-on-insertion-point.txt`

Used by: `file-on-insertion-point`

Script: `after("bar") | qed:file() | qed:upper()`
`qed:file()` at `1:16-26` (10 chars); widest spans `qed:upper()` at `1:29-40` (11 chars) — both location strings are 7 chars, no padding needed.

```
qed: warning: 1:16-26: qed:file(): qed:file() ignored for empty region
```

---

## Notes

### `external-empty-input` and `at()` on empty input

`at()` with no pattern selects the entire stream. When the stream is empty the
selected region is a zero-byte span — but it still exists. The mock receives empty
stdin, produces output, and that output becomes the stream content. This verifies
that `at()` on empty input does not short-circuit past the processor.

### `${QED_FILE}` scope in `file-then-stdin-pipeline`

The `readfile` mock declares `expected_args = ["${QED_FILE}"]` — it receives the
temp file path as its first argument and its stdout is `bar`. That stdout is then
passed as stdin to `upcase`. At the point `upcase` runs, `${QED_FILE}` is no longer
set in its environment — only the immediately downstream command of `qed:file()`
receives it. The `upcase` mock therefore declares only `input` and `stdout`, with
no `expected_args` referencing `${QED_FILE}`.

### `external-nonzero-triggers-fallback` and mock stderr

The `failcmd` mock writes to its own stderr, which qed passes through to its own
stderr. The scenario's `stderr` golden (`external-failed.txt`) therefore contains
the mock's stderr output rather than a qed-generated diagnostic — qed does not
suppress or wrap the child process's stderr, it passes it through directly.
The fallback succeeds and qed exits zero.
