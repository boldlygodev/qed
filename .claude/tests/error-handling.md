# Error Handling Scenarios

Tests covering fallback statements, `on_error` interaction with `||`,
and processor failure routing.

`on_error` basic behaviour (fail/warn/skip with no fallback) is covered in
`tests/selectors/`. This directory focuses on the `||` fallback mechanism
and the interaction between selector errors and processor errors.

---

## Directory Layout

```
tests/error-handling/
  manifest.toml
  inputs/
    three-lines.txt
    foo-only.txt
  scripts/
    fallback-selector.qed
    fallback-processor.qed
    fallback-chain.qed
    on-error-fail-triggers-fallback.qed
    on-error-warn-no-fallback.qed
    on-error-skip-no-fallback.qed
    processor-fail-triggers-fallback.qed
    fallback-fail.qed
  goldens/
    stdout/
      empty.txt
      foo-bar-baz.txt
      foo-baz.txt
      fallback-result.txt
    stderr/
      empty.txt
      warn-no-match.txt
      processor-failed.txt
      fallback-processor-failed.txt
    output/
      empty.txt
      foo-bar-baz.txt
      foo-baz.txt
      fallback-result.txt
```

---

## Manifest

```toml
# tests/error-handling/manifest.toml

# ── fallback: selector no-match ───────────────────────────────────────────────

[[scenario]]
id = "fallback-on-selector-no-match"
description = "|| fallback is triggered when the primary selector finds no match; fallback selects from original input"
script = "fallback-selector.qed"
input = "three-lines.txt"
stdout = "foo-baz.txt"
stderr = "empty.txt"
output = "foo-baz.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── fallback: processor failure ───────────────────────────────────────────────

[[scenario]]
id = "fallback-on-processor-failure"
description = "|| fallback is triggered when the processor exits non-zero; fallback processor runs against original input"
script = "fallback-processor.qed"
input = "three-lines.txt"
stdout = "foo-baz.txt"
stderr = "processor-failed.txt"
output = "foo-baz.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── fallback: chained ─────────────────────────────────────────────────────────

[[scenario]]
id = "fallback-chain"
description = "fallback is itself a full select-action expression with its own selector and processor"
script = "fallback-chain.qed"
input = "three-lines.txt"
stdout = "fallback-result.txt"
stderr = "empty.txt"
output = "fallback-result.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── on_error:fail triggers || ─────────────────────────────────────────────────

[[scenario]]
id = "on-error-fail-triggers-fallback"
description = "on_error:fail (default) triggers || fallback on no-match; on_error:warn and skip do not"
script = "on-error-fail-triggers-fallback.qed"
input = "three-lines.txt"
stdout = "foo-baz.txt"
stderr = "empty.txt"
output = "foo-baz.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── on_error:warn does not trigger || ────────────────────────────────────────

[[scenario]]
id = "on-error-warn-does-not-trigger-fallback"
description = "on_error:warn emits to stderr and succeeds; || fallback is not triggered"
script = "on-error-warn-no-fallback.qed"
input = "three-lines.txt"
stdout = "foo-bar-baz.txt"
stderr = "warn-no-match.txt"
output = "foo-bar-baz.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── on_error:skip does not trigger || ────────────────────────────────────────

[[scenario]]
id = "on-error-skip-does-not-trigger-fallback"
description = "on_error:skip succeeds silently; || fallback is not triggered"
script = "on-error-skip-no-fallback.qed"
input = "three-lines.txt"
stdout = "foo-bar-baz.txt"
stderr = "empty.txt"
output = "foo-bar-baz.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── processor failure triggers || ─────────────────────────────────────────────

[[scenario]]
id = "processor-fail-triggers-fallback"
description = "a processor error always triggers || fallback regardless of on_error"
script = "processor-fail-triggers-fallback.qed"
input = "three-lines.txt"
stdout = "foo-baz.txt"
stderr = "processor-failed.txt"
output = "foo-baz.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── fallback itself fails ─────────────────────────────────────────────────────

[[scenario]]
id = "fallback-fail"
description = "if the fallback processor also fails, qed exits non-zero; the selected line is never emitted and downstream lines are blocked behind it"
script = "fallback-fail.qed"
input = "foo-only.txt"
stdout = "empty.txt"
stderr = "fallback-processor-failed.txt"
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

### `inputs/foo-only.txt`

Used by: `fallback-fail`

A single line — used to provoke a fallback that itself fails.

```
foo
```

---

## Script Files

### `scripts/fallback-selector.qed`

Primary selector targets `quux` (not present); fallback targets `bar` and deletes it.
The fallback selects from the original input stream.

```
at("quux") | qed:delete() || at("bar") | qed:delete()
```

### `scripts/fallback-processor.qed`

Primary processor is a failing external command; fallback deletes `bar`.
`false` is a standard Unix command that always exits non-zero.

```
at("bar") | false || at("bar") | qed:delete()
```

### `scripts/fallback-chain.qed`

Primary selector targets `quux` (not present); fallback is a full select-action
targeting `foo` and uppercasing it.

```
at("quux") | qed:delete() || at("foo") | qed:upper()
```

### `scripts/on-error-fail-triggers-fallback.qed`

`on_error:fail` (the default) triggers `||` fallback on no-match.
The fallback deletes `bar`.

```
at("quux") | qed:delete() || at("bar") | qed:delete()
```

### `scripts/on-error-warn-no-fallback.qed`

`on_error:warn` succeeds with a warning — `||` fallback is not triggered even though
it is present. The stream passes through unchanged.

```
at("quux", on_error:warn) | qed:delete() || at("bar") | qed:delete()
```

### `scripts/on-error-skip-no-fallback.qed`

`on_error:skip` succeeds silently — `||` fallback is not triggered.
The stream passes through unchanged.

```
at("quux", on_error:skip) | qed:delete() || at("bar") | qed:delete()
```

### `scripts/processor-fail-triggers-fallback.qed`

The selector matches `bar`; the processor (`false`) fails.
Processor failure always triggers `||` fallback regardless of `on_error`.
Fallback deletes `bar`.

```
at("bar") | false || at("bar") | qed:delete()
```

### `scripts/fallback-fail.qed`

Both primary and fallback processors fail.
Primary matches `foo` and runs `false`; fallback also runs `false`.
qed exits non-zero.

```
at("foo") | false || at("foo") | false
```

---

## Golden Files

### `goldens/stdout/` and `goldens/output/`

#### `empty.txt`

```
```

#### `foo-bar-baz.txt`

Used by: `on-error-warn-does-not-trigger-fallback`,
`on-error-skip-does-not-trigger-fallback`

Stream passed through unchanged — fallback was not triggered.

```
foo
bar
baz
```

#### `foo-baz.txt`

Used by: `fallback-on-selector-no-match`, `on-error-fail-triggers-fallback`,
`fallback-on-processor-failure`, `processor-fail-triggers-fallback`

`bar` deleted by the fallback processor.

```
foo
baz
```

#### `fallback-result.txt`

Used by: `fallback-chain`

`foo` uppercased by the fallback select-action; `bar` and `baz` pass through.

```
FOO
bar
baz
```

---

### `goldens/stderr/`

#### `empty.txt`

```
```

#### `warn-no-match.txt`

Used by: `on-error-warn-does-not-trigger-fallback`

Script: `at("quux", on_error:warn) | qed:delete() || at("bar") | qed:delete()` —
selector at 1:1-25, second `qed:delete()` at 1:57-68 (widest: 7 chars).

```
qed: warning: 1:1-25: at("quux", on_error:warn): no lines matched
```

#### `processor-failed.txt`

Used by: `fallback-on-processor-failure`, `processor-fail-triggers-fallback`

`false` is at bytes 13–17 in both scripts (`at("bar") | false || …`).

```
qed: error:   1:13-17: false: exit code 1
```

#### `fallback-processor-failed.txt`

Used by: `fallback-fail`

In `fallback-fail.qed` the fallback `false` occupies bytes 34–38 on line 1.
The diagnostic fires for the terminal (fallback) failure.

```
qed: error:   1:34-38: false: exit code 1
```
