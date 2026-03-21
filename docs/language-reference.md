# qed Language Reference

This document is the complete reference for qed's scripting language.
For installation and a quick introduction, see the [README](../README.md).
For CLI flags, see [CLI Reference](cli-reference.md).

---

## The Select-Action Model

Every `qed` statement has the form:

```
selector | processor-chain
```

The **selector** identifies which lines to act on.
The **processor chain** transforms those lines.
Unselected lines pass through unchanged.

Multiple statements execute sequentially —
each statement sees the output of the prior one:

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

---

## Selectors

### `at(pattern?, nth?, on_error?)`

Selects lines matching a pattern.
With no pattern, selects every line in the stream.

| Parameter  | Type                       | Default         | Description                                |
| ---------- | -------------------------- | --------------- | ------------------------------------------ |
| `pattern`  | literal, regex, or name    | —               | Pattern to match; omit to select all lines |
| `nth`      | nth-expr                   | all occurrences | Which occurrences to select                |
| `on_error` | `fail` \| `warn` \| `skip` | `fail`          | Behaviour when no lines match              |

```sh
qed 'at("bar") | qed:delete()'              # lines containing "bar"
qed 'at(/^func /) | qed:upper()'            # lines matching regex
qed 'at() | qed:number()'                   # all lines
qed 'at("x", nth:2) | qed:delete()'         # second occurrence only
qed 'at("x", on_error:skip) | qed:delete()' # silent no-match
```

### `after(pattern?, nth?, on_error?)`

Insertion point immediately after lines matching `pattern`.
The processor receives empty stdin;
its stdout is inserted at the cursor.
With no pattern, targets the end of the stream.

| Parameter  | Type                       | Default         | Description                                    |
| ---------- | -------------------------- | --------------- | ---------------------------------------------- |
| `pattern`  | literal, regex, or name    | —               | Pattern to match; omit to target end of stream |
| `nth`      | nth-expr                   | all occurrences | Which occurrences to insert after              |
| `on_error` | `fail` \| `warn` \| `skip` | `fail`          | Behaviour when no lines match                  |

```sh
qed 'after("header") | echo "new line"'   # insert after each "header" line
qed 'after() | cat footer.txt'            # append to end of stream
```

### `before(pattern?, nth?, on_error?)`

Insertion point immediately before lines matching `pattern`.
With no pattern, targets the beginning of the stream.

| Parameter  | Type                       | Default         | Description                                      |
| ---------- | -------------------------- | --------------- | ------------------------------------------------ |
| `pattern`  | literal, regex, or name    | —               | Pattern to match; omit to target start of stream |
| `nth`      | nth-expr                   | all occurrences | Which occurrences to insert before               |
| `on_error` | `fail` \| `warn` \| `skip` | `fail`          | Behaviour when no lines match                    |

```sh
qed 'before(/^func /) | cat license.txt'  # insert before every function
qed 'before() | echo "---"'               # prepend to start of stream
```

### `from(pattern, nth?, on_error?)`

Selects from the matching line to the end of the stream.
Append `+` to the pattern to make the boundary inclusive
(include the matching line in the region).

| Parameter  | Type                       | Default         | Description                   |
| ---------- | -------------------------- | --------------- | ----------------------------- |
| `pattern`  | literal, regex, or name    | required        | Start boundary pattern        |
| `nth`      | nth-expr                   | all occurrences | Which occurrences to select   |
| `on_error` | `fail` \| `warn` \| `skip` | `fail`          | Behaviour when no lines match |

```sh
qed 'from("start") | qed:delete()'                   # delete from "start" to end
qed 'from("start", nth:1) | qed:delete()'            # first occurrence only
qed 'from("start", on_error:skip) | qed:delete()'    # silent no-match
```

### `to(pattern, nth?, on_error?)`

Selects from the beginning of the stream to the matching line.
Append `+` to the pattern to make the boundary inclusive.

| Parameter  | Type                       | Default         | Description                   |
| ---------- | -------------------------- | --------------- | ----------------------------- |
| `pattern`  | literal, regex, or name    | required        | End boundary pattern          |
| `nth`      | nth-expr                   | all occurrences | Which occurrences to select   |
| `on_error` | `fail` \| `warn` \| `skip` | `fail`          | Behaviour when no lines match |

