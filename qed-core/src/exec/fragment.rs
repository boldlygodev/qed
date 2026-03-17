//! Fragmentation algorithm — partitions a `Buffer` into `Passthrough` and
//! `Selected` regions based on compiled selectors.
//!
//! The algorithm has four steps:
//!
//! 1. **Match collection** — run each selector against every line in the
//!    buffer, producing a list of `MatchResult`s (line ranges tagged with
//!    statement/selector IDs).
//!
//! 2. **Boundary decomposition** — convert each match range into a pair of
//!    `BoundaryEvent`s (start and end) at specific line positions.
//!
//! 3. **Sort** — order events by line number, with start events before end
//!    events at the same line, then by statement ID.
//!
//! 4. **Sweep** — walk the sorted events, maintaining an active set of
//!    (statement, selector) tags. Each time the active set changes, emit a
//!    new fragment for the region since the last change.

use std::collections::BTreeSet;

use crate::compile::{
    CompoundSelector, CompiledPattern, CompiledSelector, PatternMatcher, RegistryEntry, SelectorOp,
};
use crate::parse::ast::NthTerm;
use crate::{SelectorId, StatementId};

use super::{Buffer, Fragment, FragmentContent, FragmentList, LineRange};

// ── Public API ─────────────────────────────────────────────────────

/// Fragment a buffer according to the given selector requests.
///
/// `requests` maps each statement to its selector.
/// `registry` is the flat selector registry from `Script`.
pub(crate) fn fragment(
    buffer: &Buffer,
    requests: &[(StatementId, SelectorId)],
    registry: &[RegistryEntry],
) -> FragmentList {
    if buffer.line_count() == 0 {
        return Vec::new();
    }

    // Step 1 — collect matches from all selectors
    let matches = collect_all_matches(buffer, requests, registry);

    // Step 2 — decompose into boundary events
    let events = decompose_events(&matches);

    // Step 3+4 — sort and sweep
    sweep(events, buffer.line_count())
}

// ── Match collection ───────────────────────────────────────────────

/// A single match: a line range tagged with the statement and selector
/// that produced it.
struct MatchResult {
    range: LineRange,
    statement_id: StatementId,
    selector_id: SelectorId,
}

fn collect_all_matches(
    buffer: &Buffer,
    requests: &[(StatementId, SelectorId)],
    registry: &[RegistryEntry],
) -> Vec<MatchResult> {
    let mut all_matches = Vec::new();

    for &(stmt_id, sel_id) in requests {
        let entry = &registry[sel_id.value()];
        match entry {
            RegistryEntry::Simple(selector) => {
                all_matches.extend(collect_simple_matches(buffer, selector, stmt_id));
            }
            RegistryEntry::Compound(compound) => {
                all_matches.extend(collect_compound_matches(
                    buffer, compound, registry, stmt_id,
                ));
            }
        }
    }

    all_matches
}

fn collect_simple_matches(
    buffer: &Buffer,
    selector: &CompiledSelector,
    stmt_id: StatementId,
) -> Vec<MatchResult> {
    let sel_id = selector.id;

    let ranges = match &selector.op {
        SelectorOp::At { pattern, nth } => collect_at(buffer, pattern, nth.as_deref()),
        SelectorOp::After { pattern } => collect_after(buffer, pattern),
        SelectorOp::Before { pattern } => collect_before(buffer, pattern),
        SelectorOp::From { pattern } => collect_from(buffer, pattern),
        SelectorOp::To { pattern } => collect_to(buffer, pattern),
    };

    ranges
        .into_iter()
        .map(|range| MatchResult {
            range,
            statement_id: stmt_id,
            selector_id: sel_id,
        })
        .collect()
}

