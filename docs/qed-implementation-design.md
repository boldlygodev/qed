# qed Implementation Design

Implementation design and architecture decisions for `qed`, a modern stream editor implemented in Rust.

---

## Pipeline

```
source → parse → AST → compile → Script → execute → output
```

---

## Key Design Decisions

### Buffering

Full buffering — the entire input is read into memory before any statement executes.
`qed` targets source files and config files, not multi-GB streams.
Full buffering simplifies execution, makes atomic in-place writes natural, and eliminates the need for two code paths.

### Internal Representation

The input is buffered as a `Buffer` — a flat `String` with a pre-computed line offset index.
The index maps line numbers to byte offsets, making line-to-byte-range lookups O(log n) via binary search.
Joining a line range for processor handoff is a single `&str` slice into the original content — no scanning, no copying until handoff.

```rust
struct Buffer {
    content: String,
    line_offsets: Vec<usize>,   // byte offset of the start of each line
}
```

`Buffer.line_offsets` covers original input content only.
Processor output is not in the original buffer — re-fragmentation over processor output builds a temporary line index on demand.
Since processor output is typically small relative to the full buffer, this is cheap and correct.

### Fragment Model

The buffer is fragmented into a list of tagged fragments.
Each fragment is either a passthrough (unselected, emitted as-is) or a selected region carrying a set of tags identifying which statements and selectors matched it.

Since selectors are strictly line-oriented, fragment boundaries always coincide with line boundaries.
Fragments reference the original buffer by line range rather than owning their content — only processor output requires an owned string.
This means most fragments — all passthroughs and unprocessed selected regions — never allocate.

```rust
struct LineRange {
    start: usize,   // line index, inclusive
    end: usize,     // line index, exclusive
}

enum FragmentContent {
    Borrowed(LineRange),   // references original buffer — no allocation
    Owned(String),         // processor output
}

enum Fragment {
    Passthrough(FragmentContent),
    Selected {
        content: FragmentContent,
        tags: Vec<(StatementId, SelectorId)>,
    },
}

type FragmentList = Vec<Fragment>;
```

**Mutations** — variable-size replacements are expressed as fragment list splices, not buffer mutations.
The original buffer is never modified — borrowed references remain valid throughout execution.
When a selected region is processed, its `Borrowed(LineRange)` fragment is replaced with an `Owned(String)` fragment carrying the processor output.
Other borrowed fragments are unaffected.

**Splice implementation** — `Vec::splice()` for fragment replacement.
Optimise to an arena-allocated index-linked list later if profiling demands it.

### Sequential Semantics

Statement N+1 sees the output of statement N, not the original input.
This is preserved by re-fragmenting processor output against all remaining statements' selectors before splicing it back into the fragment list.

The fragmentation lifecycle:

1. **Initial pass** — all selectors across all statements are resolved against the original input simultaneously, producing a fully tagged `FragmentList`
2. **Processor handoff** — statement N collects its tagged fragments, joins them, passes the joined string to its processor
3. **Output fragmentation** — processor output is immediately re-fragmented against all remaining statements' selectors (N+1 through end), tagged, and spliced back into the list in place of statement N's fragments
4. No re-scanning of the original input ever occurs

### Overlapping Regions

Multi-tagging replaces the last-declaration-wins rule from the design doc.
A fragment tagged for multiple statements is processed in statement order.
Statement N+1 sees re-fragmented output from statement N — not the original — which is the correct interpretation of sequential semantics.

---

## Identity Types

`StatementId` and `SelectorId` are newtypes over `usize`, globally scoped to the script.
Global scope means every selector in the script has a unique `SelectorId` regardless of which statement it belongs to.

```rust
struct StatementId(usize);
struct SelectorId(usize);
```

---

## Script Structure

```rust
struct Script {
    statements: Vec<Statement>,
    selectors: Vec<RegistryEntry>,
}
```

### Selector Registry

A flat `Vec<RegistryEntry>` owned by `Script`, indexed by `SelectorId`.
All selectors are compiled eagerly at parse time — compilation errors surface at startup, not during execution.
Compiled `regex::Regex` objects are reused across fragmentation passes.

