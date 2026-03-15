# Processor Edge Case Scenarios

Additional scenarios covering boundary conditions in processor behaviour.
These extend `tests/processors/` with new scenarios, inputs, scripts, and goldens.

---

## New Inputs

### `inputs/empty.txt`

Used by: `delete-empty`, `upper-empty`, `replace-no-match`, `substring-no-match`,
`trim-no-whitespace`, `indent-empty`, `wrap-short-lines`, `prefix-empty`, `number-empty`

```
```

### `inputs/single-line.txt`

Used by: `delete-single`, `duplicate-single`, `upper-single`, `number-single`

```
foo
```

### `inputs/whitespace-lines.txt`

Used by: `trim-whitespace-only`, `dedent-mixed-indent`

Lines with varying leading and trailing whitespace.

```
   
  foo  
	bar
```

### `inputs/mixed-indent.txt`

Used by: `dedent-mixed-indent`

Lines with inconsistent indentation — two spaces on some, four on others.
`dedent` removes only the common prefix (two spaces).

```
  foo
    bar
  baz
```

### `inputs/short-lines.txt`

Used by: `wrap-short-lines`

Lines already under the wrap width; `wrap` should leave them unchanged.

```
hi
ok
```

### `inputs/replace-source.txt`

Used by: `replace-literal-no-match`, `replace-regex-no-match`,
`replace-multiple-occurrences`, `replace-adjacent-matches`

```
foo bar foo
```

### `inputs/substring-source.txt`

Used by: `substring-no-match-on-line`, `substring-multiple-matches`

```
foobar
barfoo
nope
```

### `inputs/copy-move-source.txt`

Used by: `copy-range`, `move-range`

```
alpha
start
middle
end
beta
```

---

## New Scripts

### `scripts/delete-empty.qed`

```
at("foo", on_error:skip) | qed:delete()
```

### `scripts/delete-single.qed`

```
at("foo") | qed:delete()
```

### `scripts/duplicate-single.qed`

```
at("foo") | qed:duplicate()
```

### `scripts/upper-empty.qed`

```
at() | qed:upper()
```

### `scripts/upper-single.qed`

```
at() | qed:upper()
```

### `scripts/replace-literal-no-match.qed`

`qed:replace()` where the literal pattern does not appear in the selected line.
The line passes through unchanged.

```
at("foo bar foo") | qed:replace("quux", "replaced")
```

### `scripts/replace-multiple-occurrences.qed`

`qed:replace()` replaces all occurrences of the literal within the selected region.

```
at("foo bar foo") | qed:replace("foo", "baz")
```

### `scripts/replace-regex-no-match.qed`

`qed:replace()` where the regex matches no span within the selected line.

```
at("foo bar foo") | qed:replace(/\d+/, "NUM")
```

### `scripts/replace-adjacent-matches.qed`

`qed:replace()` with a regex that matches adjacent spans — verifies no span is skipped.

```
at("foo bar foo") | qed:replace(/foo/, "X")
```

### `scripts/substring-no-match-on-line.qed`

`qed:substring()` where the pattern matches some lines but not others.
Lines with no match pass through unchanged.

```
at(/bar/) | qed:substring(/bar/)
```

### `scripts/substring-multiple-matches.qed`

`qed:substring()` where the pattern matches multiple spans on a line —
only the first match is kept (RE2 leftmost semantics).

```
at("foobar") | qed:substring(/o+/)
```

### `scripts/trim-no-whitespace.qed`

```
at() | qed:trim()
```

### `scripts/trim-whitespace-only.qed`

Lines that are whitespace-only become empty lines after trim.

```
at() | qed:trim()
```

### `scripts/dedent-mixed-indent.qed`

```
at() | qed:dedent()
```

### `scripts/wrap-short-lines.qed`

Lines already under the wrap width are not modified.

```
at() | qed:wrap(width:20)
```

### `scripts/prefix-empty.qed`

```
at(on_error:skip) | qed:prefix(text:"// ")
```

### `scripts/number-empty.qed`

```
at(on_error:skip) | qed:number()
```

### `scripts/number-single.qed`

```
at() | qed:number()
```

### `scripts/copy-range.qed`

Copies a `from > to` region to after a target pattern.

