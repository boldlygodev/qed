use crate::parse::ast::{
    NthExpr, NthTerm, PatternRef, PatternRefValue, PatternValue, ProcessorChain, Program,
    QedProcessor, SelectActionNode, Selector, SelectorOp, SimpleSelector, Statement,
};
use crate::parse::error::{ParseError, ParseResult};
use crate::span::{Span, Spanned};

use super::cursor::Cursor;

// ── Program parser ──────────────────────────────────────────────────

/// Parse a complete qed program from source text.
pub(super) fn parse_program(source: &str) -> Result<Program, Vec<ParseError>> {
    let mut cursor = Cursor::new(source);
    let mut statements: Vec<Spanned<Statement>> = Vec::new();
    let mut errors: Vec<ParseError> = Vec::new();

    eat_whitespace_and_newlines(&mut cursor);

    while !cursor.is_eof() {
        let start = cursor.pos();
        match parse_statement(&mut cursor) {
            Ok(stmt) => statements.push(stmt),
            Err(e) => {
                errors.extend(e);
                // Try to recover by skipping to next line
                skip_to_newline(&mut cursor);
            }
        }
        eat_whitespace_and_newlines(&mut cursor);

        // Safety: ensure we make progress
        if cursor.pos() == start {
            let found = cursor
                .remaining()
                .chars()
                .next()
                .map_or("end of input".into(), |c| c.to_string());
            errors.push(ParseError::UnexpectedToken {
                expected: "statement".into(),
                found,
                span: cursor.span_from(start),
            });
            cursor.advance();
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(Program {
        shebang: None,
        statements,
    })
}

/// Parse a single statement (currently only select-action).
fn parse_statement(cursor: &mut Cursor) -> Result<Spanned<Statement>, Vec<ParseError>> {
    let start = cursor.pos();
    let select_action = parse_select_action(cursor)?;
    let span = cursor.span_from(start);
    Ok(Spanned {
        node: Statement::SelectAction(select_action),
        span,
    })
}

/// Parse `selector | processor_chain`.
fn parse_select_action(cursor: &mut Cursor) -> Result<SelectActionNode, Vec<ParseError>> {
    let selector = parse_selector(cursor).map_err(|e| vec![e])?;

    cursor.eat_whitespace();

    let chain = if cursor.eat_char(b'|') {
        cursor.eat_whitespace();
        let chain_start = cursor.pos();
        let chain = parse_processor_chain(cursor).map_err(|e| vec![e])?;
        let chain_span = cursor.span_from(chain_start);
        Some(Spanned {
            node: chain,
            span: chain_span,
        })
    } else {
        None
    };

    Ok(SelectActionNode {
        selector,
        chain,
        fallback: None,
    })
}

/// Parse a selector: `op("pattern")` — currently supports `at("literal")`.
fn parse_selector(cursor: &mut Cursor) -> Result<Spanned<Selector>, ParseError> {
    let start = cursor.pos();

    let op = parse_selector_op(cursor)?;

    if !cursor.eat_char(b'(') {
        return Err(ParseError::UnexpectedToken {
            expected: "'(' after selector name".into(),
            found: cursor
                .remaining()
                .chars()
                .next()
                .map_or("end of input".into(), |c| c.to_string()),
            span: cursor.span_from(cursor.pos()),
        });
    }

    cursor.eat_whitespace();

    let pattern = if cursor.peek() == Some(b'"') {
        let pat_start = cursor.pos();
        let lit = cursor.eat_string_literal().ok_or_else(|| ParseError::UnexpectedEof {
            expected: "closing '\"' for string literal".into(),
            span: cursor.span_from(pat_start),
        })?;
        let pat_span = cursor.span_from(pat_start);
        Some(Spanned {
            node: PatternRef {
                value: PatternRefValue::Inline(PatternValue::String(lit)),
                negated: false,
                inclusive: false,
            },
            span: pat_span,
        })
    } else if cursor.peek() == Some(b')') {
        // No pattern — bare selector like at()
        None
    } else {
        return Err(ParseError::UnexpectedToken {
            expected: "string literal or ')'".into(),
            found: cursor
                .remaining()
                .chars()
                .next()
                .map_or("end of input".into(), |c| c.to_string()),
            span: cursor.span_from(cursor.pos()),
        });
    };

    cursor.eat_whitespace();

    if !cursor.eat_char(b')') {
        return Err(ParseError::UnexpectedToken {
            expected: "')'".into(),
            found: cursor
                .remaining()
                .chars()
                .next()
                .map_or("end of input".into(), |c| c.to_string()),
            span: cursor.span_from(cursor.pos()),
        });
    }

    let span = cursor.span_from(start);
    Ok(Spanned {
        node: Selector {
            steps: vec![Spanned {
                node: SimpleSelector {
                    op,
                    pattern,
                    params: Vec::new(),
                },
                span,
            }],
        },
        span,
    })
}

/// Parse a selector operation keyword.
fn parse_selector_op(cursor: &mut Cursor) -> Result<SelectorOp, ParseError> {
    let start = cursor.pos();
    if cursor.eat_keyword("at") {
        return Ok(SelectorOp::At);
    }
    if cursor.eat_keyword("after") {
        return Ok(SelectorOp::After);
    }
    if cursor.eat_keyword("before") {
        return Ok(SelectorOp::Before);
    }
    if cursor.eat_keyword("from") {
        return Ok(SelectorOp::From);
    }
    if cursor.eat_keyword("to") {
        return Ok(SelectorOp::To);
    }
    Err(ParseError::UnexpectedToken {
        expected: "selector (at, after, before, from, to)".into(),
        found: cursor
            .remaining()
            .chars()
            .next()
            .map_or("end of input".into(), |c| c.to_string()),
        span: cursor.span_from(start),
    })
}

/// Parse a processor chain (single processor for now).
fn parse_processor_chain(cursor: &mut Cursor) -> Result<ProcessorChain, ParseError> {
    let processor = parse_processor(cursor)?;
    Ok(ProcessorChain {
        processors: vec![processor],
    })
}

/// Parse a single qed processor: `qed:name()`.
fn parse_processor(
    cursor: &mut Cursor,
) -> Result<Spanned<crate::parse::ast::Processor>, ParseError> {
    let start = cursor.pos();

    if !cursor.eat_keyword("qed") {
        return Err(ParseError::UnexpectedToken {
            expected: "'qed:' processor prefix".into(),
            found: cursor
                .remaining()
                .chars()
                .next()
                .map_or("end of input".into(), |c| c.to_string()),
            span: cursor.span_from(start),
        });
    }

    if !cursor.eat_char(b':') {
        return Err(ParseError::UnexpectedToken {
            expected: "':' after 'qed'".into(),
            found: cursor
                .remaining()
                .chars()
                .next()
                .map_or("end of input".into(), |c| c.to_string()),
            span: cursor.span_from(cursor.pos()),
        });
    }

    // Parse processor name (identifier)
    let name_start = cursor.pos();
    while let Some(b) = cursor.peek() {
        if b.is_ascii_alphanumeric() || b == b'_' {
            cursor.advance();
        } else {
            break;
        }
    }
    if cursor.pos() == name_start {
        return Err(ParseError::UnexpectedToken {
            expected: "processor name".into(),
            found: cursor
                .remaining()
                .chars()
                .next()
                .map_or("end of input".into(), |c| c.to_string()),
            span: cursor.span_from(name_start),
        });
    }
    let name = cursor.slice_from(name_start).to_owned();
    let name_span = cursor.span_from(name_start);

    if !cursor.eat_char(b'(') {
        return Err(ParseError::UnexpectedToken {
            expected: "'(' after processor name".into(),
            found: cursor
                .remaining()
                .chars()
                .next()
                .map_or("end of input".into(), |c| c.to_string()),
            span: cursor.span_from(cursor.pos()),
        });
    }

    cursor.eat_whitespace();

    if !cursor.eat_char(b')') {
        return Err(ParseError::UnexpectedToken {
            expected: "')'".into(),
            found: cursor
                .remaining()
                .chars()
                .next()
                .map_or("end of input".into(), |c| c.to_string()),
            span: cursor.span_from(cursor.pos()),
        });
    }

    let span = cursor.span_from(start);
    Ok(Spanned {
        node: crate::parse::ast::Processor::Qed(QedProcessor {
            name: Spanned {
                node: name,
                span: name_span,
            },
            args: Vec::new(),
            params: Vec::new(),
        }),
        span,
    })
}

/// Skip whitespace, newlines, and carriage returns.
fn eat_whitespace_and_newlines(cursor: &mut Cursor) {
    while let Some(b' ' | b'\t' | b'\n' | b'\r') = cursor.peek() {
        cursor.advance();
    }
}

/// Skip to the next newline (for error recovery).
fn skip_to_newline(cursor: &mut Cursor) {
    while let Some(b) = cursor.peek() {
        cursor.advance();
        if b == b'\n' {
            break;
        }
    }
}

/// Parse a complete nth expression: `nth-term (',' nth-term)*`.
pub(super) fn parse_nth_expr(source: &str) -> Result<ParseResult, Vec<ParseError>> {
    let mut cursor = Cursor::new(source);
    let mut terms: Vec<Spanned<NthTerm>> = Vec::new();
    let mut warnings: Vec<ParseError> = Vec::new();
    let mut errors: Vec<ParseError> = Vec::new();

    cursor.eat_whitespace();

    if cursor.is_eof() {
        return Err(vec![ParseError::UnexpectedEof {
            expected: "nth expression".into(),
            span: cursor.span_from(0),
        }]);
    }

    // Parse first term.
    match parse_nth_term(&mut cursor) {
        Ok((term, term_warnings)) => {
            warnings.extend(term_warnings);
            if let Some(t) = term {
                terms.push(t);
            }
        }
        Err(e) => errors.push(e),
    }

    // Parse remaining comma-separated terms.
    loop {
        cursor.eat_whitespace();
        if cursor.is_eof() {
            break;
        }
        if !cursor.eat_char(b',') {
            let start = cursor.pos();
            // Not a comma and not EOF — unexpected character.
            let found = cursor.remaining().chars().next().unwrap();
            errors.push(ParseError::UnexpectedToken {
                expected: "',' or end of input".into(),
                found: found.to_string(),
                span: cursor.span_from(start),
            });
            // Try to recover by skipping the bad character.
            cursor.advance();
            continue;
        }
        cursor.eat_whitespace();

        if cursor.is_eof() {
            errors.push(ParseError::UnexpectedEof {
                expected: "nth term after ','".into(),
                span: cursor.span_from(cursor.pos()),
            });
            break;
        }

        match parse_nth_term(&mut cursor) {
            Ok((term, term_warnings)) => {
                warnings.extend(term_warnings);
                if let Some(t) = term {
                    terms.push(t);
                }
            }
            Err(e) => errors.push(e),
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    if terms.is_empty() {
        return Err(vec![ParseError::InvalidNthExpr {
            reason: "all terms were zero and ignored".into(),
            span: Cursor::new(source).span_from(0),
        }]);
    }

    Ok(ParseResult {
        expr: NthExpr { terms },
        warnings,
    })
}

/// Parse a single nth term: range, step, or integer.
///
/// Returns `Ok((None, warnings))` when the term is a warned-and-ignored zero.
/// Returns `Ok((Some(term), warnings))` on a valid term.
/// Returns `Err(ParseError)` on a hard error.
fn parse_nth_term(
    cursor: &mut Cursor,
) -> Result<(Option<Spanned<NthTerm>>, Vec<ParseError>), ParseError> {
    let term_start = cursor.pos();
    let mut warnings: Vec<ParseError> = Vec::new();

    // Handle leading '+' — warn and skip.
    let had_leading_plus = cursor.eat_char(b'+');
    if had_leading_plus {
        cursor.eat_whitespace();
    }

    let negative = cursor.eat_char(b'-');
    if negative {
        cursor.eat_whitespace();
    }

    // Try to parse digits.
    let digits = parse_digits(cursor);

    // Check for 'n' — this is a step expression.
    if cursor.peek() == Some(b'n') {
        let coeff_value = match digits.as_deref() {
            Some("0") => {
                return Err(ParseError::InvalidNthExpr {
                    reason: "coefficient must be non-zero (`0n` is invalid)".into(),
                    span: cursor.span_from(term_start),
                });
            }
            Some(d) => {
                let v: i64 = d.parse().map_err(|_| ParseError::InvalidNthExpr {
                    reason: format!("coefficient too large: {d}"),
                    span: cursor.span_from(term_start),
                })?;
                if negative { -v } else { v }
            }
            None => {
                // bare 'n' or '-n'
                if negative { -1 } else { 1 }
            }
        };
        cursor.advance(); // consume 'n'

        if had_leading_plus {
            let coeff_str = if coeff_value == 1 { "n" } else { &format!("{coeff_value}n") };
            warnings.push(ParseError::NthWarning {
                reason: format!(
                    "leading '+' ignored, `+{coeff_str}` treated as `{coeff_str}`"
                ),
                span: cursor.span_from(term_start),
            });
        }

        // Check for offset: '+' or '-' followed by pos-integer.
        cursor.eat_whitespace();
        let offset = if cursor.peek() == Some(b'+') || cursor.peek() == Some(b'-') {
            let offset_negative = cursor.peek() == Some(b'-');
            cursor.advance(); // consume +/-
            cursor.eat_whitespace();
            let offset_digits =
                parse_digits(cursor).ok_or_else(|| ParseError::UnexpectedToken {
                    expected: "integer after offset sign".into(),
                    found: cursor
                        .remaining()
                        .chars()
                        .next()
                        .map_or("end of input".into(), |c| c.to_string()),
                    span: cursor.span_from(cursor.pos()),
                })?;

            // Check for zero offset: an+0 or an-0.
            if offset_digits == "0" {
                return Err(ParseError::InvalidNthExpr {
                    reason: "offset in step expression must be non-zero".into(),
                    span: cursor.span_from(term_start),
                });
            }

            let v: i64 = offset_digits.parse().map_err(|_| ParseError::InvalidNthExpr {
                reason: format!("offset too large: {offset_digits}"),
                span: cursor.span_from(term_start),
            })?;
            if offset_negative { -v } else { v }
        } else {
            0
        };

        let span = cursor.span_from(term_start);
        return Ok((
            Some(Spanned {
                node: NthTerm::Step {
                    coefficient: coeff_value,
                    offset,
                },
                span,
            }),
            warnings,
        ));
    }

    // No 'n' — must be an integer (possibly followed by '...' for range).
    let digits = digits.ok_or_else(|| {
        let found = cursor
            .remaining()
            .chars()
            .next()
            .map_or("end of input".into(), |c| c.to_string());
        ParseError::UnexpectedToken {
            expected: "integer, range, or step expression".into(),
            found,
            span: cursor.span_from(term_start),
        }
    })?;

    let int_value: i64 = digits.parse().map_err(|_| ParseError::InvalidNthExpr {
        reason: format!("integer too large: {digits}"),
        span: cursor.span_from(term_start),
    })?;
    let int_value = if negative { -int_value } else { int_value };

    // Check for zero integer.
    if int_value == 0 {
        if had_leading_plus {
            warnings.push(ParseError::NthWarning {
                reason: "leading '+' ignored".into(),
                span: cursor.span_from(term_start),
            });
        }
        warnings.push(ParseError::NthWarning {
            reason: format!(
                "`{}` is zero and will be ignored",
                if negative { "-0" } else { "0" }
            ),
            span: cursor.span_from(term_start),
        });
        return Ok((None, warnings));
    }

    if had_leading_plus {
        warnings.push(ParseError::NthWarning {
            reason: format!("leading '+' ignored, `+{int_value}` treated as `{int_value}`"),
            span: cursor.span_from(term_start),
        });
    }

    // Check for range: integer '...' integer.
    cursor.eat_whitespace();
    let range_start_pos = cursor.pos();
    if cursor.eat_char(b'.') {
        if cursor.eat_char(b'.') {
            if cursor.eat_char(b'.') {
                // Got '...' — parse the end of the range.
                cursor.eat_whitespace();
                let end_start = cursor.pos();
                let end_negative = cursor.eat_char(b'-');
                if end_negative {
                    cursor.eat_whitespace();
                }
                let end_digits =
                    parse_digits(cursor).ok_or_else(|| ParseError::UnexpectedToken {
                        expected: "integer after '...'".into(),
                        found: cursor
                            .remaining()
                            .chars()
                            .next()
                            .map_or("end of input".into(), |c| c.to_string()),
                        span: cursor.span_from(end_start),
                    })?;

                let end_value: i64 =
                    end_digits.parse().map_err(|_| ParseError::InvalidNthExpr {
                        reason: format!("integer too large: {end_digits}"),
                        span: cursor.span_from(end_start),
                    })?;
                let end_value = if end_negative { -end_value } else { end_value };

                // Check for zero in range endpoint.
                if end_value == 0 {
                    return Err(ParseError::InvalidNthExpr {
                        reason: "range endpoint cannot be zero".into(),
                        span: cursor.span_from(term_start),
                    });
                }

                // Cross-sign check.
                if (int_value > 0 && end_value < 0) || (int_value < 0 && end_value > 0) {
                    return Err(ParseError::InvalidNthExpr {
                        reason: format!(
                            "range bounds must have the same sign (`{int_value}...{end_value}` is invalid)"
                        ),
                        span: cursor.span_from(term_start),
                    });
                }

                let span = cursor.span_from(term_start);
                return Ok((
                    Some(Spanned {
                        node: NthTerm::Range {
                            start: int_value,
                            end: end_value,
                        },
                        span,
                    }),
                    warnings,
                ));
            } else {
                // Only two dots — '..' is not valid.
                return Err(ParseError::UnexpectedToken {
                    expected: "'...' (three dots for range)".into(),
                    found: "'..' (two dots)".into(),
                    span: cursor.span_from(range_start_pos),
                });
            }
        } else {
            // Single dot — e.g. `1.5`.
            return Err(ParseError::UnexpectedToken {
                expected: "'...' (three dots for range) or ',' or end of input".into(),
                found: "'.' (single dot)".into(),
                span: cursor.span_from(range_start_pos),
            });
        }
    }

    let span = cursor.span_from(term_start);
    Ok((
        Some(Spanned {
            node: NthTerm::Integer(int_value),
            span,
        }),
        warnings,
    ))
}

/// Parse a sequence of ASCII digits. Returns `None` if no digits found.
/// Does not consume a leading sign.
fn parse_digits(cursor: &mut Cursor) -> Option<String> {
    let start = cursor.pos();
    while let Some(b'0'..=b'9') = cursor.peek() {
        cursor.advance();
    }
    if cursor.pos() == start {
        return None;
    }
    Some(cursor.slice_from(start).to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_ok(input: &str) -> ParseResult {
        parse_nth_expr(input).unwrap_or_else(|errs| {
            panic!("expected Ok for {input:?}, got errors: {errs:?}")
        })
    }

    fn parse_err(input: &str) -> Vec<ParseError> {
        parse_nth_expr(input).expect_err(&format!("expected Err for {input:?}"))
    }

    // ── Valid forms ──────────────────────────────────────────────────

    #[test]
    fn integer_positive() {
        let r = parse_ok("1");
        assert_eq!(r.expr.terms.len(), 1);
        assert_eq!(r.expr.terms[0].node, NthTerm::Integer(1));
        assert_eq!(r.expr.terms[0].span.start, 0);
        assert_eq!(r.expr.terms[0].span.end, 1);
    }

    #[test]
    fn integer_negative() {
        let r = parse_ok("-1");
        assert_eq!(r.expr.terms[0].node, NthTerm::Integer(-1));
    }

    #[test]
    fn integer_multi_digit() {
        let r = parse_ok("10");
        assert_eq!(r.expr.terms[0].node, NthTerm::Integer(10));
    }

    #[test]
    fn range_positive() {
        let r = parse_ok("1...3");
        assert_eq!(
            r.expr.terms[0].node,
            NthTerm::Range { start: 1, end: 3 }
        );
        assert_eq!(r.expr.terms[0].span, crate::span::Span { start: 0, end: 5 });
    }

    #[test]
    fn range_negative() {
        let r = parse_ok("-3...-1");
        assert_eq!(
            r.expr.terms[0].node,
            NthTerm::Range { start: -3, end: -1 }
        );
    }

    #[test]
    fn step_basic() {
        let r = parse_ok("2n");
        assert_eq!(
            r.expr.terms[0].node,
            NthTerm::Step {
                coefficient: 2,
                offset: 0
            }
        );
    }

    #[test]
    fn step_with_positive_offset() {
        let r = parse_ok("2n+1");
        assert_eq!(
            r.expr.terms[0].node,
            NthTerm::Step {
                coefficient: 2,
                offset: 1
            }
        );
    }

    #[test]
    fn step_with_negative_offset() {
        let r = parse_ok("2n-1");
        assert_eq!(
            r.expr.terms[0].node,
            NthTerm::Step {
                coefficient: 2,
                offset: -1
            }
        );
    }

    #[test]
    fn step_negative_coefficient() {
        let r = parse_ok("-2n");
        assert_eq!(
            r.expr.terms[0].node,
            NthTerm::Step {
                coefficient: -2,
                offset: 0
            }
        );
    }

    #[test]
    fn step_negative_coefficient_with_offset() {
        let r = parse_ok("-2n+1");
        assert_eq!(
            r.expr.terms[0].node,
            NthTerm::Step {
                coefficient: -2,
                offset: 1
            }
        );
    }

    #[test]
    fn step_bare_n() {
        let r = parse_ok("n");
        assert_eq!(
            r.expr.terms[0].node,
            NthTerm::Step {
                coefficient: 1,
                offset: 0
            }
        );
    }

    #[test]
    fn step_bare_negative_n() {
        let r = parse_ok("-n");
        assert_eq!(
            r.expr.terms[0].node,
            NthTerm::Step {
                coefficient: -1,
                offset: 0
            }
        );
    }

    #[test]
    fn step_n_with_offset() {
        let r = parse_ok("n+3");
        assert_eq!(
            r.expr.terms[0].node,
            NthTerm::Step {
                coefficient: 1,
                offset: 3
            }
        );
    }

    #[test]
    fn step_negative_n_with_offset() {
        let r = parse_ok("-n+3");
        assert_eq!(
            r.expr.terms[0].node,
            NthTerm::Step {
                coefficient: -1,
                offset: 3
            }
        );
    }

    #[test]
    fn step_large_coefficient() {
        let r = parse_ok("100n+50");
        assert_eq!(
            r.expr.terms[0].node,
            NthTerm::Step {
                coefficient: 100,
                offset: 50
            }
        );
    }

    // ── Multi-term ──────────────────────────────────────────────────

    #[test]
    fn comma_separated() {
        let r = parse_ok("1,3,-1");
        assert_eq!(r.expr.terms.len(), 3);
        assert_eq!(r.expr.terms[0].node, NthTerm::Integer(1));
        assert_eq!(r.expr.terms[1].node, NthTerm::Integer(3));
        assert_eq!(r.expr.terms[2].node, NthTerm::Integer(-1));
    }

    #[test]
    fn range_and_integer() {
        let r = parse_ok("1...3,-2");
        assert_eq!(r.expr.terms.len(), 2);
        assert_eq!(
            r.expr.terms[0].node,
            NthTerm::Range { start: 1, end: 3 }
        );
        assert_eq!(r.expr.terms[1].node, NthTerm::Integer(-2));
    }

    #[test]
    fn step_and_integer() {
        let r = parse_ok("2n+1, 4");
        assert_eq!(r.expr.terms.len(), 2);
        assert_eq!(
            r.expr.terms[0].node,
            NthTerm::Step {
                coefficient: 2,
                offset: 1
            }
        );
        assert_eq!(r.expr.terms[1].node, NthTerm::Integer(4));
    }

    // ── Span accuracy ───────────────────────────────────────────────

    #[test]
    fn span_range() {
        let r = parse_ok("1...3");
        assert_eq!(r.expr.terms[0].span.start, 0);
        assert_eq!(r.expr.terms[0].span.end, 5);
    }

    #[test]
    fn span_multi_term() {
        let r = parse_ok("2n+1, 4");
        assert_eq!(r.expr.terms[0].span.start, 0);
        assert_eq!(r.expr.terms[0].span.end, 4);
        assert_eq!(r.expr.terms[1].span.start, 6);
        assert_eq!(r.expr.terms[1].span.end, 7);
    }

    // ── Semantic hard errors ────────────────────────────────────────

    #[test]
    fn error_zero_coefficient() {
        let errs = parse_err("0n");
        assert!(matches!(&errs[0], ParseError::InvalidNthExpr { reason, .. } if reason.contains("coefficient")));
    }

    #[test]
    fn error_cross_sign_range_neg_to_pos() {
        let errs = parse_err("-3...5");
        assert!(matches!(&errs[0], ParseError::InvalidNthExpr { reason, .. } if reason.contains("same sign")));
    }

    #[test]
    fn error_cross_sign_range_pos_to_neg() {
        let errs = parse_err("3...-1");
        assert!(matches!(&errs[0], ParseError::InvalidNthExpr { reason, .. } if reason.contains("same sign")));
    }

    #[test]
    fn error_zero_offset_plus() {
        let errs = parse_err("2n+0");
        assert!(matches!(&errs[0], ParseError::InvalidNthExpr { reason, .. } if reason.contains("non-zero")));
    }

    #[test]
    fn error_zero_offset_minus() {
        let errs = parse_err("2n-0");
        assert!(matches!(&errs[0], ParseError::InvalidNthExpr { reason, .. } if reason.contains("non-zero")));
    }

    // ── Warnings ────────────────────────────────────────────────────

    #[test]
    fn warning_zero_ignored() {
        let r = parse_ok("0,1");
        // '0' is ignored, result has just Integer(1).
        assert_eq!(r.expr.terms.len(), 1);
        assert_eq!(r.expr.terms[0].node, NthTerm::Integer(1));
        assert!(r.warnings.iter().any(|w| matches!(w, ParseError::NthWarning { reason, .. } if reason.contains("zero"))));
    }

    #[test]
    fn warning_negative_zero_ignored() {
        let r = parse_ok("-0,1");
        assert_eq!(r.expr.terms.len(), 1);
        assert_eq!(r.expr.terms[0].node, NthTerm::Integer(1));
    }

    #[test]
    fn warning_plus_n() {
        let r = parse_ok("+n");
        assert_eq!(
            r.expr.terms[0].node,
            NthTerm::Step {
                coefficient: 1,
                offset: 0
            }
        );
        assert!(r.warnings.iter().any(|w| matches!(w, ParseError::NthWarning { reason, .. } if reason.contains("+"))));
    }

    #[test]
    fn warning_plus_step() {
        let r = parse_ok("+2n+1");
        assert_eq!(
            r.expr.terms[0].node,
            NthTerm::Step {
                coefficient: 2,
                offset: 1
            }
        );
        assert!(!r.warnings.is_empty());
    }

    // ── Malformed input ─────────────────────────────────────────────

    #[test]
    fn error_empty() {
        let errs = parse_err("");
        assert!(matches!(&errs[0], ParseError::UnexpectedEof { .. }));
    }

    #[test]
    fn error_alpha() {
        let errs = parse_err("abc");
        // 'a' is not valid here — it's not 'n', not a digit, not a sign.
        // Actually 'a' would be UnexpectedToken... wait, the parser sees no digits,
        // no '-', no '+', and the first char is 'a'. It would try parse_digits, fail,
        // then check for 'n'... Let me think. The parser first checks for '+', then '-',
        // then digits, then 'n'. So 'a' would hit none of those.
        assert!(!errs.is_empty());
    }

    #[test]
    fn error_trailing_dots() {
        let errs = parse_err("1...");
        assert!(!errs.is_empty());
    }

    #[test]
    fn error_leading_dots() {
        let errs = parse_err("...3");
        assert!(!errs.is_empty());
    }

    #[test]
    fn error_two_dots() {
        let errs = parse_err("1..3");
        assert!(matches!(&errs[0], ParseError::UnexpectedToken { found, .. } if found.contains("two dots")));
    }

    #[test]
    fn error_trailing_plus() {
        let errs = parse_err("2n+");
        assert!(!errs.is_empty());
    }

    #[test]
    fn error_double_comma() {
        let errs = parse_err("1,,2");
        assert!(!errs.is_empty());
    }

    #[test]
    fn error_decimal() {
        let errs = parse_err("1.5");
        assert!(!errs.is_empty());
    }

    // ── Whitespace tolerance ────────────────────────────────────────

    #[test]
    fn whitespace_range() {
        let r = parse_ok("1 ... 3");
        assert_eq!(
            r.expr.terms[0].node,
            NthTerm::Range { start: 1, end: 3 }
        );
    }

    #[test]
    fn whitespace_step() {
        let r = parse_ok("2n + 1");
        assert_eq!(
            r.expr.terms[0].node,
            NthTerm::Step {
                coefficient: 2,
                offset: 1
            }
        );
    }

    #[test]
    fn whitespace_step_minus() {
        let r = parse_ok("2n - 1");
        assert_eq!(
            r.expr.terms[0].node,
            NthTerm::Step {
                coefficient: 2,
                offset: -1
            }
        );
    }

    #[test]
    fn whitespace_comma() {
        let r = parse_ok(" 1 , 2 ");
        assert_eq!(r.expr.terms.len(), 2);
        assert_eq!(r.expr.terms[0].node, NthTerm::Integer(1));
        assert_eq!(r.expr.terms[1].node, NthTerm::Integer(2));
    }

    // ── All-zero edge case ──────────────────────────────────────────

    #[test]
    fn error_all_zeros() {
        let errs = parse_err("0");
        assert!(matches!(&errs[0], ParseError::InvalidNthExpr { reason, .. } if reason.contains("all terms were zero")));
    }
}
