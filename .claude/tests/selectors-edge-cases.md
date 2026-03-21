# Selector Edge Case Scenarios

Additional scenarios covering boundary conditions in selector behaviour.
These extend `tests/selectors/manifest.toml` with new scenarios, inputs, scripts, and goldens.

---

## New Inputs

### `inputs/empty.txt`

Used by: `at-empty-input`, `at-entire-stream-empty`

```
```

### `inputs/single-line.txt`

Used by: `at-single-line`, `from-single-line`, `to-single-line`,
`after-first-line`, `before-last-line`

```
foo
```

### `inputs/match-first-line.txt`

Used by: `at-match-first-line`, `from-first-line-inclusive`,
`to-first-line-exclusive`, `before-first-line`

```
target
middle
last
```

### `inputs/match-last-line.txt`

Used by: `at-match-last-line`, `from-last-line-exclusive`,
`to-last-line-inclusive`, `after-last-line`

```
first
middle
target
```

### `inputs/match-only-line.txt`

Used by: `at-match-only-line`, `from-only-line`, `to-only-line`

```
target
```

### `inputs/adjacent-matches.txt`

Used by: `at-adjacent-matches`, `nth-adjacent`

Pattern appears on consecutive lines.

```
x
x
y
```

### `inputs/five-lines.txt`

Already exists in selectors. Used by new `nth` edge cases.

---

## New Scripts

### `scripts/at-empty-input.qed`

```
at("foo") | qed:delete()
```

### `scripts/at-entire-stream-empty.qed`

```
at() | qed:upper()
```

### `scripts/at-single-line.qed`

```
at("foo") | qed:delete()
```

### `scripts/at-match-first-line.qed`

```
at("target") | qed:delete()
```

### `scripts/at-match-last-line.qed`

```
at("target") | qed:delete()
```

### `scripts/at-match-only-line.qed`

```
at("target") | qed:delete()
```

### `scripts/at-adjacent-matches.qed`

```
at("x") | qed:delete()
```

### `scripts/from-first-line-inclusive.qed`

`from` matching the first line with `+` — the entire stream is selected.

```
from("target"+) | qed:delete()
```

### `scripts/from-last-line-exclusive.qed`

`from` matching the last line without `+` — nothing after it, nothing selected.

```
from("target") | qed:delete()
```

### `scripts/from-only-line.qed`

`from` on the only line, exclusive — nothing selected.

```
from("target") | qed:delete()
```

### `scripts/to-first-line-exclusive.qed`

`to` matching the first line without `+` — nothing before it, nothing selected.

```
to("target") | qed:delete()
```

### `scripts/to-last-line-inclusive.qed`

`to` matching the last line with `+` — entire stream selected.

```
to("target"+) | qed:delete()
```

### `scripts/to-only-line.qed`

`to` on the only line, exclusive — nothing selected.

```
to("target") | qed:delete()
```

### `scripts/after-first-line.qed`

```
after("foo") | echo inserted
```

### `scripts/after-last-line.qed`

```
after("target") | echo inserted
```

### `scripts/before-first-line.qed`

```
before("target") | echo inserted
```

### `scripts/before-last-line.qed`

```
before("foo") | echo inserted
```

### `scripts/at-single-line-match.qed`

```
at("foo") | qed:upper()
```

### `scripts/nth-adjacent.qed`

`nth:1` on adjacent matches — only the first of two consecutive `x` lines deleted.

```
at("x", nth:1) | qed:delete()
```

### `scripts/nth-exceeds-count.qed`

`nth:5` requested but only three `x` lines exist — no match, `on_error:skip`.

```
at("x", nth:5, on_error:skip) | qed:delete()
```

### `scripts/nth-negative-step.qed`

`nth:-2n` selects every second occurrence from the end (end-positions 2, 4, 6…).
With three `x` matches: only match 2 (the middle `x`) is at end-position 2 and is deleted.

```
at("x", nth:-2n) | qed:delete()
```

