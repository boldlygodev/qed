# Error Handling Edge Case Scenarios

Additional scenarios covering boundary conditions in error handling behaviour.
These extend `tests/error-handling/` with new scenarios, inputs, scripts, and goldens.

---

## New Inputs

### `inputs/two-lines.txt`

Used by: `multiple-failures-first-wins`, `partial-success-before-failure`

```
foo
bar
```

### `inputs/three-lines.txt`

Already exists. Used by new scenarios below.

---

## New Scripts

### `scripts/fallback-processor-chain.qed`

The fallback is a processor-chain-only form (no selector) — it runs against
the original input when triggered.

```
at("quux") | qed:delete() || qed:upper()
```

### `scripts/nested-fallback.qed`

A fallback select-action that itself has a fallback.

```
at("quux") | qed:delete() || at("quux2") | qed:delete() || at("foo") | qed:upper()
```

### `scripts/multiple-statements-first-fails.qed`

Two statements; the first fails and has no fallback, so qed exits non-zero
before the second statement executes.

```
at("quux") | qed:delete()
at("bar") | qed:upper()
```

### `scripts/multiple-statements-second-fails.qed`

Two statements; the first succeeds, the second fails with no fallback.

```
at("foo") | qed:upper()
at("quux") | qed:delete()
```

### `scripts/fallback-skips-on-warn.qed`

`on_error:warn` — the fallback must not trigger even though it is present.
The fallback would delete `bar`; if it fired, `bar` would be absent from the output.

```
at("quux", on_error:warn) | qed:delete() || at("bar") | qed:delete()
```

### `scripts/fallback-skips-on-skip.qed`

`on_error:skip` — the fallback must not trigger.

```
at("quux", on_error:skip) | qed:delete() || at("bar") | qed:delete()
```

---

## New Manifest Scenarios

```toml
# Append to tests/error-handling/manifest.toml

# ── fallback forms ────────────────────────────────────────────────────────────

[[scenario]]
id = "fallback-processor-chain-only"
description = "|| fallback can be a processor-chain without a selector; it runs against the original input"
script = "fallback-processor-chain.qed"
input = "three-lines.txt"
stdout = "foo-bar-baz-upper.txt"
stderr = "empty.txt"
output = "foo-bar-baz-upper.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "nested-fallback"
description = "a fallback select-action can itself have a fallback; the chain resolves to the first succeeding branch"
script = "nested-fallback.qed"
input = "three-lines.txt"
stdout = "foo-upper-bar-baz.txt"
stderr = "empty.txt"
output = "foo-upper-bar-baz.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── multi-statement failure behaviour ─────────────────────────────────────────

[[scenario]]
id = "multiple-statements-first-fails"
description = "when the first of multiple statements fails with no fallback, qed exits non-zero; lines with no remaining statement tags before the tagged region are emitted"
script = "multiple-statements-first-fails.qed"
input = "three-lines.txt"
stdout = "foo.txt"
stderr = "error-no-match-first-statement.txt"
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

[[scenario]]
id = "multiple-statements-second-fails"
description = "when the second of multiple statements fails with no fallback, qed exits non-zero; the first statement's output and all untagged lines are emitted before the failure"
script = "multiple-statements-second-fails.qed"
input = "three-lines.txt"
stdout = "foo-upper-bar-baz.txt"
stderr = "error-no-match-second-statement.txt"
output = "foo-upper-bar-baz.txt"
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

# ── on_error does not trigger fallback ────────────────────────────────────────

[[scenario]]
id = "on-error-warn-fallback-not-triggered"
description = "on_error:warn does not trigger || fallback even when fallback is present; stream passes through unchanged"
script = "fallback-skips-on-warn.qed"
input = "three-lines.txt"
stdout = "foo-bar-baz.txt"
stderr = "warn-no-match.txt"
output = "foo-bar-baz.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "on-error-skip-fallback-not-triggered"
description = "on_error:skip does not trigger || fallback even when fallback is present; stream passes through unchanged"
script = "fallback-skips-on-skip.qed"
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

## New Golden Files

### `goldens/stdout/` and `goldens/output/`

#### `foo.txt`

Used by: `multiple-statements-first-fails` (stdout, output)

`foo` passes through before `bar`'s tagged region is reached;
`bar` and `baz` are blocked behind the unprocessed tagged fragment when qed exits.

```
foo
```

#### `foo-bar-baz-upper.txt`

Used by: `fallback-processor-chain-only`

The processor-chain-only fallback `qed:upper()` runs against the original input —
the entire stream is uppercased.

```
FOO
BAR
BAZ
```

#### `foo-upper-bar-baz.txt`

Used by: `nested-fallback`, `multiple-statements-second-fails` (stdout, output)

The first two fallback branches find no match (`quux`, `quux2`).
The third branch matches `foo` and uppercases it.

```
FOO
bar
baz
```

#### `foo-bar-baz.txt`

Used by: `on-error-warn-fallback-not-triggered`,
`on-error-skip-fallback-not-triggered`

Stream passes through unchanged — fallback was suppressed by `on_error`.

```
foo
bar
baz
```

---

### `goldens/stderr/`

#### `error-no-match-first-statement.txt`

Used by: `multiple-statements-first-fails`

`at("quux")` is statement 1 of 2 in `multiple-statements-first-fails.qed` — line 1, bytes 1–10.
Widest span in the script is `qed:delete()` at `1:14-25` (7 chars) → location padded to 7.

```
qed: error:   1:1-10:  at("quux"): no lines matched
```

#### `error-no-match-second-statement.txt`

Used by: `multiple-statements-second-fails`

`at("quux")` is statement 2 of 2 in `multiple-statements-second-fails.qed` — line 2, bytes 1–10.
Widest span in the script is `qed:delete()` at `2:14-25` (7 chars) → location padded to 7.

```
qed: error:   2:1-10:  at("quux"): no lines matched
```

#### `warn-no-match.txt`

Used by: `on-error-warn-fallback-not-triggered`

`at("quux", on_error:warn)` occupies bytes 1–25 on line 1 of `fallback-skips-on-warn.qed`.
Widest span is the second `qed:delete()` at `1:57-68` (7 chars) → location padded to 7.

```
qed: warning: 1:1-25:  at("quux", on_error:warn): no lines matched
```