```
src_start="start"
src_end="end"
from(src_start+) > to(src_end+) | qed:copy(after:"beta")
```

### `scripts/move-range.qed`

Moves a `from > to` region to after a target pattern.

```
src_start="start"
src_end="end"
from(src_start+) > to(src_end+) | qed:move(after:"beta")
```

---

## New Manifest Scenarios

```toml
# Append to tests/processors/manifest.toml

# ── qed:delete() edge cases ───────────────────────────────────────────────────

[[scenario]]
id = "delete-empty-input"
description = "qed:delete() on empty input produces empty output"
script = "delete-empty.qed"
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
id = "delete-single-line"
description = "qed:delete() on the only line in a single-line input produces empty output"
script = "delete-single.qed"
input = "single-line.txt"
stdout = "empty.txt"
stderr = "empty.txt"
output = "empty.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── qed:duplicate() edge cases ────────────────────────────────────────────────

[[scenario]]
id = "duplicate-single-line"
description = "qed:duplicate() on a single-line input emits that line twice"
script = "duplicate-single.qed"
input = "single-line.txt"
stdout = "foo-foo.txt"
stderr = "empty.txt"
output = "foo-foo.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── qed:upper() / qed:lower() edge cases ─────────────────────────────────────

[[scenario]]
id = "upper-empty-input"
description = "qed:upper() on empty input produces empty output"
script = "upper-empty.qed"
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
id = "upper-single-line"
description = "qed:upper() on a single-line input uppercases that line"
script = "upper-single.qed"
input = "single-line.txt"
stdout = "foo-upper.txt"
stderr = "empty.txt"
output = "foo-upper.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── qed:replace() edge cases ──────────────────────────────────────────────────

[[scenario]]
id = "replace-literal-no-match"
description = "qed:replace() where the literal does not appear in the region leaves the line unchanged"
script = "replace-literal-no-match.qed"
input = "replace-source.txt"
stdout = "replace-source.txt"
stderr = "empty.txt"
output = "replace-source.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "replace-regex-no-match"
description = "qed:replace() where the regex matches nothing in the region leaves the line unchanged"
script = "replace-regex-no-match.qed"
input = "replace-source.txt"
stdout = "replace-source.txt"
stderr = "empty.txt"
output = "replace-source.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "replace-multiple-occurrences"
description = "qed:replace() replaces all occurrences of the pattern within the selected region"
script = "replace-multiple-occurrences.qed"
input = "replace-source.txt"
stdout = "replace-multiple-result.txt"
stderr = "empty.txt"
output = "replace-multiple-result.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "replace-adjacent-matches"
description = "qed:replace() with adjacent matches replaces each span without skipping"
script = "replace-adjacent-matches.qed"
input = "replace-source.txt"
stdout = "replace-adjacent-result.txt"
stderr = "empty.txt"
output = "replace-adjacent-result.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── qed:substring() edge cases ────────────────────────────────────────────────

[[scenario]]
id = "substring-no-match-on-line"
description = "qed:substring() on a line where the pattern does not match passes the line through unchanged"
script = "substring-no-match-on-line.qed"
input = "substring-source.txt"
stdout = "substring-no-match-result.txt"
stderr = "empty.txt"
output = "substring-no-match-result.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "substring-multiple-spans"
description = "qed:substring() with multiple spans on a line keeps only the first match (RE2 leftmost semantics)"
script = "substring-multiple-matches.qed"
input = "substring-source.txt"
stdout = "substring-multiple-result.txt"
stderr = "empty.txt"
output = "substring-multiple-result.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── qed:trim() edge cases ─────────────────────────────────────────────────────

[[scenario]]
id = "trim-no-whitespace"
description = "qed:trim() on lines with no leading or trailing whitespace leaves them unchanged"
script = "trim-no-whitespace.qed"
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
id = "trim-whitespace-only-lines"
description = "qed:trim() on whitespace-only lines produces empty lines"
script = "trim-whitespace-only.qed"
input = "whitespace-lines.txt"
stdout = "trim-whitespace-result.txt"
stderr = "empty.txt"
output = "trim-whitespace-result.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── qed:dedent() edge cases ───────────────────────────────────────────────────

[[scenario]]
id = "dedent-mixed-indent"
description = "qed:dedent() removes only the common leading whitespace prefix; lines with more indent retain their relative indent"
script = "dedent-mixed-indent.qed"
input = "mixed-indent.txt"
stdout = "dedent-mixed-result.txt"
stderr = "empty.txt"
output = "dedent-mixed-result.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── qed:wrap() edge cases ─────────────────────────────────────────────────────

[[scenario]]
id = "wrap-short-lines"
description = "qed:wrap() on lines already under the wrap width leaves them unchanged"
script = "wrap-short-lines.qed"
input = "short-lines.txt"
stdout = "short-lines.txt"
stderr = "empty.txt"
output = "short-lines.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── qed:number() edge cases ───────────────────────────────────────────────────

[[scenario]]
id = "number-empty-input"
description = "qed:number() on empty input produces empty output"
script = "number-empty.qed"
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
id = "number-single-line"
description = "qed:number() on a single-line input prefixes the line with 1"
script = "number-single.qed"
input = "single-line.txt"
stdout = "number-single-result.txt"
stderr = "empty.txt"
output = "number-single-result.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── qed:copy() / qed:move() edge cases ───────────────────────────────────────

[[scenario]]
id = "copy-range"
description = "qed:copy() with a from > to region as the source copies the entire range to the destination"
script = "copy-range.qed"
input = "copy-move-source.txt"
stdout = "copy-range-result.txt"
stderr = "empty.txt"
output = "copy-range-result.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "move-range"
description = "qed:move() with a from > to region removes the source and inserts it at the destination"
script = "move-range.qed"
input = "copy-move-source.txt"
stdout = "move-range-result.txt"
stderr = "empty.txt"
output = "move-range-result.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]
```