```rust
enum RegistryEntry {
    Simple(CompiledSelector),
    Compound(CompoundSelector),
}

struct CompiledSelector {
    id: SelectorId,
    op: SelectorOp,
    on_error: OnError,
}

struct CompoundSelector {
    id: SelectorId,
    steps: Vec<SelectorId>,   // simple selectors to intersect in order
}
```

`on_error` is factored into `CompiledSelector` as a cross-cutting concern valid on all operations.
Operation-specific parameters live in per-variant fields on `SelectorOp`.

```rust
enum SelectorOp {
    At     { pattern: CompiledPattern, nth: NthExpr },
    After  { pattern: CompiledPattern },
    Before { pattern: CompiledPattern },
    From   { pattern: CompiledPattern },
    To     { pattern: CompiledPattern },
}
```

`inclusive` is not a selector-level parameter.
It is a property of the pattern reference itself — the `+` suffix on a pattern in `from` or `to` position.
It is stored on `CompiledPattern` and read during match collection.
Using `+` on a pattern in `at`, `after`, or `before` position is warned and ignored during compilation.

```rust
struct CompiledPattern {
    matcher: PatternMatcher,   // compiled regex or literal string
    negated: bool,
    inclusive: bool,           // true if + suffix present; warned and ignored outside from/to
}

enum PatternMatcher {
    Literal(String),
    Regex(regex::Regex),
}
```

Compound selectors (`from > to`) resolve to a single `SelectorId` — fragments are tagged with the compound selector's ID, not the IDs of its component steps.

### Statement Structure

```rust
struct Statement {
    id: StatementId,
    selector: SelectorId,
    processor: Box<dyn Processor>,
    fallback: Option<Box<dyn Processor>>,
}
```

The fallback is also a `Box<dyn Processor>` — it can be a single processor, a chain, or anything else that implements the trait.

---

## Fragmentation Algorithm

A single fragmentation pass takes a `&str` buffer and a set of `(StatementId, SelectorId)` pairs and produces a `Vec<Fragment>`.

### Step 1 — Parallel match collection

Each selector runs independently against the full buffer.
Results are collected in parallel using `rayon`.

```rust
let matches: Vec<(LineRange, StatementId, SelectorId)> = selectors
    .par_iter()
    .flat_map(|selector| collect_matches(buffer, selector))
    .collect();
```

`collect_matches` handles `nth` filtering and `inclusive` boundary logic per selector.
All matches are line-oriented — selectors always return whole lines.
`inclusive` is read from the pattern's `CompiledPattern.inclusive` field, not from the selector.
Compound selectors resolve by running their component steps and intersecting the resulting ranges.

### Step 2 — Boundary event decomposition

Each range is decomposed into a `Start` and `End` event at its line boundaries.
Line indices are used rather than byte offsets — all match boundaries are line-aligned, which eliminates edge cases around mid-line byte positions and simplifies the sweep.

```rust
enum EventKind {
    Start(StatementId, SelectorId),
    End(StatementId, SelectorId),
}

struct BoundaryEvent {
    line: usize,    // line index
    kind: EventKind,
}
```

### Step 3 — Sort

Events are sorted by:

1. Line index ascending
2. `Start` before `End` at the same line
3. `StatementId` ascending for simultaneous starts

### Step 4 — Sweep

A `BTreeSet<(StatementId, SelectorId)>` tracks the active tag set.
A fragment is emitted whenever the active set changes.
The gap between the previous emission point and the current event line becomes a `Passthrough(Borrowed(LineRange))` if no tags are active, or a `Selected { content: Borrowed(LineRange), tags }` if tags are active.

---

## Processor Trait

All processors — internal, external, and chains — implement a single trait.

```rust
trait Processor {
    fn execute(&self, input: String) -> Result<String, ProcessorError>;
}
```

### Internal Processors

One type per operation, each implementing `Processor`.

```rust
struct DeleteProcessor;
struct SubstringProcessor { pattern: CompiledPattern }
struct ReplaceProcessor { pattern: CompiledPattern, replacement: ReplacementArg }
struct UpperProcessor;
struct LowerProcessor;
// ... one per qed: builtin
```

