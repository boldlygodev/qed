# Processor Scenarios

Tests covering all internal processors for broad coverage.
One scenario per key processor behavior.
Edge cases and combinations are added in subsequent passes.

Generation processors (`qed:uuid`, `qed:timestamp`, `qed:random`) and
stream control processors (`qed:warn`, `qed:fail`, `qed:skip`, `qed:debug:*`)
are covered in their own feature directories.

---

## Directory Layout

```
tests/processors/
  manifest.toml
  inputs/
    three-lines.txt
    indented.txt
    long-lines.txt
    placeholder.txt
    copy-source.txt
  scripts/
    delete.qed
    duplicate.qed
    copy-after.qed
    copy-before.qed
    copy-at.qed
    move-after.qed
    replace-literal.qed
    replace-regex-template.qed
    replace-pipeline.qed
    substring.qed
    trim.qed
    upper.qed
    lower.qed
    indent.qed
    dedent.qed
    wrap.qed
    prefix.qed
    suffix.qed
    number.qed
  goldens/
    stdout/
      (same filenames and content as goldens/output/)
    stderr/
      empty.txt
    output/
      empty.txt
      foo-baz.txt
      foo-bar-bar-baz.txt
      foo-bar-baz-inserted.txt
      foo-inserted-bar-baz.txt
      foo-bar-replaced-baz.txt
      foo-bar-baz.txt
      foo-bar-baz-upper.txt
      foo-bar-baz-lower.txt
      foo-bar-baz-trimmed.txt
      foo-bar-baz-indented.txt
      foo-bar-baz-dedented.txt
      foo-bar-baz-wrapped.txt
      foo-bar-baz-prefixed.txt
      foo-bar-baz-suffixed.txt
      foo-bar-baz-numbered.txt
      bar-substring.txt
      placeholder-replaced.txt
      placeholder-pipeline-replaced.txt
      copy-source-copy-after.txt
      copy-source-copy-before.txt
      copy-source-copy-at.txt
      copy-source-moved-after.txt
```

---

## Manifest