---

## New Golden Files

### `goldens/stdout/` and `goldens/output/`

#### `foo-foo.txt`

Used by: `duplicate-single-line`

```
foo
foo
```

#### `foo-upper.txt`

Used by: `upper-single-line`

```
FOO
```

#### `replace-source.txt`

Used by: `replace-literal-no-match`, `replace-regex-no-match`

Stream passes through unchanged.

```
foo bar foo
```

#### `replace-multiple-result.txt`

Used by: `replace-multiple-occurrences`

Both occurrences of `foo` replaced with `baz`.

```
baz bar baz
```

#### `replace-adjacent-result.txt`

Used by: `replace-adjacent-matches`

Both occurrences of `foo` replaced with `X`; `bar` between them is untouched.

```
X bar X
```

#### `substring-no-match-result.txt`

Used by: `substring-no-match-on-line`

`at(/bar/)` selects `foobar` and `barfoo`; `qed:substring(/bar/)` narrows each to `bar`.
`nope` is not selected and passes through unchanged.

```
bar
bar
nope
```

#### `substring-multiple-result.txt`

Used by: `substring-multiple-spans`

`at("foobar")` selects the first line; `qed:substring(/o+/)` keeps only the
first (leftmost) match of `o+`, which is `oo` in `foobar`.
`barfoo` is not selected and passes through; `nope` is not selected and passes through.

```
oo
barfoo
nope
```

#### `trim-whitespace-result.txt`

Used by: `trim-whitespace-only-lines`

The whitespace-only first line becomes an empty line; `foo` and `bar` lose
their surrounding whitespace.

```

foo
bar
```

#### `dedent-mixed-result.txt`

Used by: `dedent-mixed-indent`

Common two-space prefix removed; `bar` retains its extra two-space relative indent.

```
foo
  bar
baz
```

#### `short-lines.txt`

Used by: `wrap-short-lines`

Stream passes through unchanged.

```
hi
ok
```

#### `number-single-result.txt`

Used by: `number-single-line`

Single-line input; stream line number is 1, colon-space separator, minimal padding.

```
1: foo
```

#### `copy-range-result.txt`

Used by: `copy-range`

`start` through `end` (inclusive) copied to after `beta`; original region remains.

```
alpha
start
middle
end
beta
start
middle
end
```

#### `move-range-result.txt`

Used by: `move-range`

`start` through `end` (inclusive) moved to after `beta`; removed from original position.

```
alpha
beta
start
middle
end
```
