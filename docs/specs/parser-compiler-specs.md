# EARS Specs — Parser-Compiler

ID prefix: `PCOMP`

## Recursive Descent Parser

- [x] PCOMP-001: The parser SHALL be a hand-written recursive descent implementation (`parse/rd/`) with no external parser-generator dependency.
- [x] PCOMP-002: The cursor SHALL operate as a zero-copy scanner over `&str`, maintaining a byte-offset `pos` and providing `peek`, `advance`, `eat_*`, and `set_pos` (backtrack) operations.
- [x] PCOMP-003: WHEN `parse_statement` encounters an identifier followed by `=` (not `==`), it SHALL interpret it as a definition statement; otherwise it SHALL restore the cursor position and attempt a `SelectAction` parse.
- [x] PCOMP-004: WHEN `parse_processor` encounters a bare identifier with no argument list, it SHALL produce an `AliasRef`; WHEN it encounters an identifier with an argument list, it SHALL produce an external command processor node.
- [x] PCOMP-005: WHEN a per-statement parse error occurs, `parse_program` SHALL record the error and advance to the next newline boundary, continuing to parse subsequent statements.
- [x] PCOMP-006: WHEN a script argument ends with `\` not followed by whitespace, the parser SHALL treat it as a line continuation and join the following line into the same argument.
- [x] PCOMP-007: WHEN a script argument has trailing whitespace after a `\` continuation marker, the parser SHALL emit a hard parse error.
- [x] PCOMP-008: The parser SHALL expose `parse_program` and `parse_nth_expr` as the only public surfaces in `parse/mod.rs`.

## Two-Pass Compiler

- [x] PCOMP-010: The compiler SHALL perform two passes: pass 1 collects all `PatternDef` and `AliasDef` symbols; pass 2 compiles `SelectAction` statements against the resolved symbol table.
- [x] PCOMP-011: The compiler SHALL support forward references — a pattern or alias defined after its first use SHALL resolve correctly.
- [x] PCOMP-012: The compiler function SHALL return `Result<(Script, Vec<CompileWarning>), Vec<CompileError>>`, accumulating all errors before returning.
- [x] PCOMP-013: WHEN a pattern is a valid regex string, the compiler SHALL compile it to `PatternMatcher::Regex(regex::Regex)`.
- [x] PCOMP-014: WHEN a pattern is a literal string, the compiler SHALL compile it to `PatternMatcher::Literal(String)`.
- [x] PCOMP-015: WHEN a pattern fails regex compilation and `on_error` is `Skip`, the compiler SHALL compile it to `PatternMatcher::NeverMatch` and continue without an error.
- [x] PCOMP-016: Compiled selectors SHALL be stored in a flat `Vec<RegistryEntry>` indexed by `SelectorId` for O(1) engine lookup.

## qed:file() Fusion

- [x] PCOMP-020: WHEN the compiler encounters a `FileMarker` (from `qed:file()`) followed immediately by an external command in a processor chain, it SHALL fuse them into a single `FileHandoffProcessor`.
- [x] PCOMP-021: WHEN `qed:file()` appears at the end of a chain with no following command, the compiler SHALL emit an appropriate error rather than silently producing an incomplete processor.
- [ ] PCOMP-022: The `pending_file_span` fusion state machine in `compile_processor_chain` SHOULD be made explicit (a named state enum) to prevent silent failures when `qed:file()` appears in non-standard chain positions.

## AliasRef Resolution

- [x] PCOMP-025: WHEN an `AliasRef` name is found in the alias table, the compiler SHALL resolve it to the defined alias processor chain.
- [x] PCOMP-026: WHEN an `AliasRef` name is NOT found in the alias table, the compiler SHALL silently promote it to an external PATH-lookup command with no diagnostic.
- [ ] PCOMP-027: WHEN an `AliasRef` name is not found in the alias table, the compiler SHOULD emit a warning diagnostic to help users catch alias typos.

## Stream-Control Detection

- [x] PCOMP-030: WHEN the compiler encounters `qed:warn`, `qed:fail`, `qed:debug:count`, or `qed:debug:print` as processor names, it SHALL compile them to `StatementAction` enum variants rather than `Box<dyn Processor>`.

## Environment Variable Expansion

- [x] PCOMP-035: `expand_env_vars` SHALL substitute `${VAR}` references with their environment values during compilation.
- [x] PCOMP-036: WHEN a `\${VAR}` escaped reference is encountered, `expand_env_vars` SHALL emit `${VAR}` literally without expansion.
- [x] PCOMP-037: WHEN a referenced environment variable is unset, `expand_env_vars` SHALL expand it to an empty string and record an `UnsetVar` at the byte offset of the reference.
- [x] PCOMP-038: WHEN `no_env: true` is set in `CompileOptions`, `expand_env_vars` SHALL skip all expansion and return the input unchanged.

## Non-Features

- [D] PCOMP-040: The chumsky PEG parser SHALL NOT be used or resurrected; the hand-written RD parser is the sole parser implementation.
- [D] PCOMP-041: The compiler SHALL NOT perform environment variable expansion at parse time or execution time — only at compile time.

## References

- `qed-core/src/parse/mod.rs`
- `qed-core/src/parse/rd/cursor.rs`
- `qed-core/src/parse/rd/parser.rs`
- `qed-core/src/compile/mod.rs`
- `qed-core/src/compile/env.rs`
- `docs/llds/parser-compiler.md`
