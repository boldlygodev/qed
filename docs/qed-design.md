# qed

`qed` is a modern stream editor optimized for shell scripting,
filling the gap between `sed`'s simplicity and `vim -es`'s power.
Stream-oriented by default, composable with Unix pipelines.
Implemented in Rust.

---

## Name and Extension

The tool is named `qed` — a deliberate nod to the original `qed` text editor,
developed at Bell Labs in the 1960s and a direct predecessor of `ed` and `sed`.
Naming a modern stream editor `qed` acknowledges that lineage without being derivative of `sed`.

`qed` also carries the mathematical meaning of *quod erat demonstrandum* —
"which was to be demonstrated" — an apt connotation for a tool that transforms text
precisely and predictably.

Script files use the `.qed` extension:

```sh
qed 'at("foo") | qed:delete()' file.txt      # one-liner
qed -f transform.qed file.txt                # script file
```

---

## Design Principles

These principles have guided design decisions and should guide future ones:

- **Prefer structural solutions over regex complexity** —
  selector composition covers most cases that would otherwise require backreferences or lookaheads
- **Prefer shell composition over tool complexity** —
  multi-pass transformations belong in shell pipelines, not in qed itself
- **Complexity belongs at the right layer** —
  selection, transformation, and orchestration are separate concerns
- **Predictable behavior over maximum expressiveness** —
  RE2 semantics, linear time matching, no hidden performance cliffs
- **Warn don't fail for statically undetectable issues** —
  warnings go to stderr, the stream is unaffected, the tool exits zero
- **Surface mistakes explicitly** —
  statically detectable mistakes warn or error with helpful messages rather than silently misbehaving

---

## Core Concepts

### Select-Action Model

The core primitive is selecting a region of the input stream and piping it through a processor:

```
selector | processor
```

The call site syntax is uniform across internal processors and external tools:

```
at(region) | qed:replace(/foo/, "bar")
at(region) | qed:delete()
at(region) | jq '.name'
```

### Stream Orientation

`qed` reads from stdin and writes to stdout.
Unselected content passes through unchanged by default.
Statements execute sequentially within a single invocation.
Multi-pass transformations can also be handled by composing multiple invocations:

```sh
qed 'at("foo") | qed:delete()' file.txt | qed 'at("bar") | qed:replace(/x/, "y")'
```

### Composability

`qed` is a Unix citizen — it composes naturally with pipelines, `&&`, `||`, and other tools.
Shell pipelines are the preferred mechanism for sequencing operations.
Complex transformations that qed cannot express cleanly belong in external processors.

