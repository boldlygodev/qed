# `qed` Implementation Roadmap

Sequenced build plan for the `qed` implementation.
Phases are ordered to maximize early feedback, keep the codebase stable at
every milestone, and use the test harness as the primary development signal
from as early as possible.

---

## Guiding Principles

**Type-first where it matters; harness-first for feedback.**
The harness is independent infrastructure with no dependency on implementation
types — it can be built and run (with failing tests) before any code exists.
Shift it to Phase 1 so the test signal is available from the start of real work.
Core types still precede their consumers (parser before executor), but type
definitions are incremental rather than exhaustive upfront.

**Parser POC before full parser work.**
The recursive descent and chumsky spikes are evaluated against a representative
grammar production before the full parser is built.
Building the full parser once against a decided approach is cleaner than
restructuring a partial parser mid-way.

**Walking skeleton early.**
A minimal end-to-end path — one selector, one processor, stdin to stdout —
is established as soon as the core types and parser approach are settled.
The harness is already wired from Phase 1, so the first test going green
is the signal that the skeleton works.

**Integration tests as the primary signal from Phase 4 onward.**
Every feature beyond the skeleton is driven by test scenarios going from red to green.
The golden files are already written — integration tests become the specification
as implementation proceeds.

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

## Phase 1 — Test Harness Infrastructure

**Goal:** the integration test harness is built and ready to register failing tests
before any implementation exists.

The harness has **zero dependency on `qed-core` implementation types** at compile time.
Only `libtest-mimic` and `toml` are required.
Trials can register and fail gracefully at `eval "$INVOCATION"` until the CLI works.

### Test harness — Rust layer

- Manifest `[[scenario]]` parsing with `toml`
- `scenario.sh` generation for a single invocation
- `Trial` registration with `libtest-mimic`
- Temp directory lifecycle (create before, remove after)
- `bash run-scenario.sh <tmpdir>` invocation and pass/fail capture
- Trial naming convention: `<suite>::<scenario-id>::<invocation-index>`

### Test harness — bash layer

- `run-scenario.sh` — sources `scenario.sh`, sets up files, runs invocation, calls comparison
- `compare-golden.sh` — `.txt` exact match, `.pattern` full-string regex, `.*` glob
- No mock support yet (added in Phase 7)

### Test scenario files

Read and validate all scenario manifests in `.claude/tests/`:
- `selectors.md`, `processors.md`, `patterns.md`, `invocation.md`, `error-handling.md`,
  `generation.md`, `stream-control.md`, `external-processors.md`, `script-files.md`
- And their corresponding `-edge-cases.md` variants, plus `usecases.md`

**Checkpoint:** `cargo test --package qed-tests` runs and registers all trials.
No trials pass yet — invocations fail at `eval "$INVOCATION"` because `qed` doesn't exist.
But the harness itself is correct and ready to drive implementation from here forward.

---

## Phase 2 — Core Types and Fragmentation Algorithm

**Goal:** define the types that the parser, compiler, and executor build against.
Implement the fragmentation algorithm as a unit-tested component independent
of parser and compiler logic.

### Core Types

Define only what the parser (Phase 3) and executor need. Later phases add variants
as features are implemented. Type definitions are stable but not exhaustive.

#### `span`

- `Span { start: usize, end: usize }`
- `Spanned<T> { node: T, span: Span }`

#### `parse/ast` — Parser output

- `Program`, `Statement`, `SelectActionNode`
- `Selector`, `SimpleSelector`, `SelectorOp`
- `PatternValue`, `PatternRef`, `PatternRefValue`
- `ProcessorChain`, `Processor`, `QedProcessor`, `ExternalProcessor`
- `QedArg`, `ExternalArg`
- `Fallback`
- `Param`, `ParamValue`
- `NthExpr`, `NthTerm`

#### Identity newtypes

- `StatementId(usize)`, `SelectorId(usize)` (never pass raw `usize` to functions expecting these)

#### `exec` — buffer and fragment model

