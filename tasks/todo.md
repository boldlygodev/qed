# qed — Implementation TODOs

## Status

- [x] Phase 0 — Workspace Scaffold
- [x] Phase 1 — Test Harness Infrastructure
- [x] Phase 2 — Core Types and Fragmentation Algorithm
- [x] Phase 3 — Parser POC Evaluation
- [x] Phase 4 — Walking Skeleton
- [x] Phase 5 — Full Parser (5A ✓, 5B ✓, 5C ✓, 5D ✓, 5E deferred to Phase 11)
- [x] Phase 6 — Full Compiler (6A–6D) → **Alpha 1**
- [x] Phase 7 — Processor Coverage → **Alpha 2**
- [x] Phase 8 — Generation Processors
- [x] Phase 9 — Invocation Features → **Alpha 3**
- [ ] Phase 10 — Diagnostics
- [ ] Phase 11 — Edge Cases and Use Cases → **Alpha 4**
- [ ] Phase 12 — Release Polish → **v1.0**

---

## Phase 2 — Core Types and Fragmentation Algorithm

### 2a — Foundation types ✓

- [x] `span.rs`: define `Span { start: usize, end: usize }` and `Spanned<T> { node: T, span: Span }`
- [x] Identity newtypes: `StatementId(usize)`, `SelectorId(usize)` with accessor methods
- [x] `error.rs`: `CompileError` enum (all variants from implementation design), `SymbolKind` enum
- [x] Wire module declarations in `lib.rs` (`mod span`, `mod error`, `mod parse`, `mod compile`, `mod exec`, `mod processor`)
- [x] Checkpoint: `cargo build --workspace` and `cargo clippy --workspace` clean

### 2b — AST types ✓

- [x] `parse/ast.rs`: `Program`, `Statement`, `SelectActionNode`
- [x] `parse/ast.rs`: `Selector`, `SimpleSelector`, `SelectorOp`
- [x] `parse/ast.rs`: `PatternValue`, `PatternRef`, `PatternRefValue`
- [x] `parse/ast.rs`: `ProcessorChain`, `Processor`, `QedProcessor`, `ExternalProcessor`
- [x] `parse/ast.rs`: `QedArg`, `ExternalArg`
- [x] `parse/ast.rs`: `Fallback`
- [x] `parse/ast.rs`: `Param`, `ParamValue`
- [x] `parse/ast.rs`: `NthExpr`, `NthTerm`
- [x] Checkpoint: `cargo build --workspace` and `cargo clippy --workspace` clean

### 2c — Exec and IR types + Processor trait ✓

- [x] `exec/`: `LineRange { start, end }`, `FragmentContent` (`Borrowed(LineRange)` / `Owned(String)`), `Fragment` (`Passthrough(FragmentContent)` / `Selected { content, tags }`), `FragmentList` type alias
- [x] `exec/`: `Buffer { content: String, line_offsets: Vec<usize> }` with constructor and `slice(LineRange) -> &str`
- [x] `compile/`: `Script { statements, selectors }`, `Statement { id, selector, processor, fallback }`
- [x] `compile/`: `RegistryEntry` (`Simple(CompiledSelector)` / `Compound(CompoundSelector)`), `CompiledSelector`, `CompoundSelector`
- [x] `compile/`: `SelectorOp` with per-variant fields, `CompiledPattern { matcher, negated, inclusive }`, `PatternMatcher` (`Literal(String)` / `Regex(regex::Regex)`)
- [x] `compile/`: `OnError` enum
- [x] `processor/`: `Processor` trait (`fn execute(&self, input: &str) -> Result<String, ProcessorError>`), `ProcessorError` enum (`NoMatch`, `ProcessorFailed`, `ExternalFailed`)
- [x] Unit tests for `Buffer::new()` (line offset construction) and `Buffer::slice()` (correct line extraction)
- [x] Checkpoint: buffer unit tests pass, `cargo build --workspace` clean

### 2d — Fragmentation algorithm ✓

