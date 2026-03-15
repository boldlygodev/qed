# qed

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="assets/qed-logo-dark.svg">
  <img src="assets/qed-logo.svg" alt="qed" xheight="80">
</picture>

A modern stream editor for source files and config files.

> **⚠️ In development.** `qed` is not yet released. This documentation describes the intended design. APIs and behavior may change before the first release.

-----

## What is qed?

`qed` transforms text files using a concise select-action model: select a region of lines, pipe it through a processor. It is designed for the things `sed` and `awk` handle awkwardly: structured edits to source files, config manipulation, log processing, and code generation — tasks where you need to target a function body, a YAML block, or a range of log lines rather than a single pattern.

```sh
# Delete all TODO comments from a Go file
qed --in-place 'at(/^\s*\/\/\s*TODO:/) | qed:delete()' main.go

# Replace a version in a TOML file
qed --in-place 'at(/^version = /) | qed:replace(/=.*/, "= \"2.0.0\"")' Cargo.toml

# Extract only ERROR lines from a log
qed --extract 'at(/\bERROR\b/)' app.log

# Insert a deprecation notice before every Old* function
qed --in-place '
target=/^func Old/
before(target) | echo "// Deprecated: use New* equivalent instead."
' main.go
```

-----

## Installation

**mise** (recommended):

```sh
mise use --global github:boldlygo.dev/qed
```

Or via the cargo backend if you manage your own Rust toolchain:

```sh
mise use --global cargo:qed
```

**Homebrew:**

```sh
brew install qed
```

**cargo:**

```sh
cargo install qed
```

**Build from source:**

```sh
git clone https://github.com/your-org/qed
cd qed
cargo build --release
```

-----

## Quick Start

`qed` reads from stdin and writes to stdout by default:

```sh
echo -e "foo\nbar\nbaz" | qed 'at("bar") | qed:delete()'
# foo
# baz
```

Pass a file as the input argument:

```sh
qed 'at("bar") | qed:delete()' input.txt
```

Modify a file in place:

```sh
qed --in-place 'at("bar") | qed:delete()' input.txt
```

Use a script file with `-f` for multi-statement scripts:

```sh
qed -f transform.qed input.txt
```

-----

## CLI Reference

```
Usage: qed [OPTIONS] [SCRIPT] [FILE]
```

| Flag                | Short | Description                                                 |
| ------------------- | ----- | ----------------------------------------------------------- |
| `--file <FILE>`     | `-f`  | Load script from a file instead of inline                   |
| `--in-place`        | `-i`  | Modify the input file directly (atomic write)               |
| `--output <FILE>`   | `-o`  | Write output to a file instead of stdout                    |
| `--extract`         | `-x`  | Suppress passthrough — emit only selected regions           |
| `--dry-run`         | `-d`  | Show proposed changes as a unified diff; do not modify      |
| `--on-error <MODE>` |       | Global no-match behaviour: `fail` (default), `warn`, `skip` |
| `--no-env`          |       | Disable `${VAR}` expansion in patterns                      |

**`--in-place`** writes atomically — the original file is replaced only after the full transformation succeeds. A failing run leaves the original file unchanged.

**`--extract`** inverts the output: only selected regions are emitted, passthrough lines are suppressed. Use `qed:skip()` as the processor to select without transforming:

```sh
qed --extract 'at(/\bERROR\b/) | qed:skip()' app.log
# shorthand — qed:skip() is implied when --extract is used with no processor
qed --extract 'at(/\bERROR\b/)' app.log
```

**`--dry-run`** produces a unified diff on stdout; the input file is never modified. Composable with diff viewers:

```sh
qed --dry-run -f transform.qed main.go | delta
```

