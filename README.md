# qed

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="assets/qed-logo-dark.svg">
  <img src="assets/qed-logo.svg" alt="qed" xheight="80">
</picture>

A modern stream editor for source files and config files.

> **Alpha release.** `qed` is functional but incomplete — not all processors are implemented yet. APIs and behavior may change before 1.0.

---

## What is qed?

`qed` transforms text files using a concise select-action model:
select a region of lines, pipe it through a processor.
It is designed for the things `sed` and `awk` handle awkwardly:
structured edits to source files, config manipulation, log processing, and code generation —
tasks where you need to target a function body, a YAML block,
or a range of log lines rather than a single pattern.

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

---

## Why qed?

`qed` targets the class of text editing tasks where `sed` and `awk` become unwieldy:
multi-line region selection, structured edits to source files,
and in-place transformation with safety guarantees.

| Capability                           | sed           | awk            | perl          | qed           |
| ------------------------------------ | ------------- | -------------- | ------------- | ------------- |
| Multi-line range selection           | ✗ fragile     | △ stateful     | △ slurp+regex | ✓ native      |
| In-place editing (cross-platform)    | △ flag varies | ✗ temp file    | ✓             | ✓ atomic      |
| External processor pipeline          | ✗             | ✗              | ✗             | ✓ any command |
| Named patterns and aliases           | ✗             | ✗              | ✗             | ✓             |
| Guaranteed linear-time regex         | ✗             | △ impl-defined | ✗             | ✓ RE2         |
| Insertion at arbitrary positions     | ✗             | △              | ✗             | ✓             |
| Generation (UUID, timestamp, random) | ✗             | ✗              | △ modules     | ✓ built-in    |
| `$EDITOR` compatible                 | △             | ✗              | △             | ✓             |

**Concrete example — delete a Go function by name:**

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

**sed** — three delete commands, breaks on nested braces, `-i` varies by platform:

```sh
sed '/^func handleRequest(/,/^}/{
    /^func handleRequest(/d
    /^}/d
    d
}' main.go
```

**awk** — stateful, three rules easy to mis-order, no in-place editing:

```sh
awk '/^func handleRequest\(/{skip=1} skip && /^}/{skip=0; next} !skip' main.go
```

**perl** — slurps file into memory, O(n²) backtracking, no awareness of syntax:

```sh
perl -i -0777 -pe 's/^func handleRequest\(\) \{.*?^\}\n//ms' main.go
```

**qed** — native range selection, named patterns, atomic write:

```sh
qed --in-place '
func_start=/^func handleRequest\(/
func_end=/^\}/
from(func_start+) > to(func_end+) | qed:delete()
' main.go
```

`>` composes selectors into a range — `from > to` is the most common idiom.
`+` makes a boundary inclusive.
`--in-place` is atomic on all platforms.

---

## Installation

**mise** (recommended):

```sh
mise use --global github:boldlygodev/qed
```