- `Buffer { content: String, line_offsets: Vec<usize> }` with constructor and `slice(LineRange) -> &str`
- `LineRange { start: usize, end: usize }`
- `FragmentContent` — `Borrowed(LineRange)` / `Owned(String)`
- `Fragment` — `Passthrough(FragmentContent)` / `Selected { content, tags }`
- `FragmentList` type alias

#### `compile` — IR types (interpreter output)

- `Script { statements: Vec<Statement>, selectors: Vec<RegistryEntry> }`
- `Statement { id, selector, processor, fallback }`
- `RegistryEntry` — `Simple(CompiledSelector)` / `Compound(CompoundSelector)`
- `CompiledSelector`, `CompoundSelector`
- `SelectorOp` with per-variant fields
- `CompiledPattern { matcher, negated, inclusive }`
- `PatternMatcher` — `Literal(String)` / `Regex(regex::Regex)`
- `OnError` enum

#### `processor` — trait and error type

- `Processor` trait: `fn execute(&self, input: String) -> Result<String, ProcessorError>`
- `ProcessorError` enum — `NoMatch`, `ProcessorFailed`, `ExternalFailed`

#### `error`

- `CompileError` enum with all variants from the implementation design
- `SymbolKind` enum

### Fragmentation Algorithm

Implement the algorithm that takes a `&Buffer` and selector matches and produces
a `FragmentList`, ready for processor dispatch.

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

**Checkpoint:** `cargo test --workspace` passes with unit tests covering the
`Buffer` constructor and slice, `FragmentContent` variants, newtype accessors,
and all fragmentation edge cases.
The algorithm is correct in isolation before any selector matching logic
exists in the compiler.

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

## Phase 4 — Walking Skeleton

**Goal:** one test scenario passes end-to-end: `selectors::at-literal-single-match::0`.

This is the most important milestone in the project.
Every component touches every other at this phase.
The harness from Phase 1 is already ready — this phase wires the implementation
to make the first test green.

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

**Checkpoint:** `cargo test --package qed-tests selectors::at-literal-single-match::0` passes.
The harness is now driving implementation and will remain the primary signal
through all remaining phases.

---

## Phase 5 — Full Parser

**Goal:** the parser handles the complete `qed` grammar and drives integration
tests to green as new productions are added.

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

Most Phase 6 work was completed during sub-phases 5B–5D: two-pass symbol collection,
selector ops, nth expression compilation, regex compilation, processor chain composition.
The original checkpoint (`selectors` suite fully green, 46/46) is already achieved.
Remaining work is broken into four sub-phases, with `qed:replace()` and external
processor execution pulled forward from Phase 7 to reach the Alpha 1 milestone.

Sub-phases 6A, 6B, and 6C are complete. 169/396 integration tests pass.

### 6A — Env var expansion

- `expand_env_vars()` function: `$IDENT`, `${IDENT}`, `$$` escape
- Wire into pattern compilation and processor string args
- Thread `no_env: bool` through `compile()` (hardcode `false`; CLI wiring in Phase 9)

### 6B — Compiler warnings & validation

- Duplicate name detection in pass 1 → warning (last definition wins)
- Param validation: unknown param names, wrong param types
- `compile()` returns `(Script, Vec<CompileWarning>)`
- Warning emission infrastructure: `run()` formats and writes to stderr
- `CompileError` variant coverage audit

### 6C — Replace processor

- `qed:replace("old", "new")` — literal replacement
- `qed:replace(/pattern/, "template")` — regex with capture groups
- `qed:replace("match", qed:upper())` — pipeline (run processor on matched text)

### 6D — External processor execution

- Complete `ExternalCommandProcessor`: stdin piping, stdout capture, arg passing
- Non-zero exit → `ProcessorError::ExternalFailed`
- Mock script support in test harness

**Checkpoint:** `patterns::env-*` green; `processors::replace-*` green;
basic `external-processors::*` scenarios green. 198/396 integration tests pass.
46/46 selector tests pass. 21/27 external processor tests pass
(6 deferred to Phase 8 for `qed:file()`).

### ✦ Alpha 1 — Basic Stream Editing

