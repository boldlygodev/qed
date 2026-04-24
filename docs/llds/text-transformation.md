# Text Transformation

## Context and Design Philosophy

Implements the `Processor` trait for all deterministic, side-effect-free text transformations — the core vocabulary of the qed language. Each processor receives its selected region as a `&str` and returns a `String` (the replacement) or a `ProcessorError`. No processor in this segment spawns subprocesses, reads the clock, or reads OS randomness. Adding a new transformation processor is intentionally self-contained: create one file, implement the trait, register in `processor/mod.rs` and `compile/mod.rs`.

## Processor Taxonomy

**Deletion / structural:**
- `DeleteProcessor` — returns `Ok("")`; the empty-string return is the deletion signal interpreted by `ChainProcessor` and the execution engine
- `DuplicateProcessor` — returns `format!("{}{}", input, input)`; relies on input being well-formed (ending with `'\n'` for multi-line)
- `SkipProcessor` — identity passthrough (`Ok(input.to_string())`); used with `--extract` to select without transforming

**Case conversion:**
- `UpperProcessor` — `str::to_uppercase()`; Unicode full case folding, not ASCII-only
- `LowerProcessor` — `str::to_lowercase()`; Unicode full case folding, not ASCII-only

**Whitespace and indentation:**
- `TrimProcessor` — `str::trim()` per line via `map_lines`; Unicode whitespace aware
- `IndentProcessor { width: usize, indent_char: String }` — prepends `indent_char.repeat(width)` per line; `indent_char` is a `String` (not `char`), supporting multi-character indent units
- `DedentProcessor` — computes minimum leading whitespace from non-empty lines only; shorter lines pass through unchanged rather than panicking

**Line formatting:**
- `WrapProcessor { width: usize }` — splits on ASCII space only; words longer than `width` get their own line without hard-breaking; empty lines preserved as-is
- `PrefixProcessor { text: String }` — prepends `text` to each line via `map_lines`
- `SuffixProcessor { text: String }` — appends `text` to each line via `map_lines`
- `NumberProcessor { start: i64, width: usize }` — right-aligned `"N: line"` format; `actual_width = max(self.width, max_num.to_string().len())` for consistent alignment; `start` is `i64` allowing negative start values

**Search and replace:**
- `ReplaceProcessor { search: ReplaceSearch, replacement: ReplaceWith }` — see below
- `SubstringProcessor { search: SubstringSearch }` — narrows each line to its first match (RE2 leftmost); non-matching lines pass through unchanged

## map_lines Utility

`processor/mod.rs` exports `map_lines(input: &str, f: impl Fn(&str) -> String) -> String`. It strips a trailing `'\n'` before applying `f` per line, then re-appends it if the original had one. This ensures processors that operate per-line do not accidentally double or drop trailing newlines.

Used by: `indent`, `prefix`, `suffix`, `trim`, `substring`.
Not used by: `dedent`, `number`, `wrap` (all need custom line logic), `replace` (operates on full content), `delete`, `duplicate`, `skip`, `upper`, `lower` (operate on full string).

## Replace Processor

`ReplaceProcessor` is the most complex processor in this segment. It composes two orthogonal dimensions:

**Search:**
- `ReplaceSearch::Literal(String)` — `str::find` / `str::replacen`; not regex-aware
- `ReplaceSearch::Regex(regex::Regex)` — `Regex::find` / `Regex::replace_all`

**Replacement:**
- `ReplaceWith::Literal(String)` — `regex::NoExpand` for the regex-search path to prevent unintended `$` expansion
- `ReplaceWith::Template(String)` — capture group substitution; only valid with `ReplaceSearch::Regex` (the `(Literal, Template)` combination is rejected at compile time; a runtime defensive guard also exists)
- `ReplaceWith::Pipeline(Box<dyn Processor>)` — calls `processor.execute()` on each matched span; enables arbitrary nested transformations (e.g. `qed:replace("x", qed:upper())`). If the pipeline contains an `ExternalCommandProcessor`, the replacement gains side effects. `strip_trailing_newline_if_needed` prevents injecting `'\n'` mid-line when the matched text did not end with one.

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| Empty string as deletion signal | `Ok("")` | Separate `ProcessorResult::Delete` variant | Simplest contract; ChainProcessor already handles the empty-string case for chain short-circuit. [inferred] |
| `map_lines` utility | Shared helper in `processor/mod.rs` | Inline per processor | Consistent trailing-newline semantics without duplication; processors that need custom line logic (wrap, number, dedent) opt out. [inferred] |
| `IndentProcessor.indent_char` as `String` | `String` | `char` | Supports multi-character indent units (e.g. `"\t"`, `"  "`). [inferred] |
| `NumberProcessor.start` as `i64` | `i64` | `usize` | Allows negative start values for reverse-counting scenarios. [inferred] |
| `WrapProcessor` ASCII-space-only split | `split(' ')` on ASCII space | Unicode word-boundary split | Simpler implementation; ASCII space is the dominant word separator in prose. [inferred — possibly an oversight for Unicode content] |
| `ReplaceWith::Pipeline` via `Box<dyn Processor>` | Trait object in replacement | Inline special cases per processor type | Enables arbitrary nesting; consistent with the general processor composition model. [inferred] |
| `SubstringProcessor` first-match only | `str::find` / `Regex::find` (leftmost) | All matches, or last match | RE2 leftmost semantics are predictable; all-matches would produce a different type of output (multiple fragments per line). [inferred] |

## Open Questions & Future Decisions

### Resolved
*(none yet)*

### Deferred
1. **`WrapProcessor` Unicode word splitting** — Should `wrap_line` split on Unicode whitespace (e.g. `char::is_whitespace`) rather than ASCII space only?
2. **`ReplaceWith::Pipeline` side-effect transparency** — When a `Pipeline` contains an `ExternalCommandProcessor`, `ReplaceProcessor` gains subprocess side effects with no indication at the call site. Should this be prohibited at compile time or documented as intended?
3. **`DuplicateProcessor` trailing-newline assumption** — `format!("{}{}", input, input)` assumes well-formed input ending with `'\n'` for multi-line regions. Should this be validated or normalized?

## References

- `qed-core/src/processor/delete.rs`
- `qed-core/src/processor/duplicate.rs`
- `qed-core/src/processor/upper.rs`, `lower.rs`, `trim.rs`
- `qed-core/src/processor/indent.rs`, `dedent.rs`, `wrap.rs`
- `qed-core/src/processor/prefix.rs`, `suffix.rs`, `number.rs`
- `qed-core/src/processor/replace.rs`
- `qed-core/src/processor/substring.rs`
- `qed-core/src/processor/skip.rs`
- `docs/qed-design.md` — processor specifications (authoritative)
- `docs/arrows/text-transformation.md`
- `docs/specs/text-transformation-specs.md`