### `scripts/plus-ignored-on-at.qed`

`+` on `at` is warned and ignored; the selector still matches and the processor runs.

```
at("bar"+) | qed:delete()
```

### `scripts/plus-ignored-on-after.qed`

`+` on `after` is warned and ignored; the insertion point still fires.
`qed:upper()` receives empty stdin from the insertion point and produces empty output —
nothing is inserted, stream passes through.

```
after("bar"+) | qed:upper()
```

### `scripts/plus-ignored-on-before.qed`

`+` on `before` is warned and ignored; the insertion point still fires.

```
before("bar"+) | qed:upper()
```

### `scripts/nth-zero-warned.qed`

`nth:0` has no meaning, is warned, and the term is ignored.
With no remaining nth terms the selector matches nothing; `on_error:skip` suppresses failure.

```
at("x", nth:0, on_error:skip) | qed:delete()
```

### `scripts/nth-duplicate-bare.qed`

`nth:1,1...3` — `1` duplicates the first term of the range `1...3`.
A warning fires and the duplicate is removed, leaving `nth:1...3`.

```
at("x", nth:1,1...3) | qed:delete()
```

### `scripts/nth-duplicate-in-range.qed`

`nth:1...3,2` — `2` is already covered by the range `1...3`.
A warning fires and the duplicate is removed, leaving `nth:1...3`.

```
at("x", nth:1...3,2) | qed:delete()
```

### `scripts/from-to-same-pattern.qed`

`from` and `to` match the same line — with both exclusive, nothing is selected.

```
from("charlie") > to("charlie") | qed:delete()
```

### `scripts/from-to-adjacent.qed`

`from` and `to` match adjacent lines — with both exclusive, nothing between them.

```
from("bravo") > to("charlie") | qed:delete()
```

---

## New Manifest Scenarios