- [x] Implement parallel match collection using `rayon`
- [x] Boundary event decomposition (`Start` / `End` events per match)
- [x] Sort events (line ascending, `Start` before `End`, `StatementId` ascending)
- [x] Sweep with `BTreeSet` active tag set producing the `FragmentList`
- [x] `inclusive` boundary logic per `CompiledPattern`
- [x] `nth` filtering on match results
- [x] Unit test: single selector, single match → one `Selected` fragment flanked by `Passthrough`
- [x] Unit test: single selector, no match → all `Passthrough`
- [x] Unit test: two overlapping selectors → multi-tagged `Selected` fragment
- [x] Unit test: `nth:2` → only second match selected
- [x] Unit test: `from > to` compound → correct inclusive/exclusive boundary variants
- [x] Unit test: negated pattern → lines not matching are selected
- [x] Checkpoint: all fragmentation unit tests pass

---

## Phase 3 — Parser POC Evaluation ✓

- [x] Create `parse/error.rs` with `ParseError` enum and `ParseResult` struct
- [x] Wire `error` module and feature-gated `rd`/`chumsky` modules in `parse/mod.rs`
- [x] Implement RD spike: `Cursor` struct, `parse_nth_expr` recursive descent parser
- [x] Implement chumsky 0.9 spike: `Token` lexer, combinator parser, error conversion
- [x] Both spikes pass identical test suites (valid forms, spans, errors, warnings, whitespace)
- [x] Evaluate: RD wins on compile time (1.5s vs 2.9s), error quality, debuggability, deps (0 vs 16)
- [x] Cleanup: delete `chumsky/`, remove feature flags from `Cargo.toml`, simplify `parse/mod.rs`
- [x] Update `docs/qed-project-structure.md` with evaluation result
- [x] Remove "Switching Parsers" section from `docs/qed-dev-workflow.md`
- [x] Checkpoint: `cargo test`, `cargo build --workspace`, `cargo clippy --workspace` clean

---

## Phase 4 — Walking Skeleton ✓

- [x] `processor/delete.rs`: `DeleteProcessor` returning empty string
- [x] `parse/rd/cursor.rs`: `eat_string_literal()` and `eat_keyword()` helpers
- [x] `parse/rd/parser.rs`: `parse_program()` for `at("literal") | qed:name()` form
- [x] `parse/rd/mod.rs`, `parse/mod.rs`: re-export `parse_program`
- [x] `compile/mod.rs`: `compile()` function — AST `Program` → IR `Script`
- [x] `exec/engine.rs`: `execute()` function — fragments buffer, dispatches processors, concatenates output
- [x] `lib.rs`: public `run(script, input)` API orchestrating parse → compile → execute
- [x] `qed/src/main.rs`: clap CLI with positional script arg and `-f` flag, stdin → run → stdout
- [x] `qed-tests/src/runner.rs`: symlink `qed` binary into temp PATH for harness
- [x] `qed-tests/Cargo.toml`: switched to `[[test]]` with `harness = false` for `cargo test` compatibility
- [x] Checkpoint: `selectors::at-literal-single-match::0` green, 63 unit tests pass, clippy clean

---

## Phase 5 — Full Parser

### 5a — Patterns (regex, negation, inclusive, single-quoted strings) ✓

- [x] Cursor: `eat_single_quoted_string_literal()` for `'...'` with `\'`, `\\` escapes
- [x] Cursor: `eat_regex_literal()` for `/regex/` with `\/` escapes
- [x] Cursor: `eat_identifier()` — extract reusable `[a-zA-Z_][a-zA-Z0-9_]*` method
- [x] Parser: `parse_pattern_ref()` — `!` prefix, string/regex/identifier dispatch, `+` suffix
- [x] Parser: `parse_pattern_value()` — string vs regex discrimination
- [x] Parser: rewrite `parse_selector` to call `parse_pattern_ref`
- [x] Parser: skip `# comment` lines in `eat_whitespace_and_newlines`
- [x] Parser: handle shebang (`#!`) in `parse_program`
- [x] Unit tests for all pattern-ref forms (~12 tests)
- [x] Checkpoint: `at-regex-match`, `at-negated`, `from-inclusive`, `to-inclusive`, `patterns::inline-*` green

