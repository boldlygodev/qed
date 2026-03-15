# Generation Scenarios

Tests covering `qed:uuid()`, `qed:timestamp()`, and `qed:random()`.

Generation processors ignore stdin and produce output from parameters alone.
They compose with `qed:replace()` for placeholder substitution and with
`after`/`before` for insertion.

Non-deterministic scenarios use `.pattern` golden files for regex validation.
Deterministic scenarios (e.g. UUID v5, fixed-format timestamps) use `.txt` golden
files for exact validation alongside `.pattern` files — the manifest references `file.*`
and the harness runs all checks that exist on disk.

---

## Directory Layout

```
tests/generation/
  manifest.toml
  inputs/
    uuid-placeholder.txt
    timestamp-placeholder.txt
    random-placeholder.txt
    one-line.txt
  scripts/
    uuid-v7-replace.qed
    uuid-v4-replace.qed
    uuid-v5-replace.qed
    uuid-v7-after.qed
    timestamp-iso8601-replace.qed
    timestamp-unix-replace.qed
    timestamp-custom-format-replace.qed
    timestamp-timezone-replace.qed
    timestamp-after.qed
    random-numeric-replace.qed
    random-alpha-replace.qed
    random-alnum-replace.qed
    random-hex-replace.qed
    random-custom-alphabet-replace.qed
    random-after.qed
  goldens/
    stdout/
      empty.txt
      uuid-v7-line.pattern
      uuid-v4-line.pattern
      uuid-v5-line.txt
      uuid-v5-line.pattern
      uuid-v7-inserted.pattern
      timestamp-iso8601-line.pattern
      timestamp-unix-line.pattern
      timestamp-custom-format-line.pattern
      timestamp-timezone-line.pattern
      timestamp-inserted.pattern
      random-numeric-line.pattern
      random-alpha-line.pattern
      random-alnum-line.pattern
      random-hex-line.pattern
      random-custom-alphabet-line.pattern
      random-inserted.pattern
    stderr/
      empty.txt
    output/
      (same filenames and content as goldens/stdout/)
```

---

## Manifest