**`SubstringProcessor`** narrows its input to the matched span, discarding the rest.
Downstream processors in the chain operate on the substring alone.
This is intentionally lossy and does not break the pipeline contract.

The fragmentation layer is purely line-oriented and has no knowledge of substrings.
`SubstringProcessor` operates entirely on its `String` input after handoff — the fragment model never needs to know a substring operation occurred.
This is a cleaner separation than the old `mode:substring` model, which would have required the fragmentation layer to handle partial-line spans.

**`ReplaceProcessor`** accepts three replacement forms via `ReplacementArg`:

```rust
enum ReplacementArg {
    Literal(String),                      // "…" — always literal, no capture group interpretation
    Template(CompiledRegexTemplate),      // /…/ — regex-aware, expands capture group references
    Pipeline(Box<dyn Processor>),         // processor-chain — runs against matched span, output spliced in
}
```

`"…"` is always literal everywhere it appears.
`/…/` on the right side of `qed:replace()` is a regex template — not a pattern.
It holds capture group references (`$1`, `$name`) and is a distinct type from `CompiledPattern`.

```rust
struct CompiledRegexTemplate {
    template: String,   // raw template string with capture group references
}
```

The pipeline form runs the processor against the matched span and splices its stdout back
in place of the match. The surrounding content always survives.

**Generation processors** (`qed:uuid()`, `qed:timestamp()`, `qed:random()`) strictly ignore stdin.
They produce output purely from their parameters.
Pattern matching is not their responsibility — they compose with `qed:replace()` for placeholder
substitution and with `after`/`before` for insertion.

### External Processor

```rust
struct ExternalProcessor {
    command: String,
    args: Vec<String>,
}
```

Spawns a child process, writes `input` to stdin, reads stdout as the return value.
Non-zero exit produces `ProcessorError::ExternalFailed`.

### Processor Chain

```rust
struct ProcessorChain(Vec<Box<dyn Processor>>);

impl Processor for ProcessorChain {
    fn execute(&self, input: String) -> Result<String, ProcessorError> {
        self.0.iter().try_fold(input, |acc, p| p.execute(acc))
    }
}
```

`ProcessorChain` implements `Processor` itself — chains compose and nest freely.
`try_fold` short-circuits on the first error, which triggers the `||` fallback at the statement level.
`qed:file()` is a regular processor step that materialises its input to a temp file — it fits naturally in the chain without special casing.

---

## Error Routing

`on_error` is a routing directive — it answers "where does control go next?", not "what went wrong?".
It is consumed at the selector boundary and never inspected by statement execution.

```rust
enum SelectionResult {
    Selected(String),       // match found, input ready for processor
    Passthrough(String),    // no match, on_error was Warn or Skip — original content preserved
    Failed(ProcessorError), // on_error was Fail, or processor returned an error
}
```

Selector boundary routing:

```rust
fn resolve_selector(
    buffer: &FragmentList,
    statement: &Statement,
    selector: &CompiledSelector,
) -> SelectionResult {
    match collect_and_join_fragments(buffer, statement.id) {
        Some(input) => SelectionResult::Selected(input),
        None => match selector.on_error {
            OnError::Fail => SelectionResult::Failed(ProcessorError::NoMatch {
                selector_id: selector.id,
            }),
            OnError::Warn => {
                eprintln!("warning: selector {:?} matched nothing", selector.id);
                SelectionResult::Passthrough(buffer.join())
            },
            OnError::Skip => SelectionResult::Passthrough(buffer.join()),
        }
    }
}
```

Statement execution:

```rust
let result = match resolve_selector(buffer, statement, selector) {
    SelectionResult::Selected(input) => statement.processor.execute(input),
    SelectionResult::Passthrough(content) => return Ok(content),
    SelectionResult::Failed(e) => Err(e),
};

match result {
    Ok(output) => re_fragment_and_splice(output),
    Err(e) => match &statement.fallback {
        Some(fallback) => fallback.execute(original_input),
        None => return Err(e),
    }
}
```