**Download a prebuilt binary** from the
[latest release](https://github.com/boldlygodev/qed/releases/latest)
(macOS and Linux, x86_64 and arm64):

```sh
# Example: macOS arm64
curl -fsSL https://github.com/boldlygodev/qed/releases/latest/download/qed-aarch64-apple-darwin.tar.gz \
  | tar xz
sudo mv qed /usr/local/bin/
```

**Build from source:**

```sh
git clone https://github.com/boldlygodev/qed
cd qed
cargo build --release
# binary is at target/release/qed
```

**Coming soon:** `cargo install qed` and Homebrew.

---

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

Use a script file with `--file` for multi-statement scripts:

```sh
qed --file transform.qed input.txt
```

See [CLI Reference](docs/cli-reference.md) for all flags and options.

---

## The Select-Action Model

Every `qed` statement has the form:

```
selector | processor-chain
```

The **selector** identifies which lines to act on.
The **processor chain** transforms those lines.
Unselected lines pass through unchanged.

Multiple statements are separated by newlines or `;`:

```
at("bar") | qed:delete()      # statement 1
at("baz") | qed:upper()       # statement 2: sees output of statement 1
```

See [Language Reference](docs/language-reference.md) for the full language specification.

---

## Cheat Sheet

### Selectors

| Selector | Description                                      |
| -------- | ------------------------------------------------ |
| `at`     | Selects matching lines                           |
| `after`  | Insertion point immediately after matching lines |
| `before` | Insertion point immediately before matching lines |
| `from`   | Selects from matching line to end of stream      |
| `to`     | Selects from start of stream to matching line    |
| `>`      | Composes selectors into a range                  |

`pattern` — `"literal"`, `/regex/`, `name`, `!pattern` (negated), `pattern+` (inclusive)

`nth` — `1` · `-1` (last) · `2n` (even) · `2n+1` (odd) · `1...3` (range) · `1,3,-1` (list)

### Processors

| Processor              | Description                                    |
| ---------------------- | ---------------------------------------------- |
| `qed:delete()`         | Remove selected region                         |
| `qed:duplicate()`      | Emit selected region twice                     |
| `qed:skip()`           | No-op passthrough (useful with `--extract`)    |
| `qed:upper()`          | Convert to uppercase                           |
| `qed:lower()`          | Convert to lowercase                           |
| `qed:replace(m, r)`    | Substitute within selected region              |
| `qed:substring(p)`     | Keep only the matched span                     |
| `qed:trim()`           | Strip leading/trailing whitespace              |
| `qed:indent(width:N)`  | Add indentation                                |
| `qed:dedent()`         | Remove common leading whitespace               |
| `qed:wrap(width:N)`    | Word-wrap at column width                      |
| `qed:prefix(text:"…")` | Prepend text to each line                      |
| `qed:suffix(text:"…")` | Append text to each line                       |
| `qed:number()`         | Add line numbers                               |
| `qed:copy(dest)`       | Copy selected region to a target position      |
| `qed:move(dest)`       | Move selected region to a target position      |
| `qed:file()`           | Materialize to temp file for seekable commands |
| `qed:warn()`           | Emit to stderr, continue                       |
| `qed:fail()`           | Exit non-zero                                  |
| `qed:debug:count()`    | Emit match count to stderr                     |
| `qed:debug:print()`    | Echo selected region to stderr (passthrough)   |
| `qed:uuid()`           | Generate a UUID                                |
| `qed:timestamp()`      | Generate a timestamp                           |
| `qed:random(N)`        | Generate a random string                       |

Any command on `PATH` can also be used as a processor —
the selected region is passed as stdin.

See [Language Reference](docs/language-reference.md) for parameter details and all options.

---

## Use Cases

### Config editing

```sh
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

### AI-assisted transformation

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
```

### `$EDITOR` integration

`qed` can serve as `$EDITOR` for programs that open a file for
non-interactive transformation — `git`, `cron`, `kubectl`, and others.

```sh
#!/usr/bin/env qed --file
# ~/.config/qed/git-commit.qed
at(/\s+$/) | qed:replace(/\s+$/, "")   # trim trailing whitespace
at(/^#/) | qed:delete()                 # remove git comment lines
```

```sh
export EDITOR="qed --in-place --file ~/.config/qed/git-commit.qed"
```

```sh
# Enforce resource limits during kubectl edit
EDITOR="qed --in-place --file enforce-limits.qed" kubectl edit deployment myapp
```

See [Language Reference](docs/language-reference.md) for more examples
including template rendering, log processing, and document manipulation.

---

## Diagnostics

Warnings and errors are written to stderr in a consistent format:

```
qed: warning: 1:1-10: at("quux"): no lines matched
qed: error:   2:5-18: qed:delete(): processor failed
```

Location is `line:start-end` using 1-based byte offsets.
Each event is one line; no summary is emitted at the end of a run.

Exit codes: `0` on success, `1` on any error.
Warnings do not affect the exit code.

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for prerequisites, build instructions,
and the development workflow.
