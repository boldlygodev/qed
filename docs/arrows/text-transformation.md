# Arrow: text-transformation

Pure text-in → text-out processors — the core transformation vocabulary of the qed language.

## Status

**OK** — last audited 2026-04-25 (git SHA `ae1b9ec`).
All 25 behavioral specs implemented. No active gaps.

## References

### HLD
- `docs/high-level-design.md` — Approach section (processor primitive)

### LLD
- `docs/llds/text-transformation.md`

### EARS
- `docs/specs/text-transformation-specs.md`

### Tests
- `tests/processors/` — 16 scenarios covering all processors listed below
- `tests/processors-edge-cases/` — 20 scenarios (empty input, no-match, boundary conditions)

### Code
- `qed-core/src/processor/delete.rs`
- `qed-core/src/processor/duplicate.rs`
- `qed-core/src/processor/upper.rs`
- `qed-core/src/processor/lower.rs`
- `qed-core/src/processor/trim.rs`
- `qed-core/src/processor/indent.rs`
- `qed-core/src/processor/dedent.rs`
- `qed-core/src/processor/wrap.rs`
- `qed-core/src/processor/prefix.rs`
- `qed-core/src/processor/suffix.rs`
- `qed-core/src/processor/number.rs`
- `qed-core/src/processor/replace.rs`
- `qed-core/src/processor/substring.rs`
- `qed-core/src/processor/skip.rs`

## Architecture

**Purpose:** Implements the `Processor` trait for all deterministic, side-effect-free text transformations. Each processor receives its selected region as a `&str` and returns a transformed `String` or a `ProcessorError`.

**Key Components:**
1. `DeleteProcessor` — returns empty string (deletion signal to `ChainProcessor`)
2. `DuplicateProcessor` — emits input twice
3. `UpperProcessor` / `LowerProcessor` — Unicode-aware case conversion via `str::to_uppercase/lowercase`
4. `TrimProcessor` — Unicode-aware whitespace strip per line via `map_lines`
5. `IndentProcessor` — prepends N copies of `indent_char` (a `String`, supporting multi-char units)
6. `DedentProcessor` — removes common leading whitespace; skips blank lines for indent calculation
7. `WrapProcessor` — word-wraps at column width; splits on ASCII space only
8. `PrefixProcessor` / `SuffixProcessor` — prepend/append fixed string per line via `map_lines`
9. `NumberProcessor` — right-aligned line numbering with configurable start (i64) and width
10. `ReplaceProcessor` — find/replace with literal, regex, or `Pipeline(Box<dyn Processor>)` replacement
11. `SubstringProcessor` — narrows each line to first match of literal/regex; non-matching lines pass through
12. `SkipProcessor` — identity passthrough; used with `--extract`

## Spec Coverage

| Category | Spec IDs | Implemented | Deferred | Gaps |
|---|---|---|---|---|
| General contract | TXFM-001–TXFM-003 | 3 | 0 | 0 |
| Deletion and structural | TXFM-010–TXFM-012 | 3 | 0 | 0 |
| Case conversion | TXFM-020–TXFM-021 | 2 | 0 | 0 |
| Whitespace and indentation | TXFM-030–TXFM-033 | 4 | 0 | 0 |
| Line formatting | TXFM-040–TXFM-045 | 6 | 0 | 0 |
| Search and replace | TXFM-050–TXFM-056 | 7 | 0 | 0 |
| Non-features | TXFM-060–TXFM-061 | 0 | 2 | 0 |
| **Total** | | **25** | **2** | **0** |

**Summary:** All 25 behavioral specs implemented. Note: TXFM-001 and TXFM-002 are trait-level contract specs; add `@spec TXFM-001, TXFM-002` to `processor/mod.rs` to complete annotation coverage.

## Key Findings

1. **`map_lines` utility split** — 5 processors use `map_lines` (`indent`, `prefix`, `suffix`, `trim`, `substring`); the rest implement custom line logic. `dedent`, `number`, `wrap`, and `replace` need custom loops (multi-line output, accumulation, or pattern-level processing).
2. **`WrapProcessor` ASCII-only word splitting** — `wrap_line` splits on ASCII space only (`wrap.rs:42`). Tabs and other Unicode whitespace are not treated as word separators. Long words exceeding `width` get their own line without hard-breaking.
3. **`ReplaceProcessor::Pipeline` recursion** — Replacement can be `Box<dyn Processor>`, enabling arbitrary nested transforms (e.g. `qed:replace("x", qed:upper())`). If the pipeline contains an `ExternalCommandProcessor`, the replacement gains side effects.
4. **`(Literal, Template)` defensive guard** — `replace.rs:66–70` returns `ProcessorFailed` for this combination at runtime, with a comment that it is "rejected at compile time." The runtime guard exists but should be unreachable.
5. **`SubstringProcessor` first-match-only semantics** — Only the first match per line is kept (RE2 leftmost). Non-matching lines pass through unchanged. Documented explicitly in `processors-edge-cases` tests.
6. **`NumberProcessor.start` is `i64`** — Allows negative start values (e.g. counting from -5), which is non-obvious but intentional.

## Work Required

### Must Fix
*(none — all processors are functionally complete)*

### Should Fix
1. `WrapProcessor` ASCII-only word split: consider Unicode whitespace for correctness (TXFM specs TBD).

### Nice to Have
1. Cache `Vec<char>` alphabet in `RandomProcessor` (owned by `generation` segment, but worth noting the pattern applies here too if any processor has per-call allocation).
