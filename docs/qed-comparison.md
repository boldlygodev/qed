# qed Compared to Existing Tools

`qed` occupies a specific position in the text processing landscape — more structured than `sed`,
more composable than `awk`, less general than `perl`, and more capable than modern single-purpose
tools like `sd`. This document compares `qed` against the most common alternatives.

---

## Feature Matrix

| Feature | sed | awk | vim -es | perl | sd | qed |
|---|---|---|---|---|---|---|
| Stream processing | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Line-oriented default | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| RE2 / linear-time regex | ✗ | ✗ | ✗ | ✗ | ✓ | ✓ |
| Regex backreferences | ✓ | ✗ | ✓ | ✓ | ✓ | ✗ |
| Named capture groups | ✗ | ✗ | ✗ | ✓ | ✓ | ✓ |
| Region / range selection | ~ | ~ | ✓ | ~ | ✗ | ✓ |
| nth occurrence selection | ~ | ~ | ✓ | ~ | ✗ | ✓ |
| Named patterns | ✗ | ~ | ~ | ~ | ✗ | ✓ |
| Before / after insertion | ✓ | ~ | ✓ | ~ | ✗ | ✓ |
| Start / end insertion points | ✗ | ~ | ✓ | ~ | ✗ | ✓ |
| Inline substring replacement | ✗ | ~ | ✓ | ✓ | ✓ | ✓ |
| Inline substring extraction | ✗ | ~ | ✓ | ✓ | ✓ | ✓ |
| External tool integration | ✗ | ~ | ✓ | ✓ | ✗ | ✓ |
| Processor pipeline as replacement | ✗ | ✗ | ✗ | ~ | ✗ | ✓ |
| Processor aliases | ✗ | ~ | ✗ | ✓ | ✗ | ✓ |
| Script files | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ |
| Shebang support | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ |
| Built-in text transforms | ~ | ~ | ✓ | ✓ | ✗ | ✓ |
| Region move / copy | ✗ | ✗ | ✓ | ✗ | ✗ | ✓ |
| Generation (uuid, timestamp, random) | ✗ | ~ | ✗ | ~ | ✗ | ✓ |
| Structured error handling | ✗ | ✗ | ✗ | ~ | ✗ | ✓ |
| Fallback statements | ✗ | ✗ | ✗ | ~ | ✗ | ✓ |
| Environment variable expansion | ~ | ~ | ✗ | ✓ | ✗ | ✓ |
| In-place file editing (`$EDITOR`) | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ |
| Dry-run / diff preview | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ |
| Shell-friendly syntax | ~ | ~ | ✗ | ✗ | ✓ | ✓ |
| `//go:generate` compatible | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ |

✓ supported  ~ partial or requires workaround  ✗ not supported

---

## Tool-by-Tool Comparison

### sed

`sed` is `qed`'s closest ancestor and the most direct comparison.
Both are stream editors that process input line by line and write to stdout.
The core workflow — match a pattern, apply a transformation — is identical.

Where they diverge is in expressiveness and ergonomics. `sed`'s address syntax (`/pat/,/pat/`)
provides basic range selection but has no concept of occurrence counting, named patterns,
or nested region composition. Writing a `sed` script that operates on every third occurrence
of a pattern requires awkward counter manipulation with hold space and branch commands.
`qed` expresses this directly with `nth:3n`.

`sed` uses POSIX BRE or ERE depending on the implementation and flag, with inconsistent
behavior across GNU sed and BSD sed (macOS). This is a persistent portability headache
for shell scripts. `qed` uses RE2 semantics everywhere, with consistent behavior guaranteed.

`sed` has no concept of processor pipelines — the only way to pass a region through an
external tool is to shell out entirely. `qed`'s `| command` handoff model makes this a
first-class operation.

The syntax of `sed` is famously terse to the point of obscurity. Commands like
`sed -n '/start/,/end/{/start/d;/end/d;p}'` are correct but nearly unreadable.
`qed` trades terseness for clarity without sacrificing composability.

---

### awk

`awk` is a more powerful tool than `sed` — it has variables, arithmetic, arrays, and
user-defined functions. It excels at structured text like TSV and CSV where field-based
processing is natural.