### 5b — Selector parameters + compound selectors ✓

- [x] Cursor: `peek_at(offset)` for lookahead (disambiguate `|` vs `||`)
- [x] Parser: `parse_param_list()` — comma-separated `name:value` pairs
- [x] Parser: `parse_param_value()` — identifier, string, integer, nth-expr, pattern-ref
- [x] Parser: factor `parse_nth_expr` internals into `parse_nth_expr_from_cursor()` for mid-stream use
- [x] Parser: nth `,` vs param `,` disambiguation (lookahead: `[a-zA-Z_]` + `:` = next param)
- [x] Parser: selector params `(pattern, nth:..., on_error:...)`
- [x] Parser: compound selectors `from(p) > to(p)` with `>` operator
- [x] Parser: implicit line continuation after `>` and `,`
- [x] Compiler: compound selector compilation (multi-step → `RegistryEntry::Compound`)
- [x] Compiler: wire `nth` param → `NthExpr` on compiled selector
- [x] Compiler: wire `on_error` param → `OnError` enum
- [x] Compiler: support `at()` entire-stream (empty pattern)
- [x] Compiler: support `after`/`before`/`from`/`to` selector ops
- [x] Compiler: `UpperProcessor` and `LowerProcessor`
- [x] Exec: no-match detection with on_error routing, structured diagnostics
- [x] CLI: diagnostic output formatting
- [x] Harness: subshell eval fix for `exit` in invocations
- [x] Unit tests for param parsing, compound selectors (~15 tests)
- [x] Checkpoint: 42/46 selector tests green
- [x] Fix: infinite loop in `apply_nth_filter` with negative coefficient

### 5c — Processor arguments + chains + external processors ✓

