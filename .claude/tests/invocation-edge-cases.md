# Invocation Edge Case Scenarios

Additional scenarios covering boundary conditions in invocation behaviour.
These extend `tests/invocation/` with new scenarios, inputs, scripts, and goldens.

---

## New Inputs

### `inputs/empty.txt`

Used by: `output-flag-empty`, `in-place-empty`, `extract-no-match`, `dry-run-no-change`

```
```

### `inputs/three-lines.txt`

Already exists.

### `inputs/unset-var.txt`

Used by: `env-expand-unset-warn`

Contains a reference to an env var that will not be set.

```
foo
```

---

## New Scripts

### `scripts/delete-bar.qed`

Already exists.

### `scripts/extract-no-match.qed`

`--extract` with a pattern that matches nothing — output is empty.

```
at("quux", on_error:skip) | qed:skip()
```

### `scripts/dry-run-no-change.qed`

`--dry-run` when the script makes no changes — diff output is empty.

```
at("quux", on_error:skip) | qed:delete()
```

### `scripts/dry-run-multiple-hunks.qed`

`--dry-run` with changes in non-adjacent regions — produces multiple diff hunks.

```
at("foo") | qed:upper()
at("baz") | qed:upper()
```

### `scripts/output-flag-empty.qed`

Applied to empty input — `--output` file should be written but empty.

```
at("foo", on_error:skip) | qed:delete()
```

### `scripts/in-place-no-change.qed`

`--in-place` when nothing matches — file content unchanged.

```
at("quux", on_error:skip) | qed:delete()
```

### `scripts/env-expand-unset-warn.qed`

References an env var that is not set — should expand to empty string with a warning.

```
at("${QED_UNSET_VAR}", on_error:skip) | qed:delete()
```

### `scripts/global-on-error-overridden-by-selector.qed`

`--on-error=skip` globally, but the selector uses `on_error:fail` explicitly —
the per-selector setting wins.

```
at("quux", on_error:fail) | qed:delete()
```

---

## New Manifest Scenarios

