//! Parser for test declarations in an Oneil program.

use nom::{
    Parser,
    combinator::{all_consuming, opt},
};
use oneil_ast::{Node, Test, TestNode, TraceLevel, TraceLevelNode};
use oneil_shared::span::Span;

use crate::{
    error::{ErrorHandlingParser, ParserError},
    expression::parse as parse_expr,
    note::parse as parse_note,
    token::{
        keyword::test as test_keyword,
        structure::end_of_line,
        symbol::{colon, star, star_star},
    },
    util::{InputSpan, Result},
};

/// Parse a test declaration, e.g. `* test: x > y`.
///
/// This function **may not consume the complete input**.
pub fn parse(input: InputSpan<'_>) -> Result<'_, TestNode, ParserError> {
    test_decl(input)
}

/// Parse a test declaration
///
/// This function **fails if the complete input is not consumed**.
pub fn parse_complete(input: InputSpan<'_>) -> Result<'_, TestNode, ParserError> {
    all_consuming(test_decl).parse(input)
}

/// Parses a test declaration with optional trace level and inputs.
fn test_decl(input: InputSpan<'_>) -> Result<'_, TestNode, ParserError> {
    let (rest, trace_level_node) = opt(trace_level).parse(input)?;

    let (rest, test_keyword_token) = test_keyword
        .convert_error_to(ParserError::expect_test)
        .parse(rest)?;

    let (rest, colon_token) = colon
        .or_fail_with(ParserError::test_missing_colon(
            test_keyword_token.lexeme_span,
        ))
        .parse(rest)?;

    let (rest, expr_node) = parse_expr
        .or_fail_with(ParserError::test_missing_expr(colon_token.lexeme_span))
        .parse(rest)?;

    let (rest, linebreak_token) = end_of_line
        .or_fail_with(ParserError::test_missing_end_of_line(expr_node.span()))
        .parse(rest)?;

    let (rest, note_node) = opt(parse_note).parse(rest)?;

    let test_start_span = match &trace_level_node {
        Some(trace_level_node) => trace_level_node.span(),
        None => test_keyword_token.lexeme_span,
    };

    let (test_end_span, test_whitespace_span) = match &note_node {
        Some(note_node) => (note_node.span(), note_node.whitespace_span()),
        None => (linebreak_token.lexeme_span, linebreak_token.whitespace_span),
    };

    let test_span = Span::from_start_and_end(&test_start_span, &test_end_span);
    let test_whitespace_span = Span::from_start_and_end(&test_start_span, &test_whitespace_span);

    let test_node = Node::new(
        Test::new(trace_level_node, expr_node, note_node),
        test_span,
        test_whitespace_span,
    );

    Ok((rest, test_node))
}

/// Parse a trace level indicator (`*` or `**`).
fn trace_level(input: InputSpan<'_>) -> Result<'_, TraceLevelNode, ParserError> {
    let single_star = star.map(|token| token.into_node_with_value(TraceLevel::Trace));
    let double_star = star_star.map(|token| token.into_node_with_value(TraceLevel::Debug));

    double_star.or(single_star).convert_errors().parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Config,
        error::reason::{ExpectKind, IncompleteKind, ParserErrorReason, TestKind},
    };
    use oneil_ast::{Expr, Literal};

    mod success {
        use std::ops::Deref;

        use oneil_ast::Note;

        use super::*;

        #[test]
        fn decl_basic() {
            let input = InputSpan::new_extra("test: true\n", Config::default());
            let (rest, test) = parse(input).expect("should parse test");

            let test = test.take_value();

            let Expr::Literal(value) = test.expr().clone().take_value() else {
                panic!("expected literal");
            };
            let Literal::Boolean(value) = value.take_value() else {
                panic!("expected boolean");
            };
            assert_eq!(value, true);

            assert_eq!(test.trace_level(), None);

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn decl_at_eof() {
            let input = InputSpan::new_extra("test: true", Config::default());
            let (rest, test) = parse(input).expect("should parse test");

            let test = test.take_value();

            let Expr::Literal(value) = test.expr().clone().take_value() else {
                panic!("expected literal");
            };
            let Literal::Boolean(value) = value.take_value() else {
                panic!("expected boolean");
            };
            assert_eq!(value, true);

            assert_eq!(test.trace_level(), None);

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn decl_with_trace() {
            let input = InputSpan::new_extra("* test: true\n", Config::default());
            let (rest, test) = parse(input).expect("should parse test");

            let test = test.take_value();

            assert_eq!(
                test.trace_level().map(Node::deref).cloned(),
                Some(TraceLevel::Trace)
            );

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn decl_with_debug() {
            let input = InputSpan::new_extra("** test: true\n", Config::default());
            let (rest, test) = parse(input).expect("should parse test");

            let test = test.take_value();

            assert_eq!(
                test.trace_level().map(Node::deref).cloned(),
                Some(TraceLevel::Debug)
            );

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn decl_with_note() {
            let input = InputSpan::new_extra("test: true\n~ This is a note\n", Config::default());
            let (rest, test) = parse(input).expect("should parse test");

            assert_eq!(
                test.note().map(Node::deref).cloned(),
                Some(Note::new("This is a note".to_string()))
            );

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn decl_full() {
            let input = InputSpan::new_extra("** test: x > y\n", Config::default());
            let (rest, test) = parse(input).expect("should parse test");

            let test = test.take_value();
            assert_eq!(
                test.trace_level().map(Node::deref).cloned(),
                Some(TraceLevel::Debug)
            );

            assert_eq!(rest.fragment(), &"");
        }
    }

    mod parse_complete {
        use super::*;

        #[test]
        fn parse_complete_success() {
            let input = InputSpan::new_extra("test: true\n", Config::default());
            let (rest, test) = parse_complete(input).expect("should parse test");

            let test = test.take_value();

            let Expr::Literal(value) = test.expr().clone().take_value() else {
                panic!("expected literal");
            };

            let Literal::Boolean(value) = value.take_value() else {
                panic!("expected boolean");
            };
            assert_eq!(value, true);

            assert_eq!(test.trace_level(), None);
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        #[expect(
            clippy::assertions_on_result_states,
            reason = "we don't care about the result, just that it's an error"
        )]
        fn parse_complete_with_remaining_input() {
            let input = InputSpan::new_extra("test: true\n extra", Config::default());
            let result = parse_complete(input);
            assert!(result.is_err());
        }
    }

    mod error {
        use super::*;

        #[test]
        fn missing_test_keyword() {
            let input = InputSpan::new_extra(": true\n", Config::default());
            let result = parse(input);

            let Err(nom::Err::Error(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 0);
            assert!(matches!(
                error.reason,
                ParserErrorReason::Expect(ExpectKind::Test)
            ));
        }

        #[test]
        fn missing_colon_after_test() {
            let input = InputSpan::new_extra("test true\n", Config::default());
            let result = parse(input);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 5); // After "test"
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Test(TestKind::MissingColon),
                cause,
            } = error.reason
            else {
                panic!("Unexpected error {:?}", error.reason);
            };

            assert_eq!(cause.start().offset, 0);
            assert_eq!(cause.end().offset, 4);
        }

        #[test]
        fn missing_expression() {
            let input = InputSpan::new_extra("test:\n", Config::default());
            let result = parse(input);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 5); // After ":"
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Test(TestKind::MissingExpr),
                cause,
            } = error.reason
            else {
                panic!("Unexpected error {error:?}");
            };

            assert_eq!(cause.start().offset, 4);
            assert_eq!(cause.end().offset, 5);
        }

        #[test]
        fn missing_colon_with_debug() {
            let input = InputSpan::new_extra("** test true\n", Config::default());
            let result = parse(input);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 8); // After "test"
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Test(TestKind::MissingColon),
                cause,
            } = error.reason
            else {
                panic!("Unexpected error {error:?}");
            };

            assert_eq!(cause.start().offset, 3);
            assert_eq!(cause.end().offset, 7);
        }

        #[test]
        fn missing_expression_with_trace() {
            let input = InputSpan::new_extra("* test:\n", Config::default());
            let result = parse(input);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 7); // After ":"
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Test(TestKind::MissingExpr),
                cause,
            } = error.reason
            else {
                panic!("Unexpected error {error:?}");
            };

            assert_eq!(cause.start().offset, 6);
            assert_eq!(cause.end().offset, 7);
        }

        #[test]
        fn invalid_trace_level() {
            let input = InputSpan::new_extra("*** test: true\n", Config::default());
            let result = parse(input);

            let Err(nom::Err::Error(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 2);
            assert!(matches!(
                error.reason,
                ParserErrorReason::Expect(ExpectKind::Test)
            ));
        }

        #[test]
        fn empty_input() {
            let input = InputSpan::new_extra("", Config::default());
            let result = parse(input);

            let Err(nom::Err::Error(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 0);
            assert!(matches!(
                error.reason,
                ParserErrorReason::Expect(ExpectKind::Test)
            ));
        }

        #[test]
        fn whitespace_only() {
            let input = InputSpan::new_extra("   \n", Config::default());
            let result = parse(input);

            let Err(nom::Err::Error(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 0); // After whitespace
            assert!(matches!(
                error.reason,
                ParserErrorReason::Expect(ExpectKind::Test)
            ));
        }
    }
}
