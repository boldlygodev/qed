# Selector Scenarios

Tests covering all selector types for broad coverage.
One scenario per key selector behavior.
Edge cases and combinations are added in subsequent passes.

---

## Directory Layout

```
tests/selectors/
  manifest.toml
  inputs/
    three-lines.txt
    five-lines.txt
    repeated.txt
    narrowing.txt
  scripts/
    at-entire-stream.qed
    at-bar-delete.qed
    at-x-delete.qed
    at-regex-b-delete.qed
    at-negated-bar-delete.qed
    after-bar-insert.qed
    before-bar-insert.qed
    from-charlie-delete.qed
    from-charlie-inclusive-delete.qed
    to-charlie-delete.qed
    to-charlie-inclusive-delete.qed
    from-bravo-to-delta-delete.qed
    from-bravo-inclusive-to-delta-delete.qed
    from-bravo-to-delta-inclusive-delete.qed
    at-narrowing-delete.qed
    at-x-nth-first-delete.qed
    at-x-nth-last-delete.qed
    at-x-nth-second-delete.qed
    at-x-nth-range-delete.qed
    at-x-nth-step-delete.qed
    at-quux-fail.qed
    at-quux-warn.qed
    at-quux-skip.qed
  goldens/
    stdout/
      (same filenames and content as goldens/output/)
    stderr/
      empty.txt
      warn-no-match.txt
      error-no-match.txt
    output/
      empty.txt
      foo-bar-baz.txt
      foo-bar-baz-upper.txt
      foo-baz.txt
      bar.txt
      foo-bar-inserted-baz.txt
      foo-inserted-bar-baz.txt
      y-y.txt
      alpha-bravo.txt
      alpha-bravo-charlie.txt
      charlie-delta-echo.txt
      delta-echo.txt
      alpha-bravo-delta-echo.txt
      alpha-delta-echo.txt
      alpha-echo.txt
      five-minus-bravo.txt
      narrowing-minus-foo-bar.txt
      y-x-y-x.txt
      x-y-x-y.txt
      x-y-y-x.txt
      y-y-x.txt
      y-x-y.txt
```

---

## Manifest