```toml
# Append to tests/invocation/manifest.toml

# ── --output edge cases ───────────────────────────────────────────────────────

[[scenario]]
id = "output-flag-empty-input"
description = "--output writes an empty file when input is empty"
script = "output-flag-empty.qed"
input = "empty.txt"
stdout = "empty.txt"
stderr = "empty.txt"
output = "empty.txt"
exit_code = 0
invoke = [
  """qed -f "$SCRIPT" --output "$OUTPUT" "$INPUT" > "$STDOUT" 2> "$STDERR" """,
]

# ── --in-place edge cases ─────────────────────────────────────────────────────

[[scenario]]
id = "in-place-no-change"
description = "--in-place when nothing matches leaves the file content identical to the original"
script = "in-place-no-change.qed"
input = "three-lines.txt"
stdout = "empty.txt"
stderr = "empty.txt"
output = "foo-bar-baz.txt"
exit_code = 0
invoke = [
  """
  qed -f "$SCRIPT" --in-place "$INPUT" > "$STDOUT" 2> "$STDERR"
  cp "$INPUT" "$OUTPUT"
  """,
]

[[scenario]]
id = "in-place-empty-input"
description = "--in-place on an empty file leaves the file empty"
script = "output-flag-empty.qed"
input = "empty.txt"
stdout = "empty.txt"
stderr = "empty.txt"
output = "empty.txt"
exit_code = 0
invoke = [
  """
  qed -f "$SCRIPT" --in-place "$INPUT" > "$STDOUT" 2> "$STDERR"
  cp "$INPUT" "$OUTPUT"
  """,
]

# ── --extract edge cases ──────────────────────────────────────────────────────

[[scenario]]
id = "extract-no-match"
description = "--extract with no matching lines produces empty output"
script = "extract-no-match.qed"
input = "three-lines.txt"
stdout = "empty.txt"
stderr = "empty.txt"
output = "empty.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" --extract < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" --extract < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "extract-empty-input"
description = "--extract on empty input produces empty output"
script = "extract-no-match.qed"
input = "empty.txt"
stdout = "empty.txt"
stderr = "empty.txt"
output = "empty.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" --extract < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" --extract < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── --dry-run edge cases ──────────────────────────────────────────────────────

[[scenario]]
id = "dry-run-no-change"
description = "--dry-run when nothing changes produces empty diff output"
script = "dry-run-no-change.qed"
input = "three-lines.txt"
stdout = "empty.txt"
stderr = "empty.txt"
output = "empty.txt"
exit_code = 0
invoke = [
  """
  qed -f "$SCRIPT" --dry-run "$INPUT" > "$STDOUT" 2> "$STDERR"
  cp /dev/null "$OUTPUT"
  """,
]

[[scenario]]
id = "dry-run-multiple-hunks"
description = "--dry-run with non-adjacent changes produces multiple diff hunks"
script = "dry-run-multiple-hunks.qed"
input = "three-lines.txt"
stdout = "dry-run-multiple-hunks.txt"
stderr = "empty.txt"
output = "empty.txt"
exit_code = 0
invoke = [
  """
  qed -f "$SCRIPT" --dry-run "$INPUT" > "$STDOUT" 2> "$STDERR"
  cp /dev/null "$OUTPUT"
  """,
]

# ── --on-error edge cases ─────────────────────────────────────────────────────

[[scenario]]
id = "per-selector-on-error-overrides-global"
description = "per-selector on_error:fail overrides the global --on-error=skip; the statement fails and exits non-zero; unselected lines pass through to stdout"
script = "global-on-error-overridden-by-selector.qed"
input = "three-lines.txt"
stdout = "foo-bar-baz.txt"
stderr = "error-no-match.txt"
output = "foo-bar-baz.txt"
exit_code = 1
invoke = [
  """
  qed "$(cat "$SCRIPT")" --on-error=skip < "$INPUT" > "$STDOUT" 2> "$STDERR"
  QED_EXIT=$?
  cp "$STDOUT" "$OUTPUT"
  exit $QED_EXIT
  """,
  """
  qed -f "$SCRIPT" --on-error=skip < "$INPUT" > "$STDOUT" 2> "$STDERR"
  QED_EXIT=$?
  cp "$STDOUT" "$OUTPUT"
  exit $QED_EXIT
  """,
]

# ── --no-env edge cases ───────────────────────────────────────────────────────

[[scenario]]
id = "env-expand-unset-warns"
description = "referencing an unset env var without --no-env expands to empty string and emits a warning"
script = "env-expand-unset-warn.qed"
input = "three-lines.txt"
stdout = "foo-bar-baz.txt"
stderr = "unset-var-warn.txt"
output = "foo-bar-baz.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]
```

---

## New Golden Files

### `goldens/stdout/`

#### `foo-bar-baz.txt`

Already exists. Used by: `per-selector-on-error-overrides-global` (stdout, output) — all three lines pass through as unselected content.

#### `dry-run-multiple-hunks.txt`

Used by: `dry-run-multiple-hunks`

`foo` and `baz` are uppercased; `bar` is unchanged between them.
With three lines of context (unified diff default), the two changes are close
enough to collapse into a single hunk.

```
--- a
+++ b
@@ -1,3 +1,3 @@
-foo
+FOO
 bar
-baz
+BAZ
```

---

### `goldens/stderr/`

#### `error-no-match.txt`

Used by: `per-selector-on-error-overrides-global`

Script: `at("quux", on_error:fail) | qed:delete()` — selector at 1:1-25, `qed:delete()` at 1:29-40 (widest: 7 chars).

```
qed: error:   1:1-25:  at("quux", on_error:fail): no lines matched
```

#### `unset-var-warn.txt`

Used by: `env-expand-unset-warns`

Script: `at("${QED_UNSET_VAR}", on_error:skip) | qed:delete()` —
`${QED_UNSET_VAR}` is the env var reference at 1:5-20; `qed:delete()` at 1:41-52 (widest: 7 chars).

```
qed: warning: 1:5-20:  ${QED_UNSET_VAR}: environment variable not set, expanding to empty string
```

---

### `goldens/output/`

#### `foo-bar-baz.txt`

Already exists.
