# `qed` Implementation Roadmap

Sequenced build plan for the `qed` implementation.
Phases are ordered to maximize early feedback, keep the codebase stable at
every milestone, and use the test harness as the primary development signal
from as early as possible.

---

## Guiding Principles

**AST and fragment types first.**
Both the parser and the compiler build against these types.
Defining them before either layer exists prevents churn caused by one layer
making assumptions the other has to work around.

**Parser POC before full parser work.**
The recursive descent and chumsky spikes are evaluated against a representative
grammar production before the full parser is built.
Building the full parser once against a decided approach is cleaner than
restructuring a partial parser mid-way.

**Walking skeleton early.**
A minimal end-to-end path — one selector, one processor, stdin to stdout —
is established as soon as the core types and parser approach are settled.
The harness is wired up at the same time, so the first test going green
is the signal that the skeleton works.

**Harness-driven development from the skeleton onward.**
Every subsequent feature is driven by test scenarios going from red to green.
The golden files are already written — use them.

---

## Phase 0 — Workspace Scaffold

**Goal:** `cargo build --workspace` succeeds with empty stubs.
No logic yet — just structure.

- Create the Cargo workspace with `qed-core`, `qed`, and `qed-tests` crates
- Add `Cargo.toml` feature flags for `parser-rd` (default) and `parser-chumsky`
- Create stub `lib.rs` for `qed-core` with empty module declarations
- Create stub `main.rs` for `qed` that prints `"qed"` and exits
- Create stub `main.rs` for `qed-tests`
- Verify `cargo build --workspace` and `cargo clippy --workspace` are clean

**Checkpoint:** the workspace builds cleanly with both feature flag configurations.

---

## Phase 1 — Core Types

**Goal:** the types that every other phase builds against exist and are stable.
No parser, no compiler, no executor yet — just type definitions and their unit tests.

### `span`

- `Span { start: usize, end: usize }`
- `Spanned<T> { node: T, span: Span }`

### `parse/ast`

The complete AST type tree.
No parsing logic — just the types the parser will produce.

- `Program`, `Statement`, `SelectActionNode`
- `Selector`, `SimpleSelector`, `SelectorOp`
- `PatternValue`, `PatternRef`, `PatternRefValue`
- `ProcessorChain`, `Processor`, `QedProcessor`, `ExternalProcessor`
- `QedArg`, `ExternalArg`
- `Fallback`
- `Param`, `ParamValue`
- `NthExpr`, `NthTerm`

### Identity newtypes

- `StatementId(usize)`, `SelectorId(usize)`

### `exec` — buffer and fragment model

- `Buffer { content: String, line_offsets: Vec<usize> }` with constructor and `slice(LineRange) -> &str`
- `LineRange { start: usize, end: usize }`
- `FragmentContent` — `Borrowed(LineRange)` / `Owned(String)`
- `Fragment` — `Passthrough(FragmentContent)` / `Selected { content, tags }`
- `FragmentList` type alias

### `compile` — IR types

- `Script { statements: Vec<Statement>, selectors: Vec<RegistryEntry> }`
- `Statement { id, selector, processor, fallback }`
- `RegistryEntry` — `Simple(CompiledSelector)` / `Compound(CompoundSelector)`
- `CompiledSelector`, `CompoundSelector`
- `SelectorOp` with per-variant fields
- `CompiledPattern { matcher, negated, inclusive }`
- `PatternMatcher` — `Literal(String)` / `Regex(regex::Regex)`
- `OnError` enum

### `processor` — trait and error type

- `Processor` trait: `fn execute(&self, input: String) -> Result<String, ProcessorError>`
- `ProcessorError` enum — `NoMatch`, `ProcessorFailed`, `ExternalFailed`

### `error`

- `CompileError` enum with all variants from the implementation design
- `SymbolKind` enum

**Checkpoint:** `cargo test --workspace` passes with unit tests covering the
`Buffer` constructor and slice, `FragmentContent` variants, and newtype accessors.

---

## Phase 2 — Fragmentation Algorithm

**Goal:** the fragmentation algorithm is fully implemented and unit-tested,
independent of the parser and compiler.

The algorithm takes a `&Buffer` and a set of `(StatementId, SelectorId,
&CompiledSelector)` pairs and produces a `FragmentList`.

- Parallel match collection using `rayon`
- Boundary event decomposition (`Start` / `End` events)
- Sort (line ascending, Start before End, StatementId ascending)
- Sweep with `BTreeSet` active tag set producing the `FragmentList`
- `inclusive` boundary logic per `CompiledPattern`
- `nth` filtering on match results

Unit tests cover:

- Single selector, single match → one `Selected` fragment flanked by `Passthrough`
- Single selector, no match → all `Passthrough`
- Two overlapping selectors → multi-tagged `Selected` fragment
- `nth:2` → only second match selected
- `from > to` compound → correct inclusive/exclusive boundary variants
- Negated pattern → lines not matching are selected

**Checkpoint:** all fragmentation unit tests pass.
The algorithm compiles and is correct in isolation before any selector matching
logic exists in the compiler or executor.

---

