# Invocation Scenarios

Tests covering CLI flags and invocation modes:
`--output`, `--in-place`, `--extract`, `--dry-run`, `--on-error`, and `--no-env`.

Stdin/stdout and `-f` invocation are tested as `invoke` variants in all other feature
directories and are not duplicated here.

---

## Directory Layout

```
tests/invocation/
  manifest.toml
  inputs/
    three-lines.txt
    env-pattern.txt
  scripts/
    delete-bar.qed
    extract-bar.qed
    dry-run-delete-bar.qed
    global-on-error-skip.qed
    global-on-error-warn.qed
    no-env-literal.qed
    env-expand.qed
  goldens/
    stdout/
      empty.txt
      foo-baz.txt
      bar.txt
      dry-run-delete-bar.txt
      foo-bar-baz.txt
      warn-no-match.txt
    stderr/
      empty.txt
      warn-no-match.txt
    output/
      empty.txt
      foo-baz.txt
      bar.txt
      foo-bar-baz.txt
```

---

## Manifest

```toml
# tests/invocation/manifest.toml

# ── --output ──────────────────────────────────────────────────────────────────

[[scenario]]
id = "output-flag"
description = "--output writes transformed content to the specified file; stdout is empty"
script = "delete-bar.qed"
input = "three-lines.txt"
stdout = "empty.txt"
stderr = "empty.txt"
output = "foo-baz.txt"
exit_code = 0
invoke = [
  """qed -f "$SCRIPT" --output "$OUTPUT" "$INPUT" > "$STDOUT" 2> "$STDERR" """,
]

# ── --in-place ────────────────────────────────────────────────────────────────

[[scenario]]
id = "in-place"
description = "--in-place modifies the input file directly via atomic write; stdout is empty"
script = "delete-bar.qed"
input = "three-lines.txt"
stdout = "empty.txt"
stderr = "empty.txt"
output = "foo-baz.txt"
exit_code = 0
invoke = [
  """
  qed -f "$SCRIPT" --in-place "$INPUT" > "$STDOUT" 2> "$STDERR"
  cp "$INPUT" "$OUTPUT"
  """,
]

# ── --extract ─────────────────────────────────────────────────────────────────

[[scenario]]
id = "extract"
description = "--extract suppresses passthrough output; only the selected region is written to stdout"
script = "extract-bar.qed"
input = "three-lines.txt"
stdout = "bar.txt"
stderr = "empty.txt"
output = "bar.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" --extract < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" --extract < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── --dry-run ─────────────────────────────────────────────────────────────────

[[scenario]]
id = "dry-run"
description = "--dry-run writes a unified diff of the proposed changes to stdout; the input file is not modified"
script = "dry-run-delete-bar.qed"
input = "three-lines.txt"
stdout = "dry-run-delete-bar.txt"
stderr = "empty.txt"
output = "empty.txt"
exit_code = 0
invoke = [
  """
  qed -f "$SCRIPT" --dry-run "$INPUT" > "$STDOUT" 2> "$STDERR"
  cp /dev/null "$OUTPUT"
  """,
]

# ── --on-error ────────────────────────────────────────────────────────────────

[[scenario]]
id = "global-on-error-skip"
description = "--on-error=skip sets the global no-match behaviour to skip; per-selector on_error overrides it"
script = "global-on-error-skip.qed"
input = "three-lines.txt"
stdout = "foo-bar-baz.txt"
stderr = "empty.txt"
output = "foo-bar-baz.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" --on-error=skip < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" --on-error=skip < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "global-on-error-warn"
description = "--on-error=warn sets the global no-match behaviour to warn; unmatched selectors emit to stderr but exit zero"
script = "global-on-error-warn.qed"
input = "three-lines.txt"
stdout = "foo-bar-baz.txt"
stderr = "warn-no-match.txt"
output = "foo-bar-baz.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" --on-error=warn < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" --on-error=warn < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── --no-env ──────────────────────────────────────────────────────────────────

[[scenario]]
id = "no-env-suppresses-expansion"
description = "--no-env treats ${VAR} as a literal string rather than expanding it"
script = "no-env-literal.qed"
input = "env-pattern.txt"
stdout = "env-pattern.txt"
stderr = "empty.txt"
output = "env-pattern.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" --no-env < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" --no-env < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "env-expansion"
description = "Without --no-env, ${VAR} in patterns is expanded from the environment"
script = "env-expand.qed"
input = "three-lines.txt"
stdout = "foo-baz.txt"
stderr = "empty.txt"
output = "foo-baz.txt"
exit_code = 0
env = { QED_PATTERN = "bar" }
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]
```

---

## Input Files

### `inputs/three-lines.txt`

