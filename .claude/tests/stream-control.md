# Stream Control Scenarios

Tests covering `qed:warn()`, `qed:fail()`, `qed:skip()`,
`qed:debug:count()`, and `qed:debug:print()`.

---

## Directory Layout

```
tests/stream-control/
  manifest.toml
  inputs/
    three-lines.txt
  scripts/
    warn.qed
    fail.qed
    skip.qed
    debug-count.qed
    debug-print.qed
  goldens/
    stdout/
      empty.txt
      foo-bar-baz.txt
    stderr/
      empty.txt
      warn-region.txt
      fail-region.txt
      debug-count.txt
      debug-print.txt
    output/
      empty.txt
      foo-bar-baz.txt
```

---

## Manifest

```toml
# tests/stream-control/manifest.toml

# ── qed:warn() ────────────────────────────────────────────────────────────────

[[scenario]]
id = "warn"
description = "qed:warn() emits the selected region to stderr and continues; the stream is unaffected"
script = "warn.qed"
input = "three-lines.txt"
stdout = "foo-bar-baz.txt"
stderr = "warn-region.txt"
output = "foo-bar-baz.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── qed:fail() ────────────────────────────────────────────────────────────────

[[scenario]]
id = "fail"
description = "qed:fail() exits non-zero; lines before the selected region pass through, lines after are blocked"
script = "fail.qed"
input = "three-lines.txt"
stdout = "foo.txt"
stderr = "fail-region.txt"
output = "foo.txt"
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

# ── qed:skip() ────────────────────────────────────────────────────────────────

[[scenario]]
id = "skip"
description = "qed:skip() is a no-op passthrough; the selected region passes through unchanged"
script = "skip.qed"
input = "three-lines.txt"
stdout = "foo-bar-baz.txt"
stderr = "empty.txt"
output = "foo-bar-baz.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── qed:debug:count() ─────────────────────────────────────────────────────────

[[scenario]]
id = "debug-count"
description = "qed:debug:count() emits the match count to stderr; the stream is unaffected"
script = "debug-count.qed"
input = "three-lines.txt"
stdout = "foo-bar-baz.txt"
stderr = "debug-count.txt"
output = "foo-bar-baz.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── qed:debug:print() ─────────────────────────────────────────────────────────

[[scenario]]
id = "debug-print"
description = "qed:debug:print() echoes the selected region to stderr; the stream is unaffected"
script = "debug-print.qed"
input = "three-lines.txt"
stdout = "foo-bar-baz.txt"
stderr = "debug-print.txt"
output = "foo-bar-baz.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]
```

---

## Input Files

### `inputs/three-lines.txt`

Used by: all scenarios

```
foo
bar
baz
```

---

## Script Files

### `scripts/warn.qed`

Emits `bar` to stderr; stream passes through unchanged.

```
at("bar") | qed:warn()
```

### `scripts/fail.qed`

Matches `bar` and triggers an immediate non-zero exit.

```
at("bar") | qed:fail()
```

### `scripts/skip.qed`

`bar` is selected but passed through unchanged.

```
at("bar") | qed:skip()
```

### `scripts/debug-count.qed`

Emits the count of lines matching `bar` to stderr; stream unaffected.

```
at("bar") | qed:debug:count()
```

### `scripts/debug-print.qed`

Echoes the `bar` line to stderr; stream unaffected.

```
at("bar") | qed:debug:print()
```

---

## Golden Files

### `goldens/stdout/` and `goldens/output/`

#### `empty.txt`

Used by: (none currently — see `foo.txt` for `fail`)

```
```

#### `foo.txt`

Used by: `fail` (stdout, output)

`foo` passes through before `bar`'s tagged region is reached;
`baz` is blocked behind it when `qed:fail()` fires.

```
foo
```

#### `foo-bar-baz.txt`

Used by: `warn`, `skip`, `debug-count`, `debug-print`

```
foo
bar
baz
```

---

### `goldens/stderr/`

#### `empty.txt`

```
```

#### `warn-region.txt`

Used by: `warn`

`qed:warn()` emits the selected region to stderr as raw content. The matched line is `bar`.

```
bar
```

#### `fail-region.txt`

Used by: `fail`

`qed:fail()` emits the selected region to stderr before exiting non-zero. The matched line is `bar`.

```
bar
```

#### `debug-count.txt`

Used by: `debug-count`

Script: `at("bar") | qed:debug:count()` — selector at 1:1-9, `qed:debug:count()` at 1:13-29 (widest: 7 chars).
`at("bar")` matches 1 line in `three-lines.txt`.

```
qed: debug:   1:1-9:   at("bar"): 1 match
```

#### `debug-print.txt`

Used by: `debug-print`

`qed:debug:print()` echoes the selected region to stderr as raw content. The matched line is `bar`.

```
bar
```