## Phase 3 — Parser POC Evaluation

**Goal:** pick the parser approach and delete the loser.
This phase produces a decision and a skeleton, not a complete parser.

### Spike target

Both implementations spike the `nth` expression grammar production:

```ebnf
nth-expr = nth-term ("," nth-term)*
nth-term = integer | range | step
range    = integer "..." integer
step     = integer? "n" ("+" integer | "-" integer)?
```

This is the most syntactically complex sub-grammar in `qed`.
If either approach struggles here, it will struggle more on the full grammar.

### Recursive descent spike (`parse/rd/`)

- `Lexer` with `Cursor` over the source `&str`
- `parse_nth_expr()` → `NthExpr`
- Error type returning `Span`-bearing parse errors
- Unit tests: valid forms, malformed input, error span accuracy

### Chumsky spike (`parse/chumsky/`)

- Token enum and lexer combinator
- `nth_expr()` parser combinator → `NthExpr`
- Error recovery behavior with `ariadne`-rendered output
- Unit tests: same cases as the recursive descent spike

### Evaluation criteria

| Criterion | What to check |
|---|---|
| Error quality | Are messages clear and actionable for a `qed` user? |
| Span accuracy | Do error spans point at the right token? |
| Grammar coverage | Does it handle all `nth` forms without workarounds? |
| Debuggability | How hard was it to trace failures during the spike? |
| Compile time | How much does `chumsky` add to incremental build time? |

### Outcome

- Document the evaluation result as a short entry in `qed-project-structure.md`
- Delete the losing implementation directory
- Remove the losing feature flag from `qed-core/Cargo.toml`
- Simplify `parse/mod.rs` routing

**Checkpoint:** one parser directory remains, feature flag routing is removed,
and `cargo build --workspace` is clean.

---

## Phase 4 — Walking Skeleton + Harness

**Goal:** one test scenario passes end-to-end: `selectors::at-literal::0`.

This is the most important milestone in the project.
Every component touches every other at this phase.

### Minimal parser

Parse exactly one form: `at("literal") | qed:delete()`.
Hard-code assumptions where needed — this will be replaced in Phase 5.

- Token types and lexer for the subset
- `parse_program()` → `Program` for the one supported form
- Error type stub returning a `Vec<ParseError>`

### Minimal compiler

Compile the one AST form to a `Script`.

- `SymbolTable` construction (empty — no named patterns yet)
- Compile `at(string-literal)` → `CompiledSelector` with `PatternMatcher::Literal`
- Compile `qed:delete()` → `Box<dyn Processor>`

### `qed:delete()` processor

```rust
struct DeleteProcessor;
impl Processor for DeleteProcessor {
    fn execute(&self, _input: String) -> Result<String, ProcessorError> {
        Ok(String::new())
    }
}
```

### Execution engine

Wire the compiler output through the fragmentation algorithm to output.

- `Engine::run(script: &Script, buffer: &Buffer) -> Result<String, ProcessorError>`
- Collect fragments, route selected regions through their processors, join and emit

### CLI scaffolding

- `clap` `Cli` struct with the full flag set (even flags not yet implemented can exist as stubs)
- Read script from first positional argument
- Read input from stdin (file support deferred)
- Write output to stdout
- Wire: parse CLI → read input → parse script → compile → execute → print output

### Test harness — Rust layer

- Manifest `[[scenario]]` parsing with `toml`
- `scenario.sh` generation for a single invocation
- `Trial` registration with `libtest-mimic`
- Temp directory lifecycle (create before, remove after)
- `bash run-scenario.sh <tmpdir>` invocation and pass/fail capture

### Test harness — bash layer

- `run-scenario.sh` — sources `scenario.sh`, sets up files, runs invocation, calls comparison
- `compare-golden.sh` — `.txt` exact match, `.pattern` full-string regex, `.*` glob
- No mock support yet

**Checkpoint:** `cargo test --package qed-tests selectors::at-literal::0` passes.

---

## Phase 5 — Full Parser

**Goal:** the parser handles the complete `qed` grammar.

Build out the parser chosen in Phase 3 to cover every grammar production.
Work through productions roughly in dependency order:

1. Patterns — string literals, regex literals, negation, `+` suffix
2. Selectors — `at`, `after`, `before`, `from`, `to`, compound `from > to`
3. `nth` expression — all forms (reuse the spike implementation)
4. Params — named params with typed values
5. Processors — `qed:*` internal processors, external processors, chains
6. Statements — `PatternDef`, `AliasDef`, `SelectAction`
7. Fallback — `||` with chain and nested select-action forms
8. Shebang line
9. Line continuation — `|`, `,`, `>` at end of line

Unit test each production.
All parse errors must carry accurate `Span` values.

**Checkpoint:** the parser unit test suite passes for all grammar productions.
The harness `selectors` suite begins going green as selector forms are added.

---

## Phase 6 — Full Compiler

**Goal:** the compilation pass handles all AST forms.

- Phase 1 (symbol collection) — `PatternDef`, `AliasDef`, duplicate name warnings
- Phase 2 (compilation) — all selector ops, all processor types, param validation,
  regex compilation, env var expansion, `nth` expression compilation
