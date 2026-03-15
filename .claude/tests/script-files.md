# Script File Scenarios

Tests covering `-f` script files, shebang support, named patterns and aliases
in script context, multi-statement scripts, and line continuation.

One-liner vs `-f` invocation symmetry is tested as `invoke` variants throughout
other feature directories. This directory focuses on script-file-specific features
that have no one-liner equivalent.

---

## Directory Layout

```
tests/script-files/
  manifest.toml
  inputs/
    three-lines.txt
    five-lines.txt
  scripts/
    shebang.qed
    multi-statement.qed
    alias-simple.qed
    alias-composed.qed
    alias-override-short.qed
    line-continuation-pipe.qed
    line-continuation-comma.qed
    line-continuation-narrowing.qed
  goldens/
    stdout/
      empty.txt
      foo-baz.txt
      foo-baz-upper.txt
      foo-bar-baz-upper.txt
      foo-baz-prefixed.txt
      alpha-bravo-delta-echo.txt
    stderr/
      empty.txt
    output/
      (same filenames and content as goldens/stdout/)
```

---

## Manifest

```toml
# tests/script-files/manifest.toml

# ── shebang ───────────────────────────────────────────────────────────────────

[[scenario]]
id = "shebang"
description = "a script file with a shebang line can be executed directly; the shebang is not treated as a statement"
script = "shebang.qed"
input = "three-lines.txt"
stdout = "foo-baz.txt"
stderr = "empty.txt"
output = "foo-baz.txt"
exit_code = 0
invoke = [
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── multi-statement ───────────────────────────────────────────────────────────

[[scenario]]
id = "multi-statement"
description = "multiple statements execute sequentially; each statement sees the output of the prior one"
script = "multi-statement.qed"
input = "three-lines.txt"
stdout = "foo-baz-upper.txt"
stderr = "empty.txt"
output = "foo-baz-upper.txt"
exit_code = 0
invoke = [
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── aliases ───────────────────────────────────────────────────────────────────

[[scenario]]
id = "alias-simple"
description = "an alias binds a name to a processor chain and can be referenced in select-action statements"
script = "alias-simple.qed"
input = "three-lines.txt"
stdout = "foo-baz-prefixed.txt"
stderr = "empty.txt"
output = "foo-baz-prefixed.txt"
exit_code = 0
invoke = [
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "alias-composed"
description = "an alias can reference another alias; composed aliases chain their processor steps"
script = "alias-composed.qed"
input = "three-lines.txt"
stdout = "foo-baz-prefixed.txt"
stderr = "empty.txt"
output = "foo-baz-prefixed.txt"
exit_code = 0
invoke = [
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "alias-override-short-name"
description = "a short alias name like 'delete' resolves to the alias definition rather than PATH"
script = "alias-override-short.qed"
input = "three-lines.txt"
stdout = "foo-bar-baz-upper.txt"
stderr = "empty.txt"
output = "foo-bar-baz-upper.txt"
exit_code = 0
invoke = [
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── line continuation ─────────────────────────────────────────────────────────

[[scenario]]
id = "line-continuation-pipe"
description = "a line ending with | continues the statement on the next line"
script = "line-continuation-pipe.qed"
input = "three-lines.txt"
stdout = "foo-baz.txt"
stderr = "empty.txt"
output = "foo-baz.txt"
exit_code = 0
invoke = [
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "line-continuation-comma"
description = "a line ending with , continues the statement on the next line"
script = "line-continuation-comma.qed"
input = "three-lines.txt"
stdout = "foo-baz.txt"
stderr = "empty.txt"
output = "foo-baz.txt"
exit_code = 0
invoke = [
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "line-continuation-narrowing"
description = "a line ending with > continues the statement on the next line"
script = "line-continuation-narrowing.qed"
input = "five-lines.txt"
stdout = "alpha-bravo-delta-echo.txt"
stderr = "empty.txt"
output = "alpha-bravo-delta-echo.txt"
exit_code = 0
invoke = [
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

Used by: `line-continuation-narrowing`

```
alpha
bravo
charlie
delta
echo
```

---

## Script Files

### `scripts/shebang.qed`

The shebang line is parsed and ignored; only `at("bar") | qed:delete()` executes.

```
#!/usr/bin/env qed -f
at("bar") | qed:delete()
```

### `scripts/multi-statement.qed`

Two statements execute sequentially.
Statement 1 deletes `bar`; statement 2 sees the result and uppercases `baz`.

```
at("bar") | qed:delete()
at("baz") | qed:upper()
```

### `scripts/alias-simple.qed`

`markup` is an alias for a processor chain.
The alias is applied to the `bar` line, which is deleted, leaving `foo` and `baz`
prefixed with `> `.

```
markup=qed:prefix(text:"> ")
target="bar"
at(target) | qed:delete()
at() | markup
```

### `scripts/alias-composed.qed`

`clean` aliases `qed:delete()`; `markup` aliases `qed:prefix(text:"> ")`.
`transform` composes both. The result is the same as `alias-simple`.

```
clean=qed:delete()
markup=qed:prefix(text:"> ")
transform=clean | markup
target="bar"
at(target) | transform
at() | markup
```

> ⚠️ Note: `alias-composed` and `alias-simple` produce the same golden output —
> `foo-baz-prefixed.txt`. They test different alias mechanisms, not different outputs.

### `scripts/alias-override-short.qed`

`delete` is defined as an alias for `qed:upper()` rather than the internal `qed:delete()`.
Using `delete` in a select-action resolves to the alias, not the PATH command.
The stream is uppercased rather than deleted — demonstrating alias resolution.

```
delete=qed:upper()
at() | delete
```

### `scripts/line-continuation-pipe.qed`

Statement continues across lines because the first line ends with `|`.

```
at("bar") |
    qed:delete()
```

### `scripts/line-continuation-comma.qed`

Statement continues across lines because the first line ends with `,`.

```
at("bar",
    on_error:fail) | qed:delete()
```

### `scripts/line-continuation-narrowing.qed`

Statement continues across lines because the first line ends with `>`.
`from("bravo") > to("delta")` selects `charlie` (both boundaries exclusive);
deleting it leaves `alpha`, `bravo`, `delta`, `echo` — the same as `five-lines.txt`
minus `charlie`.

```
from("bravo") >
    to("delta") | qed:delete()
```

---

## Golden Files

### `goldens/stdout/` and `goldens/output/`

#### `empty.txt`

```
```

#### `foo-baz.txt`

Used by: `shebang`, `alias-simple`, `alias-composed`, `line-continuation-pipe`,
`line-continuation-comma`

`bar` deleted; `foo` and `baz` pass through.

```
foo
baz
```

#### `foo-baz-upper.txt`

Used by: `multi-statement`

`bar` deleted by statement 1; `baz` uppercased by statement 2.

```
foo
BAZ
```

#### `foo-bar-baz-upper.txt`

Used by: `alias-override-short-name`

All lines uppercased — `delete` alias resolved to `qed:upper()`.

```
FOO
BAR
BAZ
```

#### `foo-baz-prefixed.txt`

Used by: `alias-simple`, `alias-composed`

`bar` deleted; remaining lines prefixed with `> `.

```
> foo
> baz
```

#### `alpha-bravo-delta-echo.txt`

Used by: `line-continuation-narrowing`

`charlie` deleted; `alpha`, `bravo`, `delta`, `echo` remain.

```
alpha
bravo
delta
echo
```

---

### `goldens/stderr/`

#### `empty.txt`

```
```