- [x] Cursor: `eat_unquoted_arg()` for external processor args
- [x] Parser: rewrite `parse_processor` — dispatch `qed:*` vs external
- [x] Parser: rewrite `parse_qed_processor` — `qed:name(args, params)` with positional + named args
- [x] Parser: colon-separated processor names (`qed:debug:count()`)
- [x] Parser: nested processor chain as arg (`qed:replace("x", qed:upper())`)
- [x] Parser: `parse_external_processor()` — command/path, escaped `\`, quoted/unquoted args
- [x] Parser: rewrite `parse_processor_chain` for multi-processor piping
- [x] Parser: `|` vs `||` resolution (1-byte lookahead)
- [x] Parser: implicit line continuation after `|`
- [x] Compiler: processor chain composition
- [x] Unit tests for all processor forms (~18 tests)
- [x] Exec: fix zero-width fragment detection in `sweep()` (after/before selectors)
- [x] Exec: fix `ExternalCommandProcessor` newline handling (input-aware normalization)
- [x] Checkpoint: `at-entire-stream`, `after-literal`, `before-literal` green; chain parsing doesn't regress (46/46 selectors, 106/396 total)

### 5d — Definitions + fallback + aliases ✓

- [x] AST: add `Processor::AliasRef(String)` variant
- [x] Parser: `parse_pattern_def_value()` — `identifier = "string" | /regex/`
- [x] Parser: `parse_alias_def_value()` — `identifier = processor-chain`
- [x] Parser: `parse_statement` disambiguation — lookahead for `=` (not `==`)
- [x] Parser: alias refs in processor position — bare identifiers without args
- [x] Parser: bare identifiers with args remain external commands (backward compat)
- [x] Parser: fallback `||` in `parse_select_action` — select-action or processor-chain
- [x] Parser: `is_selector_start()` for fallback disambiguation
- [x] Parser: implicit line continuation after `||`
- [x] Parser: semicolons as statement separators
- [x] Compiler: two-pass architecture — pass 1 collects definitions, pass 2 compiles
- [x] Compiler: `HashMap` symbol tables for pattern defs and alias defs
- [x] Compiler: resolve `PatternRefValue::Named` through pattern symbol table
- [x] Compiler: resolve `Processor::AliasRef` through alias symbol table (recursive)
- [x] Compiler: `compile_single_processor_into()` flattens alias chains
- [x] Compiler: fallback compilation on `Statement`
- [x] Compiler: `qed:prefix(text:"...")` processor registration
- [x] Compiler: `extract_string_param()` helper
- [x] Processor: `PrefixProcessor` in `processor/prefix.rs`
- [x] Processor: `ChainProcessor` short-circuits on empty output (delete semantics)
- [x] Checkpoint: `patterns::named-*` 4/4, `script-files::*` 8/8, 125/396 total, no regressions

---

## Phase 6 — Full Compiler

### 6A — Env var expansion ✓

- [x] `expand_env_vars()` in `compile/env.rs`: `${IDENT}` expansion, `\${IDENT}` escape
- [x] Wire into pattern compilation (literal strings and regex)
- [x] Wire into processor string arg compilation (qed + external)
- [x] Thread `no_env: bool` through `compile()` (hardcode `false`)
- [x] `compile()` returns `(Script, Vec<CompileError>)` — warnings in Ok path
- [x] Warning emission: `run()` converts compile warnings to `RunDiagnostic`, CLI formats to stderr
- [x] Checkpoint: `patterns::env-expand-pattern`, `invocation::env-expansion` green (129/396)

### 6B — Compiler warnings & validation ✓

- [x] `CompileWarning` enum: `UnsetEnvVar`, `DuplicateName`, `InclusiveIgnored`
- [x] Move `UnsetEnvVar` from `CompileError` to `CompileWarning`
- [x] Duplicate name detection in pass 1 → warning, last definition wins
- [x] `+` on non-boundary selectors (`at`/`after`/`before`) → warning, flag cleared
- [x] Processor unknown param validation (`delete`, `upper`, `lower`, `prefix`)
- [x] Cross-map `WrongSymbolKind` detection (pattern name in alias position and vice versa)
- [x] Compile errors routed through structured `RunDiagnostic` (not Debug format)
- [x] `CompileError` variant coverage audit; reserved variants documented
- [x] Checkpoint: `duplicate-pattern-name`, `plus-ignored-on-{at,after,before}` green (155/396)

### 6C — Replace processor ✓

- [x] `ReplaceProcessor` with `ReplaceSearch`/`ReplaceWith` enums in `processor/replace.rs`
- [x] Literal→Literal: `qed:replace("old", "new")` via `str::replace`
- [x] Regex→Template: `qed:replace(/pattern/, /template/)` via `regex::replace_all`
- [x] Regex→Literal: `qed:replace(/pattern/, "literal")` via `regex::NoExpand`
- [x] Pipeline: `qed:replace("match", echo world)` — run processor per match, strip trailing newline
- [x] Compiler: `compile_replace_processor()` with arg-type dispatch, `(Literal, Template)` rejection
- [x] Parser: extend `parse_qed_arg` for bare-identifier processor chains as args
- [x] Parser: fix `is_param_start` to exclude `qed:` prefix
- [x] Checkpoint: `processors::replace-*` 6/6, `processors-edge-cases::replace-*` 8/8 (169/396)

### 6D — External processor execution ✓

- [x] Mock script generation in bash harness (`tests/harness/generate-mock.sh`)
- [x] Mock generation loop in `run-scenario.sh`
- [x] Stderr passthrough on success, captured in `ProcessorError::ExternalFailed` on failure
- [x] Fallback dispatch in execution engine (`recovered` flag, `has_unrecovered_error` tracking)
- [x] `Fallback::SelectAction` compilation (extract processor chain, ignore selector)
- [x] Bare-word alias refs fall back to `ExternalCommandProcessor`
- [x] Adjacent line merging in `collect_at` for contiguous selections
- [x] Empty buffer handling for `at()` universal pattern
- [x] Unified diagnostic format (single space before source expression)
- [x] Checkpoint: 21/27 external-processors tests green (6 deferred to Phase 8 for `qed:file()`)

### ✦ Alpha 1 checkpoint ✓

- [x] 198/396 integration tests passing
- [x] All selectors, core processors, external commands, named patterns, aliases, env vars
- [x] Update `.claude/CLAUDE.md` with current status

---

## Phase 7 — Processor Coverage

### 7A — Simple processors ✓

- [x] `map_lines()` per-line utility in `processor/mod.rs`
- [x] `qed:suffix(text:"...")`
- [x] `qed:duplicate()`
- [x] `qed:skip()`
- [x] `qed:trim()`
- [x] Fix `qed:prefix()` to use per-line semantics

### 7B — Parameterized processors ✓

- [x] `extract_int_param()` compiler helper (handles NthExpr → Integer unwrap)
- [x] `qed:number(start:N, width:N)`
- [x] `qed:indent(width:N, char:"...")`
- [x] `qed:dedent()`
- [x] `qed:wrap(width:N)`

### 7C — Pattern-based processor ✓

- [x] `qed:substring(pattern)` — literal and regex search

### 7D — Copy and move ✓

- [x] `StatementAction` enum in compile IR
- [x] `Destination` and `DestinationKind` types
- [x] `qed:copy(after:p | before:p | at:p)`
- [x] `qed:move(after:p | before:p | at:p)`
- [x] Execution engine copy/move post-processing pass

### 7E — Test verification ✓

- [x] Full `processors` suite green (47/47)
- [x] Edge case fixtures from `.claude/tests/processors-edge-cases.md` (all present)
- [x] Edge case tests passing (all `processors-edge-cases::` green)
- [x] Fix: `at(on_error:skip)` parser — params-only selector without pattern
- [x] Fix: `NumberProcessor` empty input handling

---

## Phase 9 — Invocation Features

### 9A — CLI struct expansion + input file ✓

- [x] Create missing `tests/invocation/goldens/output/env-pattern.txt` (`${QED_PATTERN}`)
- [x] Expand `Cli` struct: `in_place`, `extract`, `output`, `dry_run`, `on_error`, `no_env`, positional `args` Vec
- [x] Clap `conflicts_with`: `in_place`/`output`, `in_place`/`dry_run`; manual validation for `script`/`file` positional semantics
- [x] Post-parse validation: `--in-place` requires input file
- [x] `OnError` public with `FromStr` + `Display` impls
- [x] Wire `input` positional arg (file or stdin fallback)

### 9B — `--no-env`, `--on-error`, `--extract` ✓

- [x] `RunOptions` struct in `qed-core/src/lib.rs`
- [x] `run()` signature: accept `&RunOptions`
- [x] Wire `no_env` → `compile()` (replace hardcoded `false`)
- [x] Wire `on_error` → `compile()` as global default (per-selector overrides)
- [x] Wire `extract` → `execute()` (skip passthrough fragments)
- [x] Construct `RunOptions` from `Cli` in `main.rs`

### 9C — `--output` + `--in-place` ✓

- [x] `--output`: write `result.output` to file; suppress stdout
- [x] `--in-place`: atomic write via temp file (`.qed-tmp`) + rename in same directory
- [x] Error handling: I/O failures → stderr + exit 2; temp file cleanup on rename failure

### 9D — `--dry-run` ✓

- [x] Add `similar = "2.7"` to `qed/Cargo.toml`
- [x] `qed/src/diff.rs`: unified diff generation (check equality first, `header("a","b")`, `missing_newline_hint(false)`)
- [x] Wire `--dry-run` in `main.rs`: run script, diff original vs output, print diff

### 9E — Phase checkpoint + documentation ✓

- [x] Full integration suite: 332/396 pass
- [x] `cargo clippy --workspace -- -D warnings` clean
- [x] `cargo fmt --check` clean
- [x] Update `docs/qed-roadmap.md` — Phase 9 complete
- [x] Update `.claude/CLAUDE.md` — current phase update

## Phase 8 — Generation Processors

### 8A — Harness `.pattern` multiline fix ✓

- [x] Update `compare-golden.sh`: switch from `grep -qE` to bash `=~` with `\n` resolution
- [x] Verify no regressions (248/396 still passing)

### 8B — `qed:random()` ✓

- [x] Add `rand = "0.10"` dependency
- [x] `RandomProcessor` in `processor/random.rs`
- [x] `extract_ident_param()` helper in `compile/mod.rs`
- [x] Compiler case for `"random"` in `compile_qed_processor()`
- [x] 16 trials green (`generation::random-*`, `generation-edge-cases::random-*`)

### 8C — `qed:uuid()` ✓

- [x] Add `uuid` dependency with v4, v5, v7 features
- [x] `UuidProcessor` in `processor/uuid.rs`
- [x] Compiler case for `"uuid"` in `compile_qed_processor()`
- [x] Verify UUID v5 golden value (corrected to `4fd35a71-71ef-5a55-a9d9-aa75c889a6d0`)
- [x] Simplified `uuid-v7-after.qed` to `after("header") | qed:uuid()`
- [x] Fixed harness `\n` resolution (`$'\n'` → pre-expanded `${nl}`)
- [x] Generation processors now append `\n` for proper line termination
- [x] 12 trials green (`generation::uuid-*`, `generation-edge-cases::uuid-*`)

### 8D — `qed:timestamp()` ✓

- [x] Add `chrono` and `chrono-tz` dependencies
- [x] `TimestampProcessor` in `processor/timestamp.rs`
- [x] LDML-to-strftime translation
- [x] Compiler case for `"timestamp"` in `compile_qed_processor()`
- [x] 20 trials green (`generation::timestamp-*`, `generation-edge-cases::timestamp-*`)

## Phase 10 — Diagnostics

### 10A — Fallback on selector no-match ✓

- [x] `CompiledFallback` enum: `Chain { processor, span, text }` and `SelectAction { selector, action, spans, fallback }`
- [x] Compile `Fallback::SelectAction` with its selector (via `compile_selector()`)
- [x] Recursive nested fallback compilation
- [x] Engine: try fallback on no-match when `on_error:fail`
- [x] Engine: `CompiledFallback::Chain` runs processor against entire buffer
- [x] Engine: `CompiledFallback::SelectAction` re-fragments buffer with fallback selector
- [x] Fix partial output on error: preserve already-assembled output
- [x] Update `handle_processor_error` for new `CompiledFallback` type
- [x] Checkpoint: +12 tests green (344/396)

### 10B — Stream-control processors ✓

- [x] `StatementAction::Warn` — emit selected text to stderr, pass through
- [x] `StatementAction::Fail` — emit selected text to stderr, halt, exit non-zero
- [x] `StatementAction::DebugCount` — count matches, emit diagnostic
- [x] `StatementAction::DebugPrint` — echo selected text to stderr, pass through
- [x] Compiler: recognize `warn`, `fail`, `debug:count`, `debug:print` in `compile_statement_action()`
- [x] Engine: `stderr_lines: Vec<String>` in `ExecuteResult`
- [x] Engine: `DiagnosticLevel::Debug`
- [x] Public API: `stderr_lines` in `RunResult`, `"debug"` level string
- [x] CLI: emit `result.stderr_lines` to stderr
- [x] Checkpoint: +8 tests green (352/396)

### 10C — Location padding ✓

- [x] Compute max location width from all diagnostics
- [x] Pad location fields to uniform width in `run()`

### 10D — Phase checkpoint and documentation

- [ ] Full test suite verification
- [ ] Update `docs/qed-roadmap.md` with Phase 10 status
- [ ] Update `.claude/CLAUDE.md` with current phase
- [ ] Update `tasks/todo.md` with completion status

## Phases 11–12

See `docs/qed-roadmap.md` for full details.

---

## Deferred

- [ ] 5E: Parser error recovery — skip to next statement boundary
- [ ] 5E: Span accuracy audit across all productions
- [ ] 5E: Edge cases — empty program, comment-only, EOF without newline
- [ ] 5E: `\` line continuation in external processor expressions
- [ ] 5E: Trailing whitespace after `\` → hard error
- [ ] Switch `collect_all_matches` in `exec/fragment.rs` to `rayon` parallel iteration (dependency already present)