fn collect_compound_matches(
    buffer: &Buffer,
    compound: &CompoundSelector,
    registry: &[RegistryEntry],
    stmt_id: StatementId,
) -> Vec<MatchResult> {
    // Run each step, intersect ranges
    let mut result_range: Option<LineRange> = None;

    for &step_id in &compound.steps {
        let step_entry = &registry[step_id.value()];
        let step_selector = match step_entry {
            RegistryEntry::Simple(s) => s,
            RegistryEntry::Compound(_) => {
                // Compound steps should always resolve to simple selectors
                continue;
            }
        };

        let ranges = match &step_selector.op {
            SelectorOp::At { pattern, nth } => collect_at(buffer, pattern, nth.as_deref()),
            SelectorOp::After { pattern } => collect_after(buffer, pattern),
            SelectorOp::Before { pattern } => collect_before(buffer, pattern),
            SelectorOp::From { pattern } => collect_from(buffer, pattern),
            SelectorOp::To { pattern } => collect_to(buffer, pattern),
        };

        // Merge step ranges into a single range via union, then intersect with accumulated
        let step_range = union_ranges(&ranges);
        result_range = match (result_range, step_range) {
            (None, step) => step,
            (Some(_), None) => None,
            (Some(acc), Some(step)) => intersect_ranges(acc, step),
        };

        // Short-circuit if intersection is already empty
        if result_range.is_none() {
            break;
        }
    }

    match result_range {
        Some(range) if range.start < range.end => vec![MatchResult {
            range,
            statement_id: stmt_id,
            selector_id: compound.id,
        }],
        _ => Vec::new(),
    }
}

fn union_ranges(ranges: &[LineRange]) -> Option<LineRange> {
    if ranges.is_empty() {
        return None;
    }
    let start = ranges.iter().map(|r| r.start).min().expect("non-empty");
    let end = ranges.iter().map(|r| r.end).max().expect("non-empty");
    Some(LineRange { start, end })
}

fn intersect_ranges(a: LineRange, b: LineRange) -> Option<LineRange> {
    let start = a.start.max(b.start);
    let end = a.end.min(b.end);
    if start < end {
        Some(LineRange { start, end })
    } else {
        None
    }
}

// ── Per-op match collectors ────────────────────────────────────────

/// `at(pattern)` — selects each line that matches `pattern`, producing
/// one single-line range per match. An optional `nth` filter narrows
/// the matches to specific ordinal positions (1-based, negative = from end).
fn collect_at(
    buffer: &Buffer,
    pattern: &CompiledPattern,
    nth: Option<&[NthTerm]>,
) -> Vec<LineRange> {
    let matching_lines: Vec<usize> = (0..buffer.line_count())
        .filter(|&i| pattern_matches(pattern, buffer.line(i)))
        .collect();

    let selected = match nth {
        Some(terms) => apply_nth_filter(&matching_lines, terms),
        None => matching_lines,
    };

    selected
        .into_iter()
        .map(|i| LineRange { start: i, end: i + 1 })
        .collect()
}

/// `after(pattern)` — selects the zero-width insertion point immediately
/// after each matching line. The range is empty (`start == end`),
/// representing a position between lines rather than a line itself.
fn collect_after(buffer: &Buffer, pattern: &CompiledPattern) -> Vec<LineRange> {
    (0..buffer.line_count())
        .filter(|&i| pattern_matches(pattern, buffer.line(i)))
        .map(|i| LineRange {
            start: i + 1,
            end: i + 1,
        })
        .collect()
}

/// `before(pattern)` — selects the zero-width insertion point immediately
/// before each matching line. The range is empty (`start == end`).
fn collect_before(buffer: &Buffer, pattern: &CompiledPattern) -> Vec<LineRange> {
    (0..buffer.line_count())
        .filter(|&i| pattern_matches(pattern, buffer.line(i)))
        .map(|i| LineRange { start: i, end: i })
        .collect()
}

/// `from(pattern)` — selects from the matching line to the end of the
/// buffer. When `pattern.inclusive` is true the matched line itself is
/// included; when false, selection begins on the line after the match.
fn collect_from(buffer: &Buffer, pattern: &CompiledPattern) -> Vec<LineRange> {
    let line_count = buffer.line_count();
    (0..line_count)
        .filter(|&i| pattern_matches(pattern, buffer.line(i)))
        .map(|i| {
            if pattern.inclusive {
                LineRange {
                    start: i,
                    end: line_count,
                }
            } else {
                LineRange {
                    start: i + 1,
                    end: line_count,
                }
            }
        })
        .collect()
}

/// `to(pattern)` — selects from the beginning of the buffer up to the
/// matching line. When `pattern.inclusive` is true the matched line is
/// included; when false, selection ends just before it.
fn collect_to(buffer: &Buffer, pattern: &CompiledPattern) -> Vec<LineRange> {
    (0..buffer.line_count())
        .filter(|&i| pattern_matches(pattern, buffer.line(i)))
        .map(|i| {
            if pattern.inclusive {
                LineRange { start: 0, end: i + 1 }
            } else {
                LineRange { start: 0, end: i }
            }
        })
        .collect()
}

