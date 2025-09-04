//! Parser for test declarations in an Oneil program.
//!
//! This module provides parsing functionality for Oneil test declarations.
//! A test declaration defines a validation condition that can be used to
//! verify the correctness of a model or its components.
//!
//! # Test Declaration Syntax
//!
//! The general syntax for a test declaration is:
//!
//! ```text
//! [trace_level] test [inputs]: expression
//! ```
//!
//! Where:
//! - `trace_level` is optional and can be `*` (trace) or `**` (debug)
//! - `test` is the required keyword
//! - `:` is the required separator
//! - `expression` is the test condition that should evaluate to true
//!
//! # Examples
//!
//! ```rust
//! use oneil_parser::parse_test;
//!
//! // Simple test
//! let test = parse_test("test: x > 0", None).unwrap();
//!
//! // Test with inputs
//! let test = parse_test("test: x > y", None).unwrap();
//!
//! // Test with trace level
//! let test = parse_test("* test: temperature > 0", None).unwrap();
//!
//! // Complex test with all components
//! let test = parse_test("** test: a > b and b > c", None).unwrap();
//! ```

use nom::{
    Parser,
    combinator::{all_consuming, opt},
};
use oneil_ast::{AstSpan, Node, Test, TestNode, TraceLevel, TraceLevelNode};

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
///
/// A test declaration has the following structure:
/// `[trace_level] test [inputs]: expression`
///
/// Where:
/// - `trace_level` is optional and can be `*` (trace) or `**` (debug)
/// - `test` is the required keyword
/// - `:` is the required separator
/// - `expression` is the test condition
///
/// The function handles all combinations of these components with proper
/// error handling for missing required elements.
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns a test node containing the parsed test declaration.
///
/// # Error Handling
///
/// The function provides detailed error messages for common parsing failures:
/// - Missing `test` keyword
/// - Missing expression after colon
/// - Missing end of line after expression
/// - Missing inputs after opening brace
/// - Missing closing brace for inputs
///
/// # Examples
///
/// ```rust
/// use oneil_parser::parse_test;
///
/// // Basic test
/// let test = parse_test("test: x > 0", None).unwrap();
///
/// // Test with trace level
/// let test = parse_test("* test: temperature > 0", None).unwrap();
///
/// // Test with all components
/// let test = parse_test("** test: a > b and b > c", None).unwrap();
/// ```
fn test_decl(input: InputSpan<'_>) -> Result<'_, TestNode, ParserError> {
    let (rest, trace_level) = opt(trace_level).parse(input)?;

    let (rest, test_keyword_token) = test_keyword
        .convert_error_to(ParserError::expect_test)
        .parse(rest)?;

    let (rest, colon_token) = colon
        .or_fail_with(ParserError::test_missing_colon(&test_keyword_token))
        .parse(rest)?;

    let (rest, expr) = parse_expr
        .or_fail_with(ParserError::test_missing_expr(&colon_token))
        .parse(rest)?;

    let (rest, linebreak_token) = end_of_line
        .or_fail_with(ParserError::test_missing_end_of_line(&expr))
        .parse(rest)?;

    let (rest, note) = opt(parse_note).parse(rest)?;

    // note that for the purposes of span calculation, the note is considered
    // "whitespace"
    let whitespace_span = match &note {
        Some(note) => AstSpan::calc_span(&linebreak_token, note),
        None => AstSpan::from(&linebreak_token),
    };

    let span = match &trace_level {
        Some(trace_level) => {
            AstSpan::calc_span_with_whitespace(trace_level, &expr, &whitespace_span)
        }
        None => AstSpan::calc_span_with_whitespace(&test_keyword_token, &expr, &whitespace_span),
    };

    let test = Test::new(trace_level, expr, note);

    Ok((rest, Node::new(&span, test)))
}

/// Parse a trace level indicator (`*` or `**`).
///
/// Trace levels indicate the debugging/tracing level for a test:
/// - `*` indicates trace level (basic debugging)
/// - `**` indicates debug level (detailed debugging)
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns a trace level node if a valid trace indicator is found, or an error
/// if one isn't found.
///
/// # Examples
///
/// ```rust
/// use oneil_parser::parse_test;
///
/// // Test with trace level
/// let test = parse_test("* test: x > 0", None).unwrap();
/// assert!(test.trace_level().is_some());
///
/// // Test with debug level
/// let test = parse_test("** test: x > 0", None).unwrap();
/// assert!(test.trace_level().is_some());
/// ```
fn trace_level(input: InputSpan<'_>) -> Result<'_, TraceLevelNode, ParserError> {
    let single_star = star.map(|token| Node::new(&token, TraceLevel::Trace));
    let double_star = star_star.map(|token| Node::new(&token, TraceLevel::Debug));

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

    mod success_tests {
        use oneil_ast::Note;

        use super::*;

        #[test]
        fn test_decl_basic() {
            let input = InputSpan::new_extra("test: true\n", Config::default());
            let (rest, test) = parse(input).expect("should parse test");

            let expected_span = AstSpan::new(0, 10, 1);

            let expected_test_expr = Node::new(&AstSpan::new(6, 4, 0), Literal::boolean(true));
            let expected_test_expr =
                Node::new(&AstSpan::new(6, 4, 0), Expr::literal(expected_test_expr));

            assert_eq!(test.node_span(), expected_span);
            assert_eq!(test.trace_level(), None);
            assert_eq!(test.expr(), &expected_test_expr);

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_decl_at_eof() {
            let input = InputSpan::new_extra("test: true", Config::default());
            let (rest, test) = parse(input).expect("should parse test");

            let expected_span = AstSpan::new(0, 10, 0);

            let expected_test_expr = Node::new(&AstSpan::new(6, 4, 0), Literal::boolean(true));
            let expected_test_expr =
                Node::new(&AstSpan::new(6, 4, 0), Expr::literal(expected_test_expr));

            assert_eq!(test.node_span(), expected_span);
            assert_eq!(test.trace_level(), None);
            assert_eq!(test.expr(), &expected_test_expr);

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_decl_with_trace() {
            let input = InputSpan::new_extra("* test: true\n", Config::default());
            let (rest, test) = parse(input).expect("should parse test");

            let expected_span = AstSpan::new(0, 12, 1);

            assert_eq!(test.node_span(), expected_span);
            assert_eq!(
                test.trace_level().map(Node::node_value),
                Some(&TraceLevel::Trace)
            );

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_decl_with_debug() {
            let input = InputSpan::new_extra("** test: true\n", Config::default());
            let (rest, test) = parse(input).expect("should parse test");

            let expected_span = AstSpan::new(0, 13, 1);

            assert_eq!(test.node_span(), expected_span);
            assert_eq!(
                test.trace_level().map(Node::node_value),
                Some(&TraceLevel::Debug)
            );

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_decl_with_note() {
            let input = InputSpan::new_extra("test: true\n~ This is a note\n", Config::default());
            let (rest, test) = parse(input).expect("should parse test");

            let expected_span = AstSpan::new(0, 10, 18);

            assert_eq!(test.node_span(), expected_span);

            let note = test.note().expect("note should be present");
            assert_eq!(note.node_value(), &Note::new("This is a note".to_string()));

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_decl_full() {
            let input = InputSpan::new_extra("** test: x > y\n", Config::default());
            let (rest, test) = parse(input).expect("should parse test");

            let expected_span = AstSpan::new(0, 14, 1);

            assert_eq!(test.node_span(), expected_span);
            assert_eq!(
                test.trace_level().map(Node::node_value),
                Some(&TraceLevel::Debug)
            );

            assert_eq!(rest.fragment(), &"");
        }
    }

    mod parse_complete_tests {
        use super::*;

        #[test]
        fn test_parse_complete_success() {
            let input = InputSpan::new_extra("test: true\n", Config::default());
            let (rest, test) = parse_complete(input).expect("should parse test");

            let expected_span = AstSpan::new(0, 10, 1);

            let expected_test_expr = Node::new(&AstSpan::new(6, 4, 0), Literal::boolean(true));
            let expected_test_expr =
                Node::new(&AstSpan::new(6, 4, 0), Expr::literal(expected_test_expr));

            assert_eq!(test.node_span(), expected_span);
            assert_eq!(test.trace_level(), None);
            assert_eq!(test.expr(), &expected_test_expr);
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_parse_complete_with_remaining_input() {
            let input = InputSpan::new_extra("test: true\n extra", Config::default());
            let result = parse_complete(input);
            assert!(result.is_err());
        }
    }

    mod error_tests {
        use super::*;

        #[test]
        fn test_error_missing_test_keyword() {
            let input = InputSpan::new_extra(": true\n", Config::default());
            let result = parse(input);

            match result {
                Err(nom::Err::Error(error)) => {
                    assert_eq!(error.error_offset, 0);
                    assert!(matches!(
                        error.reason,
                        ParserErrorReason::Expect(ExpectKind::Test)
                    ));
                }
                _ => panic!("Unexpected result {result:?}"),
            }
        }

        #[test]
        fn test_error_missing_colon_after_test() {
            let input = InputSpan::new_extra("test true\n", Config::default());
            let result = parse(input);
            let expected_test_span = AstSpan::new(0, 4, 1);

            match result {
                Err(nom::Err::Failure(error)) => {
                    assert_eq!(error.error_offset, 5); // After "test"
                    match error.reason {
                        ParserErrorReason::Incomplete {
                            kind: IncompleteKind::Test(TestKind::MissingColon),
                            cause,
                        } => {
                            assert_eq!(cause, expected_test_span);
                        }
                        error => panic!("Unexpected error {error:?}"),
                    }
                }
                _ => panic!("Unexpected result {result:?}"),
            }
        }

        #[test]
        fn test_error_missing_expression() {
            let input = InputSpan::new_extra("test:\n", Config::default());
            let result = parse(input);
            let expected_colon_span = AstSpan::new(4, 1, 0);

            match result {
                Err(nom::Err::Failure(error)) => {
                    assert_eq!(error.error_offset, 5); // After ":"
                    match error.reason {
                        ParserErrorReason::Incomplete {
                            kind: IncompleteKind::Test(TestKind::MissingExpr),
                            cause,
                        } => {
                            assert_eq!(cause, expected_colon_span);
                        }
                        error => panic!("Unexpected error {error:?}"),
                    }
                }
                _ => panic!("Unexpected result {result:?}"),
            }
        }

        #[test]
        fn test_error_missing_colon_with_debug() {
            let input = InputSpan::new_extra("** test true\n", Config::default());
            let result = parse(input);
            let expected_test_span = AstSpan::new(3, 4, 1);

            match result {
                Err(nom::Err::Failure(error)) => {
                    assert_eq!(error.error_offset, 8); // After "test"
                    match error.reason {
                        ParserErrorReason::Incomplete {
                            kind: IncompleteKind::Test(TestKind::MissingColon),
                            cause,
                        } => {
                            assert_eq!(cause, expected_test_span);
                        }
                        error => panic!("Unexpected error {error:?}"),
                    }
                }
                _ => panic!("Unexpected result {result:?}"),
            }
        }

        #[test]
        fn test_error_missing_expression_with_trace() {
            let input = InputSpan::new_extra("* test:\n", Config::default());
            let result = parse(input);
            let expected_colon_span = AstSpan::new(6, 1, 0);

            match result {
                Err(nom::Err::Failure(error)) => {
                    assert_eq!(error.error_offset, 7); // After ":"
                    match error.reason {
                        ParserErrorReason::Incomplete {
                            kind: IncompleteKind::Test(TestKind::MissingExpr),
                            cause,
                        } => {
                            assert_eq!(cause, expected_colon_span);
                        }
                        error => panic!("Unexpected error {error:?}"),
                    }
                }
                _ => panic!("Unexpected result {result:?}"),
            }
        }

        #[test]
        fn test_error_invalid_trace_level() {
            let input = InputSpan::new_extra("*** test: true\n", Config::default());
            let result = parse(input);

            match result {
                Err(nom::Err::Error(error)) => {
                    assert_eq!(error.error_offset, 2);
                    assert!(matches!(
                        error.reason,
                        ParserErrorReason::Expect(ExpectKind::Test)
                    ));
                }
                _ => panic!("Unexpected result {result:?}"),
            }
        }

        #[test]
        fn test_error_empty_input() {
            let input = InputSpan::new_extra("", Config::default());
            let result = parse(input);

            match result {
                Err(nom::Err::Error(error)) => {
                    assert_eq!(error.error_offset, 0);
                    assert!(matches!(
                        error.reason,
                        ParserErrorReason::Expect(ExpectKind::Test)
                    ));
                }
                _ => panic!("Unexpected result {result:?}"),
            }
        }

        #[test]
        fn test_error_whitespace_only() {
            let input = InputSpan::new_extra("   \n", Config::default());
            let result = parse(input);

            match result {
                Err(nom::Err::Error(error)) => {
                    assert_eq!(error.error_offset, 0); // After whitespace
                    assert!(matches!(
                        error.reason,
                        ParserErrorReason::Expect(ExpectKind::Test)
                    ));
                }
                _ => panic!("Unexpected result {result:?}"),
            }
        }
    }
}
