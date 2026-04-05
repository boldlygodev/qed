//! Grammar implementation for the recursive descent parser.
//!
//! Each `parse_*` function corresponds to a production in the qed grammar.
//! Covers selectors, processors, definitions (pattern/alias), fallback (`||`),
//! semicolons, and alias references in processor position.
//!
//! Error recovery is line-based: on a parse failure the parser skips to the
//! next newline and attempts the next statement.

use crate::parse::ast::{
    ExternalArg, ExternalProcessor, Fallback, NthExpr, NthTerm, Param, ParamValue, PatternRef,
    PatternRefValue, PatternValue, ProcessorChain, Program, QedArg, QedProcessor, SelectActionNode,
    Selector, SelectorOp, SimpleSelector, Statement,
};
use crate::parse::error::{ParseError, ParseResult};
use crate::span::Spanned;

/// Helper: describe the next byte for error messages.
fn peek_found(cursor: &Cursor) -> String {
    cursor
        .remaining()
        .chars()
        .next()
        .map_or("end of input".into(), |c| c.to_string())
}

use super::cursor::Cursor;

// ── Program parser ──────────────────────────────────────────────────

/// Parse a complete qed program from source text.
pub(super) fn parse_program(source: &str) -> Result<Program, Vec<ParseError>> {
    let mut cursor = Cursor::new(source);
    let mut statements: Vec<Spanned<Statement>> = Vec::new();
    let mut errors: Vec<ParseError> = Vec::new();

    // Handle shebang line
    let shebang = parse_shebang(&mut cursor);

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
        // Eat semicolons as statement separators
        while cursor.eat_char(b';') {
            eat_whitespace_and_newlines(&mut cursor);
        }

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
        shebang,
        statements,
    })
}

/// Parse a `#!` shebang line at the very start of the source.
fn parse_shebang(cursor: &mut Cursor) -> Option<Spanned<String>> {
    let start = cursor.pos();
    if cursor.peek_at(0) == Some(b'#') && cursor.peek_at(1) == Some(b'!') {
        cursor.advance(); // #
        cursor.advance(); // !
        let content_start = cursor.pos();
        while let Some(b) = cursor.peek() {
            if b == b'\n' {
                break;
            }
            cursor.advance();
        }
        let content = cursor.slice_from(content_start).to_owned();
        // consume the newline if present
        cursor.eat_char(b'\n');
        let span = cursor.span_from(start);
        Some(Spanned {
            node: content,
            span,
        })
    } else {
        None
    }
}

/// Parse a single statement: definition or select-action.
///
/// Disambiguation: eat an identifier; if followed by `=` (not `==`),
/// parse as a definition (pattern or alias). Otherwise restore and
/// parse as a select-action.
fn parse_statement(cursor: &mut Cursor) -> Result<Spanned<Statement>, Vec<ParseError>> {
    let start = cursor.pos();

    // Try definition: identifier = ...
    let saved = cursor.pos();
    if let Some(name) = cursor.eat_identifier() {
        let name_span = cursor.span_from(saved);
        cursor.eat_whitespace();
        if cursor.peek() == Some(b'=') && cursor.peek_at(1) != Some(b'=') {
            cursor.advance(); // consume '='
            cursor.eat_whitespace();
            match cursor.peek() {
                Some(b'"') | Some(b'\'') | Some(b'/') => {
                    let stmt = parse_pattern_def_value(cursor, name, name_span)?;
                    let span = cursor.span_from(start);
                    return Ok(Spanned { node: stmt, span });
                }
                _ => {
                    let stmt = parse_alias_def_value(cursor, name, name_span)?;
                    let span = cursor.span_from(start);
                    return Ok(Spanned { node: stmt, span });
                }
            }
        }
        // Not a definition — restore position
        cursor.set_pos(saved);
    }

    let select_action = parse_select_action(cursor)?;
    let span = cursor.span_from(start);
    Ok(Spanned {
        node: Statement::SelectAction(select_action),
        span,
    })
}

/// Parse the value side of a pattern definition after `name =`.
fn parse_pattern_def_value(
    cursor: &mut Cursor,
    name: String,
    name_span: crate::span::Span,
) -> Result<Statement, Vec<ParseError>> {
    let value_start = cursor.pos();
    let value = match cursor.peek() {
        Some(b'"') => {
            let s = cursor.eat_string_literal().ok_or_else(|| {
                vec![ParseError::UnexpectedEof {
                    expected: "closing '\"' for pattern string".into(),
                    span: cursor.span_from(value_start),
                }]
            })?;
            PatternValue::String(s)
        }
        Some(b'\'') => {
            let s = cursor.eat_single_quoted_string_literal().ok_or_else(|| {
                vec![ParseError::UnexpectedEof {
                    expected: "closing \"'\" for pattern string".into(),
                    span: cursor.span_from(value_start),
                }]
            })?;
            PatternValue::String(s)
        }
        Some(b'/') => {
            let r = cursor.eat_regex_literal().ok_or_else(|| {
                vec![ParseError::UnexpectedEof {
                    expected: "closing '/' for pattern regex".into(),
                    span: cursor.span_from(value_start),
                }]
            })?;
            PatternValue::Regex(r)
        }
        _ => unreachable!("called only when peek is \", ', or /"),
    };
    let value_span = cursor.span_from(value_start);

    Ok(Statement::PatternDef {
        name: Spanned {
            node: name,
            span: name_span,
        },
        value: Spanned {
            node: value,
            span: value_span,
        },
    })
}