```toml
# Append to tests/selectors/manifest.toml

# ── at() edge cases ───────────────────────────────────────────────────────────

[[scenario]]
id = "at-empty-input"
description = "at() on empty input produces empty output; no match is found and on_error:skip suppresses the failure"
script = "at-empty-input.qed"
input = "empty.txt"
stdout = "empty.txt"
stderr = "empty.txt"
output = "empty.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" --on-error=skip < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" --on-error=skip < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "at-entire-stream-empty"
description = "at() with no pattern on empty input produces empty output"
script = "at-entire-stream-empty.qed"
input = "empty.txt"
stdout = "empty.txt"
stderr = "empty.txt"
output = "empty.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "at-single-line"
description = "at() on a single-line input deletes the only line, producing empty output"
script = "at-single-line.qed"
input = "single-line.txt"
stdout = "empty.txt"
stderr = "empty.txt"
output = "empty.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "at-match-first-line"
description = "at() correctly selects and deletes a pattern that appears on the first line"
script = "at-match-first-line.qed"
input = "match-first-line.txt"
stdout = "middle-last.txt"
stderr = "empty.txt"
output = "middle-last.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "at-match-last-line"
description = "at() correctly selects and deletes a pattern that appears on the last line"
script = "at-match-last-line.qed"
input = "match-last-line.txt"
stdout = "first-middle.txt"
stderr = "empty.txt"
output = "first-middle.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "at-match-only-line"
description = "at() deletes the only line in a single-line input, producing empty output"
script = "at-match-only-line.qed"
input = "match-only-line.txt"
stdout = "empty.txt"
stderr = "empty.txt"
output = "empty.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "at-adjacent-matches"
description = "at() correctly selects all occurrences when the pattern appears on consecutive lines"
script = "at-adjacent-matches.qed"
input = "adjacent-matches.txt"
stdout = "y-only.txt"
stderr = "empty.txt"
output = "y-only.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── from() edge cases ─────────────────────────────────────────────────────────

[[scenario]]
id = "from-first-line-inclusive"
description = "from() with + matching the first line selects the entire stream"
script = "from-first-line-inclusive.qed"
input = "match-first-line.txt"
stdout = "empty.txt"
stderr = "empty.txt"
output = "empty.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "from-last-line-exclusive"
description = "from() exclusive matching the last line selects nothing; on_error:skip suppresses failure"
script = "from-last-line-exclusive.qed"
input = "match-last-line.txt"
stdout = "first-middle-target.txt"
stderr = "empty.txt"
output = "first-middle-target.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" --on-error=skip < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" --on-error=skip < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "from-only-line-exclusive"
description = "from() exclusive on the only line selects nothing; stream passes through unchanged"
script = "from-only-line.qed"
input = "match-only-line.txt"
stdout = "target-only.txt"
stderr = "empty.txt"
output = "target-only.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" --on-error=skip < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" --on-error=skip < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── to() edge cases ───────────────────────────────────────────────────────────

[[scenario]]
id = "to-first-line-exclusive"
description = "to() exclusive matching the first line selects nothing; stream passes through unchanged"
script = "to-first-line-exclusive.qed"
input = "match-first-line.txt"
stdout = "target-middle-last.txt"
stderr = "empty.txt"
output = "target-middle-last.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" --on-error=skip < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" --on-error=skip < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "to-last-line-inclusive"
description = "to() with + matching the last line selects the entire stream"
script = "to-last-line-inclusive.qed"
input = "match-last-line.txt"
stdout = "empty.txt"
stderr = "empty.txt"
output = "empty.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "to-only-line-exclusive"
description = "to() exclusive on the only line selects nothing; stream passes through unchanged"
script = "to-only-line.qed"
input = "match-only-line.txt"
stdout = "target-only.txt"
stderr = "empty.txt"
output = "target-only.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" --on-error=skip < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" --on-error=skip < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── after() / before() edge cases ─────────────────────────────────────────────

[[scenario]]
id = "after-first-line"
description = "after() on the first line inserts content between the first and second lines"
script = "after-first-line.qed"
input = "single-line.txt"
stdout = "foo-inserted.txt"
stderr = "empty.txt"
output = "foo-inserted.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "after-last-line"
description = "after() on the last line appends content at the end of the stream"
script = "after-last-line.qed"
input = "match-last-line.txt"
stdout = "first-middle-target-inserted.txt"
stderr = "empty.txt"
output = "first-middle-target-inserted.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "before-first-line"
description = "before() on the first line prepends content at the start of the stream"
script = "before-first-line.qed"
input = "match-first-line.txt"
stdout = "inserted-target-middle-last.txt"
stderr = "empty.txt"
output = "inserted-target-middle-last.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "before-last-line"
description = "before() on the last line inserts content between the second-to-last and last lines"
script = "before-last-line.qed"
input = "single-line.txt"
stdout = "inserted-foo.txt"
stderr = "empty.txt"
output = "inserted-foo.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── from > to edge cases ──────────────────────────────────────────────────────

[[scenario]]
id = "from-to-same-pattern"
description = "from > to where both match the same line and both are exclusive selects nothing"
script = "from-to-same-pattern.qed"
input = "five-lines.txt"
stdout = "five-lines.txt"
stderr = "empty.txt"
output = "five-lines.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" --on-error=skip < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" --on-error=skip < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "from-to-adjacent"
description = "from > to where boundaries are adjacent lines and both exclusive selects nothing between them"
script = "from-to-adjacent.qed"
input = "five-lines.txt"
stdout = "five-lines.txt"
stderr = "empty.txt"
output = "five-lines.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" --on-error=skip < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" --on-error=skip < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── nth edge cases ────────────────────────────────────────────────────────────

[[scenario]]
id = "nth-adjacent"
description = "nth:1 correctly selects only the first of two consecutive matching lines"
script = "nth-adjacent.qed"
input = "adjacent-matches.txt"
stdout = "x-y.txt"
stderr = "empty.txt"
output = "x-y.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "nth-exceeds-count"
description = "nth:N where N exceeds the number of matches produces no selection; on_error:skip suppresses failure"
script = "nth-exceeds-count.qed"
input = "repeated.txt"
stdout = "repeated.txt"
stderr = "empty.txt"
output = "repeated.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "nth-negative-step"
description = "nth:-2n selects every second occurrence from the end (end-positions 2, 4, 6…)"
script = "nth-negative-step.qed"
input = "repeated.txt"
stdout = "x-y-y-x.txt"
stderr = "empty.txt"
output = "x-y-y-x.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── + warning ─────────────────────────────────────────────────────────────────

[[scenario]]
id = "plus-ignored-on-at"
description = "+ on at() is warned and ignored; the selector still matches and the processor runs"
script = "plus-ignored-on-at.qed"
input = "three-lines.txt"
stdout = "foo-baz.txt"
stderr = "plus-ignored-at.txt"
output = "foo-baz.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "plus-ignored-on-after"
description = "+ on after() is warned and ignored; the insertion point still fires"
script = "plus-ignored-on-after.qed"
input = "three-lines.txt"
stdout = "foo-bar-baz.txt"
stderr = "plus-ignored-after.txt"
output = "foo-bar-baz.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "plus-ignored-on-before"
description = "+ on before() is warned and ignored; the insertion point still fires"
script = "plus-ignored-on-before.qed"
input = "three-lines.txt"
stdout = "foo-bar-baz.txt"
stderr = "plus-ignored-before.txt"
output = "foo-bar-baz.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── nth warning ───────────────────────────────────────────────────────────────

[[scenario]]
id = "nth-zero-warned"
description = "nth:0 has no meaning; a warning is emitted and the term is ignored, leaving no selection"
script = "nth-zero-warned.qed"
input = "repeated.txt"
stdout = "repeated.txt"
stderr = "nth-zero-warned.txt"
output = "repeated.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "nth-duplicate-bare"
description = "nth:1,1...3 warns that occurrence 1 is a duplicate and deduplicates to nth:1...3"
script = "nth-duplicate-bare.qed"
input = "repeated.txt"
stdout = "y-y.txt"
stderr = "nth-duplicate-bare.txt"
output = "y-y.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "nth-duplicate-in-range"
description = "nth:1...3,2 warns that occurrence 2 is already covered by the range and deduplicates to nth:1...3"
script = "nth-duplicate-in-range.qed"
input = "repeated.txt"
stdout = "y-y.txt"
stderr = "nth-duplicate-range.txt"
output = "y-y.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]
```