Alpha 1 is reached after Phase 6D. qed is usable for common stream editing tasks:
all selectors, core processors (delete, upper, lower, prefix, replace), external
commands, named patterns and aliases, script files, fallback, env var expansion.

---

## Phase 7 — Processor Coverage

**Goal:** all remaining `qed:*` processors are implemented.

### 7A — Simple processors

- `qed:suffix(text:"...")` — append text to each line
- `qed:duplicate()` — emit region twice
- `qed:skip()` — no-op passthrough
- `qed:trim()` — strip leading/trailing whitespace per line
- Add `map_lines()` per-line utility in `processor/mod.rs`

### 7B — Parameterized processors ✓

- `qed:number(start:N, width:N)` — line numbering with alignment ✓
- `qed:indent(width:N, char:"...")` — prepend indentation per line ✓
- `qed:dedent()` — remove common leading whitespace ✓
- `qed:wrap(width:N)` — word-wrap at column width ✓

### 7C — Pattern-based processor ✓

- `qed:substring(pattern)` — narrow each line to matched span ✓

### 7D — Copy and move ✓

- `qed:copy(after:p | before:p | at:p)` — copy region to destination ✓
- `qed:move(after:p | before:p | at:p)` — move region to destination ✓
- `StatementAction` enum and execution engine post-processing pass ✓

### 7E — Test verification and edge cases ✓

- Full `processors` suite validation ✓ (47/47)
- All edge case fixtures present and passing ✓
- Fix: parser supports `at(on_error:skip)` params-only selectors ✓
- Fix: `NumberProcessor` empty input returns empty ✓

**Checkpoint:** `processors` and `processors-edge-cases` suites green.
21/27 `external-processors` green (6 deferred to Phase 8 for `qed:file()`).

### ✦ Alpha 2 — Full Processor Suite

Alpha 2 is reached after Phase 7. All text transformation processors work.
248/396 integration tests pass.

---

## Phase 8 — Generation Processors

**Goal:** `qed:uuid()`, `qed:timestamp()`, and `qed:random()` are implemented.
`qed:file()` deferred to Phase 9 or 11.

These processors ignore stdin entirely.
They compose with `qed:replace()` for substitution and with `after`/`before` for insertion.

### 8A — Harness `.pattern` multiline fix ✓

- Updated `compare-golden.sh`: switched from `grep -qE` to bash `=~` for pattern matching
- Resolves literal `\n` in `.pattern` files to actual newlines before matching
- Bash `=~` handles multiline content natively; no regressions in existing tests

### 8B — `qed:random()` ✓

- `qed:random(N)` — default numeric, configurable alphabet and length
- Named alphabets: `numeric`, `alpha`, `alnum`, `hex`, `upper`, `base32`, `crockford`,
  `bech32`, `base58`, `base62`, `base64url`, `ascii`, `symbol`, plus custom strings
- Confirmed: generation processors work directly in `after`/`before` pipelines

### 8C — `qed:uuid()` ✓

- `qed:uuid()` — v7 (default, time-ordered), v4 (random), v5 (deterministic namespace + name)
- UUID v5 golden verified (`4fd35a71-71ef-5a55-a9d9-aa75c889a6d0` for `NAMESPACE_URL` + `https://example.com`)
- Simplified `uuid-v7-after.qed` to direct `after("header") | qed:uuid()` form
- Fixed harness `\n` resolution (pre-expanded newline variable instead of inline `$'\n'`)
- Generation processors now append trailing `\n` for proper line termination in `before`/`after` pipelines

### 8D — `qed:timestamp()` ✓

- `qed:timestamp()` — ISO 8601 (default), unix epoch, unix_ms, date, time, datetime, custom LDML format
- Timezone support: UTC (default), IANA names (`"America/New_York"`), fixed offsets (`"UTC+5:30"`)
- LDML-to-strftime translation: `yyyy`→`%Y`, `MM`→`%m`, `dd`→`%d`, `HH`→`%H`, `mm`→`%M`, `ss`→`%S`