AI-powered transformations are a natural use case via external processors.
Tools like [`llm`](https://github.com/simonw/llm) receive the selected region as stdin
and return transformed text as stdout — exactly the processor model `qed` is built around:

```sh
at(/^\/\/.*/) | llm "rewrite this comment more concisely"
from(/^func /+) > to(/^\}/+) | llm "add idiomatic error handling"
```

---

## Patterns

### Pattern Syntax

Patterns are the foundation of selection.
Named patterns are defined as statements and referenced by their bare identifier.
Inline patterns embed the pattern directly at the point of use.

`"…"` denotes a literal string — no special characters, no capture group interpretation, everywhere it appears.
`/…/` denotes a regex pattern on the left side of `qed:replace()` and a regex template on the right side.
Capture groups are only ever meaningful inside `/…/`.

```
name="pattern"      # named literal string pattern definition
name=/regex/        # named regex pattern definition
name                # named pattern reference
"pattern"           # inline literal string
/regex/             # inline regex
!name               # negated named pattern
!"pattern"          # negated inline literal string
!/regex/            # negated inline regex
name+               # named, inclusive boundary
"pattern"+          # inline literal string, inclusive boundary
/regex/+            # inline regex, inclusive boundary
!name+              # negated, inclusive boundary
!"pattern"+         # negated inline literal string, inclusive boundary
!/regex/+           # negated inline regex, inclusive boundary
```

`+` marks a boundary pattern as inclusive — the matching line is included in the region.
It is only meaningful on boundary patterns in `from` and `to`.
Using `+` elsewhere emits a warning and is ignored.

Named patterns may be referenced before their definition — forward references are permitted.
Redefining a named pattern that is already defined emits a warning; the last definition wins.

Best practice is inline patterns for one-liners and named patterns for script files.

### Regex Engine

Patterns use RE2 semantics via Rust's `regex` crate.
This guarantees linear time matching — no catastrophic backtracking regardless of input.

The constraints of RE2 (no backreferences, limited lookahead/lookbehind) are intentional.
`qed`'s architecture covers the common use cases for these features structurally:

- **Backreferences in substitutions** — handled by `qed:replace()` with named capture groups in the `/…/` template form
- **Context-dependent matching** — handled by selector composition (`from > to`, chaining)
- **Boundary detection** — handled by `before` and `after` selectors
- **Multiline matching** — expressed via regex flags (`(?s)`, `(?m)`) within the pattern itself

Complex transformations that genuinely require backtracking regex are best handled by
piping through an external tool (`perl`, `sd`, etc.) in the processor pipeline.

---

## Selectors

Selectors identify regions of the input stream for transformation.
All selectors are line-oriented — they always return whole lines.
All selectors support labeled parameters.
In the examples below, `pattern`, `p1`, `p2`, `section`, `subsection`, and `region`
are named pattern references.

### Operators

```
at()                             # entire stream
at(pattern)                      # matching line(s)
after(pattern)                   # insertion point immediately after matching line (empty region)
before(pattern)                  # insertion point immediately before matching line (empty region)
from(pattern)                    # matching line to end of stream
to(pattern)                      # start of stream to matching line
from(p1) > to(p2)               # closed range between two patterns
```

`>` is the narrowing operator — it intersects two regions:

```
at(section) > at(subsection)    # match subsection within section
from(p1+) > to(p2)             # closed range, include p1
```

### Inclusion

The `+` suffix on a boundary pattern controls whether the boundary line is included in the region.
Inclusion defaults to exclusive — the boundary line is not included:

```
from(p1+) > to(p2)             # include p1, exclude p2
from(p1+) > to(p2+)            # include both
from(p1) > to(p2)              # exclude both, default
```

### Insertion Points

`after` and `before` are insertion points — empty regions.
The processor receives empty stdin and whatever it writes to stdout is inserted at the cursor position.
Side-effect-only processors write nothing and nothing is inserted:

```
after(pattern) | echo "new line"          # insert static text
after(pattern) | date +%Y-%m-%d           # insert generated content
before(pattern) | generate-header         # insert processor output
after(pattern) | notify.sh                # side effect only, nothing inserted
```

`+` and `qed:file()` are meaningless for `after` and `before` — both emit a warning and are ignored:

```
after(/pattern/+) | command               # warns: + ignored on insertion point
after(/pattern/) | qed:file() | command   # warns: qed:file() ignored for empty region
```

### Common Parameters

All selectors support these labeled parameters.

**Occurrence** (`nth`) selects which occurrences to operate on,
using an `an+b` expression language inspired by CSS `nth-child`:

```
at(pattern, nth:1)              # first occurrence
at(pattern, nth:3)              # third occurrence
at(pattern, nth:-1)             # last occurrence
at(pattern, nth:-2)             # second to last
at(pattern, nth:2n)             # every second
at(pattern, nth:2n+1)           # every second starting from first
at(pattern, nth:2n-1)           # every second, offset back one
at(pattern, nth:-2n)            # every second from end
at(pattern, nth:-2n+1)          # every second from end, offset forward
at(pattern, nth:1...3)          # first through third, inclusive
at(pattern, nth:-3...-1)        # third from last through last
at(pattern, nth:1,3,-1)         # first, third, and last
at(pattern, nth:1...3,-2)       # first through third, and second to last
```

When `nth` is omitted, the selector matches **all** occurrences — equivalent to `nth:1n`.
When `nth` is present, the selector returns the **union** of all matched occurrences,
in **source order**, with duplicates deduplicated.
Duplicate detection and warnings depend on the expression form:

- `b` and `...` forms — duplicates are statically detectable, warn and deduplicate
- `an+b` forms — silent deduplication, overlap not statically knowable

```
1,1...3        # warns: duplicate occurrence 1, deduplicates
1...3,2        # warns: duplicate occurrence 2, deduplicates
2n,1...3       # silent deduplication
```

Edge cases:

- `0` or `-0` — zero has no meaning, term warned and ignored
- `-3...5` — cross-boundary range, hard error with suggestion to use explicit form e.g. `1...5,-3...-1`

**On-error** controls no-match behavior:

```
at(pattern, on_error:fail)      # no-match is a failure, default
at(pattern, on_error:warn)      # no-match emits to stderr, statement succeeds
at(pattern, on_error:skip)      # no-match is silent, statement succeeds
```

---

## Processors

### Handoff Model

The processor receives the selected region as stdin by default.
`qed:file()` is an internal processor that materializes the region to a temp file for processors
that require seekable input:

```
at(region) | command                                     # pass region as stdin, default
at(region) | qed:file() | command "${QED_FILE}"          # materialize; path via ${QED_FILE}
at(region) | qed:file() | command --input "${QED_FILE}"  # flag-based example
```

When `qed:file()` is in the pipeline, qed materializes the selected region to a
temp file and injects its path as `${QED_FILE}` into the environment of the
downstream external command.
`${QED_FILE}` can be referenced anywhere in the external command's argument list —
as a positional argument, as a flag value, or embedded in an unquoted argument.
`${QED_FILE}` refers to the most recently materialized file in the pipeline,
consistent with left-to-right execution order.
`${QED_FILE}` is only set when `qed:file()` has been invoked in the current
pipeline — referencing it otherwise expands to empty string with a warning,
following the same rules as other unset env vars.

`qed:file()` transforms the handoff mechanism, not the content.
Future expansion points like `qed:lsp()` would follow the same pattern.

### Internal Processors

Internal processors are always referenced with the `qed:` namespace.
`qed:` names are not aliasable or overridable — they always resolve to the internal processor.

**Region manipulation**

```
at(region) | qed:delete()
at(region) | qed:duplicate()                              # emit region twice
at(region) | qed:copy(after:p)                           # insert copy after p
at(region) | qed:copy(before:p)                          # insert copy before p
at(region) | qed:copy(at:p)                              # overwrite p with copy
at(region) | qed:copy(from:p1, to:p2)                   # overwrite range with copy
at(region) | qed:copy(from:p1+, to:p2)                  # range with inclusion control
at(region) | qed:copy(from:p1)                           # p1 to end of stream
at(region) | qed:copy(to:p2)                             # start of stream to p2
at(region) | qed:move(after:p)                           # same params as copy, removes source
at(region) | qed:move(at:p)
at(region) | qed:move(from:p1, to:p2)
```

`qed:copy` and `qed:move` share the same destination params.
Params are mutually exclusive between point-based (`at`, `after`, `before`) and
range-based (`from`, `to`) — mixing is an error.
Omitting both `from` and `to` while not specifying a point param is also an error.

**Text transformation**

`qed:replace()` accepts three replacement forms:

```
at(region) | qed:replace("literal", "replacement")        # literal match, literal replacement
at(region) | qed:replace(/(\d+)/, /$1 items/)             # regex match, template replacement
at(region) | qed:replace("{{placeholder}}", processor)    # literal match, pipeline replacement
```

The pipeline form runs the processor against the matched span and splices its stdout back
in place of the match. The surrounding content always survives.

```
at(region) | qed:substring(pattern)                       # narrow region to matched span, discard rest
at(region) | qed:trim()                                   # strip leading/trailing whitespace
at(region) | qed:upper()                                  # uppercase
at(region) | qed:lower()                                  # lowercase
at(region) | qed:indent(width:4)                          # indent by width
at(region) | qed:indent(width:4, char:"\t")               # indent with custom char
at(region) | qed:dedent()                                 # remove common leading whitespace
at(region) | qed:wrap(width:80)                           # word wrap at width
at(region) | qed:prefix(text:"// ")                       # prepend to each line
at(region) | qed:suffix(text:" \\")                       # append to each line
at(region) | qed:number()                                 # stream line numbers, colon-space separator — 3: foo
at(region) | qed:number(width:4)                          # right-align to minimum 4 digits —    3: foo
at(region) | qed:number(start:1)                          # region-relative numbering — 1: foo, 2: bar
at(region) | qed:number(start:1, width:4)                 # region-relative, aligned —    1: foo
```

**`qed:substring(pattern)`** narrows the selected region to exactly the matched span.
The rest of the line is discarded — downstream processors in the chain operate on the
substring alone. This is intentionally lossy; it does not break the pipeline contract.

Inversion — keeping everything *except* the match — is handled by composition:

```
at(region) | qed:replace(pattern, "")                     # delete matched span, keep the rest
```

**Stream control**

```
at(region) | qed:file() | command                         # materialize to temp file
at(region) | qed:warn()                                   # emit region to stderr, continue
at(region) | qed:fail()                                   # exit non-zero
at(region) | qed:skip()                                   # no-op, passthrough
at(region) | qed:debug:count()                            # emit match count to stderr
at(region) | qed:debug:print()                            # echo region to stderr, stream unchanged
```

**Generation**

Generation processors ignore stdin — they produce output from parameters alone.
They compose with `qed:replace()` for inline placeholder substitution,
and with `after` and `before` for direct insertion.
When used in an `after` or `before` pipeline, the processor receives empty stdin
and its output is inserted as a new line at the cursor position:

```
at(/{{uuid}}/) | qed:replace("{{uuid}}", qed:uuid())               # v7, default
at(/{{uuid}}/) | qed:replace("{{uuid}}", qed:uuid(version:4))    # random
at(/{{uuid}}/) | qed:replace("{{uuid}}", qed:uuid(version:5, namespace:url, name:https://example.com))

at(/{{ts}}/) | qed:replace("{{ts}}", qed:timestamp())                          # ISO 8601 UTC, default
at(/{{ts}}/) | qed:replace("{{ts}}", qed:timestamp(format:iso8601))            # explicit default
at(/{{ts}}/) | qed:replace("{{ts}}", qed:timestamp(format:iso8601z))           # ISO 8601 UTC explicit
at(/{{ts}}/) | qed:replace("{{ts}}", qed:timestamp(format:rfc2822))            # email/HTTP dates
at(/{{ts}}/) | qed:replace("{{ts}}", qed:timestamp(format:rfc3339))            # stricter ISO 8601 superset
at(/{{ts}}/) | qed:replace("{{ts}}", qed:timestamp(format:unix))               # unix epoch seconds
at(/{{ts}}/) | qed:replace("{{ts}}", qed:timestamp(format:unix_ms))            # unix epoch milliseconds
at(/{{ts}}/) | qed:replace("{{ts}}", qed:timestamp(format:unix_ns))            # unix epoch nanoseconds
at(/{{ts}}/) | qed:replace("{{ts}}", qed:timestamp(format:date))               # yyyy-MM-dd
at(/{{ts}}/) | qed:replace("{{ts}}", qed:timestamp(format:time))               # HH:mm:ss
at(/{{ts}}/) | qed:replace("{{ts}}", qed:timestamp(format:datetime))           # yyyy-MM-dd HH:mm:ss
at(/{{ts}}/) | qed:replace("{{ts}}", qed:timestamp(format:"yyyy/MM/dd HH:mm")) # custom LDML string
at(/{{ts}}/) | qed:replace("{{ts}}", qed:timestamp(timezone:UTC))              # explicit UTC, default
at(/{{ts}}/) | qed:replace("{{ts}}", qed:timestamp(timezone:"America/New_York")) # IANA, DST-aware
at(/{{ts}}/) | qed:replace("{{ts}}", qed:timestamp(timezone:"UTC+5:30"))       # fixed offset, no DST
at(/{{ts}}/) | qed:replace("{{ts}}", qed:timestamp(format:datetime, timezone:"Asia/Tokyo"))

# direct insertion — output becomes a new line at the cursor position
after(header)  | qed:uuid()           # insert a UUID after header
before(footer) | qed:timestamp()      # insert a timestamp before footer
after(section) | qed:random(16)       # insert a random string after section

after(header) | qed:random(16)                           # numeric 0-9, default
after(header) | qed:random(16, alphabet:alpha)           # a-z
after(header) | qed:random(16, alphabet:upper)           # A-Z
after(header) | qed:random(16, alphabet:alnum)           # a-zA-Z0-9
after(header) | qed:random(16, alphabet:hex)             # 0-9a-f
after(header) | qed:random(16, alphabet:HEX)             # 0-9A-F
after(header) | qed:random(16, alphabet:base32)          # RFC 4648, A-Z2-7
after(header) | qed:random(16, alphabet:crockford)       # 0-9A-Z excluding I, L, O, U
after(header) | qed:random(16, alphabet:bech32)          # qpzry9x8gf2tvdw0s3jn54khce6mua7l
after(header) | qed:random(16, alphabet:base58)          # Bitcoin alphabet, no 0/O/I/l
after(header) | qed:random(16, alphabet:base62)          # a-zA-Z0-9
after(header) | qed:random(16, alphabet:base64url)       # URL-safe base64
after(header) | qed:random(16, alphabet:ascii)           # all printable ASCII
after(header) | qed:random(16, alphabet:symbol)          # printable non-alphanumeric ASCII
after(header) | qed:random(16, alphabet:"abc123")        # custom alphabet
```

### Aliases

Aliases bind a name to a processor chain.
They are statements, using `=` as the assignment operator:

```
trim=qed:replace(/^\s+/, "") |
     qed:replace(/\s+$/, "")

at(header) | trim
at(footer) | trim
```

Aliases can compose:

```
clean=trim | normalize
```

Internal processors are exposed as short names through aliasing.
This means short names can be overridden:

```
replace=qed:replace
delete=qed:delete
```

### Name Resolution

- `qed:name` — internal processor, always
- `name` — alias if defined, otherwise resolved via PATH
- `\name` — bypass alias, resolve via PATH only

```
at(region) | qed:replace(/foo/, "bar")    # always internal
at(region) | replace(/foo/, "bar")        # alias if defined, else PATH
at(region) | \replace(/foo/, "bar")       # bypass alias, PATH only
```

---

## Statements

A statement is either a named pattern definition, an alias definition,
or a complete select-action expression.
Statements are separated by `;` or newlines.
Statements execute sequentially — each statement sees the output of the prior one,
not the original input. This is consistent with how multiple `qed` invocations
compose via shell pipelines.
Conflict resolution between overlapping regions follows last-declaration-wins.
Overlapping regions are not recommended — when regions overlap behavior may be surprising.
Prefer non-overlapping regions or split into separate invocations.

```
header=/^#/                                                # pattern definition
trim=qed:replace(/^\s+/, "") |
     qed:replace(/\s+$/, "")                              # alias definition
at(header) | qed:delete()                                 # select-action
at(/bar/) | qed:replace(/x/, "y")                         # select-action
```

### Line Continuation

Statements are not restricted to a single line.
A newline does not end a statement when the line ends
(ignoring trailing whitespace) with `>`, `|`, `||`, or `,`.
In the examples below, `pattern` and `p2` are named pattern references:

```
at(pattern) |
    qed:delete()                          # implicit continuation on |

at(pattern,
    nth:1...3) | qed:delete()             # implicit continuation on ,

at(pattern) >
    to(p2) | qed:delete()                 # implicit continuation on >
```

Within a processor expression, `\` can be used for explicit line continuation.
Trailing whitespace after `\` is an error — the parser will emit a clear message:

```
at(pattern) | command \
    --flag \
    --other                             # explicit \ within processor expression

at(pattern) \                          # error: \ not valid outside processor expression
    | command
```

---

## Error Handling

Two distinct failure layers with separate handling.

### Selector Errors

No-match behavior is controlled by the `on_error` parameter (default: `fail`).
The global `--on-error` flag sets the baseline; per-selector `on_error` overrides it:

- `on_error:fail` — no-match triggers `||` fallback if present, exits non-zero if not
- `on_error:warn` — no-match emits to stderr, statement succeeds, `||` not triggered
- `on_error:skip` — no-match is silent, statement succeeds, `||` not triggered

### Processor Errors

Processor errors always trigger `||` fallback.

### Fallback Statements

`||` introduces a fallback statement — a full select-action expression with its own selector
and processor chain. The fallback selects from the original input stream:

```
at("foo") | qed:delete() || at("bar") | qed:delete()          # fallback to different selection
at("foo", on_error:skip) | qed:delete() || qed:warn()          # only processor fail triggers fallback
at("foo") | qed:delete() || qed:fail()                         # explicit failure
```

### Output on Failure

`qed` emits output lines as soon as all statements have finished with them —
fragments are freed as they go rather than held until exit.
On failure, lines already emitted are already emitted.

When piping `qed` output into another tool, `set -o pipefail` is strongly recommended
so that a non-zero exit from `qed` fails the pipeline visibly regardless of downstream tool behavior:

```sh
set -o pipefail
qed -f transform.qed file.txt | grep "pattern"
```

With `--on-error=skip` or `--on-error=warn`, the user has explicitly accepted responsibility
for downstream behavior and `set -o pipefail` is optional.

---

## Environment Variable Expansion

Env vars are expanded implicitly everywhere — in pattern values and processor arguments.
Behavior is consistent between one-liners and script files.

- `${VAR}` — replaced with value of `VAR`; expands to empty string with a warning to stderr if unset
- `\${VAR}` — literal `${VAR}`, no expansion
- `--no-env` — disables all expansion globally, treating all `${VAR}` as literal

```
at("${PATTERN}") | qed:delete()          # expanded
at("\${PATTERN}") | qed:delete()         # literal ${PATTERN}
```

Only `${VAR}` form is supported — `$VAR` is not, avoiding spacing ambiguity.

---

## Diagnostics

All diagnostic messages are written to stderr.
The format is consistent across errors, warnings, and debug output:

```
qed: error:   5:12-20: at("foo"): no lines matched
qed: warning: 5:8-15:  at("foo"+): + ignored on at
qed: debug:   5:32-44: false: exit code 1
```

**Format:** `qed: <severity>: <location>: <source-expression>: <message>`

**Severity** is one of `error:`, `warning:`, or `debug:`.
Severity keywords are padded to `warning:` width so the location always starts at the same column.

**Location** is `line:start-end` using 1-based line and byte offsets.
Location fields are padded to the width of the widest span in the script,
computed from the AST before any statement executes.
The source expression and message are not padded — they follow naturally after the location.

**Source expression** echoes the span of source text that produced the diagnostic —
a selector, a processor, a parameter, or any expression.
This is always source text, never an internal identifier.

One diagnostic is emitted per event.
No end-of-run summary is emitted by default.

**`--dry-run` context lines:** unified diffs include 3 lines of unchanged context around each hunk,
matching the standard `diff` and `git diff` default.

---

## Invocation

```sh
qed 'at("foo") | qed:delete()' file.txt                       # one-liner
qed -f script.qed file.txt                                     # script file
qed --on-error=skip 'at("foo") | qed:delete()'                # global error mode override
qed --no-env 'at("${VAR}") | qed:delete()'                    # disable env expansion
qed --extract 'at("foo") | qed:delete()'                      # suppress passthrough
qed --output=result.txt 'at("foo") | qed:delete()' file.txt   # write to file instead of stdout
qed --in-place -f transform.qed file.txt                       # modify file directly, enables $EDITOR use
qed --dry-run -f transform.qed file.txt                        # preview changes as a unified diff
```

`--dry-run` produces a unified diff on stdout.
File paths in the `---`/`+++` header use fixed `a` / `b` placeholders regardless
of whether input comes from a file or stdin.
Timestamps are omitted from the header.
This format is stable, machine-readable, and composable with diff viewers like `delta`:

```sh
qed --dry-run -f transform.qed file.txt | delta
```

`--output` writes to the specified file instead of stdout, creating it if it doesn't exist
and overwriting if it does.
This makes `qed` usable in contexts where shell redirection is unavailable,
such as `//go:generate` directives:

```go
//go:generate qed --output=generated.go -f transform.qed input.go
```

`--in-place` modifies the input file directly using an atomic write — `qed` writes to a
temp file and renames it on success. This enables `qed` to be used as `$EDITOR`:

```sh
export EDITOR="qed --in-place -f ~/.config/qed/transform.qed"
```

---

## Todo

- [ ] **LSP integration** — future expansion of the handoff model to support LSP servers
- [ ] **Two-pattern forms** — `qed:replace(p1 > p2, replacement)` and `qed:substring(p1, p2)` for marking begin and end points of a span explicitly, as an alternative to regex

---

## Formal Grammar

```ebnf
(* Lexer pre-processing
   The lexer handles two steps before tokenisation:

   1. Statement continuation — a line ending with a continuation trigger
      ('>' '|' '||' ',') causes subsequent ignorable lines to be consumed
      and the next meaningful token treated as continuing the current statement.

   2. Explicit continuation — '\' immediately followed by '\n' within an
      external processor expression is consumed as whitespace. '\' is invalid
      outside external processor expressions and produces a hard error.

   The grammar below operates on the resulting token stream. *)


(* Top-level structure *)

program         ::= shebang? line* eof

line            ::= ws* (comment | statement | empty) line-end
                  | ws* statement ';'

shebang         ::= '#!' [^\n]* line-end

comment         ::= '#' [^\n]*

empty           ::= ws*

line-end        ::= '\n' | '\r\n'

ignorable-line  ::= ws* (comment | empty) line-end

ws              ::= ' ' | '\t'

eof             ::= (* end of input *)


(* Statements *)

statement       ::= pattern-def
                  | alias-def
                  | select-action

pattern-def     ::= identifier '=' pattern-value

alias-def       ::= identifier '=' processor-chain


(* Patterns *)

pattern-value   ::= string
                  | regex

string          ::= '"' dq-char* '"'
                  | "'" sq-char* "'"

dq-char         ::= [^"\\]
                  | '\\' .

sq-char         ::= [^'\\]
                  | '\\' .

regex           ::= '/' regex-char* '/'

regex-char      ::= [^/\\]
                  | '\\' .

pattern-ref     ::= negation? identifier inclusion?
                  | negation? string inclusion?
                  | negation? regex inclusion?

negation        ::= '!'

inclusion       ::= '+'


(* Select-action *)

select-action   ::= selector ws* '|' ws* processor-chain
                    (ws* '||' ws* fallback)?

fallback        ::= select-action
                  | processor-chain

selector        ::= simple-selector (ws* '>' ws* simple-selector)*

simple-selector ::= selector-op '(' ws* ')'
                  | selector-op '(' ws* pattern-ref ws* ')'
                  | selector-op '(' ws* pattern-ref ws* ',' ws* param-list ws* ')'

selector-op     ::= 'at'
                  | 'after'
                  | 'before'
                  | 'from'
                  | 'to'

param-list      ::= param (ws* ',' ws* param)*

param           ::= identifier ':' param-value

param-value     ::= identifier
                  | string
                  | integer
                  | nth-expr
                  | pattern-ref


(* Processor chain *)

processor-chain ::= processor (ws* '|' ws* processor)*

processor       ::= qed-processor
                  | external-processor


(* qed internal processors *)

qed-processor   ::= 'qed:' qed-name '(' ws* ')'
                  | 'qed:' qed-name '(' ws* arg-list ws* ')'

qed-name        ::= identifier (':' identifier)*

arg-list        ::= positional-args
                  | positional-args ws* ',' ws* param-list
                  | param-list

positional-args ::= positional-arg (ws* ',' ws* positional-arg)*

positional-arg  ::= pattern-ref
                  | string
                  | regex
                  | integer
                  | processor-chain


(* External processors *)

external-processor ::= ext-command ext-arg*

ext-command     ::= escape? command-name
                  | escape? path

command-name    ::= [a-zA-Z_] [a-zA-Z0-9_-]*

escape          ::= '\\'

path            ::= absolute-path
                  | relative-path

absolute-path   ::= ('/' path-segment)+

relative-path   ::= ('.' | '..') ('/' path-segment)*
                  | './' path-segment ('/' path-segment)*

path-segment    ::= [a-zA-Z0-9_.-]+

ext-arg         ::= ws+ ext-arg-value

ext-arg-value   ::= string
                  | unquoted-arg

unquoted-arg    ::= unquoted-char+

unquoted-char   ::= [^ \t\n|\\;'"]


(* nth expression language *)

nth-expr        ::= nth-term (ws* ',' ws* nth-term)*

nth-term        ::= range
                  | step
                  | integer

range           ::= integer ws* '...' ws* integer

step            ::= coefficient 'n' (ws* ('+' | '-') ws* pos-integer)?

coefficient     ::= '-'? pos-integer
                  | '-'

integer         ::= '-'? pos-integer
                  | '0'

pos-integer     ::= [1-9] [0-9]*


(* Identifiers *)

identifier      ::= [a-zA-Z_] [a-zA-Z0-9_]*
```

### Constraints

The following are semantic constraints enforced after parsing:

- `0` or `-0` in `nth-expr` — warned and ignored
- `0n` in `nth-expr` — hard error
- `+n` in `nth-expr` — warned, treated as `n`
- Range bounds in `nth-expr` must have the same sign — hard error with suggestion
- `b` in `an+b` or `an-b` must be non-zero — hard error
- `+` on non-boundary patterns (`at`, `after`, `before`) — warned and ignored
- `qed:file()` in `after` or `before` pipeline — warned and ignored
- Mixing point params (`at`, `after`, `before`) and range params (`from`, `to`) in `qed:copy` or `qed:move` — hard error
- Omitting all destination params in `qed:copy` or `qed:move` — hard error
- `${VAR}` referencing unset env var — warned, expanded to empty string
- `\` outside external processor expression — hard error
- Trailing whitespace after `\` in external processor — hard error
- Overlapping regions — last-declaration-wins, no warning
- Named pattern redefined — warned, last definition wins

---

## Changelog

### [next]

- **Specified named pattern redefinition behavior** — redefining a named pattern
  emits a warning; the last definition wins.
  Forward references (referencing a pattern before its definition) are permitted.
  Both rules added to the Pattern Syntax section and the Constraints list.
- **Removed `mode` parameter** from all selectors.
  Line mode is the only mode — selectors always return whole lines.
  Multiline region selection continues to be handled by `from > to`.
- **Removed the `@` sigil** from pattern references.
  Strings (`"…"`), regexes (`/…/`), and bare identifiers are self-describing;
  the sigil was redundant.
- **Added `qed:substring(pattern)`** processor.
  Narrows the selected region to the matched span, discarding the rest of the line.
  Replaces the primary use case of `mode:substring`.
- **Extended `qed:replace()`** to accept an operator pipeline as the replacement argument.
  Handles inline generation and transformation without requiring generation processors
  to manage pattern matching themselves.
- **Added regex template form `/…/`** for replacement arguments in `qed:replace()`.
  Establishes full symmetry between pattern and replacement sigils:
  `"…"` is always literal, `/…/` is always regex-aware.
- **Simplified generation processors** (`qed:uuid()`, `qed:timestamp()`, `qed:random()`).
  Pattern matching is no longer their responsibility — delegate to `qed:replace()`.
- **Clarified `nth` default behaviour** — omitting `nth` matches all occurrences,
  equivalent to `nth:1n`. Updated `nth:1` table comment to remove the misleading
  "default" label.
- **Documented `${QED_FILE}` environment variable** — when `qed:file()` is in the
  pipeline, qed injects the temp file path as `${QED_FILE}` into the downstream
  external command's environment. Added usage examples and scope rules.
- **Specified `--dry-run` output format** — unified diff on stdout, with fixed
  `a` / `b` placeholders in `---`/`+++` headers and timestamps omitted.
  Composable with diff viewers such as `delta`.
- **Specified `qed:number()` params** — colon-space separator, minimal padding by default.
  Optional `width` param for fixed-width right-alignment.
  Optional `start` param to set origin (defaults to stream line number; `start:1` gives region-relative numbering).
- **Confirmed `after` / `before` | generation-processor** — generation processors used
  directly in `after` or `before` pipelines insert their output as a new line at the
  cursor position. Both forms (`qed:replace()` substitution and direct insertion) are valid.
- **Added Diagnostics section** — specifies the full stderr diagnostic format:
  `qed: <severity>: <location>: <source-expression>: <message>`.
  Severity padded to `warning:` width; location padded to widest span in script;
  source expression and message unpadded. 1-based line and byte offsets.
  One diagnostic per event, no end-of-run summary.
  `--dry-run` confirmed to use 3 lines of context (standard default).
- **Documented output-on-failure behaviour** — qed emits lines as soon as all statements
  are done with them, freeing fragments as it goes. Documents `set -o pipefail` recommendation
  when piping qed output. `--on-error=skip/warn` users accept downstream responsibility.
