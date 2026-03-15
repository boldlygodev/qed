# Generation Edge Case Scenarios

Additional scenarios covering boundary conditions in generation processor behaviour.
These extend `tests/generation/` with new scenarios, inputs, scripts, and goldens.

---

## New Inputs

### `inputs/multi-placeholder.txt`

Used by: `uuid-multiple-placeholders`, `random-multiple-placeholders`

Multiple placeholder lines — verifies each is replaced independently.

```
{{uuid}}
{{uuid}}
```

### `inputs/mixed-placeholder.txt`

Used by: `timestamp-mixed-content`

A placeholder embedded mid-line alongside static content.

```
generated at {{ts}} by qed
```

---

## New Scripts

### `scripts/uuid-multiple-placeholders.qed`

Each `{{uuid}}` line is replaced independently; both should produce valid UUID v7 values
and they should differ from each other.

```
at(/\{\{uuid\}\}/) | qed:replace("{{uuid}}", qed:uuid())
```

### `scripts/timestamp-mixed-content.qed`

`{{ts}}` is embedded within a line of static content; the surrounding content
must survive the replacement.

```
at(/\{\{ts\}\}/) | qed:replace("{{ts}}", qed:timestamp(format:date))
```

### `scripts/random-length-boundary.qed`

`qed:random(1)` — minimum meaningful length.

```
at(/\{\{token\}\}/) | qed:replace("{{token}}", qed:random(1, alphabet:alnum))
```

### `scripts/random-large-length.qed`

`qed:random(128)` — verifies no length restriction is silently imposed.

```
at(/\{\{token\}\}/) | qed:replace("{{token}}", qed:random(128, alphabet:alnum))
```

### `scripts/uuid-before.qed`

`qed:uuid()` composing with `before()` — inserts a UUID before the matching line.

```
before("header") | qed:uuid()
```

### `scripts/timestamp-format-date.qed`

`qed:timestamp(format:date)` — `yyyy-MM-dd` format; deterministic structure, non-deterministic value.

```
at(/\{\{ts\}\}/) | qed:replace("{{ts}}", qed:timestamp(format:date))
```

### `scripts/timestamp-format-time.qed`

`qed:timestamp(format:time)` — `HH:mm:ss` format.

```
at(/\{\{ts\}\}/) | qed:replace("{{ts}}", qed:timestamp(format:time))
```

### `scripts/timestamp-format-unix-ms.qed`

`qed:timestamp(format:unix_ms)` — milliseconds since epoch.

```
at(/\{\{ts\}\}/) | qed:replace("{{ts}}", qed:timestamp(format:unix_ms))
```

### `scripts/timestamp-fixed-offset-tz.qed`

`qed:timestamp(timezone:"UTC+5:30")` — fixed offset, no DST.

```
at(/\{\{ts\}\}/) | qed:replace("{{ts}}", qed:timestamp(format:datetime, timezone:"UTC+5:30"))
```

---

## New Manifest Scenarios