**Checkpoint:** `generation` and `generation-edge-cases` suites green. 298/396 integration tests pass.
`qed:file()` deferred to Phase 9 or 11.

---

## Phase 9 — Invocation Features

**Goal:** all CLI flags are fully implemented.

| Feature | Notes |
|---|---|
| Input file positional | Read from file instead of stdin; stdin fallback when absent |
| `--extract` (`-x`) | Suppress passthrough output; only selected regions emitted |
| `--no-env` | Disable env var expansion in patterns and args |
| `--on-error` | Global default `fail`/`warn`/`skip`; per-selector overrides |
| `--output` (`-o`) | Write to file instead of stdout |
| `--in-place` (`-i`) | Atomic write via temp file + rename |
| `--dry-run` (`-d`) | Unified diff output, 3 context lines, `a`/`b` placeholders |

### 9A — CLI struct expansion + input file ✓

- `Cli` struct expanded: all flags (`-i`, `-x`, `-o`, `-d`, `--on-error`, `--no-env`) + positional `args` Vec
- Positional arg reinterpretation: without `-f`, args[0]=script, args[1]=file; with `-f`, args[0]=file
- Clap `conflicts_with` for `in_place`/`output` and `in_place`/`dry_run`; post-parse `--in-place` requires file
- `OnError` made `pub` with `FromStr` + `Display` impls; re-exported from `qed_core`
- Created missing golden `tests/invocation/goldens/output/env-pattern.txt`

### 9B — `--no-env`, `--on-error`, `--extract` ✓

- `RunOptions` struct in `qed-core` public API; `run()` accepts `&RunOptions`
- `no_env` → `compile()`; `on_error` → `compile()` as `global_on_error` default; `extract` → `execute()` passthrough suppression
- Per-selector `on_error:X` still overrides global; invocation 10/13, edge cases 8/13

### 9C — `--output` + `--in-place` ✓

- `--output`: write result to file via `std::fs::write()`, suppress stdout
- `--in-place`: atomic write via `.qed-tmp` temp file + `std::fs::rename()`, with cleanup on failure

### 9D — `--dry-run`

- `similar = "2.7"` in `qed/Cargo.toml` (CLI concern, not core)
- `qed/src/diff.rs`: unified diff generation with `header("a","b")`, equality check, `missing_newline_hint(false)`

### 9E — Phase checkpoint + documentation

**Checkpoint:** `invocation`, `invocation-edge-cases`, `stream-control`, and `script-files` integration
suites are green. ~330/396 integration tests pass.

### ✦ Alpha 3 — Generation + Full CLI

Alpha 3 is reached after Phase 9. Content generation and all invocation modes work.
~330/396 integration tests pass.

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

### ✦ Alpha 4 — Feature Complete

Alpha 4 is reached after Phase 11. Full test suite green. All edge cases and
use cases pass. 396/396 integration tests pass.

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

| Phase | Deliverable | Key checkpoint | Alpha |
|---|---|---|---|
| 0 | Workspace scaffold | `cargo build --workspace` clean | |
| 1 | Test harness infrastructure | `cargo test --package qed-tests` registers all trials (failing) | |
| 2 | Core types + fragmentation algorithm | Buffer, fragment, and fragmentation unit tests pass | |
| 3 | Parser POC evaluation | One parser remains, routing removed | |
| 4 | Walking skeleton | `selectors::at-literal-single-match::0` green | |
| 5 | Full parser | All grammar productions parsed; `selectors` suite green | |
| 6 | Full compiler (6A–6D) | Env vars, warnings, replace, external processors | **Alpha 1** |
| 7 | Processor coverage | `processors` + `external-processors` suites green | **Alpha 2** |
| 8 | Generation processors | `generation` suite green | |
| 9 | Invocation features | `invocation` + `stream-control` suites green | **Alpha 3** |
| 10 | Diagnostics | `error-handling` suites green | |
| 11 | Edge cases + use cases | `cargo test --workspace` fully green | **Alpha 4** |
| 12 | Release polish | Completions, README, clippy clean | **v1.0** |
