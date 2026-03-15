# Pattern Scenarios

Tests covering named patterns, inline patterns, negation, `+` inclusion,
and environment variable expansion in patterns.

Basic inline literal and regex pattern usage is exercised throughout other
feature directories. This directory focuses on the pattern system itself —
definition, reference, negation, inclusion, and env var expansion.

---

## Directory Layout

```
tests/patterns/
  manifest.toml
  inputs/
    three-lines.txt
    five-lines.txt
  scripts/
    named-literal.qed
    named-regex.qed
    named-negated.qed
    named-inclusive.qed
    inline-literal.qed
    inline-regex.qed
    inline-negated-literal.qed
    inline-negated-regex.qed
    env-expand-pattern.qed
    env-expand-escaped.qed
  goldens/
    stdout/
      empty.txt
      foo-baz.txt
      bar.txt
      alpha-bravo.txt
      alpha-bravo-charlie.txt
      foo-bar-baz.txt
      env-pattern-result.txt
      env-escaped-result.txt
    stderr/
      empty.txt
      unset-var-warn.txt
    output/
      (same filenames and content as goldens/stdout/)
```

---

## Manifest

```toml
# tests/patterns/manifest.toml

# ── named patterns ────────────────────────────────────────────────────────────

[[scenario]]
id = "named-literal"
description = "a named literal pattern is defined and referenced by bare identifier in a selector"
script = "named-literal.qed"
input = "three-lines.txt"
stdout = "foo-baz.txt"
stderr = "empty.txt"
output = "foo-baz.txt"
exit_code = 0
invoke = [
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "named-regex"
description = "a named regex pattern is defined and referenced by bare identifier in a selector"
script = "named-regex.qed"
input = "three-lines.txt"
stdout = "foo-baz.txt"
stderr = "empty.txt"
output = "foo-baz.txt"
exit_code = 0
invoke = [
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "named-negated"
description = "a named pattern prefixed with ! negates the match; lines not matching the pattern are selected"
script = "named-negated.qed"
input = "three-lines.txt"
stdout = "bar.txt"
stderr = "empty.txt"
output = "bar.txt"
exit_code = 0
invoke = [
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "named-inclusive"
description = "a named pattern with + in from position includes the matching line in the selected region"
script = "named-inclusive.qed"
input = "five-lines.txt"
stdout = "alpha-bravo.txt"
stderr = "empty.txt"
output = "alpha-bravo.txt"
exit_code = 0
invoke = [
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── inline patterns ───────────────────────────────────────────────────────────

[[scenario]]
id = "inline-literal"
description = "an inline literal string pattern matches the exact text; no regex interpretation"
script = "inline-literal.qed"
input = "three-lines.txt"
stdout = "foo-baz.txt"
stderr = "empty.txt"
output = "foo-baz.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "inline-regex"
description = "an inline regex pattern uses RE2 semantics for matching"
script = "inline-regex.qed"
input = "three-lines.txt"
stdout = "foo-baz.txt"
stderr = "empty.txt"
output = "foo-baz.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "inline-negated-literal"
description = "an inline literal prefixed with ! selects lines that do not contain the literal"
script = "inline-negated-literal.qed"
input = "three-lines.txt"
stdout = "bar.txt"
stderr = "empty.txt"
output = "bar.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "inline-negated-regex"
description = "an inline regex prefixed with ! selects lines that do not match the regex"
script = "inline-negated-regex.qed"
input = "three-lines.txt"
stdout = "bar.txt"
stderr = "empty.txt"
output = "bar.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── environment variable expansion ───────────────────────────────────────────

[[scenario]]
id = "env-expand-pattern"
description = "${VAR} in a pattern value is expanded from the environment before matching"
script = "env-expand-pattern.qed"
input = "three-lines.txt"
stdout = "foo-baz.txt"
stderr = "empty.txt"
output = "foo-baz.txt"
exit_code = 0
env = { QED_PAT = "bar" }
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "env-expand-escaped"
description = "\\${VAR} is treated as the literal string ${VAR} and is not expanded"
script = "env-expand-escaped.qed"
input = "three-lines.txt"
stdout = "foo-bar-baz.txt"
stderr = "empty.txt"
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

Used by: most scenarios

```
foo
bar
baz
```

### `inputs/five-lines.txt`

Used by: `named-inclusive`

```
alpha
bravo
charlie
delta
echo
```

---

## Script Files

### `scripts/named-literal.qed`

```
target="bar"
at(target) | qed:delete()
```

### `scripts/named-regex.qed`

```
target=/^bar$/
at(target) | qed:delete()
```

### `scripts/named-negated.qed`

Selects all lines that do not match `bar`; deletes them, leaving only `bar`.

```
target="bar"
at(!target) | qed:delete()
```

### `scripts/named-inclusive.qed`

`from` with the named pattern marked `+` includes `charlie` in the selected region;
deleting from `charlie+` to end of stream leaves only `alpha` and `bravo`.

```
boundary="charlie"
from(boundary+) | qed:delete()
```

### `scripts/inline-literal.qed`

```
at("bar") | qed:delete()
```

### `scripts/inline-regex.qed`

```
at(/^bar$/) | qed:delete()
```

### `scripts/inline-negated-literal.qed`

Selects lines not matching `bar` literally; deletes them, leaving only `bar`.

```
at(!"bar") | qed:delete()
```

### `scripts/inline-negated-regex.qed`

Selects lines not matching `/^bar$/`; deletes them, leaving only `bar`.

```
at(!/^bar$/) | qed:delete()
```

### `scripts/env-expand-pattern.qed`

`${QED_PAT}` is expanded from the environment.
When invoked with `QED_PAT=bar`, this deletes `bar`.

```
at("${QED_PAT}") | qed:delete()
```

### `scripts/env-expand-escaped.qed`

`\${QED_PAT}` is the literal string `${QED_PAT}` — not expanded.
It matches no lines in `three-lines.txt`, so the stream passes through unchanged.
Uses `on_error:skip` to avoid a no-match failure.

```
at("\${QED_PAT}", on_error:skip) | qed:delete()
```

---

## Golden Files

### `goldens/stdout/` and `goldens/output/`

#### `empty.txt`

```
```

#### `foo-baz.txt`

Used by: `named-literal`, `named-regex`, `inline-literal`, `inline-regex`,
`env-expand-pattern`

`bar` deleted; `foo` and `baz` pass through.

```
foo
baz
```

#### `bar.txt`

Used by: `named-negated`, `inline-negated-literal`, `inline-negated-regex`

All lines except `bar` deleted; only `bar` remains.

```
bar
```

#### `alpha-bravo.txt`

Used by: `named-inclusive`

`from(boundary+)` selects `charlie` through `echo`; deleting them leaves `alpha` and `bravo`.

```
alpha
bravo
```

#### `foo-bar-baz.txt`

Used by: `env-expand-escaped`

Stream passes through unchanged — escaped `\${QED_PAT}` matched nothing.

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
