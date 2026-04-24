# EARS Specs — Text Transformation

ID prefix: `TXFM`

## General Contract

- [x] TXFM-001: Every text-transformation processor SHALL receive its selected region as `&str` and return a `String` or `ProcessorError`.
- [x] TXFM-002: Text-transformation processors SHALL NOT spawn subprocesses, read the system clock, or read OS randomness.
- [x] TXFM-003: `map_lines(input, fn)` SHALL strip a trailing `'\n'` before applying `fn` per line, then re-append the `'\n'` if the original had one.

## Deletion and Structural

- [x] TXFM-010: `DeleteProcessor.execute()` SHALL return `Ok("")`, signaling deletion to `ChainProcessor` and the execution engine.
- [x] TXFM-011: `DuplicateProcessor.execute()` SHALL return the input concatenated with itself (`format!("{}{}", input, input)`).
- [x] TXFM-012: `SkipProcessor.execute()` SHALL return the input unchanged; it is an identity passthrough.

## Case Conversion

- [x] TXFM-020: `UpperProcessor.execute()` SHALL apply Unicode full case folding to the entire input via `str::to_uppercase()`.
- [x] TXFM-021: `LowerProcessor.execute()` SHALL apply Unicode full case folding to the entire input via `str::to_lowercase()`.

## Whitespace and Indentation

- [x] TXFM-030: `TrimProcessor.execute()` SHALL apply `str::trim()` per line using `map_lines`; it SHALL be Unicode-whitespace aware.
- [x] TXFM-031: `IndentProcessor.execute()` SHALL prepend `indent_char.repeat(width)` to each line via `map_lines`; `indent_char` SHALL be a `String` to support multi-character indent units.
- [x] TXFM-032: `DedentProcessor.execute()` SHALL compute the minimum leading whitespace from non-empty lines only, then strip that many characters from the start of each line.
- [x] TXFM-033: WHEN a line in `DedentProcessor` input is shorter than the minimum indent, the processor SHALL pass that line through unchanged rather than panicking.

## Line Formatting

- [x] TXFM-040: `WrapProcessor.execute()` SHALL split on ASCII space characters only; words longer than `width` SHALL each occupy their own line without hard-breaking at the width boundary.
- [x] TXFM-041: `WrapProcessor.execute()` SHALL preserve empty lines as-is.
- [x] TXFM-042: `PrefixProcessor.execute()` SHALL prepend its `text` field to each line via `map_lines`.
- [x] TXFM-043: `SuffixProcessor.execute()` SHALL append its `text` field to each line via `map_lines`.
- [x] TXFM-044: `NumberProcessor.execute()` SHALL produce right-aligned `"N: line"` format where the actual width is `max(self.width, max_num.to_string().len())` for consistent column alignment.
- [x] TXFM-045: `NumberProcessor.start` SHALL be `i64` to allow negative start values.

## Search and Replace

- [x] TXFM-050: `ReplaceProcessor` SHALL support two search modes: `Literal(String)` (using `str::find` / `str::replacen`) and `Regex(regex::Regex)` (using `Regex::find` / `Regex::replace_all`).
- [x] TXFM-051: WHEN `ReplaceWith::Literal` is used with `ReplaceSearch::Regex`, the replacement SHALL use `regex::NoExpand` to prevent unintended `$` capture-group expansion.
- [x] TXFM-052: `ReplaceWith::Template` SHALL support capture-group substitution; it SHALL only be valid with `ReplaceSearch::Regex`.
- [x] TXFM-053: WHEN the `(Literal search, Template replacement)` combination is attempted, the compiler SHALL reject it as an error.
- [x] TXFM-054: `ReplaceWith::Pipeline(Box<dyn Processor>)` SHALL call `processor.execute()` on each matched span, enabling arbitrary nested transformations.
- [x] TXFM-055: WHEN `ReplaceWith::Pipeline` produces output that would inject a mid-line `'\n'`, `strip_trailing_newline_if_needed` SHALL strip the trailing newline from the pipeline result.
- [x] TXFM-056: `SubstringProcessor.execute()` SHALL narrow each line to its first (RE2 leftmost) match; non-matching lines SHALL pass through unchanged.

## Non-Features

- [D] TXFM-060: `WrapProcessor` SHALL NOT split on Unicode whitespace by default; ASCII space splitting is the intended behavior.
- [D] TXFM-061: `SubstringProcessor` SHALL NOT return all matches or the last match; leftmost-first semantics are intentional.

## References

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
- `docs/llds/text-transformation.md`
