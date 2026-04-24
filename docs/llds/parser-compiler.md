# Parser-Compiler

## Context and Design Philosophy

Transforms a qed source string into a validated, executable `Script` IR — or returns accumulated errors. Two distinct sub-concerns are intentionally kept in one segment because they are tightly coupled: the compiler reads every AST node type produced by the parser, and adding a new language construct requires changing both. Separating them would force every new-feature PR to span two segments with no clean cascade boundary between them.

## Recursive Descent Parser

**Cursor** (`parse/rd/cursor.rs`) — zero-copy scanner over `&str`. Maintains a byte-offset `pos`. Core operations: `peek`, `advance`, `eat_char`, `eat_whitespace`, `eat_string_literal`, `eat_regex_literal`, `eat_identifier`, `eat_unquoted_arg`, `eat_keyword`, and `set_pos` for backtracking. Operates on bytes (`u8`); non-ASCII bytes in string literals are assembled via `ch as char` which is only correct for single-byte code points.

**Parser** (`parse/rd/parser.rs`) — implements the full grammar as a collection of mutually recursive `parse_*` functions. Notable structural choices:

- **Statement disambiguation** — `parse_statement` saves cursor position, eats an identifier, checks for `=` (but not `==`), and restores on mismatch. This lookahead-and-backtrack handles the `pattern = "x"` vs `at("x") | ...` ambiguity at the statement level.
- **Processor disambiguation** — `parse_processor` uses a three-way lookahead: bare identifier with no args → `AliasRef`; identifier with args → external command. Alias vs external ambiguity is resolved at parse time, not compile time.
- **Error recovery** — `parse_program` catches per-statement parse errors and skips to the next newline. Multi-line constructs with mid-statement errors lose the remainder of the construct silently.
- **Backslash line continuation** — Trailing `\` at end of argument continues to next line. Trailing whitespace after `\` is a hard parse error (Phase 12D addition).
- **Let-chains** — qed processor name parsing uses edition-2024 `let … && let` chains in an `if` condition.

**Module facades** — `parse/mod.rs` exposes `parse_program` and `parse_nth_expr` to the rest of `qed-core` and was designed to isolate any future parser-backend feature flag. Currently rd is the sole backend; the chumsky comment at `parse/mod.rs:8–11` is stale.

## Two-Pass Compiler

`compile/mod.rs` transforms a `Program` AST into a `Script` IR. Errors accumulate into `Vec<CompileError>`; warnings into `Vec<CompileWarning>`. The function signature is:

```
compile(program, source, options) -> Result<(Script, Vec<CompileWarning>), Vec<CompileError>>
```

**Pass 1 — Symbol collection:** Iterates all `Statement`s, registering `PatternDef` entries (literal or regex) and `AliasDef` entries into separate `HashMap`s. Forward references work because all definitions are collected before any are resolved.

**Pass 2 — Statement compilation:** Each `SelectAction` is compiled into a `Statement` IR record containing:
- `CompiledSelector` or `CompoundSelector` — pattern resolved, nth compiled, `on_error` set
- `StatementAction` — `Process(Box<dyn Processor>)`, `CopyTo`, `MoveTo`, `Warn`, `Fail`, `DebugCount`, `DebugPrint`
- Optional `CompiledFallback` — `Chain(Box<dyn Processor>)` or `SelectAction`

**Selector registry** — compiled selectors are stored in a flat registry (`Vec<RegistryEntry>`) indexed by `SelectorId`. The engine uses this registry during fragmentation.

**Pattern compilation** — patterns resolve to `PatternMatcher::Literal(String)`, `PatternMatcher::Regex(regex::Regex)`, or `PatternMatcher::NeverMatch` (for regex compile failure with `on_error:skip`).

**`qed:file()` fusion** — `compile_processor_chain` detects a `FileMarker` (via `is_file_marker()`) and fuses it with the immediately following external command into a `FileHandoffProcessor`. The fusion uses a `pending_file_span` flag; the state machine is implicit and may be fragile if `qed:file()` appears in non-standard chain positions.

**AliasRef resolution** — if an `AliasRef` name is not found in the alias table, the compiler silently promotes it to an external PATH-lookup command. This means an alias typo becomes a subprocess invocation with no diagnostic.

**Stream-control actions** — `warn`, `fail`, `debug:count`, `debug:print` are detected by name during processor chain compilation and compiled to `StatementAction` variants rather than `Box<dyn Processor>`. The `_ => unreachable!()` at `compile/mod.rs:884` is safe within the current string-match arm but would silently break if new statement-level processors are added without updating the branch.

## Environment Variable Expansion

`compile/env.rs` implements `${VAR}` substitution called during pattern and argument compilation:

```
expand_env_vars(input: &str, no_env: bool) -> (String, Vec<UnsetVar>)
```

- `\${VAR}` emits `${VAR}` literally (escaped dollar)
- Unset variables expand to empty string and produce an `UnsetVar` record (byte offset into input)
- `no_env: true` skips all expansion and returns input unchanged
- Non-ASCII bytes outside `${...}` sequences are emitted via `bytes[i] as char` — same byte-cast issue as the cursor

Callers in `compile/mod.rs` translate `UnsetVar` offsets relative to the source span; this two-level offset arithmetic is a coupling point that could silently produce wrong diagnostic positions if the span math drifts.

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| Hand-written RD parser | `parse/rd/` recursive descent | chumsky PEG (prototyped, removed in Phase 3) | RD gives precise error spans, simpler recovery, no external compile-time dependency. [confirmed — Phase 3 history] |
| Accumulate compiler errors | `Vec<CompileError>` | Abort on first error | Users can see and fix all errors in one pass. [inferred] |
| Two-pass compiler | Collect definitions first, then compile | Single-pass with forward-ref resolution table | Enables forward references in alias/pattern definitions; simpler than inline lazy resolution. [inferred] |
| AliasRef falls through to external command | Silent promotion to PATH lookup | Hard error on undefined alias | Allows bare command names in scripts without alias declarations; convenience over strictness. [inferred] |
| Env expansion at compile time | `compile/env.rs` during compilation | At parse time; at execution time | Compile-time expansion catches unset-variable warnings before execution; expansion result is baked into compiled patterns. [inferred] |

## Open Questions & Future Decisions

### Resolved
1. ✅ Chumsky vs hand-written RD — RD chosen and chumsky removed (Phase 3).

### Deferred
1. **Non-ASCII byte handling** — `cursor.rs` and `env.rs` both use `as char` byte casting. Should the cursor operate on `char` (Unicode scalars) instead of bytes? Impact on performance and span arithmetic needs evaluation.
2. **Silent alias-typo promotion** — Should unresolved `AliasRef` emit a diagnostic (warning or error) rather than silently becoming a PATH lookup? Strictness vs. ergonomics trade-off.
3. **`qed:file()` fusion fragility** — The `pending_file_span` state machine in `compile_processor_chain` is implicit. Should it be made explicit (a mini-state enum)?
4. **Wide function refactoring** — `compile_fallback` and `compile_simple_selector` each have 8 parameters with `#[allow(clippy::too_many_arguments)]`. A context struct would improve readability.
5. **`detect_nth_duplicates` gaps** — Negative indices and `Step` terms are not checked. Intentional (complexity trade-off) or oversight?

## References

- `qed-core/src/parse/mod.rs`
- `qed-core/src/parse/rd/cursor.rs`
- `qed-core/src/parse/rd/parser.rs`
- `qed-core/src/parse/rd/mod.rs`
- `qed-core/src/compile/mod.rs`
- `qed-core/src/compile/env.rs`
- `docs/qed-design.md` — grammar specification (authoritative)
- `docs/qed-implementation-design.md` — compilation pass design
- `docs/arrows/parser-compiler.md`
- `docs/specs/parser-compiler-specs.md`