- `CompileError` emission for all error variants
- Warning emission to stderr during compilation

**Checkpoint:** the compilation pass unit tests pass.
`selectors` integration suite is fully green.

---

## Phase 7 — Processor Coverage

**Goal:** all `qed:*` processors are implemented.

Work through processors roughly in order of increasing complexity:

| Processor | Notes |
|---|---|
| `qed:delete()` | Done in Phase 4 |
| `qed:upper()`, `qed:lower()` | Trivial transforms |
| `qed:prepend()`, `qed:append()` | String concatenation |
| `qed:number()` | Line numbering with `width` and `start` params |
| `qed:substring()` | Pattern matching, narrowing semantics |
| `qed:replace()` | Literal, regex template, and pipeline forms |
| External processors | Spawn, stdin/stdout, non-zero exit handling |
| `ProcessorChain` | `try_fold` composition |
| Fallback (`\|\|`) | Invoke fallback on processor error |

Add mock support to the harness bash layer when external processors are reached.

**Checkpoint:** `processors` and `external-processors` integration suites are green.

---

## Phase 8 — Generation Processors

**Goal:** `qed:uuid()`, `qed:timestamp()`, and `qed:random()` are implemented.

- `qed:uuid()` — v4 (random), v5 (namespace + name), v7 (time-ordered)
- `qed:timestamp()` — ISO 8601, unix epoch, custom format, timezone
- `qed:random()` — configurable alphabet and length

These processors ignore stdin entirely.
They compose with `qed:replace()` for substitution and with `after`/`before` for insertion.

**Checkpoint:** `generation` integration suite is green.
`.pattern` golden matching is exercised here — verify harness handles it correctly.

---

## Phase 9 — Invocation Features

**Goal:** all CLI flags are fully implemented.

| Feature | Notes |
|---|---|
| `-f` / `--file` | Read script from file instead of inline argument |
| `--in-place` | Atomic write via temp file + rename |
| `--extract` | Suppress passthrough output |
| `--output` | Write to file instead of stdout |
| `--dry-run` | Unified diff output, 3 context lines, `a`/`b` placeholders |
| `--on-error` | `fail` / `warn` / `skip` routing |
| `--no-env` | Disable env var expansion in patterns and args |

**Checkpoint:** `invocation`, `stream-control`, and `script-files` integration
suites are green.

---

## Phase 10 — Diagnostics

**Goal:** all diagnostic output matches the confirmed format.

- Diagnostic formatter: `qed: <severity>: <location>: <source>: <message>`
- Severity padding to `warning:` width
- Location padding to widest span in script (computed from AST pre-execution)
- Warning emission for: `+` on `at`/`after`/`before`, `nth:0`, duplicate `nth`
  values, unset env vars, duplicate pattern names, `qed:file()` on insertion point
- `qed:debug:count()` processor

**Checkpoint:** all warning scenarios in edge-case files are green.
`error-handling` and `error-handling-edge-cases` suites are green.

---

## Phase 11 — Edge Cases and Use Cases

**Goal:** the full test suite is green.

Work through the edge-case scenario files:

- `selectors-edge-cases`
- `processors-edge-cases`
- `patterns-edge-cases`
- `external-processors-edge-cases`
- `invocation-edge-cases`
- `error-handling-edge-cases`
- `script-files-edge-cases`
- `generation-edge-cases`
- `stream-control` (if not already green)

Then the use case suites:

- `usecases/code-editing`
- `usecases/config-manipulation`
- `usecases/log-processing`
- `usecases/code-generation`
- `usecases/template-rendering`
- `usecases/document-processing`
- `usecases/editor-integration`

**Checkpoint:** `cargo test --workspace` is fully green.

---

## Phase 12 — Release Polish

**Goal:** the project is ready for a first public release.

- Shell completions via `clap_complete` (bash, zsh, fish)
- `--version` flag wired to `Cargo.toml` version
- README review — verify all examples work against the final implementation
- `cargo clippy --workspace -- -D warnings` is clean
- `cargo fmt --check` passes
- Final README pass: installation instructions, quick reference, comparison table

---

## Summary

| Phase | Deliverable | Key checkpoint |
|---|---|---|
| 0 | Workspace scaffold | `cargo build --workspace` clean |
| 1 | Core types | Buffer and fragment unit tests pass |
| 2 | Fragmentation algorithm | All fragmentation unit tests pass |
| 3 | Parser POC evaluation | One parser remains, routing removed |
| 4 | Walking skeleton + harness | `selectors::at-literal::0` green |
| 5 | Full parser | All grammar productions parsed and unit-tested |
| 6 | Full compiler | `selectors` integration suite green |
| 7 | Processor coverage | `processors` + `external-processors` suites green |
| 8 | Generation processors | `generation` suite green |
| 9 | Invocation features | `invocation` + `stream-control` suites green |
| 10 | Diagnostics | `error-handling` suites green |
| 11 | Edge cases + use cases | `cargo test --workspace` fully green |
| 12 | Release polish | Completions, README, clippy clean |