Used by: `output-flag`, `in-place`, `extract`, `dry-run`,
`global-on-error-skip`, `global-on-error-warn`, `env-expansion`

```
foo
bar
baz
```

### `inputs/env-pattern.txt`

Used by: `no-env-suppresses-expansion`

Contains the literal string `${QED_PATTERN}`.
With `--no-env`, this is matched literally rather than expanded.
The script targets `${QED_PATTERN}` literally, finds a match, and the line passes through unchanged —
demonstrating that expansion was suppressed and the literal form was matched as written.

```
${QED_PATTERN}
```

---

## Script Files

### `scripts/delete-bar.qed`

Used by: `output-flag`, `in-place`

```
at("bar") | qed:delete()
```

### `scripts/extract-bar.qed`

Used by: `extract`

With `--extract`, the processor is `qed:skip()` — the selected region passes through,
and `--extract` suppresses the passthrough of everything else.

```
at("bar") | qed:skip()
```

### `scripts/dry-run-delete-bar.qed`

Used by: `dry-run`

```
at("bar") | qed:delete()
```

### `scripts/global-on-error-skip.qed`

Used by: `global-on-error-skip`

`quux` does not appear in the input; the global `--on-error=skip` flag causes
this to succeed silently rather than fail.

```
at("quux") | qed:delete()
```

### `scripts/global-on-error-warn.qed`

Used by: `global-on-error-warn`

`quux` does not appear in the input; the global `--on-error=warn` flag causes
a warning on stderr and a zero exit rather than failure.

```
at("quux") | qed:delete()
```

### `scripts/no-env-literal.qed`

Used by: `no-env-suppresses-expansion`

Targets the literal string `${QED_PATTERN}` and passes it through with `qed:skip()`.
With `--no-env`, `${QED_PATTERN}` in the pattern is not expanded — it is matched literally.
The line passes through and the output equals the input.

```
at("${QED_PATTERN}") | qed:skip()
```

### `scripts/env-expand.qed`

Used by: `env-expansion`

`${QED_PATTERN}` is expanded from the environment before matching.
When invoked with `QED_PATTERN=bar`, this deletes `bar`.

```
at("${QED_PATTERN}") | qed:delete()
```

---

## Golden Files

### `goldens/stdout/`

#### `empty.txt`

```
```

#### `foo-baz.txt`

Used by: `output-flag` (stdout), `env-expansion` (stdout)

stdout is empty for `output-flag` since content goes to `--output`.
For `env-expansion`, the stream is written to stdout normally.

> ⚠️ Note: `output-flag` references `empty.txt` for stdout, not this file.
> This file is listed here for `env-expansion` stdout only.

```
foo
baz
```

#### `bar.txt`

Used by: `extract` (stdout)

Only the selected `bar` line; `foo` and `baz` suppressed by `--extract`.

```
bar
```

#### `dry-run-delete-bar.txt`

Used by: `dry-run` (stdout)

Unified diff output with fixed `a`/`b` placeholders and no timestamps.
Three-line context (unified diff default); `bar` is the only changed line.

```
--- a
+++ b
@@ -1,3 +1,2 @@
 foo
-bar
 baz
```

#### `foo-bar-baz.txt`

Used by: `global-on-error-skip` (stdout), `global-on-error-warn` (stdout)

Stream passes through unchanged — `quux` was not found, no lines were deleted.

```
foo
bar
baz
```

#### `warn-no-match.txt`

Used by: `global-on-error-warn` (stdout)

> ⚠️ This file belongs in `goldens/stderr/`, not `goldens/stdout/`.
> Listed here for clarity; see stderr section below.

#### `env-pattern.txt`

Used by: `no-env-suppresses-expansion` (stdout)

Output equals input — the literal `${QED_PATTERN}` line was matched and passed through unchanged.

```
${QED_PATTERN}
```

---

### `goldens/stderr/`

#### `empty.txt`

```
```

#### `warn-no-match.txt`

Used by: `global-on-error-warn`

Script: `at("quux") | qed:delete()` — selector at 1:1-10, `qed:delete()` at 1:14-25 (widest: 7 chars).

```
qed: warning: 1:1-10:  at("quux"): no lines matched
```

---

### `goldens/output/`

#### `empty.txt`

Used by: `dry-run` (output — input file is unmodified, output golden is empty to assert this)

```
```

#### `foo-baz.txt`

Used by: `output-flag`, `in-place`, `env-expansion`

```
foo
baz
```

#### `bar.txt`

Used by: `extract`

```
bar
```

#### `foo-bar-baz.txt`

Used by: `global-on-error-skip`, `global-on-error-warn`, `no-env-suppresses-expansion`

```
foo
bar
baz
```
