# Script File Edge Case Scenarios

Additional scenarios covering boundary conditions in script file behaviour.
These extend `tests/script-files/` with new scenarios, inputs, scripts, and goldens.

---

## New Inputs

### `inputs/three-lines.txt`

Already exists.

---

## New Scripts

### `scripts/empty.qed`

An empty script file — no statements. Input passes through unchanged.

```
```

### `scripts/comments-only.qed`

A script containing only comments and blank lines — no statements.

```
# this is a comment

# another comment
```

### `scripts/comment-inline.qed`

A comment on the same line as a statement — the comment must not be parsed as part
of the statement.

```
at("bar") | qed:delete() # remove bar
```

### `scripts/semicolon-separator.qed`

Two statements on one line separated by `;`.

```
at("bar") | qed:delete(); at("foo") | qed:upper()
```

### `scripts/alias-forward-ref.qed`

An alias referenced before its definition — forward references are permitted for
aliases as well as patterns.

```
at("bar") | trim
trim=qed:replace(/^\s+/, "") | qed:replace(/\s+$/, "")
```

### `scripts/alias-shadowed-by-path.qed`

An alias name that also exists as a command on PATH.
Without `\`, the alias takes precedence over the PATH command.

```
cat=qed:upper()
at("bar") | cat
```

### `scripts/multiline-alias.qed`

An alias definition spanning multiple lines via implicit continuation on `|`.

```
transform=qed:upper() |
          qed:prefix(text:">> ")
at("bar") | transform
```

### `scripts/semicolon-after-continuation.qed`

A statement using implicit continuation followed by a `;`-separated statement
on the next line.

```
at("foo") |
    qed:upper(); at("bar") | qed:delete()
```

### `scripts/pattern-and-alias-same-name.qed`

A name defined first as a pattern, then redefined as an alias.
Last definition wins; a warning is emitted at compile time.

```
target="bar"
target=qed:upper()
at("bar") | target
```

---

## New Manifest Scenarios

```toml
# Append to tests/script-files/manifest.toml

# ── empty and comment-only scripts ────────────────────────────────────────────

[[scenario]]
id = "empty-script"
description = "an empty script file produces no transformations; input passes through unchanged"
script = "empty.qed"
input = "three-lines.txt"
stdout = "foo-bar-baz.txt"
stderr = "empty.txt"
output = "foo-bar-baz.txt"
exit_code = 0
invoke = [
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "comments-only-script"
description = "a script containing only comments and blank lines produces no transformations"
script = "comments-only.qed"
input = "three-lines.txt"
stdout = "foo-bar-baz.txt"
stderr = "empty.txt"
output = "foo-bar-baz.txt"
exit_code = 0
invoke = [
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "inline-comment"
description = "a comment appearing after a statement on the same line is ignored; the statement executes correctly"
script = "comment-inline.qed"
input = "three-lines.txt"
stdout = "foo-baz.txt"
stderr = "empty.txt"
output = "foo-baz.txt"
exit_code = 0
invoke = [
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── statement separators ──────────────────────────────────────────────────────

[[scenario]]
id = "semicolon-separator"
description = "two statements on one line separated by ; execute sequentially"
script = "semicolon-separator.qed"
input = "three-lines.txt"
stdout = "foo-upper-baz.txt"
stderr = "empty.txt"
output = "foo-upper-baz.txt"
exit_code = 0
invoke = [
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "semicolon-after-continuation"
description = "a ; separator works correctly after a statement that used implicit line continuation"
script = "semicolon-after-continuation.qed"
input = "three-lines.txt"
stdout = "foo-upper-baz.txt"
stderr = "empty.txt"
output = "foo-upper-baz.txt"
exit_code = 0
invoke = [
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── alias edge cases ──────────────────────────────────────────────────────────

[[scenario]]
id = "alias-forward-ref"
description = "an alias can be referenced before its definition; forward references are permitted"
script = "alias-forward-ref.qed"
input = "three-lines.txt"
stdout = "foo-baz.txt"
stderr = "empty.txt"
output = "foo-baz.txt"
exit_code = 0
invoke = [
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "alias-takes-precedence-over-path"
description = "without \\, an alias name takes precedence over a same-named command on PATH"
script = "alias-shadowed-by-path.qed"
input = "three-lines.txt"
stdout = "foo-bar-upper-baz.txt"
stderr = "empty.txt"
output = "foo-bar-upper-baz.txt"
exit_code = 0
invoke = [
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "multiline-alias"
description = "an alias definition that spans multiple lines via implicit continuation is parsed correctly"
script = "multiline-alias.qed"
input = "three-lines.txt"
stdout = "foo-bar-transformed-baz.txt"
stderr = "empty.txt"
output = "foo-bar-transformed-baz.txt"
exit_code = 0
invoke = [
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "name-redefined-last-wins"
description = "when a name is defined twice the last definition wins; a warning is emitted"
script = "pattern-and-alias-same-name.qed"
input = "three-lines.txt"
stdout = "foo-bar-upper-baz.txt"
stderr = "redefine-warn.txt"
output = "foo-bar-upper-baz.txt"
exit_code = 0
invoke = [
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]
```

---

## New Golden Files

### `goldens/stdout/` and `goldens/output/`

#### `foo-bar-baz.txt`

Already exists.

#### `foo-baz.txt`

Already exists.

#### `foo-upper-baz.txt`

Used by: `semicolon-separator`, `semicolon-after-continuation`

Statement 1 uppercases `foo`; statement 2 deletes `bar`.
`baz` passes through both.

```
FOO
baz
```

#### `foo-bar-upper-baz.txt`

Used by: `alias-takes-precedence-over-path`, `name-redefined-last-wins`

`bar` uppercased by the `cat` alias resolving to `qed:upper()`.

```
foo
BAR
baz
```

#### `foo-bar-transformed-baz.txt`

Used by: `multiline-alias`

`bar` uppercased then prefixed by the multi-line alias; `foo` and `baz` pass through.

```
foo
>> BAR
baz
```

---

### `goldens/stderr/`

#### `empty.txt`

Already exists.

#### `redefine-warn.txt`

Used by: `name-redefined-last-wins`

> ⚠️ Placeholder — update to match actual implementation output once stderr diagnostic
> message format is defined (see `qed-implementation-design.md` § Open Concerns).

```
qed: warning: "target" redefined; last definition wins
```