**`--on-error`** sets the global no-match behaviour, overridable per-selector with the `on_error` parameter. See [`on_error`](#on_error--no-match-behaviour).

**Pipelines and `set -o pipefail`:** `qed` emits lines as soon as they are processed — it does not buffer the entire output before writing. If a downstream command exits early (e.g. `head`), `qed` may receive `SIGPIPE`. Use `set -o pipefail` in scripts that pipe `qed` output to catch failures in any stage:

```sh
set -o pipefail
qed 'at(/\bERROR\b/)' app.log | wc -l
```

-----

## Quick Reference

### Selectors

| Selector | Description |
| -------- | ----------- |
| `after`  | Insertion point immediately after matching lines |
| `at`     | Selects matching lines |
| `before` | Insertion point immediately before matching lines |
| `from`   | Selects lines beginning from matching lines |
| `to`     | Selects lines up to matching lines |

```
at(pattern?, nth:…, on_error:fail|warn|skip)
after(pattern?, nth:…, on_error:fail|warn|skip)
before(pattern?, nth:…, on_error:fail|warn|skip)
from(pattern, on_error:fail|warn|skip) > to(pattern)
```

`pattern` — `"literal"`, `/regex/`, `name`, `!pattern` (negated), `pattern+` (inclusive boundary)

`nth` — `1` · `3` · `-1` (last) · `-2` (second to last) · `2n` · `2n+1` · `1...3` · `-3...-1` · `1,3,-1`

### Built-in processors

```
qed:delete()
qed:duplicate()
qed:skip()
qed:upper()
qed:lower()
qed:replace("literal", "replacement")
qed:replace(/regex/, /template/)
qed:replace("literal", processor-pipeline)
qed:substring(pattern)
qed:trim()
qed:indent(width:N, char:" ")
qed:dedent()
qed:wrap(width:N)
qed:prefix(text:"…")
qed:suffix(text:"…")
qed:number(start:N, width:N)
qed:copy(after|before|at:pattern)
qed:move(after|before|at:pattern)
qed:file()
qed:warn()
qed:fail()
qed:debug:count()
qed:debug:print()
```

### Generation processors

```
qed:uuid(version:4|5|7, namespace:url|dns|oid|x500, name:"…")
qed:timestamp(format:iso8601|unix|datetime|"strftime", timezone:"…")
qed:random(N, alphabet:numeric|alpha|upper|alnum|hex|HEX|
           base32|crockford|bech32|base58|base62|base64url|ascii|symbol|"custom")
```

-----

## The Select-Action Model

Every `qed` statement has the form:

```
selector | processor-chain
```

The **selector** identifies which lines to act on.
The **processor chain** transforms those lines.
Unselected lines pass through unchanged.

Multiple statements execute sequentially — each statement sees the output of the prior one:

```
at("bar") | qed:delete()      # statement 1: delete bar
at("baz") | qed:upper()       # statement 2: sees output of statement 1
```

Statements are separated by newlines or `;`.
A line ending with `|`, `>`, `||`, or `,` continues on the next line:

```
from("start") >
    to("end") | qed:delete()
```

-----

## Selectors

### `at(pattern?, nth?, on_error?)`

Selects lines matching a pattern.
With no pattern, selects every line in the stream.

|Parameter |Type                    |Default        |Description                               |
|----------|------------------------|---------------|------------------------------------------|
|`pattern` |literal, regex, or name |—              |Pattern to match; omit to select all lines|
|`nth`     |nth-expr                |all occurrences|Which occurrences to select               |
|`on_error`|`fail` | `warn` | `skip`|`fail`         |Behaviour when no lines match             |

```sh
qed 'at("bar") | qed:delete()'              # lines containing "bar"
qed 'at(/^func /) | qed:upper()'            # lines matching regex
qed 'at() | qed:number()'                   # all lines
qed 'at("x", nth:2) | qed:delete()'         # second occurrence only
qed 'at("x", on_error:skip) | qed:delete()' # silent no-match
```

### `after(pattern?, nth?, on_error?)`

Insertion point immediately after lines matching `pattern`.
The processor receives empty stdin; its stdout is inserted at the cursor.
With no pattern, targets the end of the stream.

|Parameter |Type                    |Default        |Description                                   |
|----------|------------------------|---------------|----------------------------------------------|
|`pattern` |literal, regex, or name |—              |Pattern to match; omit to target end of stream|
|`nth`     |nth-expr                |all occurrences|Which occurrences to insert after             |
|`on_error`|`fail` | `warn` | `skip`|`fail`         |Behaviour when no lines match                 |

```sh
qed 'after("header") | echo "new line"'   # insert after each "header" line
qed 'after() | cat footer.txt'            # append to end of stream
```

### `before(pattern?, nth?, on_error?)`

Insertion point immediately before lines matching `pattern`.
With no pattern, targets the beginning of the stream.

|Parameter |Type                    |Default        |Description                                     |
|----------|------------------------|---------------|------------------------------------------------|
|`pattern` |literal, regex, or name |—              |Pattern to match; omit to target start of stream|
|`nth`     |nth-expr                |all occurrences|Which occurrences to insert before              |
|`on_error`|`fail` | `warn` | `skip`|`fail`         |Behaviour when no lines match                   |

```sh
qed 'before(/^func /) | cat license.txt'  # insert before every function
qed 'before() | echo "---"'               # prepend to start of stream
```

### `from(pattern, on_error?) > to(pattern)`

Selects a range of lines between two boundary patterns.
Boundaries are exclusive by default — use `+` on a boundary pattern to make it inclusive.

|Parameter     |Type                    |Default  |Description                                                 |
|--------------|------------------------|---------|------------------------------------------------------------|
|`from` pattern|literal, regex, or name |required |Start boundary                                              |
|`to` pattern  |literal, regex, or name |required |End boundary                                                |
|`+` suffix    |boundary modifier       |exclusive|Include the matching boundary line in the region            |
|`on_error`    |`fail` | `warn` | `skip`|`fail`   |Behaviour when boundaries are not found; specified on `from`|

```sh
qed 'from("start") > to("end") | qed:delete()'       # exclude both boundaries
qed 'from("start"+) > to("end") | qed:delete()'      # include start, exclude end
qed 'from("start"+) > to("end"+) | qed:delete()'     # include both
qed 'from("start", on_error:skip) > to("end") | qed:delete()'
```

### Patterns

`"…"` is a literal string — no special characters interpreted, no capture groups.
`/…/` is a RE2 regex — guaranteed linear-time matching, no backreferences.

```sh
qed 'at("foo.bar") | ...'   # matches the literal string foo.bar
qed 'at(/foo.bar/) | ...'   # matches foo, any character, bar
```

Prefix with `!` to negate — select lines that do not match:

```sh
qed 'at(!"bar") | qed:upper()'   # all lines except those containing "bar"
```

Named patterns are defined as statements and referenced by bare identifier.
Forward references are permitted. Redefining a name warns; last definition wins.

```sh
qed -f - <<'EOF'
target=/^func Old/
before(target) | echo "// Deprecated"
at(target) | qed:upper()
EOF
```

Environment variables expand within patterns by default:

```sh
qed 'at("${TARGET}") | qed:delete()'           # expands $TARGET from the environment
qed --no-env 'at("${TARGET}") | qed:delete()'  # treat ${TARGET} literally
```

### `nth` — occurrence selection

`nth` selects which occurrences of a match to act on.
When omitted, all occurrences are selected.

```
nth:1          first occurrence
nth:3          third occurrence
nth:-1         last occurrence
nth:-2         second to last
nth:2n         every second occurrence
nth:2n+1       every odd occurrence (1st, 3rd, 5th…)
nth:2n-1       every second, offset back one
nth:-2n        every second from the end
nth:1...3      first through third, inclusive
nth:-3...-1    third from last through last
nth:1,3,-1     first, third, and last
nth:1...3,-2   first through third, and second to last
```

### `on_error` — no-match behaviour

|Value |Description                        |
|------|-----------------------------------|
|`fail`|Exit non-zero (default)            |
|`warn`|Emit a warning to stderr, exit zero|
|`skip`|Silently continue, exit zero       |

Per-selector `on_error` overrides the global `--on-error` flag:

```sh
qed --on-error=skip '
at("required", on_error:fail) | qed:delete()   # always fails if not found
at("optional") | qed:delete()                  # inherits global: skip
' input.txt
```

-----

## Built-in Processors

All processors use the `qed:` namespace.
Parameters use labeled syntax: `param:value`.

### `qed:delete()`

Removes the selected region from the stream. No parameters.

```sh
qed 'at("bar") | qed:delete()'
```

### `qed:duplicate()`

Emits the selected region twice consecutively. No parameters.

```sh
qed 'at("bar") | qed:duplicate()'
```

### `qed:skip()`

No-op passthrough. Primarily useful with `--extract`. No parameters.

```sh
qed --extract 'at(/\bERROR\b/) | qed:skip()'
```

### `qed:upper()` / `qed:lower()`

Convert the selected region to uppercase or lowercase. No parameters.

```sh
qed 'at(/^const /) | qed:upper()'
```

### `qed:replace(match, replacement)`

Substitutes within the selected region. Three replacement forms:

```sh
# Literal match, literal replacement
qed 'at("bar") | qed:replace("bar", "baz")'

# Regex match, template replacement — capture groups expand as $1, $2, …
qed 'at(/\d+-\d+-\d+/) | qed:replace(/(\d{4})-(\d{2})-(\d{2})/, /$3\/$2\/$1/)'

# Literal match, pipeline replacement — processor output spliced in place
qed 'at(/\{\{uuid\}\}/) | qed:replace("{{uuid}}", qed:uuid())'
qed 'at(/\{\{body\}\}/) | qed:replace("{{body}}", generate-content)'
```

The surrounding content always survives in the pipeline form.

### `qed:substring(pattern)`

Narrows the selected region to the matched span; the rest of the line is discarded.

|Parameter|Type            |Required|Description                       |
|---------|----------------|--------|----------------------------------|
|`pattern`|literal or regex|yes     |Pattern whose matched span to keep|

```sh
qed 'at(/\d+/) | qed:substring(/\d+/)'   # keep only the matched digits on each line
```

To delete the matched span instead of keeping it, use `qed:replace()`:

```sh
qed 'at(/\s+$/) | qed:replace(/\s+$/, "")'   # trim trailing whitespace
```

### `qed:trim()`

Strips leading and trailing whitespace from each line in the selected region.
No parameters.

```sh
qed 'at() | qed:trim()'
```

### `qed:indent(width, char?)`

Prepends each line in the selected region with `width` repetitions of `char`.

|Parameter|Type   |Default      |Description                     |
|---------|-------|-------------|--------------------------------|
|`width`  |integer|required     |Number of characters to indent  |
|`char`   |string |`" "` (space)|Character to use for indentation|

```sh
qed 'at() | qed:indent(width:4)'              # indent by 4 spaces
qed 'at() | qed:indent(width:1, char:"\t")'   # indent by one tab
```

### `qed:dedent()`

Removes the common leading whitespace prefix from all lines in the selected region.
No parameters.

```sh
qed 'at() | qed:dedent()'
```

### `qed:wrap(width)`

Word-wraps each line in the selected region at the given column width.

|Parameter|Type   |Required|Description            |
|---------|-------|--------|-----------------------|
|`width`  |integer|yes     |Column width to wrap at|

```sh
qed 'at(/^[^|].{80,}/) | qed:wrap(width:80)'
```

### `qed:prefix(text)`

Prepends `text` to each line in the selected region.

|Parameter|Type  |Required|Description    |
|---------|------|--------|---------------|
|`text`   |string|yes     |Text to prepend|

```sh
qed 'at() | qed:prefix(text:"// ")'
```

### `qed:suffix(text)`

Appends `text` to each line in the selected region.

|Parameter|Type  |Required|Description   |
|---------|------|--------|--------------|
|`text`   |string|yes     |Text to append|

```sh
qed 'at() | qed:suffix(text:" \\")'
```

### `qed:number(start?, width?)`

Prefixes each line in the selected region with its line number and a colon-space separator.

|Parameter|Type   |Default           |Description                                                       |
|---------|-------|------------------|------------------------------------------------------------------|
|`start`  |integer|stream line number|Origin for numbering; `start:1` gives region-relative numbering   |
|`width`  |integer|minimal           |Minimum digit width; numbers are right-aligned with leading spaces|

```sh
qed 'at() | qed:number()'                  # stream line numbers:    3: foo
qed 'at() | qed:number(start:1)'           # region-relative:        1: foo
qed 'at() | qed:number(width:4)'           # right-aligned:       3: foo
qed 'at() | qed:number(start:1, width:4)'  # both:                1: foo
```

### `qed:copy(after|before|at)`

Inserts a copy of the selected region at a target position.
Exactly one destination parameter is required.

|Parameter|Type                   |Description                                      |
|---------|-----------------------|-------------------------------------------------|
|`after`  |literal, regex, or name|Insert copy after lines matching this pattern    |
|`before` |literal, regex, or name|Insert copy before lines matching this pattern   |
|`at`     |literal, regex, or name|Overwrite lines matching this pattern with a copy|

```sh
qed 'at("source") | qed:copy(after:"target")'    # copy to after target
qed 'at("source") | qed:copy(before:"target")'   # copy to before target
qed 'at("source") | qed:copy(at:"target")'       # overwrite target with copy
```

### `qed:move(after|before|at)`

Moves the selected region to a target position and removes the original.
Same destination parameters as `qed:copy()`.

```sh
qed 'at("source") | qed:move(after:"target")'
```

### `qed:file()`

Materializes the selected region to a temp file and injects its path as `${QED_FILE}`
into the downstream command’s environment.
Used when a command requires a seekable file rather than stdin. No parameters.

```sh
qed 'at(region) | qed:file() | sort "${QED_FILE}"'
qed 'at(region) | qed:file() | command --input "${QED_FILE}"'
```

### `qed:warn()` / `qed:fail()`

`qed:warn()` emits the selected region to stderr and continues.
`qed:fail()` exits non-zero immediately. Neither takes parameters.

```sh
qed 'at(/FORBIDDEN/) | qed:warn()'   # log to stderr, keep processing
qed 'at(/FORBIDDEN/) | qed:fail()'   # abort
```

### `qed:debug:count()` / `qed:debug:print()`

Debugging aids. `qed:debug:count()` emits the match count to stderr.
`qed:debug:print()` echoes the selected region to stderr while passing it through unchanged.
Neither takes parameters.

-----

## Generation Processors

Generation processors ignore stdin and produce output from their parameters.
They compose with `qed:replace()` for placeholder substitution
and with `after`/`before` for direct insertion.

### `qed:uuid(version?, namespace?, name?)`

Generates a UUID.

|Parameter  |Type                          |Default|Description                                                           |
|-----------|------------------------------|-------|----------------------------------------------------------------------|
|`version`  |`4` | `5` | `7`               |`7`    |UUID version                                                          |
|`namespace`|`url` | `dns` | `oid` | `x500`|—      |Required for v5                                                       |
|`name`     |string                        |—      |Required for v5; hashed with namespace to produce a deterministic UUID|

```sh
qed 'at(/\{\{uuid\}\}/) | qed:replace("{{uuid}}", qed:uuid())'
qed 'at(/\{\{uuid\}\}/) | qed:replace("{{uuid}}", qed:uuid(version:4))'
qed 'at(/\{\{uuid\}\}/) | qed:replace("{{uuid}}", qed:uuid(version:5, namespace:url, name:"https://example.com"))'
after(header) | qed:uuid()     # insert directly as a new line
```

### `qed:timestamp(format?, timezone?)`

Generates a timestamp.

|Parameter |Type                                                    |Default        |Description            |
|----------|--------------------------------------------------------|---------------|-----------------------|
|`format`  |`iso8601` | `unix` | `datetime` | custom strftime string|`iso8601`      |Output format          |
|`timezone`|IANA timezone name or `UTC`                             |system timezone|Timezone for formatting|

```sh
qed 'at(/\{\{ts\}\}/) | qed:replace("{{ts}}", qed:timestamp())'
qed 'at(/\{\{ts\}\}/) | qed:replace("{{ts}}", qed:timestamp(format:unix))'
qed 'at(/\{\{ts\}\}/) | qed:replace("{{ts}}", qed:timestamp(format:datetime, timezone:UTC))'
qed 'at(/\{\{ts\}\}/) | qed:replace("{{ts}}", qed:timestamp(format:"%d %b %Y"))'
after(header) | qed:timestamp(format:datetime, timezone:UTC)
```

### `qed:random(length, alphabet?)`

Generates a random string of the given length drawn from the given alphabet.

|Parameter |Type                           |Default  |Description                     |
|----------|-------------------------------|---------|--------------------------------|
|`length`  |integer                        |required |Number of characters to generate|
|`alphabet`|named alphabet or custom string|`numeric`|Character set to draw from      |

Named alphabets:

|Name       |Characters                              |
|-----------|----------------------------------------|
|`numeric`  |`0-9`                                   |
|`alpha`    |`a-z`                                   |
|`upper`    |`A-Z`                                   |
|`alnum`    |`a-zA-Z0-9`                             |
|`hex`      |`0-9a-f`                                |
|`HEX`      |`0-9A-F`                                |
|`base32`   |RFC 4648 `A-Z2-7`                       |
|`crockford`|`0-9A-Z` excluding `I`, `L`, `O`, `U`   |
|`bech32`   |`qpzry9x8gf2tvdw0s3jn54khce6mua7l`      |
|`base58`   |Bitcoin alphabet — no `0`, `O`, `I`, `l`|
|`base62`   |`a-zA-Z0-9`                             |
|`base64url`|URL-safe base64                         |
|`ascii`    |All printable ASCII                     |
|`symbol`   |Printable non-alphanumeric ASCII        |

```sh
qed 'at(/\{\{token\}\}/) | qed:replace("{{token}}", qed:random(32))'
qed 'at(/\{\{token\}\}/) | qed:replace("{{token}}", qed:random(32, alphabet:base62))'
qed 'at(/\{\{token\}\}/) | qed:replace("{{token}}", qed:random(8, alphabet:"abc123"))'
after(header) | qed:random(16, alphabet:hex)
```

-----

## External Processors

Any command on `PATH` can be used as a processor.
The selected region is passed as stdin; the command’s stdout replaces the region.

```sh
at("bar") | tr 'a-z' 'A-Z'
at(/^\{/) | jq '.name'
from(start+) > to(end+) | sort
```

Chain multiple processors in a pipeline:

```sh
at() | upcase | trim | add-prefix
```

Use `qed:file()` for commands that require a seekable file:

```sh
at(region) | qed:file() | sort "${QED_FILE}"
```

### Aliases

Aliases bind a name to a processor chain, enabling reuse in script files.

```sh
trim=qed:replace(/^\s+/, "") | qed:replace(/\s+$/, "")
at(header) | trim
at(footer) | trim
```

Aliases compose:

```sh
clean=trim | normalize
```

### Name resolution

```
qed:upper()    — internal processor, always
upper          — alias if defined, else resolved via PATH
\upper         — bypass alias, PATH only
```

-----

## Script Files

Use `-f` to load a script from a file:

```sh
qed -f transform.qed input.txt
```

Script files support shebangs for direct execution:

```sh
#!/usr/bin/env qed -f
# transform.qed

at(/^\s*\/\/\s*TODO:/) | qed:delete()
at(/^func /) | qed:upper()
```

```sh
chmod +x transform.qed
./transform.qed < input.go
```

-----

## Comparison

`qed` targets the class of text editing tasks where `sed` and `awk` become
unwieldy: multi-line region selection, structured edits to source files, and
in-place transformation with safety guarantees.

The example below — deleting a named Go function — is representative.
It requires finding a pattern, collecting lines until a closing brace, and
removing the whole block. It is a routine task that comes up constantly in
automated refactoring, code generation, and migration scripts.

**Task:** delete the `handleRequest` function from a Go source file.

```go
// input: main.go
package main

func keep() {
    println("keep")
}

func handleRequest() {
    println("handle")
}

func alsoKeep() {
    println("also keep")
}
```

**sed:**

```sh
sed '/^func handleRequest(/,/^}/{
    /^func handleRequest(/d
    /^}/d
    d
}' main.go
```

This deletes lines between the function signature and closing brace but
requires three separate delete commands to handle the boundaries correctly.
It also matches any `}` in the file — a nested struct literal or closure
would terminate the range prematurely.
`sed` has no native concept of balanced delimiters, so the pattern breaks on
real-world Go functions containing inner braces.
Adding the `--in-place` equivalent (`-i`) varies by platform: `-i ''` on macOS,
`-i` on GNU/Linux.

**awk:**

```awk
awk '/^func handleRequest\(/{skip=1} skip && /^}/{skip=0; next} !skip' main.go
```

Better than `sed` in that a single pass can track state, but the logic is
split across three pattern-action rules that are easy to mis-order.
Testing and extending it requires understanding how awk evaluates rules in sequence.
In-place editing requires a temp file and `mv`.

**perl:**

```sh
perl -i -0777 -pe 's/^func handleRequest\(\) \{.*?^\}\n//ms' main.go
```

`-0777` slurps the entire file into memory so that `.` matches newlines.
The `.*?` non-greedy match relies on backtracking, which is O(n²) in the
worst case and fails on pathological inputs.
The regex has no awareness of Go syntax — a comment containing `}` on its
own line would terminate the match incorrectly.

**qed:**

```sh
qed --in-place '
func_start=/^func handleRequest\(/
func_end=/^\}/
from(func_start+) > to(func_end+) | qed:delete()
' main.go
```

`from > to` is `qed`’s native range primitive.
`func_start+` and `func_end+` make both boundaries inclusive — the signature
and closing brace are deleted along with the body.
The match is line-oriented: `func_end` matches the first `}` at column 1
after the start of the function, which is the correct closing brace for
top-level Go functions.
`--in-place` is atomic on all platforms — the original file is replaced only
after the full transformation succeeds.

-----

### Summary

|Capability                          |sed          |awk           |perl         |qed          |
|------------------------------------|-------------|--------------|-------------|-------------|
|Multi-line range selection          |✗ fragile    |△ stateful    |△ slurp+regex|✓ native     |
|In-place editing (cross-platform)   |△ flag varies|✗ temp file   |✓            |✓ atomic     |
|External processor pipeline         |✗            |✗             |✗            |✓ any command|
|Named patterns and aliases          |✗            |✗             |✗            |✓            |
|Guaranteed linear-time regex        |✗            |△ impl-defined|✗            |✓ RE2        |
|Insertion at arbitrary positions    |✗            |△             |✗            |✓            |
|Generation (UUID, timestamp, random)|✗            |✗             |△ modules    |✓ built-in   |
|`$EDITOR` compatible                |△            |✗             |△            |✓            |

-----

## Use Cases

### Code editing

```sh
# Remove all TODO comments
qed 'at(/^\s*\/\/\s*TODO:/) | qed:delete()' main.go

# Delete a function by name
qed --in-place '
func_start=/^func handleRequest\(/
func_end=/^\}/
from(func_start+) > to(func_end+) | qed:delete()
' main.go

# Add a deprecation notice above every Old* function
qed --in-place '
target=/^func Old/
before(target) | echo "// Deprecated: use New* equivalent instead."
' main.go

# Indent a selected region
qed 'from(/^func /+) > to(/^\}/+) | qed:indent(width:4)' main.go

# Remove trailing whitespace
qed 'at(/\s+$/) | qed:replace(/\s+$/, "")' file.txt
```

### Config manipulation

```sh
# Update a version field in Cargo.toml
qed --in-place 'at(/^version = /) | qed:replace(/=.*/, "= \"2.0.0\"")' Cargo.toml

# Comment out a section in an INI file
qed --in-place '
section=/^\[database\]/
section_end=/^\[/
from(section+) > to(section_end) | qed:prefix(text:"# ")
' config.ini

# Replace a value using an environment variable
qed --in-place '
at(/^  image:/) | qed:replace(/:.*/, ": myrepo/myimage:${IMAGE_TAG}")
' deployment.yaml

# Delete a YAML key block
qed --in-place '
key=/^  annotations:/
next_key=/^  [a-z]/
from(key+) > to(next_key) | qed:delete()
' deployment.yaml
```

### Log processing

```sh
# Extract only ERROR lines
qed --extract 'at(/\bERROR\b/)' app.log

# Delete DEBUG lines
qed 'at(/\bDEBUG\b/) | qed:delete()' app.log

# Add line numbers
qed 'at() | qed:number()' app.log

# Extract a time range
qed --extract '
start=/2026-02-26 09:00/
end=/2026-02-26 10:00/
from(start+) > to(end+) | qed:skip()
' app.log

# Reformat timestamps from yyyy-mm-dd to dd/mm/yyyy
qed 'at(/^\d{4}-\d{2}-\d{2}/) | qed:replace(/(\d{4})-(\d{2})-(\d{2})/, /$3\/$2\/$1/)' app.log
```

### Code generation

```sh
#!/usr/bin/env qed -f
# generate-routes.qed — replace region between markers with fresh output
marker_start=/\/\/ CODE GENERATED START/
marker_end=/\/\/ CODE GENERATED END/
from(marker_start+) > to(marker_end) | ./scripts/generate-routes.sh
```

```go
//go:generate qed --in-place -f generate-routes.qed routes.go
```

```sh
# Stamp a build version into source
qed --in-place '
at(/^var Version = /) | qed:replace(/=.*/, "= \"${BUILD_VERSION}\"")
' version.go
```

### Template rendering

```sh
# Replace multiple placeholders
qed '
at(/\{\{APP_NAME\}\}/) | qed:replace("{{APP_NAME}}", "${APP_NAME}")
at(/\{\{VERSION\}\}/)  | qed:replace("{{VERSION}}", "${VERSION}")
' template.yaml

# Inject a UUID
qed 'at(/\{\{uuid\}\}/) | qed:replace("{{uuid}}", qed:uuid())' template.sql

# Inject a timestamp
qed '
at(/\{\{generated_at\}\}/) |
    qed:replace("{{generated_at}}", qed:timestamp(format:datetime, timezone:UTC))
' report-template.md
```

### Document processing

```sh
# Promote all headings by one level
qed 'at(/^##/) | qed:replace(/^##/, "#")' doc.md

# Extract all fenced code blocks
qed --extract '
fence=/^```/
from(fence+) > to(fence+) | qed:skip()
' doc.md

# Wrap long lines
qed 'at(/^[^|].{80,}/) | qed:wrap(width:80)' README.md

# Add a note after a section heading
qed --in-place '
after(/^## Installation/) | printf "\n_Last updated: %s_\n" "$(date +%Y-%m-%d)"
' README.md
```

-----

## AI-Assisted Transformation

`qed` composes naturally with AI CLI tools like [`llm`](https://github.com/simonw/llm).
The selected region is passed as stdin — the AI’s output replaces it.

```sh
# Implement a TODO comment
qed --in-place '
todo=/\/\/ TODO:.*/
at(todo) | llm "implement this based on the comment"
' main.go

# Add error handling to a function
qed --in-place '
func_start=/^func /
func_end=/^\}/
from(func_start+) > to(func_end+) |
    llm "add idiomatic Go error handling to this function"
' main.go

# Rewrite comments to be more concise
qed --in-place 'at(/^\/\/.*/) | llm "rewrite this comment more concisely"' main.go

# Generate a docstring for a function
qed --in-place '
func_start=/^func /
before(func_start) |
    llm -s "You are a Go expert. Write a godoc comment for this function." \
        --stdin-as-file
' main.go
```

-----

## `$EDITOR` Integration

`qed` can serve as `$EDITOR` for programs that open a file for non-interactive
transformation — `git`, `cron`, `kubectl`, `mutt`, and others.

**Git commit message cleanup:**

```sh
#!/usr/bin/env qed -f
# ~/.config/qed/git-commit.qed

at(/\s+$/) | qed:replace(/\s+$/, "")   # trim trailing whitespace
at(/^#/) | qed:delete()                 # remove git comment lines
```

```sh
export EDITOR="qed --in-place -f ~/.config/qed/git-commit.qed"
```

**`kubectl edit` — enforce resource limits:**

```sh
#!/usr/bin/env qed -f
limits=/^\s*limits:/
after(limits) |
    printf "          cpu: \"500m\"\n          memory: \"256Mi\"\n"
```

```sh
EDITOR="qed --in-place -f enforce-limits.qed" kubectl edit deployment myapp
```

**mutt — append signature:**

```sh
#!/usr/bin/env qed -f
after() | cat ~/.signature
```

```sh
export EDITOR="qed --in-place -f ~/.config/qed/mutt-sign.qed"
```

The `-i` shorthand keeps `$EDITOR` assignments concise:

```sh
export EDITOR="qed -if ~/.config/qed/transform.qed"
```

-----

## Diagnostics

Warnings and errors are written to stderr in a consistent format:

```
qed: warning: 1:1-10:  at("quux"): no lines matched
qed:   error: 2:5-18:  qed:delete(): processor failed
```

Location is `line:start-end` using 1-based byte offsets.
Each event is one line; no summary is emitted at the end of a run.

Exit codes: `0` on success, `1` on any error.
Warnings do not affect the exit code.