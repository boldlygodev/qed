# Pattern Edge Case Scenarios

Additional scenarios covering boundary conditions in pattern behaviour.
These extend `tests/patterns/` with new scenarios, inputs, scripts, and goldens.

---

## New Inputs

### `inputs/empty.txt`

Used by: `named-literal-no-match`, `inline-regex-no-match`

```
```

### `inputs/three-lines.txt`

Already exists.

### `inputs/special-chars.txt`

Used by: `literal-special-chars`, `regex-special-chars`

Lines containing characters that are special in regex but must be treated
as literals when the pattern is a string.

```
foo.bar
foo*bar
foo[bar]
```

### `inputs/unicode.txt`

Used by: `regex-unicode`

```
café
naïve
日本語
```

### `inputs/multiline-regex-flags.txt`

Used by: `regex-multiline-flag`

```
foo
bar
baz
```

### `inputs/forward-ref.txt`

Used by: `named-pattern-forward-ref`

```
target
other
```

### `inputs/range-source.txt`

Used by: `named-pattern-in-from-to`

A stream with named marker lines framed by context lines.
`start` and `end` are the range boundaries; `alpha` and `echo` are the survivors.

```
alpha
start
bravo
charlie
delta
end
echo
```

---

## New Scripts

### `scripts/named-literal-no-match.qed`

Named literal pattern that matches nothing — `on_error:skip` suppresses failure.

```
target="quux"
at(target, on_error:skip) | qed:delete()
```

### `scripts/inline-regex-no-match.qed`

Inline regex that matches nothing.

```
at(/quux/, on_error:skip) | qed:delete()
```

### `scripts/literal-special-chars.qed`

A literal string pattern containing regex metacharacters — `.` must match
a literal dot, not any character.

```
at("foo.bar") | qed:delete()
```

### `scripts/regex-special-chars.qed`

A regex pattern containing metacharacters that do match structurally.
`/foo.bar/` matches `foo.bar` and `foo*bar` (`.` matches any char) but not `foo[bar]`
because `.` matches only one character.

```
at(/foo.bar/) | qed:upper()
```

### `scripts/regex-unicode.qed`

Regex matching Unicode characters.

```
at(/caf\p{L}/) | qed:delete()
```

### `scripts/regex-multiline-flag.qed`

`(?m)` flag changes `^` and `$` semantics within the pattern.
Without it, `^bar$` on a line-oriented stream already anchors to the full line,
so the flag makes no difference at the line level — verifies it does not break anything.

```
at(/(?m)^bar$/) | qed:delete()
```

### `scripts/named-pattern-forward-ref.qed`

A named pattern referenced before its definition — forward references are permitted.

```
at(target) | qed:delete()
target="target"
```

### `scripts/named-pattern-reused.qed`

A named pattern referenced in multiple statements.

```
target="target"
at(target) | qed:upper()
at(target) | qed:prefix(text:">> ")
```

### `scripts/negated-on-all-lines.qed`

Negated pattern that matches all lines — `!` applied to a pattern that nothing matches,
so all lines are selected.

```
at(!"quux") | qed:upper()
```

### `scripts/double-negation.qed`

`!` cannot be doubled in the grammar — this scenario uses two separate statements
to confirm that negation composes correctly through the select-action model rather
than within a single pattern expression.

First statement deletes lines matching `bar`; second deletes lines matching `baz`.

```
at("bar") | qed:delete()
at("baz") | qed:delete()
```

### `scripts/named-regex-in-from-to.qed`

Named regex pattern used in `from > to` position.

```
start=/^start$/
end=/^end$/
from(start+) > to(end+) | qed:delete()
```

### `scripts/inclusive-in-to.qed`

`+` on `to` boundary includes the matching line in the region.

```
from("start") > to("end"+) | qed:delete()
```

### `scripts/named-pattern-in-from-to.qed`

Named literal patterns used in `from > to` boundary positions.
Verifies that named literals resolve correctly as range boundaries.

```
start="start"
end="end"
from(start+) > to(end+) | qed:delete()
```

