# qed — Implementation TODOs

## Status

- [x] Phase 0 — Workspace Scaffold
- [x] Phase 1 — Test Harness Infrastructure
- [x] Phase 2 — Core Types and Fragmentation Algorithm
- [x] Phase 3 — Parser POC Evaluation
- [x] Phase 4 — Walking Skeleton
- [ ] Phase 5 — Full Parser
- [ ] Phase 6 — Full Compiler
- [ ] Phase 7 — Processor Coverage
- [ ] Phase 8 — Generation Processors
- [ ] Phase 9 — Invocation Features
- [ ] Phase 10 — Diagnostics
- [ ] Phase 11 — Edge Cases and Use Cases
- [ ] Phase 12 — Release Polish

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

### 5a — Patterns (regex, negation, inclusive, single-quoted strings)

- [ ] Cursor: `eat_single_quoted_string_literal()` for `'...'` with `\'`, `\\` escapes
- [ ] Cursor: `eat_regex_literal()` for `/regex/` with `\/` escapes
- [ ] Cursor: `eat_identifier()` — extract reusable `[a-zA-Z_][a-zA-Z0-9_]*` method
- [ ] Parser: `parse_pattern_ref()` — `!` prefix, string/regex/identifier dispatch, `+` suffix
- [ ] Parser: `parse_pattern_value()` — string vs regex discrimination
- [ ] Parser: rewrite `parse_selector` to call `parse_pattern_ref`
- [ ] Parser: skip `# comment` lines in `eat_whitespace_and_newlines`
- [ ] Parser: handle shebang (`#!`) in `parse_program`
- [ ] Unit tests for all pattern-ref forms (~12 tests)
- [ ] Checkpoint: `at-regex-match`, `at-negated`, `from-inclusive`, `to-inclusive`, `patterns::inline-*` green

### 5b — Selector parameters + compound selectors

- [ ] Cursor: `peek_at(offset)` for lookahead (disambiguate `|` vs `||`)
- [ ] Parser: `parse_param_list()` — comma-separated `name:value` pairs
- [ ] Parser: `parse_param_value()` — identifier, string, integer, nth-expr, pattern-ref
- [ ] Parser: factor `parse_nth_expr` internals into `parse_nth_expr_from_cursor()` for mid-stream use
- [ ] Parser: nth `,` vs param `,` disambiguation (lookahead: `[a-zA-Z_]` + `:` = next param)
- [ ] Parser: selector params `(pattern, nth:..., on_error:...)`
- [ ] Parser: compound selectors `from(p) > to(p)` with `>` operator
- [ ] Parser: implicit line continuation after `>` and `,`
- [ ] Compiler: compound selector compilation (multi-step → `RegistryEntry::Compound`)
- [ ] Compiler: wire `nth` param → `NthExpr` on compiled selector
- [ ] Compiler: wire `on_error` param → `OnError` enum
- [ ] Compiler: support `at()` entire-stream (empty pattern)
- [ ] Compiler: support `after`/`before`/`from`/`to` selector ops
- [ ] Unit tests for param parsing, compound selectors (~15 tests)
- [ ] Checkpoint: `from-to-*`, `at-narrowing`, `nth-*`, `on-error-*`, `after-literal`, `before-literal`, `from-literal`, `to-literal` green

### 5c — Processor arguments + chains + external processors

- [ ] Cursor: `eat_unquoted_arg()` for external processor args
- [ ] Parser: rewrite `parse_processor` — dispatch `qed:*` vs external
- [ ] Parser: rewrite `parse_qed_processor` — `qed:name(args, params)` with positional + named args
- [ ] Parser: colon-separated processor names (`qed:debug:count()`)
- [ ] Parser: nested processor chain as arg (`qed:replace("x", qed:upper())`)
- [ ] Parser: `parse_external_processor()` — command/path, escaped `\`, quoted/unquoted args
- [ ] Parser: rewrite `parse_processor_chain` for multi-processor piping
- [ ] Parser: `|` vs `||` resolution (1-byte lookahead)
- [ ] Parser: implicit line continuation after `|`
- [ ] Compiler: register `UpperProcessor` and `LowerProcessor`
- [ ] Compiler: processor chain composition
- [ ] Unit tests for all processor forms (~18 tests)
- [ ] Checkpoint: `at-entire-stream` green, chain parsing doesn't regress

### 5d — Definitions + fallback + line continuation

- [ ] Parser: `parse_pattern_def()` — `identifier = pattern-value`
- [ ] Parser: `parse_alias_def()` — `identifier = processor-chain`
- [ ] Parser: update `parse_statement` — disambiguate PatternDef vs AliasDef vs SelectAction
- [ ] Parser: fallback `||` in `parse_select_action` — select-action or processor-chain
- [ ] Parser: implicit line continuation after `||`
- [ ] Parser: semicolons as statement separators
- [ ] Compiler: symbol table — collect `PatternDef` and `AliasDef` entries
- [ ] Compiler: resolve `PatternRefValue::Named` through symbol table
- [ ] Compiler: fallback compilation on `Statement`
- [ ] Unit tests for definitions, fallback, semicolons, line continuation (~15 tests)
- [ ] Checkpoint: `patterns::named-*`, script-file scenarios green

### 5e — Error recovery + polish

- [ ] Parser: improved error recovery — skip to next statement boundary
- [ ] Parser: span accuracy audit across all productions
- [ ] Parser: edge cases — empty program, comment-only, EOF without newline
- [ ] Parser: `\` line continuation in external processor expressions
- [ ] Parser: trailing whitespace after `\` → hard error
- [ ] Unit tests for error recovery, spans, edge cases (~10 tests)
- [ ] Checkpoint: all parser unit tests pass, grammar is complete

---

## Phases 6–12

See `docs/qed-roadmap.md` for full details.

---

## Deferred

- [ ] Switch `collect_all_matches` in `exec/fragment.rs` to `rayon` parallel iteration (dependency already present)