```toml
# tests/processors/manifest.toml

# ── qed:delete() ──────────────────────────────────────────────────────────────

[[scenario]]
id = "delete"
description = "qed:delete() removes the selected region from the stream"
script = "delete.qed"
input = "three-lines.txt"
stdout = "foo-baz.txt"
stderr = "empty.txt"
output = "foo-baz.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── qed:duplicate() ───────────────────────────────────────────────────────────

[[scenario]]
id = "duplicate"
description = "qed:duplicate() emits the selected region twice consecutively"
script = "duplicate.qed"
input = "three-lines.txt"
stdout = "foo-bar-bar-baz.txt"
stderr = "empty.txt"
output = "foo-bar-bar-baz.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── qed:copy() ────────────────────────────────────────────────────────────────

[[scenario]]
id = "copy-after"
description = "qed:copy(after:p) inserts a copy of the selected region after the target pattern; the original remains in place"
script = "copy-after.qed"
input = "copy-source.txt"
stdout = "copy-source-copy-after.txt"
stderr = "empty.txt"
output = "copy-source-copy-after.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "copy-before"
description = "qed:copy(before:p) inserts a copy of the selected region before the target pattern; the original remains in place"
script = "copy-before.qed"
input = "copy-source.txt"
stdout = "copy-source-copy-before.txt"
stderr = "empty.txt"
output = "copy-source-copy-before.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "copy-at"
description = "qed:copy(at:p) overwrites the target pattern's lines with a copy of the selected region"
script = "copy-at.qed"
input = "copy-source.txt"
stdout = "copy-source-copy-at.txt"
stderr = "empty.txt"
output = "copy-source-copy-at.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── qed:move() ────────────────────────────────────────────────────────────────

[[scenario]]
id = "move-after"
description = "qed:move(after:p) inserts a copy of the selected region after the target pattern and removes the original"
script = "move-after.qed"
input = "copy-source.txt"
stdout = "copy-source-moved-after.txt"
stderr = "empty.txt"
output = "copy-source-moved-after.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── qed:replace() ─────────────────────────────────────────────────────────────

[[scenario]]
id = "replace-literal"
description = "qed:replace() with literal match and literal replacement substitutes the matched span; surrounding content survives"
script = "replace-literal.qed"
input = "three-lines.txt"
stdout = "foo-bar-replaced-baz.txt"
stderr = "empty.txt"
output = "foo-bar-replaced-baz.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "replace-regex-template"
description = "qed:replace() with a regex pattern and /template/ replacement expands capture group references"
script = "replace-regex-template.qed"
input = "three-lines.txt"
stdout = "foo-bar-replaced-baz.txt"
stderr = "empty.txt"
output = "foo-bar-replaced-baz.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "replace-pipeline"
description = "qed:replace() with a processor pipeline as the replacement argument runs the pipeline against the matched span and splices the output in place"
script = "replace-pipeline.qed"
input = "placeholder.txt"
stdout = "placeholder-pipeline-replaced.txt"
stderr = "empty.txt"
output = "placeholder-pipeline-replaced.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── qed:substring() ───────────────────────────────────────────────────────────

[[scenario]]
id = "substring"
description = "qed:substring() narrows the selected region to the matched span; the rest of the line is discarded"
script = "substring.qed"
input = "three-lines.txt"
stdout = "bar-substring.txt"
stderr = "empty.txt"
output = "bar-substring.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── qed:trim() ────────────────────────────────────────────────────────────────

[[scenario]]
id = "trim"
description = "qed:trim() strips leading and trailing whitespace from each line in the selected region"
script = "trim.qed"
input = "indented.txt"
stdout = "foo-bar-baz-trimmed.txt"
stderr = "empty.txt"
output = "foo-bar-baz-trimmed.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── qed:upper() ───────────────────────────────────────────────────────────────

[[scenario]]
id = "upper"
description = "qed:upper() converts all characters in the selected region to uppercase"
script = "upper.qed"
input = "three-lines.txt"
stdout = "foo-bar-baz-upper.txt"
stderr = "empty.txt"
output = "foo-bar-baz-upper.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── qed:lower() ───────────────────────────────────────────────────────────────

[[scenario]]
id = "lower"
description = "qed:lower() converts all characters in the selected region to lowercase"
script = "lower.qed"
input = "foo-bar-baz-upper.txt"
stdout = "foo-bar-baz-lower.txt"
stderr = "empty.txt"
output = "foo-bar-baz-lower.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── qed:indent() ──────────────────────────────────────────────────────────────

[[scenario]]
id = "indent"
description = "qed:indent() prepends each line in the selected region with the specified number of spaces"
script = "indent.qed"
input = "three-lines.txt"
stdout = "foo-bar-baz-indented.txt"
stderr = "empty.txt"
output = "foo-bar-baz-indented.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── qed:dedent() ──────────────────────────────────────────────────────────────

[[scenario]]
id = "dedent"
description = "qed:dedent() removes the common leading whitespace prefix from all lines in the selected region"
script = "dedent.qed"
input = "indented.txt"
stdout = "foo-bar-baz-dedented.txt"
stderr = "empty.txt"
output = "foo-bar-baz-dedented.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── qed:wrap() ────────────────────────────────────────────────────────────────

[[scenario]]
id = "wrap"
description = "qed:wrap() word-wraps each line in the selected region at the specified column width"
script = "wrap.qed"
input = "long-lines.txt"
stdout = "foo-bar-baz-wrapped.txt"
stderr = "empty.txt"
output = "foo-bar-baz-wrapped.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── qed:prefix() ──────────────────────────────────────────────────────────────

[[scenario]]
id = "prefix"
description = "qed:prefix() prepends the specified text to each line in the selected region"
script = "prefix.qed"
input = "three-lines.txt"
stdout = "foo-bar-baz-prefixed.txt"
stderr = "empty.txt"
output = "foo-bar-baz-prefixed.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── qed:suffix() ──────────────────────────────────────────────────────────────

[[scenario]]
id = "suffix"
description = "qed:suffix() appends the specified text to each line in the selected region"
script = "suffix.qed"
input = "three-lines.txt"
stdout = "foo-bar-baz-suffixed.txt"
stderr = "empty.txt"
output = "foo-bar-baz-suffixed.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── qed:number() ──────────────────────────────────────────────────────────────

[[scenario]]
id = "number"
description = "qed:number() prefixes each line in the selected region with its line number"
script = "number.qed"
input = "three-lines.txt"
stdout = "foo-bar-baz-numbered.txt"
stderr = "empty.txt"
output = "foo-bar-baz-numbered.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]
```