---

## New Golden Files

### `goldens/stdout/` and `goldens/output/`

#### `middle-last.txt`

Used by: `at-match-first-line`

`target` deleted from the first line; `middle` and `last` pass through.

```
middle
last
```

#### `first-middle.txt`

Used by: `at-match-last-line`

`target` deleted from the last line; `first` and `middle` pass through.

```
first
middle
```

#### `y-only.txt`

Used by: `at-adjacent-matches`

Both consecutive `x` lines deleted; only `y` remains.

```
y
```

#### `first-middle-target.txt`

Used by: `from-last-line-exclusive`

`from("target")` exclusive on the last line selects nothing; all three lines pass through.

```
first
middle
target
```

#### `target-only.txt`

Used by: `from-only-line-exclusive`, `to-only-line-exclusive`

The single `target` line passes through unchanged.

```
target
```

#### `target-middle-last.txt`

Used by: `to-first-line-exclusive`

`to("target")` exclusive on the first line selects nothing; all three lines pass through.

```
target
middle
last
```

#### `foo-inserted.txt`

Used by: `after-first-line`

`inserted` appears after the only line `foo`.

```
foo
inserted
```

#### `first-middle-target-inserted.txt`

Used by: `after-last-line`

`inserted` appended after the last line `target`.

```
first
middle
target
inserted
```

#### `inserted-target-middle-last.txt`

Used by: `before-first-line`

`inserted` prepended before the first line `target`.

```
inserted
target
middle
last
```

#### `inserted-foo.txt`

Used by: `before-last-line`