// ── Pattern matching ───────────────────────────────────────────────

fn pattern_matches(pattern: &CompiledPattern, line: &str) -> bool {
    // Strip trailing newline for matching purposes
    let line = line.strip_suffix('\n').unwrap_or(line);

    let raw_match = match &pattern.matcher {
        PatternMatcher::Literal(lit) => line.contains(lit.as_str()),
        PatternMatcher::Regex(re) => re.is_match(line),
    };

    if pattern.negated { !raw_match } else { raw_match }
}

// ── Nth filtering ──────────────────────────────────────────────────

/// Filter a set of matching line indices to only those at specified ordinal
/// positions. Terms are evaluated as a union: `1,3,-1` selects the first,
/// third, and last match. Step terms (`2n+1`) generate a repeating series.
/// All positions are 1-based; negative values count from the end.
fn apply_nth_filter(matching_lines: &[usize], terms: &[NthTerm]) -> Vec<usize> {
    let count = matching_lines.len();
    if count == 0 {
        return Vec::new();
    }

    let mut selected_indices: BTreeSet<usize> = BTreeSet::new();

    for term in terms {
        match *term {
            NthTerm::Integer(n) => {
                let idx = resolve_1based(n, count);
                if let Some(i) = idx {
                    selected_indices.insert(i);
                }
            }
            NthTerm::Range { start, end } => {
                let s = resolve_1based(start, count);
                let e = resolve_1based(end, count);
                if let (Some(s), Some(e)) = (s, e) {
                    let lo = s.min(e);
                    let hi = s.max(e);
                    for i in lo..=hi {
                        selected_indices.insert(i);
                    }
                }
            }
            NthTerm::Step { coefficient, offset } => {
                // Generates 1-based indices: coefficient * k + offset for k = 0, 1, 2, ...
                // Then converts to 0-based
                for k in 0.. {
                    let one_based = coefficient * k + offset;
                    if one_based < 1 {
                        continue;
                    }
                    let zero_based = (one_based - 1) as usize;
                    if zero_based >= count {
                        break;
                    }
                    selected_indices.insert(zero_based);
                }
            }
        }
    }

    // Return the actual line indices in source order
    selected_indices
        .into_iter()
        .map(|i| matching_lines[i])
        .collect()
}

/// Resolve a 1-based index (possibly negative) to a 0-based index.
fn resolve_1based(n: i64, count: usize) -> Option<usize> {
    if n > 0 {
        let idx = (n - 1) as usize;
        if idx < count { Some(idx) } else { None }
    } else if n < 0 {
        let from_end = (-n) as usize;
        if from_end <= count {
            Some(count - from_end)
        } else {
            None
        }
    } else {
        None // 0 is not a valid 1-based index
    }
}

// ── Boundary events ────────────────────────────────────────────────

/// Whether a boundary event opens or closes a selected region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EventKind {
    Start,
    End,
}

/// A point in the line-index space where the active set of selectors changes.
/// Each `MatchResult` decomposes into one `Start` and one `End` event.
#[derive(Debug, Clone, Copy)]
struct BoundaryEvent {
    line: usize,
    kind: EventKind,
    statement_id: StatementId,
    selector_id: SelectorId,
}

fn decompose_events(matches: &[MatchResult]) -> Vec<BoundaryEvent> {
    let mut events = Vec::with_capacity(matches.len() * 2);

    for m in matches {
        events.push(BoundaryEvent {
            line: m.range.start,
            kind: EventKind::Start,
            statement_id: m.statement_id,
            selector_id: m.selector_id,
        });
        events.push(BoundaryEvent {
            line: m.range.end,
            kind: EventKind::End,
            statement_id: m.statement_id,
            selector_id: m.selector_id,
        });
    }

    // Sort: line ascending, Start before End, StatementId ascending
    events.sort_by(|a, b| {
        a.line
            .cmp(&b.line)
            .then_with(|| {
                let a_ord = match a.kind {
                    EventKind::Start => 0,
                    EventKind::End => 1,
                };
                let b_ord = match b.kind {
                    EventKind::Start => 0,
                    EventKind::End => 1,
                };
                a_ord.cmp(&b_ord)
            })
            .then_with(|| a.statement_id.cmp(&b.statement_id))
    });

    events
}