Fallback is invoked identically whether the failure came from a no-match or a processor error.

### Error Taxonomy

```rust
enum ProcessorError {
    NoMatch {
        selector_id: SelectorId,
    },
    ProcessorFailed {
        processor: String,   // human-readable name for diagnostics
        reason: String,
    },
    ExternalFailed {
        command: String,
        exit_code: Option<i32>,
        stderr: String,
    },
}
```

---

## Parser

**Decision deferred to implementation phase.**

Two viable options:

**Option D — `chumsky`**
A parser combinator library with first-class support for high-quality error messages and error recovery.
Pairs with `ariadne` for rendered, annotated diagnostic output.
Evaluate against a spike implementation of the `nth-expr` grammar production.

Evaluation criteria:
- Current major version is stable, no breaking changes anticipated
- Active maintenance — commits, responsive issues, clear ownership
- `nth-expr` spike produces diagnostics matching the quality bar the design doc implies, without significant manual effort
- `ariadne` integration produces production-quality rendered output out of the box

**Option C — Hand-written recursive descent**
One function per grammar production, full control over diagnostics.
The primary con — volume of boilerplate — is substantially reduced with AI-assisted implementation.
Fall back to this option if chumsky does not meet the evaluation criteria.

---

## CLI Interface

Argument parsing uses `clap` with derive macros.
Generated help text, error messages, and shell completions are part of the product quality surface.

### Flag Design

Short flags are provided for all flags commonly used in one-liners, `$EDITOR` assignments, and `//go:generate` directives.
Flags that weaken error handling or are rarely used in one-liners are long-only — the extra keystrokes are an intentional cost.

| Long flag | Short | Notes |
|---|---|---|
| `--file` | `-f` | Script file |
| `--in-place` | `-i` | Modify file directly |
| `--extract` | `-x` | Suppress passthrough output |
| `--output` | `-o` | Write to file instead of stdout |
| `--dry-run` | `-d` | Preview as diff |
| `--on-error` | — | Long only — cost to relaxing error handling |
| `--no-env` | — | Long only — rarely used in one-liners |

`$EDITOR` assignment with short flags:

```sh
export EDITOR="qed -if ~/.config/qed/transform.qed"
```

### Cli Struct

```rust
#[derive(Parser)]
#[command(name = "qed")]
struct Cli {
    /// Inline script
    script: Option<String>,

    /// Script file
    #[arg(short = 'f', long)]
    file: Option<PathBuf>,

    /// Modify file directly
    #[arg(short = 'i', long)]
    in_place: bool,

    /// Suppress passthrough output
    #[arg(short = 'x', long)]
    extract: bool,

    /// Write output to file instead of stdout
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,

    /// Preview changes as a diff
    #[arg(short = 'd', long)]
    dry_run: bool,

    /// Global on-error mode
    #[arg(long, default_value = "fail")]
    on_error: OnError,

    /// Disable environment variable expansion
    #[arg(long)]
    no_env: bool,

    /// Input file to process. Reads from stdin if not provided.
    #[arg(value_name = "FILE")]
    input: Option<PathBuf>,
}
```

`clap` renders the usage line as:

```
Usage: qed [OPTIONS] [FILE]
```

`[FILE]` communicates the optional nature clearly.
The doc comment on `input` makes the stdin fallback explicit in `--help` output.
When `input` is `None`, `qed` reads from stdin.

### Validation Constraints

Caught at the CLI layer before execution.
Mutual exclusions are expressed declaratively with `clap`'s `conflicts_with`.
The `--in-place` requires-input constraint is a simple `Option` check after parsing.

| Constraint | Kind |
|---|---|
| `--in-place` requires `input` to be `Some` | Manual post-parse |
| `--in-place` and `--output` are mutually exclusive | `conflicts_with` |
| `--in-place` and `--dry-run` are mutually exclusive | `conflicts_with` |
| `--script` and `--file` are mutually exclusive | `conflicts_with` |

---

## AST

The AST is the direct output of parsing.
It represents the structure of the source before any compilation or validation.
It is close to the grammar, carries source span information for diagnostics, and makes no semantic decisions.