---

## Input Files

### `inputs/three-lines.txt`

Used by: `delete`, `duplicate`, `replace-literal`, `replace-regex-template`,
`substring`, `upper`, `prefix`, `suffix`, `number`

```
foo
bar
baz
```

### `inputs/indented.txt`

Used by: `trim`, `dedent`

All three lines share a common two-space indent.
`trim` strips all leading and trailing whitespace.
`dedent` removes the common two-space prefix, leaving unindented lines.

```
  foo
  bar
  baz
```

### `inputs/long-lines.txt`

Used by: `wrap`

A single long line that exceeds the wrap width of 20 characters.

```
the quick brown fox jumps over the lazy dog
```

### `inputs/placeholder.txt`

Used by: `replace-pipeline`

Contains a `{{name}}` placeholder that the pipeline replacement will transform.

```
hello {{name}} world
```

### `inputs/copy-source.txt`

Used by: `copy-after`, `copy-before`, `copy-at`, `move-after`

Four lines; `source` is the line to copy or move, `target` is the destination marker.

```
alpha
source
beta
target
```

---

## Script Files

### `scripts/delete.qed`

```
at("bar") | qed:delete()
```

### `scripts/duplicate.qed`

```
at("bar") | qed:duplicate()
```

### `scripts/copy-after.qed`

Copies the `source` line to after the `target` line; `source` remains in place.

```
at("source") | qed:copy(after:"target")
```

### `scripts/copy-before.qed`

Copies the `source` line to before the `target` line; `source` remains in place.

```
at("source") | qed:copy(before:"target")
```

### `scripts/copy-at.qed`

Overwrites the `target` line with a copy of the `source` line; `source` remains in place.

```
at("source") | qed:copy(at:"target")
```

### `scripts/move-after.qed`

Moves the `source` line to after the `target` line; `source` is removed from its original position.

```
at("source") | qed:move(after:"target")
```

### `scripts/replace-literal.qed`

Replaces the literal string `bar` with `replaced` on matching lines.

```
at("bar") | qed:replace("bar", "replaced")
```

### `scripts/replace-regex-template.qed`

Replaces the matched content using a regex with a named capture group and a template replacement.
Input is the same `three-lines.txt`; the regex captures `bar` and the template produces `replaced`.

```
at(/^(bar)$/) | qed:replace(/(bar)/, /replaced/)
```

### `scripts/replace-pipeline.qed`

Replaces the `{{name}}` placeholder using a pipeline processor.
`echo world` is the pipeline — it receives the matched span and its output is spliced in.

```
at(/\{\{name\}\}/) | qed:replace("{{name}}", echo world)
```

### `scripts/substring.qed`

Narrows the selected region to the span matching `ba` within each line;
the rest of each line is discarded.
Applied to `three-lines.txt`, this selects `ba` from `bar` and `ba` from `baz`,
leaving only those two substrings on their own lines plus `foo` passing through unchanged.

```
at(/ba/) | qed:substring(/ba/)
```

### `scripts/trim.qed`

```
at() | qed:trim()
```

### `scripts/upper.qed`

```
at() | qed:upper()
```

### `scripts/lower.qed`

```
at() | qed:lower()
```

### `scripts/indent.qed`