```toml
# tests/generation/manifest.toml

# ── qed:uuid() ────────────────────────────────────────────────────────────────

[[scenario]]
id = "uuid-v7-replace"
description = "qed:uuid() default produces a UUID v7 replacing a placeholder; output matches UUID v7 format"
script = "uuid-v7-replace.qed"
input = "uuid-placeholder.txt"
stdout = "uuid-v7-line.pattern"
stderr = "empty.txt"
output = "uuid-v7-line.pattern"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "uuid-v4-replace"
description = "qed:uuid(version:4) produces a random UUID v4 replacing a placeholder"
script = "uuid-v4-replace.qed"
input = "uuid-placeholder.txt"
stdout = "uuid-v4-line.pattern"
stderr = "empty.txt"
output = "uuid-v4-line.pattern"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "uuid-v5-replace"
description = "qed:uuid(version:5, namespace:url, name:...) produces a deterministic UUID v5; exact output is verified"
script = "uuid-v5-replace.qed"
input = "uuid-placeholder.txt"
stdout = "uuid-v5-line.*"
stderr = "empty.txt"
output = "uuid-v5-line.*"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "uuid-v7-after"
description = "qed:uuid() composes with after() to insert a UUID after a matching line"
script = "uuid-v7-after.qed"
input = "one-line.txt"
stdout = "uuid-v7-inserted.pattern"
stderr = "empty.txt"
output = "uuid-v7-inserted.pattern"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── qed:timestamp() ───────────────────────────────────────────────────────────

[[scenario]]
id = "timestamp-iso8601-replace"
description = "qed:timestamp() default produces an ISO 8601 UTC timestamp replacing a placeholder"
script = "timestamp-iso8601-replace.qed"
input = "timestamp-placeholder.txt"
stdout = "timestamp-iso8601-line.pattern"
stderr = "empty.txt"
output = "timestamp-iso8601-line.pattern"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "timestamp-unix-replace"
description = "qed:timestamp(format:unix) produces a Unix epoch seconds timestamp replacing a placeholder"
script = "timestamp-unix-replace.qed"
input = "timestamp-placeholder.txt"
stdout = "timestamp-unix-line.pattern"
stderr = "empty.txt"
output = "timestamp-unix-line.pattern"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "timestamp-custom-format-replace"
description = "qed:timestamp(format:\"...\") produces a timestamp in a custom LDML format replacing a placeholder"
script = "timestamp-custom-format-replace.qed"
input = "timestamp-placeholder.txt"
stdout = "timestamp-custom-format-line.pattern"
stderr = "empty.txt"
output = "timestamp-custom-format-line.pattern"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "timestamp-timezone-replace"
description = "qed:timestamp(timezone:...) produces a timestamp adjusted to the specified IANA timezone"
script = "timestamp-timezone-replace.qed"
input = "timestamp-placeholder.txt"
stdout = "timestamp-timezone-line.pattern"
stderr = "empty.txt"
output = "timestamp-timezone-line.pattern"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "timestamp-after"
description = "qed:timestamp() composes with after() to insert a timestamp after a matching line"
script = "timestamp-after.qed"
input = "one-line.txt"
stdout = "timestamp-inserted.pattern"
stderr = "empty.txt"
output = "timestamp-inserted.pattern"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── qed:random() ──────────────────────────────────────────────────────────────

[[scenario]]
id = "random-numeric-replace"
description = "qed:random(N) default produces a numeric string of length N replacing a placeholder"
script = "random-numeric-replace.qed"
input = "random-placeholder.txt"
stdout = "random-numeric-line.pattern"
stderr = "empty.txt"
output = "random-numeric-line.pattern"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "random-alpha-replace"
description = "qed:random(N, alphabet:alpha) produces a lowercase alphabetic string of length N"
script = "random-alpha-replace.qed"
input = "random-placeholder.txt"
stdout = "random-alpha-line.pattern"
stderr = "empty.txt"
output = "random-alpha-line.pattern"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "random-alnum-replace"
description = "qed:random(N, alphabet:alnum) produces an alphanumeric string of length N"
script = "random-alnum-replace.qed"
input = "random-placeholder.txt"
stdout = "random-alnum-line.pattern"
stderr = "empty.txt"
output = "random-alnum-line.pattern"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "random-hex-replace"
description = "qed:random(N, alphabet:hex) produces a lowercase hex string of length N"
script = "random-hex-replace.qed"
input = "random-placeholder.txt"
stdout = "random-hex-line.pattern"
stderr = "empty.txt"
output = "random-hex-line.pattern"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "random-custom-alphabet-replace"
description = "qed:random(N, alphabet:\"...\") produces a string of length N drawn from a custom character set"
script = "random-custom-alphabet-replace.qed"
input = "random-placeholder.txt"
stdout = "random-custom-alphabet-line.pattern"
stderr = "empty.txt"
output = "random-custom-alphabet-line.pattern"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "random-after"
description = "qed:random() composes with after() to insert a random string after a matching line"
script = "random-after.qed"
input = "one-line.txt"
stdout = "random-inserted.pattern"
stderr = "empty.txt"
output = "random-inserted.pattern"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]
```

---

## Input Files

### `inputs/uuid-placeholder.txt`

Used by: `uuid-v7-replace`, `uuid-v4-replace`, `uuid-v5-replace`

```
{{uuid}}
```

### `inputs/timestamp-placeholder.txt`

Used by: `timestamp-iso8601-replace`, `timestamp-unix-replace`,
`timestamp-custom-format-replace`, `timestamp-timezone-replace`

```
{{ts}}
```

### `inputs/random-placeholder.txt`

Used by: `random-numeric-replace`, `random-alpha-replace`, `random-alnum-replace`,
`random-hex-replace`, `random-custom-alphabet-replace`

```
{{token}}
```

### `inputs/one-line.txt`

Used by: `uuid-v7-after`, `timestamp-after`, `random-after`

```
header
```

---

## Script Files

### `scripts/uuid-v7-replace.qed`

```
at(/\{\{uuid\}\}/) | qed:replace("{{uuid}}", qed:uuid())
```

### `scripts/uuid-v4-replace.qed`

```
at(/\{\{uuid\}\}/) | qed:replace("{{uuid}}", qed:uuid(version:4))
```

### `scripts/uuid-v5-replace.qed`

UUID v5 is deterministic given a fixed namespace and name.

```
at(/\{\{uuid\}\}/) | qed:replace("{{uuid}}", qed:uuid(version:5, namespace:url, name:"https://example.com"))
```

### `scripts/uuid-v7-after.qed`

```
after("header") | qed:replace("", qed:uuid())
```

### `scripts/timestamp-iso8601-replace.qed`

```
at(/\{\{ts\}\}/) | qed:replace("{{ts}}", qed:timestamp())
```

### `scripts/timestamp-unix-replace.qed`

```
at(/\{\{ts\}\}/) | qed:replace("{{ts}}", qed:timestamp(format:unix))
```

### `scripts/timestamp-custom-format-replace.qed`

```
at(/\{\{ts\}\}/) | qed:replace("{{ts}}", qed:timestamp(format:"yyyy/MM/dd"))
```