/// Parse the value side of an alias definition after `name =`.
fn parse_alias_def_value(
    cursor: &mut Cursor,
    name: String,
    name_span: crate::span::Span,
) -> Result<Statement, Vec<ParseError>> {
    let chain_start = cursor.pos();
    let chain = parse_processor_chain(cursor).map_err(|e| vec![e])?;
    let chain_span = cursor.span_from(chain_start);

    Ok(Statement::AliasDef {
        name: Spanned {
            node: name,
            span: name_span,
        },
        chain: Spanned {
            node: chain,
            span: chain_span,
        },
    })
}

/// Parse `selector | processor_chain (|| fallback)?`.
fn parse_select_action(cursor: &mut Cursor) -> Result<SelectActionNode, Vec<ParseError>> {
    let selector = parse_selector(cursor).map_err(|e| vec![e])?;

    cursor.eat_whitespace();

    // Eat `|` but not `||` (which is fallback)
    let chain = if cursor.peek() == Some(b'|') && cursor.peek_at(1) != Some(b'|') {
        cursor.advance(); // consume the `|`
        eat_whitespace_and_newlines(cursor);
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

    // Parse optional fallback: `|| ...`
    let saved = cursor.pos();
    cursor.eat_whitespace();
    let fallback = if cursor.peek() == Some(b'|') && cursor.peek_at(1) == Some(b'|') {
        cursor.advance(); // consume first |
        cursor.advance(); // consume second |
        eat_whitespace_and_newlines(cursor);
        let fb_start = cursor.pos();
        if is_selector_start(cursor) {
            let fb_action = parse_select_action(cursor)?;
            let fb_span = cursor.span_from(fb_start);
            Some(Spanned {
                node: Fallback::SelectAction(Box::new(fb_action)),
                span: fb_span,
            })
        } else {
            let fb_chain = parse_processor_chain(cursor).map_err(|e| vec![e])?;
            let fb_span = cursor.span_from(fb_start);
            Some(Spanned {
                node: Fallback::Chain(fb_chain),
                span: fb_span,
            })
        }
    } else {
        cursor.set_pos(saved);
        None
    };

    Ok(SelectActionNode {
        selector,
        chain,
        fallback,
    })
}

/// Parse a selector: simple-selector ('>' simple-selector)*
fn parse_selector(cursor: &mut Cursor) -> Result<Spanned<Selector>, ParseError> {
    let start = cursor.pos();

    let first_step = parse_simple_selector(cursor)?;
    let mut steps = vec![first_step];

    // Parse compound selectors with '>'
    loop {
        let saved = cursor.pos();
        cursor.eat_whitespace();
        if cursor.peek() != Some(b'>') {
            // Rewind — don't include trailing whitespace in the span
            cursor.set_pos(saved);
            break;
        }
        cursor.advance(); // consume '>'
        // Implicit line continuation after '>'
        eat_whitespace_and_newlines(cursor);
        let step = parse_simple_selector(cursor)?;
        steps.push(step);
    }

    let span = cursor.span_from(start);
    Ok(Spanned {
        node: Selector { steps },
        span,
    })
}

/// Parse a single simple selector: `op(pattern-ref, params...)`.
fn parse_simple_selector(cursor: &mut Cursor) -> Result<Spanned<SimpleSelector>, ParseError> {
    let start = cursor.pos();

    let op = parse_selector_op(cursor)?;

    if !cursor.eat_char(b'(') {
        return Err(ParseError::UnexpectedToken {
            expected: "'(' after selector name".into(),
            found: peek_found(cursor),
            span: cursor.span_from(cursor.pos()),
        });
    }

    cursor.eat_whitespace();

    // Check for empty selector: at()
    if cursor.peek() == Some(b')') {
        cursor.advance();
        let span = cursor.span_from(start);
        return Ok(Spanned {
            node: SimpleSelector {
                op,
                pattern: None,
                params: Vec::new(),
            },
            span,
        });
    }

    // Parse pattern ref — but if the first token is a named param (e.g.
    // `on_error:skip`), skip pattern parsing and go straight to params.
    let pattern = if is_param_start(cursor) {
        None
    } else {
        Some(parse_pattern_ref(cursor)?)
    };

    cursor.eat_whitespace();

    // Parse optional params after comma (or directly if pattern was None).
    let mut params = Vec::new();
    if pattern.is_none() && cursor.peek() != Some(b')') {
        // No pattern — first token is already a param.
        let param = parse_param(cursor)?;
        params.push(param);
        cursor.eat_whitespace();
    }
    while cursor.peek() == Some(b',') {
        cursor.advance(); // consume ','
        // Implicit line continuation after ','
        eat_whitespace_and_newlines(cursor);
        let param = parse_param(cursor)?;
        params.push(param);
        cursor.eat_whitespace();
    }

    if !cursor.eat_char(b')') {
        return Err(ParseError::UnexpectedToken {
            expected: "')' or ',' in selector".into(),
            found: peek_found(cursor),
            span: cursor.span_from(cursor.pos()),
        });
    }

    let span = cursor.span_from(start);
    Ok(Spanned {
        node: SimpleSelector {
            op,
            pattern,
            params,
        },
        span,
    })
}

/// Parse a pattern reference: `!? pattern-value +?`
fn parse_pattern_ref(cursor: &mut Cursor) -> Result<Spanned<PatternRef>, ParseError> {
    let start = cursor.pos();

    // Check for negation prefix
    let negated = cursor.eat_char(b'!');

    let value = parse_pattern_value(cursor)?;

    // Check for inclusive suffix
    let inclusive = cursor.eat_char(b'+');

    let span = cursor.span_from(start);
    Ok(Spanned {
        node: PatternRef {
            value,
            negated,
            inclusive,
        },
        span,
    })
}

/// Parse a pattern value: string, single-quoted string, regex, or identifier (named ref).
fn parse_pattern_value(cursor: &mut Cursor) -> Result<PatternRefValue, ParseError> {
    match cursor.peek() {
        Some(b'"') => {
            let s = cursor
                .eat_string_literal()
                .ok_or_else(|| ParseError::UnexpectedEof {
                    expected: "closing '\"' for string literal".into(),
                    span: cursor.span_from(cursor.pos()),
                })?;
            Ok(PatternRefValue::Inline(PatternValue::String(s)))
        }
        Some(b'\'') => {
            let s = cursor.eat_single_quoted_string_literal().ok_or_else(|| {
                ParseError::UnexpectedEof {
                    expected: "closing \"'\" for string literal".into(),
                    span: cursor.span_from(cursor.pos()),
                }
            })?;
            Ok(PatternRefValue::Inline(PatternValue::String(s)))
        }
        Some(b'/') => {
            let r = cursor
                .eat_regex_literal()
                .ok_or_else(|| ParseError::UnexpectedEof {
                    expected: "closing '/' for regex literal".into(),
                    span: cursor.span_from(cursor.pos()),
                })?;
            Ok(PatternRefValue::Inline(PatternValue::Regex(r)))
        }
        Some(b) if b.is_ascii_alphabetic() || b == b'_' => {
            let name = cursor
                .eat_identifier()
                .expect("identifier start already checked");
            Ok(PatternRefValue::Named(name))
        }
        _ => Err(ParseError::UnexpectedToken {
            expected: "pattern (string, regex, or identifier)".into(),
            found: peek_found(cursor),
            span: cursor.span_from(cursor.pos()),
        }),
    }
}

/// Parse a named parameter: `name:value`.
fn parse_param(cursor: &mut Cursor) -> Result<Spanned<Param>, ParseError> {
    let start = cursor.pos();

    let name_start = cursor.pos();
    let name = cursor
        .eat_identifier()
        .ok_or_else(|| ParseError::UnexpectedToken {
            expected: "parameter name".into(),
            found: peek_found(cursor),
            span: cursor.span_from(name_start),
        })?;
    let name_span = cursor.span_from(name_start);

    if !cursor.eat_char(b':') {
        return Err(ParseError::UnexpectedToken {
            expected: "':' after parameter name".into(),
            found: peek_found(cursor),
            span: cursor.span_from(cursor.pos()),
        });
    }

    // No whitespace between ':' and value for params
    let value = parse_param_value(cursor)?;

    let span = cursor.span_from(start);
    Ok(Spanned {
        node: Param {
            name: Spanned {
                node: name,
                span: name_span,
            },
            value,
        },
        span,
    })
}

/// Parse a parameter value: identifier, string, integer, nth-expr, or pattern-ref.
fn parse_param_value(cursor: &mut Cursor) -> Result<Spanned<ParamValue>, ParseError> {
    let start = cursor.pos();

    match cursor.peek() {
        Some(b'"') => {
            let s = cursor
                .eat_string_literal()
                .ok_or_else(|| ParseError::UnexpectedEof {
                    expected: "closing '\"' for string literal".into(),
                    span: cursor.span_from(start),
                })?;
            let span = cursor.span_from(start);
            Ok(Spanned {
                node: ParamValue::String(s),
                span,
            })
        }
        Some(b) if b.is_ascii_digit() || b == b'-' || b == b'+' || b == b'n' => {
            // Could be integer, nth-expr — try parsing as nth-expr from cursor
            let nth_result = parse_nth_expr_from_cursor(cursor)?;
            let span = cursor.span_from(start);
            Ok(Spanned {
                node: ParamValue::NthExpr(nth_result),
                span,
            })
        }
        Some(b) if b.is_ascii_alphabetic() || b == b'_' => {
            let ident = cursor.eat_identifier().expect("identifier start checked");
            let span = cursor.span_from(start);
            Ok(Spanned {
                node: ParamValue::Identifier(ident),
                span,
            })
        }
        _ => Err(ParseError::UnexpectedToken {
            expected: "parameter value".into(),
            found: peek_found(cursor),
            span: cursor.span_from(start),
        }),
    }
}

/// Parse an nth expression inline from a cursor (for use inside param lists).
/// Stops at `)` or `,` followed by an identifier (next param).
fn parse_nth_expr_from_cursor(cursor: &mut Cursor) -> Result<NthExpr, ParseError> {
    let mut terms: Vec<Spanned<NthTerm>> = Vec::new();

    let first = parse_nth_term(cursor)?;
    if let Some(term) = first.0 {
        terms.push(term);
    }

    loop {
        cursor.eat_whitespace();
        if cursor.peek() != Some(b',') {
            break;
        }

        // Disambiguate: comma followed by identifier + colon = next param, not next nth term
        let saved_pos = cursor.pos();
        cursor.advance(); // consume ','
        cursor.eat_whitespace();

        // Check if this looks like the start of a named param: `identifier:`
        if is_param_start(cursor) {
            // Rewind — this comma belongs to the outer param list
            cursor.set_pos(saved_pos);
            break;
        }

        let term_result = parse_nth_term(cursor)?;
        if let Some(term) = term_result.0 {
            terms.push(term);
        }
    }

    Ok(NthExpr { terms })
}

/// Check if the cursor is at the start of a named parameter: `identifier:`.
fn is_param_start(cursor: &Cursor) -> bool {
    let remaining = cursor.remaining();
    let bytes = remaining.as_bytes();

    // Must start with identifier char
    if bytes.is_empty() || !(bytes[0].is_ascii_alphabetic() || bytes[0] == b'_') {
        return false;
    }

    // Find end of identifier
    let mut i = 1;
    while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
        i += 1;
    }

    // Must be followed by ':'
    if !(i < bytes.len() && bytes[i] == b':') {
        return false;
    }

    // Exclude `qed:` — that's a processor prefix, not a parameter name.
    let ident = &remaining[..i];
    ident != "qed"
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

/// Parse a processor chain: `processor ('|' processor)*`.
/// Stops before `||` (fallback) or end-of-statement.
fn parse_processor_chain(cursor: &mut Cursor) -> Result<ProcessorChain, ParseError> {
    let mut processors = vec![parse_processor(cursor)?];

    loop {
        let saved = cursor.pos();
        cursor.eat_whitespace();
        // `|` but not `||`
        if cursor.peek() == Some(b'|') && cursor.peek_at(1) != Some(b'|') {
            cursor.advance(); // consume `|`
            // Implicit line continuation after `|`
            eat_whitespace_and_newlines(cursor);
            processors.push(parse_processor(cursor)?);
        } else {
            cursor.set_pos(saved); // don't consume trailing whitespace
            break;
        }
    }

    Ok(ProcessorChain { processors })
}

/// Parse a single processor: `qed:*`, external command, or alias reference.
///
/// Bare identifiers without arguments are parsed as alias references.
/// Bare identifiers followed by arguments are parsed as external commands.
fn parse_processor(
    cursor: &mut Cursor,
) -> Result<Spanned<crate::parse::ast::Processor>, ParseError> {
    if cursor.remaining().starts_with("qed:") {
        return parse_qed_processor(cursor);
    }
    // Explicit external-command prefixes: `\`, `/path`, `./path`
    if matches!(cursor.peek(), Some(b'\\') | Some(b'/')) || cursor.remaining().starts_with("./") {
        return parse_external_processor(cursor);
    }
    // Bare identifier: disambiguate alias ref vs external command
    let saved = cursor.pos();
    if let Some(name) = cursor.eat_identifier() {
        let ident_end = cursor.pos();
        cursor.eat_whitespace();
        match cursor.peek() {
            // At a statement/chain delimiter → alias ref (no arguments)
            None | Some(b'|') | Some(b'\n') | Some(b'\r') | Some(b';') | Some(b')') => {
                cursor.set_pos(ident_end);
                let span = cursor.span_from(saved);
                return Ok(Spanned {
                    node: crate::parse::ast::Processor::AliasRef(name),
                    span,
                });
            }
            // Has arguments → external command
            _ => {
                cursor.set_pos(saved);
                return parse_external_processor(cursor);
            }
        }
    }
    Err(ParseError::UnexpectedToken {
        expected: "processor (qed:name(), \\command, or alias name)".into(),
        found: peek_found(cursor),
        span: cursor.span_from(cursor.pos()),
    })
}

/// Parse a qed processor: `qed:name(args, params)`.
/// Name can be colon-separated: `qed:debug:count()`.
fn parse_qed_processor(
    cursor: &mut Cursor,
) -> Result<Spanned<crate::parse::ast::Processor>, ParseError> {
    let start = cursor.pos();

    // Consume `qed:`
    if !cursor.eat_keyword("qed") {
        return Err(ParseError::UnexpectedToken {
            expected: "'qed:' processor prefix".into(),
            found: peek_found(cursor),
            span: cursor.span_from(start),
        });
    }
    if !cursor.eat_char(b':') {
        return Err(ParseError::UnexpectedToken {
            expected: "':' after 'qed'".into(),
            found: peek_found(cursor),
            span: cursor.span_from(cursor.pos()),
        });
    }

    // Parse processor name: identifier (':' identifier)*
    let name_start = cursor.pos();
    loop {
        match cursor.peek() {
            Some(b) if b.is_ascii_alphabetic() || b == b'_' => {
                cursor.advance();
            }
            _ => break,
        }
        while let Some(b) = cursor.peek() {
            if b.is_ascii_alphanumeric() || b == b'_' {
                cursor.advance();
            } else {
                break;
            }
        }
        // Continue on `:` followed by alpha/underscore (colon-separated name)
        if cursor.peek() == Some(b':')
            && let Some(next) = cursor.peek_at(1)
            && (next.is_ascii_alphabetic() || next == b'_')
        {
            cursor.advance(); // consume `:`
            continue;
        }
        break;
    }

    if cursor.pos() == name_start {
        return Err(ParseError::UnexpectedToken {
            expected: "processor name".into(),
            found: peek_found(cursor),
            span: cursor.span_from(name_start),
        });
    }
    let name = cursor.slice_from(name_start).to_owned();
    let name_span = cursor.span_from(name_start);

    if !cursor.eat_char(b'(') {
        return Err(ParseError::UnexpectedToken {
            expected: "'(' after processor name".into(),
            found: peek_found(cursor),
            span: cursor.span_from(cursor.pos()),
        });
    }

    cursor.eat_whitespace();

    // Parse args and params inside parens
    let mut args: Vec<Spanned<QedArg>> = Vec::new();
    let mut params: Vec<Spanned<Param>> = Vec::new();
    let mut in_params = false;

    if cursor.peek() != Some(b')') {
        loop {
            cursor.eat_whitespace();
            if cursor.peek() == Some(b')') {
                break;
            }

            // Switch to params once we see `identifier:`
            if !in_params && is_param_start(cursor) {
                in_params = true;
            }

            if in_params {
                params.push(parse_param(cursor)?);
            } else {
                args.push(parse_qed_arg(cursor)?);
            }

            cursor.eat_whitespace();
            if !cursor.eat_char(b',') {
                break;
            }
        }
    }

    if !cursor.eat_char(b')') {
        return Err(ParseError::UnexpectedToken {
            expected: "')'".into(),
            found: peek_found(cursor),
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
            args,
            params,
        }),
        span,
    })
}