// ── Sweep ──────────────────────────────────────────────────────────

/// Walk sorted boundary events and emit fragments.
///
/// Maintains a `BTreeSet` of active `(StatementId, SelectorId)` tags.
/// Each time the active set changes at a new line position, the region
/// since the previous change is emitted as either `Passthrough` (empty
/// active set) or `Selected` (with the current tags).
fn sweep(events: Vec<BoundaryEvent>, line_count: usize) -> FragmentList {
    let mut fragments = Vec::new();
    let mut active: BTreeSet<(StatementId, SelectorId)> = BTreeSet::new();
    let mut prev_line: usize = 0;

    for event in &events {
        let current_line = event.line;

        // Emit fragment for [prev_line..current_line) if non-empty
        if current_line > prev_line {
            let range = LineRange {
                start: prev_line,
                end: current_line,
            };
            if active.is_empty() {
                fragments.push(Fragment::Passthrough(FragmentContent::Borrowed(range)));
            } else {
                fragments.push(Fragment::Selected {
                    content: FragmentContent::Borrowed(range),
                    tags: active.iter().copied().collect(),
                });
            }
            prev_line = current_line;
        }

        // Update active set
        match event.kind {
            EventKind::Start => {
                active.insert((event.statement_id, event.selector_id));
            }
            EventKind::End => {
                active.remove(&(event.statement_id, event.selector_id));
            }
        }
    }

    // Trailing passthrough
    if prev_line < line_count {
        let range = LineRange {
            start: prev_line,
            end: line_count,
        };
        if active.is_empty() {
            fragments.push(Fragment::Passthrough(FragmentContent::Borrowed(range)));
        } else {
            fragments.push(Fragment::Selected {
                content: FragmentContent::Borrowed(range),
                tags: active.iter().copied().collect(),
            });
        }
    }

    fragments
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile::OnError;

    /// Helper: build a simple CompiledSelector with a literal pattern.
    fn literal_at(id: usize, text: &str, nth: Option<Vec<NthTerm>>) -> CompiledSelector {
        CompiledSelector {
            id: SelectorId::new(id),
            op: SelectorOp::At {
                pattern: CompiledPattern {
                    matcher: PatternMatcher::Literal(text.to_string()),
                    negated: false,
                    inclusive: false,
                },
                nth,
            },
            on_error: OnError::Fail,
        }
    }

    fn literal_pattern(text: &str, negated: bool, inclusive: bool) -> CompiledPattern {
        CompiledPattern {
            matcher: PatternMatcher::Literal(text.to_string()),
            negated,
            inclusive,
        }
    }

    /// Extract the LineRange from a Borrowed fragment content.
    fn borrowed_range(f: &Fragment) -> LineRange {
        match f {
            Fragment::Passthrough(FragmentContent::Borrowed(r)) => *r,
            Fragment::Selected {
                content: FragmentContent::Borrowed(r),
                ..
            } => *r,
            _ => panic!("expected Borrowed fragment"),
        }
    }

    fn is_passthrough(f: &Fragment) -> bool {
        matches!(f, Fragment::Passthrough(_))
    }

    fn tags_of(f: &Fragment) -> &[(StatementId, SelectorId)] {
        match f {
            Fragment::Selected { tags, .. } => tags,
            _ => panic!("expected Selected fragment"),
        }
    }

    #[test]
    fn single_selector_single_match() {
        let buf = Buffer::new("aaa\nbb\nccc\n".into());
        let selector = literal_at(0, "bb", None);
        let registry = vec![RegistryEntry::Simple(selector)];
        let requests = vec![(StatementId::new(0), SelectorId::new(0))];

        let frags = fragment(&buf, &requests, &registry);

        assert_eq!(frags.len(), 3);

        assert!(is_passthrough(&frags[0]));
        assert_eq!(borrowed_range(&frags[0]), LineRange { start: 0, end: 1 });

        assert!(!is_passthrough(&frags[1]));
        assert_eq!(borrowed_range(&frags[1]), LineRange { start: 1, end: 2 });
        assert_eq!(
            tags_of(&frags[1]),
            &[(StatementId::new(0), SelectorId::new(0))]
        );

        assert!(is_passthrough(&frags[2]));
        assert_eq!(borrowed_range(&frags[2]), LineRange { start: 2, end: 3 });
    }

    #[test]
    fn single_selector_no_match() {
        let buf = Buffer::new("aaa\nbb\n".into());
        let selector = literal_at(0, "zzz", None);
        let registry = vec![RegistryEntry::Simple(selector)];
        let requests = vec![(StatementId::new(0), SelectorId::new(0))];

        let frags = fragment(&buf, &requests, &registry);

        assert_eq!(frags.len(), 1);
        assert!(is_passthrough(&frags[0]));
        assert_eq!(borrowed_range(&frags[0]), LineRange { start: 0, end: 2 });
    }

    #[test]
    fn two_overlapping_selectors() {
        // stmt 0: at("bb") selects line 1
        // stmt 1: from("aaa") inclusive selects lines 0..3
        // line 1 should have both tags
        let buf = Buffer::new("aaa\nbb\nccc\n".into());

        let sel0 = literal_at(0, "bb", None);
        let sel1 = CompiledSelector {
            id: SelectorId::new(1),
            op: SelectorOp::From {
                pattern: literal_pattern("aaa", false, true),
            },
            on_error: OnError::Fail,
        };

        let registry = vec![
            RegistryEntry::Simple(sel0),
            RegistryEntry::Simple(sel1),
        ];
        let requests = vec![
            (StatementId::new(0), SelectorId::new(0)),
            (StatementId::new(1), SelectorId::new(1)),
        ];

        let frags = fragment(&buf, &requests, &registry);

        // Expected: [Selected(0..1, {stmt1}), Selected(1..2, {stmt0, stmt1}), Selected(2..3, {stmt1})]
        assert_eq!(frags.len(), 3);

        // Line 0: only stmt 1
        assert!(!is_passthrough(&frags[0]));
        assert_eq!(borrowed_range(&frags[0]), LineRange { start: 0, end: 1 });
        assert_eq!(
            tags_of(&frags[0]),
            &[(StatementId::new(1), SelectorId::new(1))]
        );

        // Line 1: both stmt 0 and stmt 1
        assert!(!is_passthrough(&frags[1]));
        assert_eq!(borrowed_range(&frags[1]), LineRange { start: 1, end: 2 });
        let t = tags_of(&frags[1]);
        assert_eq!(t.len(), 2);
        assert!(t.contains(&(StatementId::new(0), SelectorId::new(0))));
        assert!(t.contains(&(StatementId::new(1), SelectorId::new(1))));

        // Line 2: only stmt 1
        assert!(!is_passthrough(&frags[2]));
        assert_eq!(borrowed_range(&frags[2]), LineRange { start: 2, end: 3 });
        assert_eq!(
            tags_of(&frags[2]),
            &[(StatementId::new(1), SelectorId::new(1))]
        );
    }

    #[test]
    fn nth_second_match() {
        // at("x", nth:2) on "x\ny\nx\nz\nx\n" → only second "x" (line 2) selected
        let buf = Buffer::new("x\ny\nx\nz\nx\n".into());

        let selector = literal_at(0, "x", Some(vec![NthTerm::Integer(2)]));
        let registry = vec![RegistryEntry::Simple(selector)];
        let requests = vec![(StatementId::new(0), SelectorId::new(0))];

        let frags = fragment(&buf, &requests, &registry);

        assert_eq!(frags.len(), 3);

        assert!(is_passthrough(&frags[0]));
        assert_eq!(borrowed_range(&frags[0]), LineRange { start: 0, end: 2 });

        assert!(!is_passthrough(&frags[1]));
        assert_eq!(borrowed_range(&frags[1]), LineRange { start: 2, end: 3 });

        assert!(is_passthrough(&frags[2]));
        assert_eq!(borrowed_range(&frags[2]), LineRange { start: 3, end: 5 });
    }

    #[test]
    fn from_to_compound() {
        // from("start") exclusive > to("end") exclusive on "aaa\nstart\nmid\nend\nzzz\n"
        // from("start") exclusive → lines 2..5
        // to("end") exclusive → lines 0..3
        // intersection → lines 2..3 → "mid"
        let buf = Buffer::new("aaa\nstart\nmid\nend\nzzz\n".into());

        let sel_from = CompiledSelector {
            id: SelectorId::new(0),
            op: SelectorOp::From {
                pattern: literal_pattern("start", false, false),
            },
            on_error: OnError::Fail,
        };
        let sel_to = CompiledSelector {
            id: SelectorId::new(1),
            op: SelectorOp::To {
                pattern: literal_pattern("end", false, false),
            },
            on_error: OnError::Fail,
        };
        let compound = CompoundSelector {
            id: SelectorId::new(2),
            steps: vec![SelectorId::new(0), SelectorId::new(1)],
        };

        let registry = vec![
            RegistryEntry::Simple(sel_from),
            RegistryEntry::Simple(sel_to),
            RegistryEntry::Compound(compound),
        ];
        let requests = vec![(StatementId::new(0), SelectorId::new(2))];

        let frags = fragment(&buf, &requests, &registry);

        assert_eq!(frags.len(), 3);

        assert!(is_passthrough(&frags[0]));
        assert_eq!(borrowed_range(&frags[0]), LineRange { start: 0, end: 2 });

        assert!(!is_passthrough(&frags[1]));
        assert_eq!(borrowed_range(&frags[1]), LineRange { start: 2, end: 3 });
        assert_eq!(
            tags_of(&frags[1]),
            &[(StatementId::new(0), SelectorId::new(2))]
        );

        assert!(is_passthrough(&frags[2]));
        assert_eq!(borrowed_range(&frags[2]), LineRange { start: 3, end: 5 });
    }

    #[test]
    fn negated_pattern() {
        // at(!"skip") on "keep\nskip\nkeep\n" → lines 0 and 2 selected
        let buf = Buffer::new("keep\nskip\nkeep\n".into());

        let selector = CompiledSelector {
            id: SelectorId::new(0),
            op: SelectorOp::At {
                pattern: literal_pattern("skip", true, false),
                nth: None,
            },
            on_error: OnError::Fail,
        };
        let registry = vec![RegistryEntry::Simple(selector)];
        let requests = vec![(StatementId::new(0), SelectorId::new(0))];

        let frags = fragment(&buf, &requests, &registry);

        assert_eq!(frags.len(), 3);

        // Line 0: selected (not "skip")
        assert!(!is_passthrough(&frags[0]));
        assert_eq!(borrowed_range(&frags[0]), LineRange { start: 0, end: 1 });

        // Line 1: passthrough (matches "skip", negated → not selected)
        assert!(is_passthrough(&frags[1]));
        assert_eq!(borrowed_range(&frags[1]), LineRange { start: 1, end: 2 });

        // Line 2: selected (not "skip")
        assert!(!is_passthrough(&frags[2]));
        assert_eq!(borrowed_range(&frags[2]), LineRange { start: 2, end: 3 });
    }

    #[test]
    fn empty_buffer() {
        let buf = Buffer::new(String::new());
        let selector = literal_at(0, "x", None);
        let registry = vec![RegistryEntry::Simple(selector)];
        let requests = vec![(StatementId::new(0), SelectorId::new(0))];

        let frags = fragment(&buf, &requests, &registry);
        assert!(frags.is_empty());
    }

    #[test]
    fn nth_negative_index() {
        // at("x", nth:-1) on "x\ny\nx\nz\nx\n" → last "x" (line 4) selected
        let buf = Buffer::new("x\ny\nx\nz\nx\n".into());

        let selector = literal_at(0, "x", Some(vec![NthTerm::Integer(-1)]));
        let registry = vec![RegistryEntry::Simple(selector)];
        let requests = vec![(StatementId::new(0), SelectorId::new(0))];

        let frags = fragment(&buf, &requests, &registry);

        assert_eq!(frags.len(), 2);

        assert!(is_passthrough(&frags[0]));
        assert_eq!(borrowed_range(&frags[0]), LineRange { start: 0, end: 4 });

        assert!(!is_passthrough(&frags[1]));
        assert_eq!(borrowed_range(&frags[1]), LineRange { start: 4, end: 5 });
    }

    #[test]
    fn nth_step_filter() {
        // at("x", nth:2n+1) selects 1st, 3rd, 5th... matches (odd positions)
        // "x\ny\nx\nz\nx\nw\nx\n" has "x" at lines 0, 2, 4, 6
        // 2n+1: positions 1, 3 → lines 0, 4
        let buf = Buffer::new("x\ny\nx\nz\nx\nw\nx\n".into());

        let selector = literal_at(0, "x", Some(vec![NthTerm::Step { coefficient: 2, offset: 1 }]));
        let registry = vec![RegistryEntry::Simple(selector)];
        let requests = vec![(StatementId::new(0), SelectorId::new(0))];

        let frags = fragment(&buf, &requests, &registry);

        // Selected: lines 0 and 4
        // Expected: Selected(0..1), Pass(1..4), Selected(4..5), Pass(5..7)
        assert_eq!(frags.len(), 4);

        assert!(!is_passthrough(&frags[0]));
        assert_eq!(borrowed_range(&frags[0]), LineRange { start: 0, end: 1 });

        assert!(is_passthrough(&frags[1]));
        assert_eq!(borrowed_range(&frags[1]), LineRange { start: 1, end: 4 });

        assert!(!is_passthrough(&frags[2]));
        assert_eq!(borrowed_range(&frags[2]), LineRange { start: 4, end: 5 });

        assert!(is_passthrough(&frags[3]));
        assert_eq!(borrowed_range(&frags[3]), LineRange { start: 5, end: 7 });
    }

    #[test]
    fn from_inclusive_vs_exclusive() {
        let buf = Buffer::new("aaa\ntarget\nafter\n".into());

        // Inclusive: includes the matched line
        let sel_inc = CompiledSelector {
            id: SelectorId::new(0),
            op: SelectorOp::From {
                pattern: literal_pattern("target", false, true),
            },
            on_error: OnError::Fail,
        };
        let registry = vec![RegistryEntry::Simple(sel_inc)];
        let requests = vec![(StatementId::new(0), SelectorId::new(0))];
        let frags = fragment(&buf, &requests, &registry);

        assert_eq!(frags.len(), 2);
        assert!(is_passthrough(&frags[0]));
        assert_eq!(borrowed_range(&frags[0]), LineRange { start: 0, end: 1 });
        assert!(!is_passthrough(&frags[1]));
        assert_eq!(borrowed_range(&frags[1]), LineRange { start: 1, end: 3 });

        // Exclusive: skips the matched line
        let sel_exc = CompiledSelector {
            id: SelectorId::new(0),
            op: SelectorOp::From {
                pattern: literal_pattern("target", false, false),
            },
            on_error: OnError::Fail,
        };
        let registry = vec![RegistryEntry::Simple(sel_exc)];
        let frags = fragment(&buf, &requests, &registry);

        assert_eq!(frags.len(), 2);
        assert!(is_passthrough(&frags[0]));
        assert_eq!(borrowed_range(&frags[0]), LineRange { start: 0, end: 2 });
        assert!(!is_passthrough(&frags[1]));
        assert_eq!(borrowed_range(&frags[1]), LineRange { start: 2, end: 3 });
    }

    #[test]
    fn to_inclusive_vs_exclusive() {
        let buf = Buffer::new("before\ntarget\nafter\n".into());

        // Inclusive: includes the matched line
        let sel_inc = CompiledSelector {
            id: SelectorId::new(0),
            op: SelectorOp::To {
                pattern: literal_pattern("target", false, true),
            },
            on_error: OnError::Fail,
        };
        let registry = vec![RegistryEntry::Simple(sel_inc)];
        let requests = vec![(StatementId::new(0), SelectorId::new(0))];
        let frags = fragment(&buf, &requests, &registry);

        assert_eq!(frags.len(), 2);
        assert!(!is_passthrough(&frags[0]));
        assert_eq!(borrowed_range(&frags[0]), LineRange { start: 0, end: 2 });
        assert!(is_passthrough(&frags[1]));
        assert_eq!(borrowed_range(&frags[1]), LineRange { start: 2, end: 3 });

        // Exclusive: excludes the matched line
        let sel_exc = CompiledSelector {
            id: SelectorId::new(0),
            op: SelectorOp::To {
                pattern: literal_pattern("target", false, false),
            },
            on_error: OnError::Fail,
        };
        let registry = vec![RegistryEntry::Simple(sel_exc)];
        let frags = fragment(&buf, &requests, &registry);

        assert_eq!(frags.len(), 2);
        assert!(!is_passthrough(&frags[0]));
        assert_eq!(borrowed_range(&frags[0]), LineRange { start: 0, end: 1 });
        assert!(is_passthrough(&frags[1]));
        assert_eq!(borrowed_range(&frags[1]), LineRange { start: 1, end: 3 });
    }
}