### Spans

Every AST node is wrapped in `Spanned<T>`, carrying its source location for diagnostic messages.

```rust
struct Span {
    start: usize,   // byte offset into source
    end: usize,
}

struct Spanned<T> {
    node: T,
    span: Span,
}
```

### Top Level

`SelectActionNode` is shared between `Statement::SelectAction` and `Fallback::SelectAction`.
This enforces at the type level that only a select-action can appear as a fallback — not a pattern or alias definition.

```rust
struct Program {
    shebang: Option<Spanned<String>>,
    statements: Vec<Spanned<Statement>>,
}

struct SelectActionNode {
    selector: Spanned<Selector>,
    chain: Spanned<ProcessorChain>,
    fallback: Option<Spanned<Fallback>>,
}

enum Statement {
    PatternDef {
        name: Spanned<String>,
        value: Spanned<PatternValue>,
    },
    AliasDef {
        name: Spanned<String>,
        chain: Spanned<ProcessorChain>,
    },
    SelectAction(SelectActionNode),
}
```

`PatternDef` and `AliasDef` both parse as `identifier = ...`.
The RHS disambiguates at parse time — a string or regex is a `PatternDef`, a processor name is an `AliasDef`.

### Fallback

```rust
enum Fallback {
    SelectAction(Box<SelectActionNode>),   // Box to avoid infinite size via SelectActionNode.fallback
    Chain(ProcessorChain),                 // processor-chain only
}
```

### Patterns

```rust
enum PatternValue {
    String(String),
    Regex(String),
}

struct PatternRef {
    value: PatternRefValue,
    negated: bool,
    inclusive: bool,
}

enum PatternRefValue {
    Named(String),          // bare identifier reference
    Inline(PatternValue),   // string or regex inline
}
```

### Selectors

```rust
struct Selector {
    steps: Vec<Spanned<SimpleSelector>>,   // one or more steps narrowed by > operator
}

struct SimpleSelector {
    op: SelectorOp,
    pattern: Option<Spanned<PatternRef>>,
    params: Vec<Spanned<Param>>,
}

enum SelectorOp {
    At,
    After,
    Before,
    From,
    To,
}
```

### Params

```rust
struct Param {
    name: Spanned<String>,
    value: Spanned<ParamValue>,
}

enum ParamValue {
    Identifier(String),
    String(String),
    Integer(i64),
    NthExpr(NthExpr),
    PatternRef(PatternRef),
}
```

Params are validated during compilation, not parsing.
The AST carries them as-is — unknown or invalid params surface as compilation errors.

### Processor Chain

```rust
struct ProcessorChain {
    processors: Vec<Spanned<Processor>>,
}

enum Processor {
    Qed(QedProcessor),
    External(ExternalProcessor),
}
```

### qed Internal Processor

```rust
struct QedProcessor {
    name: Spanned<String>,                 // everything after qed: e.g. "replace", "debug:count"
    args: Vec<Spanned<QedArg>>,
    params: Vec<Spanned<Param>>,
}

enum QedArg {
    PatternRef(PatternRef),
    String(String),
    Regex(String),
    Integer(i64),
    ProcessorChain(Box<ProcessorChain>),   // Box to break recursive cycle — pipeline replacement form
}
```

`ProcessorChain` inside `QedArg` is the pipeline replacement form of `qed:replace()`.
`Box` is required to break the recursive cycle: `QedArg` → `ProcessorChain` → `Processor` → `QedProcessor` → `QedArg`.

### External Processor

```rust
struct ExternalProcessor {
    command: Spanned<String>,
    escaped: bool,               // true if \ prefix present — bypass alias, resolve via PATH only
    args: Vec<Spanned<ExternalArg>>,
}

enum ExternalArg {
    Quoted(String),
    Unquoted(String),
}
```

### nth Expression Language

```rust
struct NthExpr {
    terms: Vec<Spanned<NthTerm>>,
}

enum NthTerm {
    Integer(i64),
    Range { start: i64, end: i64 },
    Step { coefficient: i64, offset: Option<i64> },
}
```

---

## Compilation Pass