```toml
# Append to tests/generation/manifest.toml

# ── qed:uuid() edge cases ─────────────────────────────────────────────────────

[[scenario]]
id = "uuid-multiple-placeholders"
description = "qed:uuid() called on multiple placeholder lines produces independent UUID values for each"
script = "uuid-multiple-placeholders.qed"
input = "multi-placeholder.txt"
stdout = "uuid-two-lines.pattern"
stderr = "empty.txt"
output = "uuid-two-lines.pattern"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "uuid-before"
description = "qed:uuid() composes with before() to insert a UUID before a matching line"
script = "uuid-before.qed"
input = "one-line.txt"
stdout = "uuid-before-inserted.pattern"
stderr = "empty.txt"
output = "uuid-before-inserted.pattern"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── qed:timestamp() edge cases ────────────────────────────────────────────────

[[scenario]]
id = "timestamp-mixed-content"
description = "qed:timestamp() replacement leaves surrounding content on the line intact"
script = "timestamp-mixed-content.qed"
input = "mixed-placeholder.txt"
stdout = "timestamp-mixed-result.pattern"
stderr = "empty.txt"
output = "timestamp-mixed-result.pattern"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "timestamp-format-date"
description = "qed:timestamp(format:date) produces a yyyy-MM-dd formatted date"
script = "timestamp-format-date.qed"
input = "timestamp-placeholder.txt"
stdout = "timestamp-date-line.pattern"
stderr = "empty.txt"
output = "timestamp-date-line.pattern"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "timestamp-format-time"
description = "qed:timestamp(format:time) produces an HH:mm:ss formatted time"
script = "timestamp-format-time.qed"
input = "timestamp-placeholder.txt"
stdout = "timestamp-time-line.pattern"
stderr = "empty.txt"
output = "timestamp-time-line.pattern"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "timestamp-format-unix-ms"
description = "qed:timestamp(format:unix_ms) produces a Unix epoch milliseconds timestamp"
script = "timestamp-format-unix-ms.qed"
input = "timestamp-placeholder.txt"
stdout = "timestamp-unix-ms-line.pattern"
stderr = "empty.txt"
output = "timestamp-unix-ms-line.pattern"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "timestamp-fixed-offset-tz"
description = "qed:timestamp(timezone:\"UTC+5:30\") applies a fixed offset without DST adjustment"
script = "timestamp-fixed-offset-tz.qed"
input = "timestamp-placeholder.txt"
stdout = "timestamp-timezone-line.pattern"
stderr = "empty.txt"
output = "timestamp-timezone-line.pattern"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── qed:random() edge cases ───────────────────────────────────────────────────

[[scenario]]
id = "random-length-one"
description = "qed:random(1) produces a single character from the specified alphabet"
script = "random-length-boundary.qed"
input = "random-placeholder.txt"
stdout = "random-length-one.pattern"
stderr = "empty.txt"
output = "random-length-one.pattern"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "random-length-large"
description = "qed:random(128) produces a string of exactly 128 characters"
script = "random-large-length.qed"
input = "random-placeholder.txt"
stdout = "random-length-large.pattern"
stderr = "empty.txt"
output = "random-length-large.pattern"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]
```

---

## New Golden Files

### `goldens/stdout/` and `goldens/output/`

All files in this section are `.pattern` files — generation output is non-deterministic.

#### `uuid-two-lines.pattern`

Used by: `uuid-multiple-placeholders`

Two lines, each a valid UUID v7. The pattern does not assert they differ —
that would require a more expressive assertion mechanism than regex.

```
^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}\n[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$
```

#### `uuid-before-inserted.pattern`

Used by: `uuid-before`

A UUID v7 line appears before `header`.

```
^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}\nheader$
```

#### `timestamp-mixed-result.pattern`

Used by: `timestamp-mixed-content`

`{{ts}}` replaced with a `yyyy-MM-dd` date; surrounding content `generated at ` and
` by qed` must be preserved exactly.

```
^generated at \d{4}-\d{2}-\d{2} by qed$
```

#### `timestamp-date-line.pattern`

Used by: `timestamp-format-date`

`yyyy-MM-dd` format.

```
^\d{4}-\d{2}-\d{2}$
```

#### `timestamp-time-line.pattern`

Used by: `timestamp-format-time`

`HH:mm:ss` format.

```
^\d{2}:\d{2}:\d{2}$
```

#### `timestamp-unix-ms-line.pattern`

Used by: `timestamp-format-unix-ms`

Unix epoch milliseconds — more digits than seconds.

```
^\d{13}$
```

#### `random-length-one.pattern`

Used by: `random-length-one`

Single alphanumeric character.

```
^[a-zA-Z0-9]$
```

#### `random-length-large.pattern`

Used by: `random-length-large`

Exactly 128 alphanumeric characters.

```
^[a-zA-Z0-9]{128}$
```

#### `timestamp-timezone-line.pattern`

Used by: `timestamp-fixed-offset-tz`

`format:datetime` with `timezone:"UTC+5:30"` — produces `yyyy-MM-dd HH:mm:ss` in the
fixed-offset local time.
The offset is not embedded in the output; `format:datetime` always produces bare
`yyyy-MM-dd HH:mm:ss` regardless of timezone.
This golden is structurally identical to the one in `tests/generation/` for
`timestamp-timezone-replace` — the scenario is a smoke test that `UTC+5:30` syntax
is accepted and the output is well-formed.

```
^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$
```