```toml
# tests/selectors/manifest.toml

# ── at() ──────────────────────────────────────────────────────────────────────

[[scenario]]
id = "at-entire-stream"
description = "at() with no pattern selects the entire stream; every line is transformed"
script = "at-entire-stream.qed"
input = "three-lines.txt"
stdout = "foo-bar-baz-upper.txt"
stderr = "empty.txt"
output = "foo-bar-baz-upper.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── at(pattern) ───────────────────────────────────────────────────────────────

[[scenario]]
id = "at-literal-single-match"
description = "at() with a literal string selects all matching lines; unselected lines pass through"
script = "at-bar-delete.qed"
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
id = "at-literal-multi-match"
description = "at() selects all occurrences when the pattern appears on multiple lines"
script = "at-x-delete.qed"
input = "repeated.txt"
stdout = "y-y.txt"
stderr = "empty.txt"
output = "y-y.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "at-regex-match"
description = "at() with a regex selects all lines matching the pattern"
script = "at-regex-b-delete.qed"
input = "five-lines.txt"
stdout = "five-minus-bravo.txt"
stderr = "empty.txt"
output = "five-minus-bravo.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "at-negated"
description = "at() with a negated pattern selects all lines that do not match; matching lines pass through"
script = "at-negated-bar-delete.qed"
input = "three-lines.txt"
stdout = "bar.txt"
stderr = "empty.txt"
output = "bar.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── after(pattern) ────────────────────────────────────────────────────────────

[[scenario]]
id = "after-literal"
description = "after() is an empty insertion point immediately after the matching line; processor stdout is inserted there"
script = "after-bar-insert.qed"
input = "three-lines.txt"
stdout = "foo-bar-inserted-baz.txt"
stderr = "empty.txt"
output = "foo-bar-inserted-baz.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── before(pattern) ───────────────────────────────────────────────────────────

[[scenario]]
id = "before-literal"
description = "before() is an empty insertion point immediately before the matching line; processor stdout is inserted there"
script = "before-bar-insert.qed"
input = "three-lines.txt"
stdout = "foo-inserted-bar-baz.txt"
stderr = "empty.txt"
output = "foo-inserted-bar-baz.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── from(pattern) ─────────────────────────────────────────────────────────────

[[scenario]]
id = "from-exclusive"
description = "from() selects from after the matching line to end of stream; the matching line is excluded"
script = "from-charlie-delete.qed"
input = "five-lines.txt"
stdout = "alpha-bravo-charlie.txt"
stderr = "empty.txt"
output = "alpha-bravo-charlie.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "from-inclusive"
description = "from() with + includes the matching line in the selected region"
script = "from-charlie-inclusive-delete.qed"
input = "five-lines.txt"
stdout = "alpha-bravo.txt"
stderr = "empty.txt"
output = "alpha-bravo.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── to(pattern) ───────────────────────────────────────────────────────────────

[[scenario]]
id = "to-exclusive"
description = "to() selects from start of stream to before the matching line; the matching line is excluded"
script = "to-charlie-delete.qed"
input = "five-lines.txt"
stdout = "charlie-delta-echo.txt"
stderr = "empty.txt"
output = "charlie-delta-echo.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "to-inclusive"
description = "to() with + includes the matching line in the selected region"
script = "to-charlie-inclusive-delete.qed"
input = "five-lines.txt"
stdout = "delta-echo.txt"
stderr = "empty.txt"
output = "delta-echo.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── from > to ─────────────────────────────────────────────────────────────────

[[scenario]]
id = "from-to-both-exclusive"
description = "from > to selects lines between two patterns; both boundary lines are excluded by default"
script = "from-bravo-to-delta-delete.qed"
input = "five-lines.txt"
stdout = "alpha-bravo-delta-echo.txt"
stderr = "empty.txt"
output = "alpha-bravo-delta-echo.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "from-to-from-inclusive"
description = "from+ > to includes the from boundary line; the to boundary line remains excluded"
script = "from-bravo-inclusive-to-delta-delete.qed"
input = "five-lines.txt"
stdout = "alpha-delta-echo.txt"
stderr = "empty.txt"
output = "alpha-delta-echo.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "from-to-both-inclusive"
description = "from+ > to+ includes both boundary lines in the selected region"
script = "from-bravo-to-delta-inclusive-delete.qed"
input = "five-lines.txt"
stdout = "alpha-echo.txt"
stderr = "empty.txt"
output = "alpha-echo.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── > narrowing ───────────────────────────────────────────────────────────────

[[scenario]]
id = "at-narrowing"
description = "> intersects two at() regions; only lines matched by both selectors are selected"
script = "at-narrowing-delete.qed"
input = "narrowing.txt"
stdout = "narrowing-minus-foo-bar.txt"
stderr = "empty.txt"
output = "narrowing-minus-foo-bar.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── nth ───────────────────────────────────────────────────────────────────────

[[scenario]]
id = "nth-first"
description = "nth:1 selects only the first occurrence of a matching line"
script = "at-x-nth-first-delete.qed"
input = "repeated.txt"
stdout = "y-x-y-x.txt"
stderr = "empty.txt"
output = "y-x-y-x.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "nth-last"
description = "nth:-1 selects only the last occurrence of a matching line"
script = "at-x-nth-last-delete.qed"
input = "repeated.txt"
stdout = "x-y-x-y.txt"
stderr = "empty.txt"
output = "x-y-x-y.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "nth-specific"
description = "nth:N selects only the Nth occurrence of a matching line"
script = "at-x-nth-second-delete.qed"
input = "repeated.txt"
stdout = "x-y-y-x.txt"
stderr = "empty.txt"
output = "x-y-y-x.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "nth-range"
description = "nth:a...b selects occurrences a through b inclusive"
script = "at-x-nth-range-delete.qed"
input = "repeated.txt"
stdout = "y-y-x.txt"
stderr = "empty.txt"
output = "y-y-x.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "nth-step"
description = "nth:an+b selects every ath occurrence offset by b; 2n+1 selects the 1st, 3rd, 5th, ..."
script = "at-x-nth-step-delete.qed"
input = "repeated.txt"
stdout = "y-x-y.txt"
stderr = "empty.txt"
output = "y-x-y.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── on_error ──────────────────────────────────────────────────────────────────

[[scenario]]
id = "on-error-fail"
description = "on_error:fail (default) exits non-zero when no lines match; unselected lines pass through to stdout before the failure is detected"
script = "at-quux-fail.qed"
input = "three-lines.txt"
stdout = "foo-bar-baz.txt"
stderr = "error-no-match.txt"
output = "foo-bar-baz.txt"
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
id = "on-error-warn"
description = "on_error:warn emits a warning to stderr and exits zero when no lines match; the stream passes through unchanged"
script = "at-quux-warn.qed"
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
id = "on-error-skip"
description = "on_error:skip exits zero silently when no lines match; the stream passes through unchanged"
script = "at-quux-skip.qed"
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

Used by: `at-literal-single-match`, `at-negated`, `after-literal`, `before-literal`,
`on-error-fail`, `on-error-warn`, `on-error-skip`

```
foo
bar
baz
```

### `inputs/five-lines.txt`

Used by: `at-regex-match`, `from-exclusive`, `from-inclusive`, `to-exclusive`, `to-inclusive`,
`from-to-both-exclusive`, `from-to-from-inclusive`, `from-to-both-inclusive`

```
alpha
bravo
charlie
delta
echo
```

### `inputs/repeated.txt`

Used by: `at-literal-multi-match`, `nth-first`, `nth-last`, `nth-specific`, `nth-range`, `nth-step`

The pattern `x` appears three times, at positions 1, 3, and 5.

```
x
y
x
y
x
```

### `inputs/narrowing.txt`

Used by: `at-narrowing`

Each line contains two space-separated tokens.
`at(/foo/) > at(/bar/)` intersects to select only `foo bar`.

```
foo bar
foo baz
qux bar
qux baz
```

---

## Script Files

### `scripts/at-entire-stream.qed`

```
at() | qed:upper()
```

### `scripts/at-bar-delete.qed`

```
at("bar") | qed:delete()
```

### `scripts/at-x-delete.qed`

```
at("x") | qed:delete()
```

### `scripts/at-regex-b-delete.qed`

```
at(/^b/) | qed:delete()
```

### `scripts/at-negated-bar-delete.qed`

```
at(!"bar") | qed:delete()
```

### `scripts/after-bar-insert.qed`

```
after("bar") | echo inserted
```

### `scripts/before-bar-insert.qed`

```
before("bar") | echo inserted
```

### `scripts/from-charlie-delete.qed`

```
from("charlie") | qed:delete()
```

### `scripts/from-charlie-inclusive-delete.qed`

```
from("charlie"+) | qed:delete()
```

### `scripts/to-charlie-delete.qed`

```
to("charlie") | qed:delete()
```

### `scripts/to-charlie-inclusive-delete.qed`

```
to("charlie"+) | qed:delete()
```

### `scripts/from-bravo-to-delta-delete.qed`

```
from("bravo") > to("delta") | qed:delete()
```

### `scripts/from-bravo-inclusive-to-delta-delete.qed`

```
from("bravo"+) > to("delta") | qed:delete()
```

### `scripts/from-bravo-to-delta-inclusive-delete.qed`

```
from("bravo"+) > to("delta"+) | qed:delete()
```

### `scripts/at-narrowing-delete.qed`

```
at(/foo/) > at(/bar/) | qed:delete()
```

### `scripts/at-x-nth-first-delete.qed`

```
at("x", nth:1) | qed:delete()
```

### `scripts/at-x-nth-last-delete.qed`

```
at("x", nth:-1) | qed:delete()
```

### `scripts/at-x-nth-second-delete.qed`

```
at("x", nth:2) | qed:delete()
```

### `scripts/at-x-nth-range-delete.qed`

```
at("x", nth:1...2) | qed:delete()
```

### `scripts/at-x-nth-step-delete.qed`

Selects the 1st and 3rd `x` (positions 1 and 5 in `repeated.txt`).

```
at("x", nth:2n+1) | qed:delete()
```

### `scripts/at-quux-fail.qed`

Relies on the default `on_error:fail` behaviour.

```
at("quux") | qed:delete()
```

### `scripts/at-quux-warn.qed`

```
at("quux", on_error:warn) | qed:delete()
```

### `scripts/at-quux-skip.qed`

```
at("quux", on_error:skip) | qed:delete()
```

---

## Golden Files

`goldens/stdout/` and `goldens/output/` contain files with identical content.
They are listed once below; both directories contain a copy of each.
`goldens/stderr/` is listed separately.

### `goldens/stdout/` and `goldens/output/`

#### `empty.txt`

```
```

#### `foo-bar-baz.txt`

Used by: `on-error-fail` (stdout, output), `on-error-warn` (stdout, output), `on-error-skip` (stdout, output)

```
foo
bar
baz
```

#### `foo-bar-baz-upper.txt`

Used by: `at-entire-stream` (stdout, output)

```
FOO
BAR
BAZ
```

#### `foo-baz.txt`

Used by: `at-literal-single-match` (stdout, output)

`bar` has been deleted; `foo` and `baz` pass through.

```
foo
baz
```

#### `bar.txt`

Used by: `at-negated` (stdout, output)

All lines except `bar` deleted; only `bar` remains.

```
bar
```

#### `foo-bar-inserted-baz.txt`

Used by: `after-literal` (stdout, output)

`inserted` appears on a new line after `bar`.

```
foo
bar
inserted
baz
```

#### `foo-inserted-bar-baz.txt`

Used by: `before-literal` (stdout, output)

`inserted` appears on a new line before `bar`.

```
foo
inserted
bar
baz
```

#### `y-y.txt`

Used by: `at-literal-multi-match` (stdout, output)

All three `x` lines deleted; only the two `y` lines remain.

```
y
y
```

#### `alpha-bravo-charlie.txt`

Used by: `from-exclusive` (stdout, output)

`from("charlie")` selects `delta` and `echo` (charlie excluded);
deleting them leaves `alpha`, `bravo`, `charlie`.

```
alpha
bravo
charlie
```

#### `alpha-bravo.txt`

Used by: `from-inclusive` (stdout, output)

`from("charlie"+)` selects `charlie`, `delta`, `echo`;
deleting them leaves `alpha`, `bravo`.

```
alpha
bravo
```

#### `charlie-delta-echo.txt`

Used by: `to-exclusive` (stdout, output)

`to("charlie")` selects `alpha`, `bravo` (charlie excluded);
deleting them leaves `charlie`, `delta`, `echo`.

```
charlie
delta
echo
```

#### `delta-echo.txt`

Used by: `to-inclusive` (stdout, output)

`to("charlie"+)` selects `alpha`, `bravo`, `charlie`;
deleting them leaves `delta`, `echo`.

```
delta
echo
```

#### `alpha-bravo-delta-echo.txt`

Used by: `from-to-both-exclusive` (stdout, output)

`from("bravo") > to("delta")` selects `charlie` only (both boundaries excluded);
deleting it leaves `alpha`, `bravo`, `delta`, `echo`.

```
alpha
bravo
delta
echo
```

#### `alpha-delta-echo.txt`

Used by: `from-to-from-inclusive` (stdout, output)

`from("bravo"+) > to("delta")` selects `bravo`, `charlie` (delta excluded);
deleting them leaves `alpha`, `delta`, `echo`.

```
alpha
delta
echo
```

#### `alpha-echo.txt`

Used by: `from-to-both-inclusive` (stdout, output)

`from("bravo"+) > to("delta"+)` selects `bravo`, `charlie`, `delta`;
deleting them leaves `alpha`, `echo`.

```
alpha
echo
```

#### `five-minus-bravo.txt`

Used by: `at-regex-match` (stdout, output)

`/^b/` matches `bravo`; deleting it leaves the other four lines.

```
alpha
charlie
delta
echo
```

#### `narrowing-minus-foo-bar.txt`

Used by: `at-narrowing` (stdout, output)

`at(/foo/) > at(/bar/)` matches only `foo bar` (the intersection);
deleting it leaves the other three lines.

```
foo baz
qux bar
qux baz
```

#### `y-x-y-x.txt`

Used by: `nth-first` (stdout, output)

`nth:1` deletes the 1st `x` (line 1); lines 2–5 remain.

```
y
x
y
x
```

#### `x-y-x-y.txt`

Used by: `nth-last` (stdout, output)

`nth:-1` deletes the 3rd `x` (line 5); lines 1–4 remain.

```
x
y
x
y
```

#### `x-y-y-x.txt`

Used by: `nth-specific` (stdout, output)

`nth:2` deletes the 2nd `x` (line 3); lines 1, 2, 4, 5 remain.

```
x
y
y
x
```

#### `y-y-x.txt`

Used by: `nth-range` (stdout, output)

`nth:1...2` deletes the 1st and 2nd `x` (lines 1 and 3); lines 2, 4, 5 remain.

```
y
y
x
```

#### `y-x-y.txt`

Used by: `nth-step` (stdout, output)

`nth:2n+1` deletes the 1st and 3rd `x` (lines 1 and 5); lines 2, 3, 4 remain.

```
y
x
y
```

---

### `goldens/stderr/`

#### `empty.txt`

```
```

#### `error-no-match.txt`

Used by: `on-error-fail`

Script: `at("quux") | qed:delete()` — selector at 1:1-10, `qed:delete()` at 1:14-25 (widest: 7 chars).

```
qed: error:   1:1-10: at("quux"): no lines matched
```

#### `warn-no-match.txt`

Used by: `on-error-warn`

Script: `at("quux", on_error:warn) | qed:delete()` — selector at 1:1-25, `qed:delete()` at 1:29-40 (widest: 7 chars).

```
qed: warning: 1:1-25: at("quux", on_error:warn): no lines matched
```