The compilation pass takes a `Program` AST and produces a `Script`.
It assigns global IDs, resolves names, compiles regex patterns, expands environment variables, validates semantic constraints, and builds the selector registry and statement list.

### Error Collection

The pass collects all errors before returning, rather than failing on the first error.
Users see all problems in one pass — essential for a good scripting experience.
Sentinel values (e.g. `CompiledPattern::Invalid`) allow compilation to continue after an error so subsequent errors are still surfaced.

Warnings (unset env vars, `+` on non-boundary patterns, etc.) are accumulated separately and emitted to stderr regardless of whether compilation succeeds.

The pass returns `Ok(Script)` if no errors were collected, or `Err(Vec<CompileError>)` if any were.

### Symbol Table

Named patterns and aliases are available script-wide — forward references are permitted.
The symbol table is populated in a pre-pass before the main compilation pass resolves references.

```rust
enum Symbol {
    Pattern(Spanned<PatternValue>),   // from PatternDef
    Alias(Spanned<ProcessorChain>),   // from AliasDef
}

struct SymbolTable {
    symbols: HashMap<String, Spanned<Symbol>>,
}
```

Lookups enforce both existence and kind.
Referencing an undefined name or using a pattern name in a processor position (or vice versa) both produce typed errors.
Duplicate name definitions at the definition site emit a warning — last definition wins.

### Pass Structure

**Phase 1 — Symbol collection**

Walk all statements, collect `PatternDef` and `AliasDef` entries into the symbol table.
No regex compilation yet.
Detect and warn on duplicate name definitions.

**Phase 2 — Compilation**

Walk all statements in order. For each statement:

- Resolve pattern references via the symbol table
- Compile regex strings into `regex::Regex`
- Expand environment variables in pattern values and processor args
- Validate semantic constraints
- Assign `StatementId` and `SelectorId`
- Build `CompiledSelector`, `RegistryEntry`, and `Box<dyn Processor>`
- Accumulate errors into `Vec<CompileError>`

### Error Type

```rust
enum CompileError {
    UndefinedName {
        name: String,
        span: Span,
    },
    WrongSymbolKind {
        name: String,
        expected: SymbolKind,
        found: SymbolKind,
        span: Span,
    },
    InvalidRegex {
        pattern: String,
        reason: String,
        span: Span,
    },
    InvalidParam {
        processor: String,
        param: String,
        span: Span,
    },
    ConflictingParams {
        processor: String,
        params: Vec<String>,
        span: Span,
    },
    InvalidNthExpr {
        reason: String,
        span: Span,
    },
    UnsetEnvVar {
        name: String,
        span: Span,   // warning only — compilation continues with empty string
    },
}

enum SymbolKind {
    Pattern,
    Alias,
}
```

## Resolved Concerns

### stderr diagnostic message format

**Resolved.** The full format is:

```
qed: error:   5:12-20: at("foo"): no lines matched
qed: warning: 5:8-15:  at("foo"+): + ignored on at
qed: debug:   5:32-44: false: exit code 1
```

**Format:** `qed: <severity>: <location>: <source-expression>: <message>`

- **Severity** — `error:`, `warning:`, or `debug:`.
  Padded to `warning:` width so location always starts at the same column.
- **Location** — `line:start-end`, 1-based line and byte offsets.
  Padded to the width of the widest span in the script.
  Widest span is computable from the AST before any statement executes — no runtime buffering required.
- **Source expression** — the span of source text that produced the diagnostic.
  Always echoed as source text, never as an internal identifier.
  Covers selectors, processors, parameters, and any other expression.
- **Message** — free-form, no padding.
- One diagnostic per event; no end-of-run summary.

The `warn:` spelling in `eprintln!` calls in `resolve_selector` must be updated to match
the confirmed `qed: warning:` prefix and location format.

### stdout behaviour on non-zero exit

**Resolved.** `qed` emits lines as soon as all statements are done with them,
freeing fragments as they go rather than holding the full output buffer until exit.
On failure, lines already emitted remain emitted — there is no rollback.

This follows from the fragment model: a line with no remaining tags for future statements
can be emitted and its memory freed immediately.