/// Parse a single positional argument to a qed processor.
fn parse_qed_arg(cursor: &mut Cursor) -> Result<Spanned<QedArg>, ParseError> {
    let start = cursor.pos();
    cursor.eat_whitespace();

    match cursor.peek() {
        Some(b'"') => {
            let s = cursor
                .eat_string_literal()
                .ok_or_else(|| ParseError::UnexpectedEof {
                    expected: "closing '\"'".into(),
                    span: cursor.span_from(start),
                })?;
            Ok(Spanned {
                node: QedArg::String(s),
                span: cursor.span_from(start),
            })
        }
        Some(b'/') => {
            let r = cursor
                .eat_regex_literal()
                .ok_or_else(|| ParseError::UnexpectedEof {
                    expected: "closing '/'".into(),
                    span: cursor.span_from(start),
                })?;
            Ok(Spanned {
                node: QedArg::Regex(r),
                span: cursor.span_from(start),
            })
        }
        // Integer (possibly negative)
        Some(b)
            if b.is_ascii_digit()
                || (b == b'-' && cursor.peek_at(1).is_some_and(|n| n.is_ascii_digit())) =>
        {
            let neg = cursor.eat_char(b'-');
            let num_start = cursor.pos();
            while cursor.peek().is_some_and(|b| b.is_ascii_digit()) {
                cursor.advance();
            }
            let digits = cursor.slice_from(num_start);
            let val: i64 = digits.parse().map_err(|_| ParseError::UnexpectedToken {
                expected: "integer".into(),
                found: digits.to_owned(),
                span: cursor.span_from(start),
            })?;
            Ok(Spanned {
                node: QedArg::Integer(if neg { -val } else { val }),
                span: cursor.span_from(start),
            })
        }
        // Pattern ref (negated `!` or single-quoted `'`)
        Some(b'!') | Some(b'\'') => {
            let pat = parse_pattern_ref(cursor)?;
            Ok(Spanned {
                node: QedArg::PatternRef(pat.node),
                span: pat.span,
            })
        }
        // Processor chain: `qed:name(...)`, bare command, `\command`, `/path`, `./path`
        _ if cursor.peek().is_some_and(|b| {
            b.is_ascii_alphabetic() || b == b'_' || b == b'\\' || b == b'.' || b == b'/'
        }) =>
        {
            let chain = parse_processor_chain(cursor)?;
            Ok(Spanned {
                node: QedArg::ProcessorChain(Box::new(chain)),
                span: cursor.span_from(start),
            })
        }
        _ => Err(ParseError::UnexpectedToken {
            expected: "processor argument (string, regex, integer, pattern, or processor chain)"
                .into(),
            found: peek_found(cursor),
            span: cursor.span_from(start),
        }),
    }
}