### `scripts/timestamp-timezone-replace.qed`

```
at(/\{\{ts\}\}/) | qed:replace("{{ts}}", qed:timestamp(format:datetime, timezone:"America/New_York"))
```

### `scripts/timestamp-after.qed`

```
after("header") | qed:timestamp()
```

### `scripts/random-numeric-replace.qed`

```
at(/\{\{token\}\}/) | qed:replace("{{token}}", qed:random(16))
```

### `scripts/random-alpha-replace.qed`

```
at(/\{\{token\}\}/) | qed:replace("{{token}}", qed:random(16, alphabet:alpha))
```

### `scripts/random-alnum-replace.qed`

```
at(/\{\{token\}\}/) | qed:replace("{{token}}", qed:random(16, alphabet:alnum))
```

### `scripts/random-hex-replace.qed`

```
at(/\{\{token\}\}/) | qed:replace("{{token}}", qed:random(16, alphabet:hex))
```

### `scripts/random-custom-alphabet-replace.qed`

Generates a 16-character string drawn only from `abc123`.

```
at(/\{\{token\}\}/) | qed:replace("{{token}}", qed:random(16, alphabet:"abc123"))
```

### `scripts/random-after.qed`

```
after("header") | qed:random(16, alphabet:alnum)
```

---

## Golden Files

`goldens/stdout/` and `goldens/output/` contain files with identical content.
They are listed once below; both directories contain a copy of each.

### `goldens/stdout/` and `goldens/output/`

#### `empty.txt`

```
```

#### `uuid-v7-line.pattern`

UUID v7 format: time-ordered, version bit `7`, variant bits `8`, `9`, `a`, or `b`.

```
^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$
```

#### `uuid-v4-line.pattern`

UUID v4 format: random, version bit `4`.

```
^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$
```

#### `uuid-v5-line.txt`

UUID v5 of `https://example.com` in the URL namespace is deterministic.

```
c5c17c18-a4a4-5a46-bcd1-b7d8e9c05694
```

#### `uuid-v5-line.pattern`

UUID v5 format: name-based SHA-1, version bit `5`.

```
^[0-9a-f]{8}-[0-9a-f]{4}-5[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$
```

#### `uuid-v7-inserted.pattern`

The original `header` line followed by a UUID v7 on the next line.

```
^header\n[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$
```

#### `timestamp-iso8601-line.pattern`

ISO 8601 UTC — e.g. `2026-02-28T13:45:00Z`.

```
^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$
```

#### `timestamp-unix-line.pattern`

Unix epoch seconds — a positive integer.

```
^\d+$
```

#### `timestamp-custom-format-line.pattern`

`yyyy/MM/dd` format — e.g. `2026/02/28`.

```
^\d{4}/\d{2}/\d{2}$
```

#### `timestamp-timezone-line.pattern`

`datetime` format in `America/New_York` — e.g. `2026-02-28 08:45:00`.

```
^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$
```

#### `timestamp-inserted.pattern`

The original `header` line followed by an ISO 8601 UTC timestamp on the next line.

```
^header\n\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$
```

#### `random-numeric-line.pattern`

16-character numeric string.

```
^\d{16}$
```

#### `random-alpha-line.pattern`

16-character lowercase alphabetic string.

```
^[a-z]{16}$
```

#### `random-alnum-line.pattern`

16-character alphanumeric string.

```
^[a-zA-Z0-9]{16}$
```

#### `random-hex-line.pattern`

16-character lowercase hex string.

```
^[0-9a-f]{16}$
```

#### `random-custom-alphabet-line.pattern`

16-character string drawn from `abc123` only.

```
^[abc123]{16}$
```

#### `random-inserted.pattern`

The original `header` line followed by a 16-character alphanumeric string.

```
^header\n[a-zA-Z0-9]{16}$
```

---

### `goldens/stderr/`

#### `empty.txt`

```
```

---

## Notes

### `uuid-v7-after` script

The `after()` insertion point delivers empty stdin to the processor.
`qed:uuid()` ignores stdin, so `after("header") | qed:replace("", qed:uuid())`
is used to compose the generation into the replace pipeline.
An alternative simpler form — `after("header") | qed:uuid()` — should also work
if generation processors are permitted directly in an `after` pipeline without
a `qed:replace()` wrapper.
This is worth confirming during implementation; the script may need updating.

### UUID v5 golden

The exact UUID v5 value in `uuid-v5-line.txt` (`c5c17c18-a4a4-5a46-bcd1-b7d8e9c05694`)
must be verified against the actual implementation output and updated if incorrect.
UUID v5 is deterministic by spec but the value depends on the SHA-1 implementation
and namespace byte encoding.