Indents every line by four spaces.

```
at() | qed:indent(width:4)
```

### `scripts/dedent.qed`

```
at() | qed:dedent()
```

### `scripts/wrap.qed`

Word-wraps at 20 characters.

```
at() | qed:wrap(width:20)
```

### `scripts/prefix.qed`

Prepends `// ` to every line.

```
at() | qed:prefix(text:"// ")
```

### `scripts/suffix.qed`

Appends `;` to every line.

```
at() | qed:suffix(text:";")
```

### `scripts/number.qed`

```
at() | qed:number()
```

---

## Golden Files

`goldens/stdout/` and `goldens/output/` contain files with identical content.
They are listed once below; both directories contain a copy of each.

### `goldens/stdout/` and `goldens/output/`

#### `empty.txt`

```
```

#### `foo-baz.txt`

Used by: `delete`

```
foo
baz
```

#### `foo-bar-bar-baz.txt`

Used by: `duplicate`

`bar` appears twice consecutively.

```
foo
bar
bar
baz
```

#### `copy-source-copy-after.txt`

Used by: `copy-after`

`source` copied to after `target`; original `source` still present.

```
alpha
source
beta
target
source
```

#### `copy-source-copy-before.txt`

Used by: `copy-before`

`source` copied to before `target`; original `source` still present.

```
alpha
source
beta
source
target
```

#### `copy-source-copy-at.txt`

Used by: `copy-at`

`target` overwritten with a copy of `source`; original `source` still present.

```
alpha
source
beta
source
```

#### `copy-source-moved-after.txt`

Used by: `move-after`

`source` moved to after `target`; removed from its original position.

```
alpha
beta
target
source
```

#### `foo-bar-replaced-baz.txt`

Used by: `replace-literal`, `replace-regex-template`

`bar` replaced with `replaced`; `foo` and `baz` pass through unchanged.

```
foo
replaced
baz
```

#### `placeholder-pipeline-replaced.txt`

Used by: `replace-pipeline`

`{{name}}` replaced with `world` via the pipeline processor; surrounding content survives.

```
hello world world
```

#### `bar-substring.txt`

Used by: `substring`

`qed:substring(/ba/)` applied to `three-lines.txt`:
`foo` has no match and passes through unchanged;
`bar` is narrowed to `ba`; `baz` is narrowed to `ba`.

```
foo
ba
ba
```

#### `foo-bar-baz-trimmed.txt`

Used by: `trim`

Leading two-space indent stripped from all lines.

```
foo
bar
baz
```

#### `foo-bar-baz-upper.txt`

Used by: `upper`

```
FOO
BAR
BAZ
```

#### `foo-bar-baz-lower.txt`

Used by: `lower`

Input is `foo-bar-baz-upper.txt` (`FOO`, `BAR`, `BAZ`); lowercased back to `foo`, `bar`, `baz`.

```
foo
bar
baz
```

#### `foo-bar-baz-indented.txt`

Used by: `indent`

Four spaces prepended to each line.

```
    foo
    bar
    baz
```

#### `foo-bar-baz-dedented.txt`

Used by: `dedent`

Common two-space prefix removed; lines are now flush left.

```
foo
bar
baz
```

#### `foo-bar-baz-wrapped.txt`

Used by: `wrap`

`the quick brown fox jumps over the lazy dog` wrapped at 20 characters.

```
the quick brown fox
jumps over the lazy
dog
```

#### `foo-bar-baz-prefixed.txt`

Used by: `prefix`

`// ` prepended to each line.

```
// foo
// bar
// baz
```

#### `foo-bar-baz-suffixed.txt`

Used by: `suffix`

`;` appended to each line.

```
foo;
bar;
baz;
```

#### `foo-bar-baz-numbered.txt`

Used by: `number`

Stream line numbers, colon-space separator, minimal padding (no fixed width set).

```
1: foo
2: bar
3: baz
```

---

### `goldens/stderr/`

#### `empty.txt`

```
```