For unstructured text transformation, `awk`'s power becomes noise. The `BEGIN`/`END`/pattern
rule model is flexible but requires the user to manage state explicitly for anything beyond
per-line transforms. Range selection requires tracking state with boolean variables.
Occurrence counting requires a counter that the user increments and tests manually.

`qed` handles all of this structurally — `nth`, `from > to`, and selector composition
replace the manual state management that `awk` requires. For text transformation tasks,
`qed` scripts are significantly shorter and more readable than equivalent `awk` programs.

Where `awk` wins is arithmetic, field processing, and aggregation. These are not `qed`'s
domain — `qed` defers to `awk`, `cut`, `jq`, and similar tools via the processor pipeline
rather than trying to subsume them.

---

### vim -es

`vim -es` (ex mode, silent) is `qed`'s most capable predecessor.
It supports region selection, nth occurrence, external tool integration, move and copy operations,
and a rich set of built-in transforms. On paper it can do nearly everything `qed` can.

In practice, `vim -es` is cumbersome as a scripting tool. Its syntax is inherited from an
interactive editor — designed for human typing, not programmatic generation. Quoting `vim`
commands for shell invocation is notoriously difficult. The ex command language has no
concept of named patterns, aliases, or structured error handling. The learning curve assumes
familiarity with vim's modal editing model, which is a significant barrier for users who want
a scripting tool without the editor background.

`qed` takes the capabilities of `vim -es` and rebuilds them with a syntax designed from the
start for shell scripting — labeled parameters, explicit quoting rules, formal grammar, and
composability with pipelines.

---

### perl

`perl -pe` and `perl -ne` are the classic escape hatch when `sed` and `awk` are insufficient.
Perl has full PCRE regex including backreferences and lookaheads, arbitrary scripting power,
and a rich module ecosystem.

The cost is Perl itself. Writing correct, readable Perl for text transformation requires
fluency in a general-purpose language with complex scoping, context sensitivity, and
decades of accumulated idioms. For simple transformations it's overkill; for complex ones
it becomes a maintenance burden.

`qed` covers the common cases that Perl is reached for — region selection, occurrence
counting, insertion, and external tool integration — with a purpose-built syntax.
When a transformation genuinely requires Perl's power (lookaheads, complex logic,
module ecosystem), `qed` integrates Perl as an external processor:

```
at(/pattern/) | perl -pe 's/complex/transformation/g'
```

`qed` does not try to replace Perl for the cases where Perl is genuinely the right tool.

---

### sd

`sd` is a modern `sed` replacement implemented in Rust, with an ergonomic syntax and
RE2 semantics via the same `regex` crate `qed` uses. It is excellent at its narrow scope:
find-and-replace across a stream or set of files.

`sd` has no concept of region selection, occurrence targeting, before/after insertion,
external tool integration, or script files. It is intentionally minimal — a better `sed s///`
and nothing more.

`qed` and `sd` are not really in competition. `sd` is the right tool when you need a clean,
portable substitution and nothing else. `qed` is the right tool when the transformation
requires selection, composition, or any of the capabilities `sd` deliberately omits.
They compose naturally in a pipeline:

```sh
qed 'from(/^## /+) > to(/^## /)' doc.md | sd 'foo' 'bar'
```

---

## Summary

`qed` was conceived as an answer to the `$EDITOR` problem — the gap between line editors
like `ed` and full terminal editors like `vim`, specifically for scripting and automation.
Programs that invoke `$EDITOR` or `$VISUAL` expect a tool that can transform a file
non-interactively. `qed` fills that role with a design built from the ground up for
scripting rather than adapted from an interactive editor:

```sh
export EDITOR="qed --in-place -f ~/.config/qed/transform.qed"
```

More broadly, `qed` is not trying to replace any single existing tool — it fills the gap
between them. `sed` and `sd` handle simple substitutions. `awk` handles structured data.
`perl` handles complex transformations. `vim -es` handles interactive editing workflows.

`qed` handles the space in between: structured text transformations that require region
selection, composition, and integration with the Unix tool ecosystem, expressed in a syntax
designed for shell scripting rather than adapted from an interactive editor.

The features most absent from existing tools — structured error handling with fallback
statements, nth occurrence selection, named patterns, processor aliases, generation
functions, processor pipelines as replacement arguments, and dry-run preview — reflect
the accumulated workarounds that scripters reach for repeatedly when existing tools fall short.