```sh
qed 'to("end") | qed:delete()'                       # delete from start to "end"
qed 'to("end", on_error:skip) | qed:delete()'        # silent no-match
```

### `>` — Range Composition

The `>` operator composes two or more selectors into a compound selector.
The result is the intersection — lines must satisfy all steps.

The most common idiom is `from() > to()` for closed ranges,
but `>` works between any selectors:

```sh
from("start") > to("end") | qed:delete()         # closed range
from("start"+) > to("end"+) | qed:delete()       # inclusive both boundaries
at(section) > at(subsection) | qed:upper()        # subsection within section
```

Append `+` to a `from` or `to` pattern to make that boundary inclusive.
By default, boundaries are exclusive —
the matching line is not included in the region.

```sh
from("start") > to("end")       # exclude both boundaries
from("start"+) > to("end")      # include start, exclude end
from("start"+) > to("end"+)     # include both
```

`on_error` can be specified on any step:

```sh
from("start", on_error:skip) > to("end") | qed:delete()
```

---

## Patterns

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
Forward references are permitted.
Redefining a name warns; last definition wins.

```sh
qed --file - <<'EOF'
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

---

## `nth` — Occurrence Selection

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

---

## `on_error` — No-Match Behaviour

| Value  | Description                         |
| ------ | ----------------------------------- |
| `fail` | Exit non-zero (default)             |
| `warn` | Emit a warning to stderr, exit zero |
| `skip` | Silently continue, exit zero        |

Per-selector `on_error` overrides the global `--on-error` flag:

```sh
qed --on-error=skip '
at("required", on_error:fail) | qed:delete()   # always fails if not found
at("optional") | qed:delete()                  # inherits global: skip
' input.txt
```

---

## Built-in Processors

All processors use the `qed:` namespace.
Parameters use labeled syntax: `param:value`.

### `qed:delete()`

Removes the selected region from the stream.
No parameters.

```sh
qed 'at("bar") | qed:delete()'
```

### `qed:duplicate()`

Emits the selected region twice consecutively.
No parameters.

```sh
qed 'at("bar") | qed:duplicate()'
```

### `qed:skip()`

No-op passthrough.
Primarily useful with `--extract`.
No parameters.

```sh
qed --extract 'at(/\bERROR\b/) | qed:skip()'
```

### `qed:upper()` / `qed:lower()`

Convert the selected region to uppercase or lowercase.
No parameters.

```sh
qed 'at(/^const /) | qed:upper()'
```

### `qed:replace(match, replacement)`

Substitutes within the selected region.
Three replacement forms:

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

Narrows the selected region to the matched span;
the rest of the line is discarded.

| Parameter | Type             | Required | Description                        |
| --------- | ---------------- | -------- | ---------------------------------- |
| `pattern` | literal or regex | yes      | Pattern whose matched span to keep |

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

| Parameter | Type    | Default       | Description                      |
| --------- | ------- | ------------- | -------------------------------- |
| `width`   | integer | required      | Number of characters to indent   |
| `char`    | string  | `" "` (space) | Character to use for indentation |

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

| Parameter | Type    | Required | Description             |
| --------- | ------- | -------- | ----------------------- |
| `width`   | integer | yes      | Column width to wrap at |

```sh
qed 'at(/^[^|].{80,}/) | qed:wrap(width:80)'
```

### `qed:prefix(text)`

Prepends `text` to each line in the selected region.

| Parameter | Type   | Required | Description     |
| --------- | ------ | -------- | --------------- |
| `text`    | string | yes      | Text to prepend |

```sh
qed 'at() | qed:prefix(text:"// ")'
```

### `qed:suffix(text)`

Appends `text` to each line in the selected region.

| Parameter | Type   | Required | Description    |
| --------- | ------ | -------- | -------------- |
| `text`    | string | yes      | Text to append |

```sh
qed 'at() | qed:suffix(text:" \\")'
```

### `qed:number(start?, width?)`

Prefixes each line in the selected region with its line number and a colon-space separator.

| Parameter | Type    | Default            | Description                                                        |
| --------- | ------- | ------------------ | ------------------------------------------------------------------ |
| `start`   | integer | stream line number | Origin for numbering; `start:1` gives region-relative numbering    |
| `width`   | integer | minimal            | Minimum digit width; numbers are right-aligned with leading spaces |

```sh
qed 'at() | qed:number()'                  # stream line numbers:    3: foo
qed 'at() | qed:number(start:1)'           # region-relative:        1: foo
qed 'at() | qed:number(width:4)'           # right-aligned:       3: foo
qed 'at() | qed:number(start:1, width:4)'  # both:                1: foo
```

### `qed:copy(after|before|at)`

Inserts a copy of the selected region at a target position.
Exactly one destination parameter is required.

| Parameter | Type                    | Description                                       |
| --------- | ----------------------- | ------------------------------------------------- |
| `after`   | literal, regex, or name | Insert copy after lines matching this pattern     |
| `before`  | literal, regex, or name | Insert copy before lines matching this pattern    |
| `at`      | literal, regex, or name | Overwrite lines matching this pattern with a copy |

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
into the downstream command's environment.
Used when a command requires a seekable file rather than stdin.
No parameters.

```sh
qed 'at(region) | qed:file() | sort "${QED_FILE}"'
qed 'at(region) | qed:file() | command --input "${QED_FILE}"'
```

### `qed:warn()` / `qed:fail()`

`qed:warn()` emits the selected region to stderr and continues.
`qed:fail()` exits non-zero immediately.
Neither takes parameters.

```sh
qed 'at(/FORBIDDEN/) | qed:warn()'   # log to stderr, keep processing
qed 'at(/FORBIDDEN/) | qed:fail()'   # abort
```

### `qed:debug:count()` / `qed:debug:print()`

Debugging aids.
`qed:debug:count()` emits the match count to stderr.
`qed:debug:print()` echoes the selected region to stderr
while passing it through unchanged.
Neither takes parameters.

---

## Generation Processors

Generation processors ignore stdin and produce output from their parameters.
They compose with `qed:replace()` for placeholder substitution
and with `after`/`before` for direct insertion.

### `qed:uuid(version?, namespace?, name?)`

Generates a UUID.

| Parameter   | Type                              | Default | Description                                                            |
| ----------- | --------------------------------- | ------- | ---------------------------------------------------------------------- |
| `version`   | `4` \| `5` \| `7`                 | `7`     | UUID version                                                           |
| `namespace` | `url` \| `dns` \| `oid` \| `x500` | —       | Required for v5                                                        |
| `name`      | string                            | —       | Required for v5; hashed with namespace to produce a deterministic UUID |

```sh
qed 'at(/\{\{uuid\}\}/) | qed:replace("{{uuid}}", qed:uuid())'
qed 'at(/\{\{uuid\}\}/) | qed:replace("{{uuid}}", qed:uuid(version:4))'
qed 'at(/\{\{uuid\}\}/) | qed:replace("{{uuid}}", qed:uuid(version:5, namespace:url, name:"https://example.com"))'
after(header) | qed:uuid()     # insert directly as a new line
```

### `qed:timestamp(format?, timezone?)`

Generates a timestamp.

| Parameter  | Type                                                        | Default         | Description             |
| ---------- | ----------------------------------------------------------- | --------------- | ----------------------- |
| `format`   | `iso8601` \| `unix` \| `datetime` \| custom strftime string | `iso8601`       | Output format           |
| `timezone` | IANA timezone name or `UTC`                                 | system timezone | Timezone for formatting |

```sh
qed 'at(/\{\{ts\}\}/) | qed:replace("{{ts}}", qed:timestamp())'
qed 'at(/\{\{ts\}\}/) | qed:replace("{{ts}}", qed:timestamp(format:unix))'
qed 'at(/\{\{ts\}\}/) | qed:replace("{{ts}}", qed:timestamp(format:datetime, timezone:UTC))'
qed 'at(/\{\{ts\}\}/) | qed:replace("{{ts}}", qed:timestamp(format:"%d %b %Y"))'
after(header) | qed:timestamp(format:datetime, timezone:UTC)
```

### `qed:random(length, alphabet?)`

Generates a random string of the given length drawn from the given alphabet.

| Parameter  | Type                            | Default   | Description                      |
| ---------- | ------------------------------- | --------- | -------------------------------- |
| `length`   | integer                         | required  | Number of characters to generate |
| `alphabet` | named alphabet or custom string | `numeric` | Character set to draw from       |

Named alphabets:

| Name        | Characters                               |
| ----------- | ---------------------------------------- |
| `numeric`   | `0-9`                                    |
| `alpha`     | `a-z`                                    |
| `upper`     | `A-Z`                                    |
| `alnum`     | `a-zA-Z0-9`                              |
| `hex`       | `0-9a-f`                                 |
| `HEX`       | `0-9A-F`                                 |
| `base32`    | RFC 4648 `A-Z2-7`                        |
| `crockford` | `0-9A-Z` excluding `I`, `L`, `O`, `U`    |
| `bech32`    | `qpzry9x8gf2tvdw0s3jn54khce6mua7l`       |
| `base58`    | Bitcoin alphabet — no `0`, `O`, `I`, `l` |
| `base62`    | `a-zA-Z0-9`                              |
| `base64url` | URL-safe base64                          |
| `ascii`     | All printable ASCII                      |
| `symbol`    | Printable non-alphanumeric ASCII         |

```sh
qed 'at(/\{\{token\}\}/) | qed:replace("{{token}}", qed:random(32))'
qed 'at(/\{\{token\}\}/) | qed:replace("{{token}}", qed:random(32, alphabet:base62))'
qed 'at(/\{\{token\}\}/) | qed:replace("{{token}}", qed:random(8, alphabet:"abc123"))'
after(header) | qed:random(16, alphabet:hex)
```

---

## External Processors

Any command on `PATH` can be used as a processor.
The selected region is passed as stdin;
the command's stdout replaces the region.

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

---

## Aliases

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

### Name Resolution

```
qed:upper()    — internal processor, always
upper          — alias if defined, else resolved via PATH
\upper         — bypass alias, PATH only
```

---

## Script Files

Use `--file` to load a script from a file:

```sh
qed --file transform.qed input.txt
```

Script files support shebangs for direct execution:

```sh
#!/usr/bin/env qed --file
# transform.qed