### `scripts/duplicate-pattern-name.qed`

`target` is defined twice — a warning fires and the last definition wins.
`at(target)` matches lines containing `"baz"` (the second definition)
and deletes the `baz` line. `foo` and `bar` pass through.

```
target="bar"
target="baz"
at(target) | qed:delete()
```

---

## New Manifest Scenarios

```toml
# Append to tests/patterns/manifest.toml

# ── no-match edge cases ───────────────────────────────────────────────────────

[[scenario]]
id = "named-literal-no-match"
description = "a named literal pattern that matches nothing succeeds silently with on_error:skip"
script = "named-literal-no-match.qed"
input = "three-lines.txt"
stdout = "foo-bar-baz.txt"
stderr = "empty.txt"
output = "foo-bar-baz.txt"
exit_code = 0
invoke = [
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "inline-regex-no-match"
description = "an inline regex that matches nothing succeeds silently with on_error:skip"
script = "inline-regex-no-match.qed"
input = "three-lines.txt"
stdout = "foo-bar-baz.txt"
stderr = "empty.txt"
output = "foo-bar-baz.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── literal vs regex metacharacters ──────────────────────────────────────────

[[scenario]]
id = "literal-special-chars"
description = "a literal string pattern treats regex metacharacters as plain characters; only the exact line matches"
script = "literal-special-chars.qed"
input = "special-chars.txt"
stdout = "special-chars-literal-result.txt"
stderr = "empty.txt"
output = "special-chars-literal-result.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "regex-special-chars"
description = "a regex pattern interprets metacharacters structurally; . matches any single character"
script = "regex-special-chars.qed"
input = "special-chars.txt"
stdout = "special-chars-regex-result.txt"
stderr = "empty.txt"
output = "special-chars-regex-result.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── unicode ───────────────────────────────────────────────────────────────────

[[scenario]]
id = "regex-unicode"
description = "regex patterns support Unicode character classes"
script = "regex-unicode.qed"
input = "unicode.txt"
stdout = "unicode-result.txt"
stderr = "empty.txt"
output = "unicode-result.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── regex flags ───────────────────────────────────────────────────────────────

[[scenario]]
id = "regex-multiline-flag"
description = "(?m) flag within a regex pattern does not break line-oriented matching"
script = "regex-multiline-flag.qed"
input = "three-lines.txt"
stdout = "foo-baz.txt"
stderr = "empty.txt"
output = "foo-baz.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── named pattern features ────────────────────────────────────────────────────

[[scenario]]
id = "named-pattern-forward-ref"
description = "a named pattern can be referenced before its definition; forward references are permitted"
script = "named-pattern-forward-ref.qed"
input = "forward-ref.txt"
stdout = "other-only.txt"
stderr = "empty.txt"
output = "other-only.txt"
exit_code = 0
invoke = [
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "named-pattern-reused"
description = "a named pattern can be referenced in multiple statements without redefinition"
script = "named-pattern-reused.qed"
input = "forward-ref.txt"
stdout = "named-reused-result.txt"
stderr = "empty.txt"
output = "named-reused-result.txt"
exit_code = 0
invoke = [
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── negation edge cases ───────────────────────────────────────────────────────

[[scenario]]
id = "negated-matches-all"
description = "a negated pattern that nothing matches selects all lines"
script = "negated-on-all-lines.qed"
input = "three-lines.txt"
stdout = "foo-bar-baz-upper.txt"
stderr = "empty.txt"
output = "foo-bar-baz-upper.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── named pattern in range positions ─────────────────────────────────────────

[[scenario]]
id = "named-regex-in-from-to"
description = "named regex patterns work correctly in from > to boundary positions"
script = "named-regex-in-from-to.qed"
input = "copy-move-source.txt"
stdout = "range-deleted-result.txt"
stderr = "empty.txt"
output = "range-deleted-result.txt"
exit_code = 0
invoke = [
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "named-pattern-in-from-to"
description = "named literal patterns work correctly in from > to boundary positions"
script = "named-pattern-in-from-to.qed"
input = "range-source.txt"
stdout = "alpha-echo.txt"
stderr = "empty.txt"
output = "alpha-echo.txt"
exit_code = 0
invoke = [
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "inclusive-to-boundary"
description = "+ on a to() boundary includes the matching line in the deleted region"
script = "inclusive-in-to.qed"
input = "copy-move-source.txt"
stdout = "inclusive-to-result.txt"
stderr = "empty.txt"
output = "inclusive-to-result.txt"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

# ── duplicate pattern name warning ────────────────────────────────────────────

[[scenario]]
id = "duplicate-pattern-name"
description = "redefining a named pattern emits a warning and uses the last definition"
script = "duplicate-pattern-name.qed"
input = "three-lines.txt"
stdout = "foo-bar.txt"
stderr = "duplicate-pattern-name.txt"
output = "foo-bar.txt"
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

#### `special-chars-literal-result.txt`

Used by: `literal-special-chars`

Only `foo.bar` deleted — `foo*bar` and `foo[bar]` do not match the literal `"foo.bar"`.

```
foo*bar
foo[bar]
```

#### `special-chars-regex-result.txt`

Used by: `regex-special-chars`

`/foo.bar/` matches any line where `foo` is followed by any single character then `bar`.
`foo.bar` and `foo*bar` both match and are uppercased; `foo[bar]` does not match
because `[bar]` is four characters, not one.

```
FOO.BAR
FOO*BAR
foo[bar]
```

#### `unicode-result.txt`

Used by: `regex-unicode`

`caf\p{L}` matches `café` (letter after `caf`); it is deleted.
`naïve` and `日本語` do not match and pass through.

```
naïve
日本語
```

#### `other-only.txt`

Used by: `named-pattern-forward-ref`

`target` deleted; `other` passes through.

```
other
```

#### `named-reused-result.txt`

Used by: `named-pattern-reused`

Statement 1 sees `target` and uppercases it.
Statement 2 sees `TARGET` (output of statement 1) — `target` pattern no longer
matches because the line is now uppercase. `>> ` prefix is not applied.
`other` passes through both statements unchanged.

> ⚠️ This golden captures the sequential semantics: statement N+1 sees statement N's
> output, not the original input. If the implementation matches case-insensitively
> or re-selects from original input, this golden would differ. Verify carefully.

```
TARGET
other
```

#### `foo-bar-baz-upper.txt`

Used by: `negated-matches-all`

All lines uppercased — negated `"quux"` matched every line.

```
FOO
BAR
BAZ
```

#### `range-deleted-result.txt`

Used by: `named-regex-in-from-to`

`start` through `end` inclusive deleted from `copy-move-source.txt`;
`alpha` and `beta` remain.

```
alpha
beta
```

#### `inclusive-to-result.txt`

Used by: `inclusive-to-boundary`

`from("start")` exclusive, `to("end"+)` inclusive.
The region runs from the line after `start` through `end` inclusive —
that is `middle` and `end` — and is deleted.
`alpha`, `start`, and `beta` remain.

```
alpha
start
beta
```

#### `alpha-echo.txt`

Used by: `named-pattern-in-from-to`

`from(start+) > to(end+)` on `range-source.txt` selects `start` through `end` inclusive
and deletes them.
`alpha` and `echo` are outside the range and pass through.

```
alpha
echo
```

#### `foo-bar.txt`

Used by: `duplicate-pattern-name`

`target="baz"` wins; `at(target) | qed:delete()` removes the `baz` line.
`foo` and `bar` pass through.

```
foo
bar
```

---

### `goldens/stderr/`

#### `duplicate-pattern-name.txt`

Used by: `duplicate-pattern-name`

Script (3 lines):
```
target="bar"
target="baz"
at(target) | qed:delete()
```
`target="baz"` at `2:1-13` (12 chars); widest span `qed:delete()` at `3:14-26` (12 chars) → location width 7.

```
qed: warning: 2:1-13: target="baz": pattern target already defined, using last definition
```