`inserted` appears before the only line `foo`.

```
inserted
foo
```

#### `five-lines.txt`

Used by: `from-to-same-pattern`, `from-to-adjacent`

Stream passes through unchanged — nothing was selected.

```
alpha
bravo
charlie
delta
echo
```

#### `x-y.txt`

Used by: `nth-adjacent`

First `x` deleted; second `x` and `y` remain.

```
x
y
```

#### `repeated.txt`

Used by: `nth-exceeds-count`

Stream passes through unchanged — `nth:5` found no fifth occurrence.

```
x
y
x
y
x
```

#### `x-y-y-x.txt`

Used by: `nth-negative-step`

Input `repeated.txt` is `x / y / x / y / x` — three `x` matches at match-positions 1, 2, 3.
`nth:-2n` selects every second match counting from the end (end-positions 2, 4, 6…).
With three matches: end-position 1 is match 3, end-position 2 is match 2, end-position 3 is match 1.
Only match 2 (the middle `x`, at stream line 3) is selected and deleted.
`x`, `y`, `y`, and `x` remain.

```
x
y
y
x
```

#### `foo-bar-baz.txt`

Already exists.
Reused by: `plus-ignored-on-after`, `plus-ignored-on-before`

Nothing inserted — `qed:upper()` on an empty insertion point produces empty output.

#### `foo-baz.txt`

Already exists.
Reused by: `plus-ignored-on-at`

`+` ignored; `at("bar")` selects and deletes `bar`.

#### `repeated.txt`

Used by: `nth-zero-warned`

`nth:0` term is warned and ignored; with no remaining nth terms the selector matches nothing.
`on_error:skip` suppresses failure. Stream passes through unchanged.

---

### `goldens/stderr/`

#### `plus-ignored-at.txt`

Used by: `plus-ignored-on-at`

Script: `at("bar"+) | qed:delete()`
`at("bar"+)` at `1:1-11` (10 chars); widest span `qed:delete()` at `1:14-26` (12 chars) → location width 7.

```
qed: warning: 1:1-11: at("bar"+): + ignored on at
```

#### `plus-ignored-after.txt`

Used by: `plus-ignored-on-after`

Script: `after("bar"+) | qed:upper()`
`after("bar"+)` at `1:1-14` (13 chars); widest span `qed:upper()` at `1:17-28` (11 chars) → location width 7.

```
qed: warning: 1:1-14: after("bar"+): + ignored on after
```

#### `plus-ignored-before.txt`

Used by: `plus-ignored-on-before`

Script: `before("bar"+) | qed:upper()`
`before("bar"+)` at `1:1-15` (14 chars); widest span `qed:upper()` at `1:18-29` (11 chars) → location width 7.

```
qed: warning: 1:1-15: before("bar"+): + ignored on before
```

#### `nth-zero-warned.txt`

Used by: `nth-zero-warned`

Script: `at("x", nth:0, on_error:skip) | qed:delete()`
`nth:0` at `1:9-14` (5 chars); widest span `qed:delete()` at `1:33-45` (12 chars) → location width 7.

```
qed: warning: 1:9-14: nth:0: 0 has no meaning in nth, term ignored
```

#### `nth-duplicate-bare.txt`

Used by: `nth-duplicate-bare`

Script: `at("x", nth:1,1...3) | qed:delete()`
`nth:1,1...3` at `1:9-20` (11 chars); widest span `qed:delete()` at `1:24-36` (12 chars) → location width 7.

```
qed: warning: 1:9-20: nth:1,1...3: duplicate occurrence 1, deduplicated
```

#### `nth-duplicate-range.txt`

Used by: `nth-duplicate-in-range`

Script: `at("x", nth:1...3,2) | qed:delete()`
`nth:1...3,2` at `1:9-20` (11 chars); widest span `qed:delete()` at `1:24-36` (12 chars) → location width 7.

```
qed: warning: 1:9-20: nth:1...3,2: duplicate occurrence 2, deduplicated
```