at(/^\s*\/\/\s*TODO:/) | qed:delete()
at(/^func /) | qed:upper()
```

```sh
chmod +x transform.qed
./transform.qed < input.go
```

---

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
#!/usr/bin/env qed --file
# generate-routes.qed — replace region between markers with fresh output
marker_start=/\/\/ CODE GENERATED START/
marker_end=/\/\/ CODE GENERATED END/
from(marker_start+) > to(marker_end) | ./scripts/generate-routes.sh
```

```go
//go:generate qed --in-place --file generate-routes.qed routes.go
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

````sh
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
````

---

## AI-Assisted Transformation

`qed` composes naturally with AI CLI tools like [`llm`](https://github.com/simonw/llm).
The selected region is passed as stdin — the AI's output replaces it.

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

---

## `$EDITOR` Integration

`qed` can serve as `$EDITOR` for programs that open a file for non-interactive
transformation — `git`, `cron`, `kubectl`, `mutt`, and others.

**Git commit message cleanup:**

```sh
#!/usr/bin/env qed --file
# ~/.config/qed/git-commit.qed

at(/\s+$/) | qed:replace(/\s+$/, "")   # trim trailing whitespace
at(/^#/) | qed:delete()                 # remove git comment lines
```

```sh
export EDITOR="qed --in-place --file ~/.config/qed/git-commit.qed"
```

**`kubectl edit` — enforce resource limits:**

```sh
#!/usr/bin/env qed --file
limits=/^\s*limits:/
after(limits) |
    printf "          cpu: \"500m\"\n          memory: \"256Mi\"\n"
```

```sh
EDITOR="qed --in-place --file enforce-limits.qed" kubectl edit deployment myapp
```

**mutt — append signature:**

```sh
#!/usr/bin/env qed --file
after() | cat ~/.signature
```

```sh
export EDITOR="qed --in-place --file ~/.config/qed/mutt-sign.qed"
```