/// Parse an external processor: `command arg1 arg2 ...`.
/// Stops at `|`, `||`, `)`, newline, `;`, or EOF.
fn parse_external_processor(
    cursor: &mut Cursor,
) -> Result<Spanned<crate::parse::ast::Processor>, ParseError> {
    let start = cursor.pos();

    // Check for `\` escape prefix
    let escaped = cursor.eat_char(b'\\');

    // Parse command name or path
    let cmd_start = cursor.pos();
    match cursor.peek() {
        // Path starting with `/` or `.`
        Some(b'/') | Some(b'.') => {
            while let Some(b) = cursor.peek() {
                match b {
                    b' ' | b'\t' | b'\n' | b'|' | b';' | b')' => break,
                    _ => {
                        cursor.advance();
                    }
                }
            }
        }
        // Command name: [a-zA-Z_][a-zA-Z0-9_-]*
        Some(b) if b.is_ascii_alphabetic() || b == b'_' => {
            cursor.advance();
            while let Some(b) = cursor.peek() {
                if b.is_ascii_alphanumeric() || b == b'_' || b == b'-' {
                    cursor.advance();
                } else {
                    break;
                }
            }
        }
        _ => {
            return Err(ParseError::UnexpectedToken {
                expected: "processor (qed:name() or external command)".into(),
                found: peek_found(cursor),
                span: cursor.span_from(start),
            });
        }
    }

    if cursor.pos() == cmd_start {
        return Err(ParseError::UnexpectedToken {
            expected: "command name".into(),
            found: peek_found(cursor),
            span: cursor.span_from(cmd_start),
        });
    }

    let command = cursor.slice_from(cmd_start).to_owned();
    let command_span = cursor.span_from(cmd_start);

    // Parse arguments: space-separated, quoted or unquoted.
    // `\<newline>` joins lines (explicit line continuation).
    // `\<whitespace><newline>` is a hard error.
    let mut args: Vec<Spanned<ExternalArg>> = Vec::new();
    loop {
        let saved = cursor.pos();
        cursor.eat_whitespace();

        // Handle backslash line continuation
        if cursor.peek() == Some(b'\\') {
            let bs_pos = cursor.pos();
            cursor.advance(); // consume `\`
            match cursor.peek() {
                Some(b'\n') => {
                    cursor.advance(); // consume newline — join lines
                    continue;
                }
                Some(b' ') | Some(b'\t') => {
                    // Trailing whitespace after `\` — hard error
                    return Err(ParseError::UnexpectedToken {
                        expected: "newline after '\\'".into(),
                        found: "trailing whitespace".into(),
                        span: cursor.span_from(bs_pos),
                    });
                }
                _ => {
                    // Not a continuation — backtrack so the `\` is handled
                    // as a command escape prefix or causes a normal stop
                    cursor.set_pos(bs_pos);
                }
            }
        }

        match cursor.peek() {
            None | Some(b'|') | Some(b')') | Some(b'\n') | Some(b';') => {
                cursor.set_pos(saved);
                break;
            }
            Some(b'"') => {
                let arg_start = cursor.pos();
                let s = cursor
                    .eat_string_literal()
                    .ok_or_else(|| ParseError::UnexpectedEof {
                        expected: "closing '\"'".into(),
                        span: cursor.span_from(arg_start),
                    })?;
                args.push(Spanned {
                    node: ExternalArg::Quoted(s),
                    span: cursor.span_from(arg_start),
                });
            }
            Some(b'\'') => {
                let arg_start = cursor.pos();
                let s = cursor.eat_single_quoted_string_literal().ok_or_else(|| {
                    ParseError::UnexpectedEof {
                        expected: "closing '''".into(),
                        span: cursor.span_from(arg_start),
                    }
                })?;
                args.push(Spanned {
                    node: ExternalArg::Quoted(s),
                    span: cursor.span_from(arg_start),
                });
            }
            _ => {
                let arg_start = cursor.pos();
                let s = cursor
                    .eat_unquoted_arg()
                    .ok_or_else(|| ParseError::UnexpectedToken {
                        expected: "argument".into(),
                        found: peek_found(cursor),
                        span: cursor.span_from(arg_start),
                    })?;
                args.push(Spanned {
                    node: ExternalArg::Unquoted(s),
                    span: cursor.span_from(arg_start),
                });
            }
        }
    }

    let span = cursor.span_from(start);
    Ok(Spanned {
        node: crate::parse::ast::Processor::External(ExternalProcessor {
            command: Spanned {
                node: command,
                span: command_span,
            },
            escaped,
            args,
        }),
        span,
    })
}