The `--on-error` flag governs downstream responsibility:

- **`--on-error=fail`** (default) — qed exits non-zero on failure.
  Users piping output should use `set -o pipefail` so the pipeline fails visibly.
- **`--on-error=skip/warn`** — user opted in, exit zero.
  User accepts downstream contract.

Implementation note: `SelectionResult::Passthrough` and the output path in statement execution
should emit and free fragments as they become tag-free rather than collecting them.

### `${QED_FILE}` injection for `qed:file()`

**Resolved.** Implementation notes:

- The temp file is written atomically before the downstream command is spawned
- `${QED_FILE}` is added to the child process environment in `ExternalProcessor::execute()`
  alongside any other env vars already in scope
- `${QED_FILE}` is scoped to the immediately downstream command — it does not
  persist to subsequent pipeline stages
- Temp files are cleaned up when `${QED_FILE}` goes out of scope — when the downstream command exits
- If `qed:file()` appears multiple times in a pipeline (e.g. in a fallback chain),
  each invocation sets `${QED_FILE}` to its own temp file path for its own downstream command
- Referencing `${QED_FILE}` in a pipeline that does not include `qed:file()` expands
  to empty string with a warning, consistent with unset env var behaviour

### `${QED_FILE}` and the mock test harness

Mock scripts generated by the test harness validate file content by reading `${QED_FILE}`
directly from their own environment — qed injects it before spawning the mock, so no
special harness channel is needed.
The mock reads the file, compares its content against the declared `expected_file_content`,
and exits non-zero with a diagnostic to stderr if they do not match.

### `--dry-run` diff format

**Resolved.**

- **File paths** — `---`/`+++` header lines use fixed `a` / `b` placeholders.
- **Timestamps** — omitted from the header.
- **Context lines** — 3 lines (standard `diff` / `git diff` default).

The diff body follows standard unified diff format:
`@@` hunk markers, `-` prefix for removed lines, `+` prefix for added lines,
unchanged context lines with no prefix.

---

## Open Concerns

### End-of-run diagnostic summary

Whether to emit a summary line at the end of a run (e.g. `2 warnings, 1 error`)
is deferred. No summary is emitted by default. Revisit once the full diagnostic
format is validated against real scripts.

---

## Changelog

### [next]

- **Named pattern redefinition** — the compilation pass must emit a warning when
  a `PatternDef` assigns to an identifier that already exists in the pattern
  registry, then overwrite the entry with the new definition.
  Forward references are resolved after all definitions are collected, so
  definition order within the source does not affect resolution.
- **Expanded Open Concerns section** — replaced the placeholder "all major concerns
  have been addressed" with three active concerns:
  stderr diagnostic message format, stdout behaviour on non-zero exit, and
  `${QED_FILE}` injection implementation notes for `qed:file()`.
- **Documented `${QED_FILE}` and the mock test harness** — mock scripts read
  `${QED_FILE}` directly from their environment; no special harness channel needed.
- **Documented `--dry-run` diff format decisions** — unified diff, fixed `a` / `b`
  placeholders, timestamps omitted; format confirmed as stable and golden-file-safe.
- **Resolved stderr diagnostic message format** — full format specified:
  `qed: <severity>: <location>: <source-expression>: <message>`.
  Severity padded to `warning:` width; location padded to widest span (computed from AST);
  source expression and message unpadded. 1-based line and byte offsets.
  One diagnostic per event, no end-of-run summary.
- **Resolved stdout behaviour on non-zero exit** — emit lines as fragments become tag-free,
  free memory as you go. No output rollback on failure. Document `set -o pipefail`
  recommendation. `--on-error=skip/warn` users accept downstream contract.
- **Resolved `${QED_FILE}` cleanup timing** — temp files are cleaned up when
  `${QED_FILE}` goes out of scope, i.e. when the downstream command exits.
- **Confirmed `--dry-run` context line count** — 3 lines (standard default).
- **Restructured Open Concerns** — resolved concerns moved to new Resolved Concerns section.
  One open concern remains: end-of-run diagnostic summary (deferred).