/// Check whether the cursor is at the start of a selector keyword.
fn is_selector_start(cursor: &Cursor) -> bool {
    let r = cursor.remaining();
    r.starts_with("at(")
        || r.starts_with("after(")
        || r.starts_with("before(")
        || r.starts_with("from(")
        || r.starts_with("to(")
}

/// Skip whitespace, newlines, carriage returns, and `# comment` lines.
fn eat_whitespace_and_newlines(cursor: &mut Cursor) {
    loop {
        match cursor.peek() {
            Some(b' ' | b'\t' | b'\n' | b'\r') => {
                cursor.advance();
            }
            Some(b'#') => {
                // Skip comment line (but not shebang — that's handled separately)
                while let Some(b) = cursor.peek() {
                    cursor.advance();
                    if b == b'\n' {
                        break;
                    }
                }
            }
            _ => break,
        }
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
            let coeff_str = if coeff_value == 1 {
                "n"
            } else {
                &format!("{coeff_value}n")
            };
            warnings.push(ParseError::NthWarning {
                reason: format!("leading '+' ignored, `+{coeff_str}` treated as `{coeff_str}`"),
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

            let v: i64 = offset_digits
                .parse()
                .map_err(|_| ParseError::InvalidNthExpr {
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
        parse_nth_expr(input)
            .unwrap_or_else(|errs| panic!("expected Ok for {input:?}, got errors: {errs:?}"))
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
        assert_eq!(r.expr.terms[0].node, NthTerm::Range { start: 1, end: 3 });
        assert_eq!(r.expr.terms[0].span, crate::span::Span { start: 0, end: 5 });
    }

    #[test]
    fn range_negative() {
        let r = parse_ok("-3...-1");
        assert_eq!(r.expr.terms[0].node, NthTerm::Range { start: -3, end: -1 });
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
        assert_eq!(r.expr.terms[0].node, NthTerm::Range { start: 1, end: 3 });
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
        assert!(
            matches!(&errs[0], ParseError::InvalidNthExpr { reason, .. } if reason.contains("coefficient"))
        );
    }

    #[test]
    fn error_cross_sign_range_neg_to_pos() {
        let errs = parse_err("-3...5");
        assert!(
            matches!(&errs[0], ParseError::InvalidNthExpr { reason, .. } if reason.contains("same sign"))
        );
    }

    #[test]
    fn error_cross_sign_range_pos_to_neg() {
        let errs = parse_err("3...-1");
        assert!(
            matches!(&errs[0], ParseError::InvalidNthExpr { reason, .. } if reason.contains("same sign"))
        );
    }

    #[test]
    fn error_zero_offset_plus() {
        let errs = parse_err("2n+0");
        assert!(
            matches!(&errs[0], ParseError::InvalidNthExpr { reason, .. } if reason.contains("non-zero"))
        );
    }

    #[test]
    fn error_zero_offset_minus() {
        let errs = parse_err("2n-0");
        assert!(
            matches!(&errs[0], ParseError::InvalidNthExpr { reason, .. } if reason.contains("non-zero"))
        );
    }

    // ── Warnings ────────────────────────────────────────────────────

    #[test]
    fn warning_zero_ignored() {
        let r = parse_ok("0,1");
        // '0' is ignored, result has just Integer(1).
        assert_eq!(r.expr.terms.len(), 1);
        assert_eq!(r.expr.terms[0].node, NthTerm::Integer(1));
        assert!(r.warnings.iter().any(
            |w| matches!(w, ParseError::NthWarning { reason, .. } if reason.contains("zero"))
        ));
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
        assert!(
            r.warnings.iter().any(
                |w| matches!(w, ParseError::NthWarning { reason, .. } if reason.contains("+"))
            )
        );
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
        assert!(
            matches!(&errs[0], ParseError::UnexpectedToken { found, .. } if found.contains("two dots"))
        );
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
        assert_eq!(r.expr.terms[0].node, NthTerm::Range { start: 1, end: 3 });
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
    fn all_zeros_returns_empty() {
        // Zero terms are stripped at parse time; the compiler emits a warning.
        let r = parse_ok("0");
        assert!(r.expr.terms.is_empty());
    }

    // ── Program parser tests ────────────────────────────────────────

    fn program_ok(input: &str) -> Program {
        parse_program(input)
            .unwrap_or_else(|errs| panic!("expected Ok for {input:?}, got errors: {errs:?}"))
    }

    fn program_err(input: &str) -> Vec<ParseError> {
        parse_program(input).expect_err(&format!("expected Err for {input:?}"))
    }

    // ── Pattern ref forms ───────────────────────────────────────────

    #[test]
    fn pattern_string_literal() {
        let p = program_ok(r#"at("hello") | qed:delete()"#);
        assert_eq!(p.statements.len(), 1);
        let stmt = &p.statements[0].node;
        if let Statement::SelectAction(sa) = stmt {
            let pat = sa.selector.node.steps[0].node.pattern.as_ref().unwrap();
            assert_eq!(
                pat.node.value,
                PatternRefValue::Inline(PatternValue::String("hello".into()))
            );
            assert!(!pat.node.negated);
            assert!(!pat.node.inclusive);
        } else {
            panic!("expected SelectAction");
        }
    }

    #[test]
    fn pattern_single_quoted_string() {
        let p = program_ok("at('hello') | qed:delete()");
        let stmt = &p.statements[0].node;
        if let Statement::SelectAction(sa) = stmt {
            let pat = sa.selector.node.steps[0].node.pattern.as_ref().unwrap();
            assert_eq!(
                pat.node.value,
                PatternRefValue::Inline(PatternValue::String("hello".into()))
            );
        } else {
            panic!("expected SelectAction");
        }
    }

    #[test]
    fn pattern_regex() {
        let p = program_ok("at(/^hello/) | qed:delete()");
        let stmt = &p.statements[0].node;
        if let Statement::SelectAction(sa) = stmt {
            let pat = sa.selector.node.steps[0].node.pattern.as_ref().unwrap();
            assert_eq!(
                pat.node.value,
                PatternRefValue::Inline(PatternValue::Regex("^hello".into()))
            );
        } else {
            panic!("expected SelectAction");
        }
    }

    #[test]
    fn pattern_negated() {
        let p = program_ok(r#"at(!"hello") | qed:delete()"#);
        let stmt = &p.statements[0].node;
        if let Statement::SelectAction(sa) = stmt {
            let pat = sa.selector.node.steps[0].node.pattern.as_ref().unwrap();
            assert!(pat.node.negated);
            assert_eq!(
                pat.node.value,
                PatternRefValue::Inline(PatternValue::String("hello".into()))
            );
        } else {
            panic!("expected SelectAction");
        }
    }

    #[test]
    fn pattern_negated_regex() {
        let p = program_ok("at(!/^b/) | qed:delete()");
        let stmt = &p.statements[0].node;
        if let Statement::SelectAction(sa) = stmt {
            let pat = sa.selector.node.steps[0].node.pattern.as_ref().unwrap();
            assert!(pat.node.negated);
            assert_eq!(
                pat.node.value,
                PatternRefValue::Inline(PatternValue::Regex("^b".into()))
            );
        } else {
            panic!("expected SelectAction");
        }
    }

    #[test]
    fn pattern_inclusive() {
        let p = program_ok(r#"from("hello"+) | qed:delete()"#);
        let stmt = &p.statements[0].node;
        if let Statement::SelectAction(sa) = stmt {
            let pat = sa.selector.node.steps[0].node.pattern.as_ref().unwrap();
            assert!(pat.node.inclusive);
        } else {
            panic!("expected SelectAction");
        }
    }

    #[test]
    fn pattern_negated_inclusive() {
        let p = program_ok(r#"from(!"hello"+) | qed:delete()"#);
        let stmt = &p.statements[0].node;
        if let Statement::SelectAction(sa) = stmt {
            let pat = sa.selector.node.steps[0].node.pattern.as_ref().unwrap();
            assert!(pat.node.negated);
            assert!(pat.node.inclusive);
        } else {
            panic!("expected SelectAction");
        }
    }

    #[test]
    fn pattern_named_ref() {
        let p = program_ok("at(mypattern) | qed:delete()");
        let stmt = &p.statements[0].node;
        if let Statement::SelectAction(sa) = stmt {
            let pat = sa.selector.node.steps[0].node.pattern.as_ref().unwrap();
            assert_eq!(pat.node.value, PatternRefValue::Named("mypattern".into()));
        } else {
            panic!("expected SelectAction");
        }
    }

    // ── Error cases ─────────────────────────────────────────────────

    #[test]
    fn error_unterminated_regex() {
        let errs = program_err("at(/unterminated) | qed:delete()");
        assert!(!errs.is_empty());
    }

    #[test]
    fn error_unterminated_single_quote() {
        let errs = program_err("at('unterminated) | qed:delete()");
        assert!(!errs.is_empty());
    }

    // ── Shebang ─────────────────────────────────────────────────────

    #[test]
    fn shebang_preserved() {
        let p = program_ok("#!/usr/bin/env qed\nat(\"x\") | qed:delete()");
        assert!(p.shebang.is_some());
        assert_eq!(p.shebang.unwrap().node, "/usr/bin/env qed");
        assert_eq!(p.statements.len(), 1);
    }

    #[test]
    fn no_shebang() {
        let p = program_ok(r#"at("x") | qed:delete()"#);
        assert!(p.shebang.is_none());
    }

    // ── Comments ────────────────────────────────────────────────────

    #[test]
    fn comment_skipped() {
        let p = program_ok("# this is a comment\nat(\"x\") | qed:delete()");
        assert_eq!(p.statements.len(), 1);
    }

    #[test]
    fn comment_only_program() {
        let p = program_ok("# just a comment\n");
        assert_eq!(p.statements.len(), 0);
    }

    // ── Compound selectors ──────────────────────────────────────────

    #[test]
    fn compound_selector() {
        let p = program_ok(r#"from("start") > to("end") | qed:delete()"#);
        let stmt = &p.statements[0].node;
        if let Statement::SelectAction(sa) = stmt {
            assert_eq!(sa.selector.node.steps.len(), 2);
            assert_eq!(sa.selector.node.steps[0].node.op, SelectorOp::From);
            assert_eq!(sa.selector.node.steps[1].node.op, SelectorOp::To);
        } else {
            panic!("expected SelectAction");
        }
    }

    // ── Bare selector ───────────────────────────────────────────────

    #[test]
    fn bare_at_selector() {
        let p = program_ok("at() | qed:delete()");
        let stmt = &p.statements[0].node;
        if let Statement::SelectAction(sa) = stmt {
            assert!(sa.selector.node.steps[0].node.pattern.is_none());
        } else {
            panic!("expected SelectAction");
        }
    }

    // ── Backslash line continuation ────────────────────────────────

    #[test]
    fn backslash_continuation_joins_args() {
        use crate::parse::ast::{ExternalArg, Processor as AstProc};
        let p = program_ok("at(\"x\") | sed \\\n-e 's/x/y/'");
        let stmt = &p.statements[0].node;
        if let Statement::SelectAction(sa) = stmt {
            let chain = sa.chain.as_ref().unwrap();
            if let AstProc::External(ext) = &chain.node.processors[0].node {
                assert_eq!(ext.command.node, "sed");
                assert_eq!(ext.args.len(), 2);
                assert_eq!(ext.args[0].node, ExternalArg::Unquoted("-e".into()));
                assert_eq!(ext.args[1].node, ExternalArg::Quoted("s/x/y/".into()));
            } else {
                panic!("expected External processor");
            }
        } else {
            panic!("expected SelectAction");
        }
    }

    #[test]
    fn backslash_continuation_multiple_lines() {
        use crate::parse::ast::{ExternalArg, Processor as AstProc};
        let p = program_ok("at(\"x\") | cmd \\\narg1 \\\narg2");
        let stmt = &p.statements[0].node;
        if let Statement::SelectAction(sa) = stmt {
            let chain = sa.chain.as_ref().unwrap();
            if let AstProc::External(ext) = &chain.node.processors[0].node {
                assert_eq!(ext.command.node, "cmd");
                assert_eq!(ext.args.len(), 2);
                assert_eq!(ext.args[0].node, ExternalArg::Unquoted("arg1".into()));
                assert_eq!(ext.args[1].node, ExternalArg::Unquoted("arg2".into()));
            } else {
                panic!("expected External processor");
            }
        } else {
            panic!("expected SelectAction");
        }
    }

    #[test]
    fn backslash_trailing_whitespace_error() {
        let errs = program_err("at(\"x\") | cmd \\  \narg1");
        assert!(!errs.is_empty());
        match &errs[0] {
            ParseError::UnexpectedToken { found, .. } => {
                assert!(found.contains("trailing whitespace"));
            }
            other => panic!("expected UnexpectedToken, got {other:?}"),
        }
    }
}
