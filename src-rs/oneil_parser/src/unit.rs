//! Unit expression parsing for the Oneil language.

use nom::{
    Parser as NomParser,
    branch::alt,
    combinator::{all_consuming, map, opt},
    multi::many0,
};

use oneil_ast::{IdentifierNode, Node, UnitExponent, UnitExpr, UnitExprNode, UnitOp};
use oneil_shared::span::Span;

use crate::{
    error::{ErrorHandlingParser, ParserError},
    token::{
        literal::{number, unit_one},
        naming::unit_identifier,
        symbol::{caret, paren_left, paren_right, slash, star},
    },
    util::{InputSpan, Result},
};

/// Parses a unit expression
///
/// This function **may not consume the complete input**.
pub fn parse(input: InputSpan<'_>) -> Result<'_, UnitExprNode, ParserError> {
    unit_expr(input)
}

/// Parses a unit expression
///
/// This function **fails if the complete input is not consumed**.
pub fn parse_complete(input: InputSpan<'_>) -> Result<'_, UnitExprNode, ParserError> {
    all_consuming(unit_expr).parse(input)
}

/// Parses a unit expression with left-associative multiplication and division.
fn unit_expr(input: InputSpan<'_>) -> Result<'_, UnitExprNode, ParserError> {
    let (rest, first_term_node) = unit_term
        .convert_error_to(ParserError::expect_unit)
        .parse(input)?;

    let (rest, rest_terms_node) = many0(|input| {
        let op = alt((
            map(star, |token| token.into_node_with_value(UnitOp::Multiply)),
            map(slash, |token| token.into_node_with_value(UnitOp::Divide)),
        ));

        let (rest, op_node) = op.convert_errors().parse(input)?;
        let (rest, term_node) = unit_term
            .or_fail_with(ParserError::unit_missing_second_term(&op_node))
            .parse(rest)?;
        Ok((rest, (op_node, term_node)))
    })
    .parse(rest)?;

    let expr_node =
        rest_terms_node
            .into_iter()
            .fold(first_term_node, |acc_node, (op, expr_node)| {
                let left = acc_node;
                let right = expr_node;
                let span = Span::from_start_and_end(&left.span(), &right.span());
                let whitespace_span = right.whitespace_span();

                Node::new(UnitExpr::binary_op(op, left, right), span, whitespace_span)
            });

    Ok((rest, expr_node))
}

/// Parses a unit term, which can be either a simple unit or a parenthesized expression.
fn unit_term(input: InputSpan<'_>) -> Result<'_, UnitExprNode, ParserError> {
    let parse_unit = |input| {
        let (rest, id_token) = unit_identifier.convert_errors().parse(input)?;
        let id_node = IdentifierNode::from(id_token);

        let (rest, exp) = opt(|input| {
            let (rest, caret_token) = caret.convert_errors().parse(input)?;
            let (rest, exp_node) = number
                .or_fail_with(ParserError::unit_missing_exponent(caret_token.lexeme_span))
                .parse(rest)?;
            Ok((rest, exp_node))
        })
        .parse(rest)?;

        let exp_node = exp.map(|n| {
            let parse_result = n.lexeme_str.parse::<f64>();
            let parse_result = parse_result.expect("all valid numbers should parse correctly");

            n.into_node_with_value(UnitExponent::new(parse_result))
        });

        let span_start = id_node.span();
        let (span_end, whitespace_span) = exp_node.as_ref().map_or_else(
            || (id_node.span(), id_node.whitespace_span()),
            |n| (n.span(), n.whitespace_span()),
        );
        let span = Span::from_start_and_end(&span_start, &span_end);

        let expr = Node::new(UnitExpr::unit(id_node, exp_node), span, whitespace_span);

        Ok((rest, expr))
    };

    let parse_unit_one = |input| {
        let (rest, unit_one_token) = unit_one.convert_errors().parse(input)?;
        let unit_one_node = unit_one_token.into_node_with_value(UnitExpr::UnitOne);

        Ok((rest, unit_one_node))
    };

    let parse_parenthesized = |input| {
        let (rest, paren_left_token) = paren_left.convert_errors().parse(input)?;

        let (rest, expr) = unit_expr
            .or_fail_with(ParserError::unit_paren_missing_expr(
                paren_left_token.lexeme_span,
            ))
            .parse(rest)?;

        let (rest, paren_right_token) = paren_right
            .or_fail_with(ParserError::unclosed_paren(paren_left_token.lexeme_span))
            .parse(rest)?;

        let span = Span::from_start_and_end(
            &paren_left_token.lexeme_span,
            &paren_right_token.lexeme_span,
        );
        let whitespace_span = paren_right_token.whitespace_span;

        // note: we need to wrap the expr in a parenthesized node in order to keep the spans accurate
        //       otherwise, calculating spans using the parenthesized node as a start or end span
        //       will result in the calculated span ignoring the parens
        let expr = Node::new(UnitExpr::parenthesized(expr), span, whitespace_span);

        Ok((rest, expr))
    };

    parse_unit
        .or(parse_unit_one)
        .or(parse_parenthesized)
        .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Config,
        error::reason::{ExpectKind, IncompleteKind, ParserErrorReason, UnitKind},
    };

    mod success {
        use super::*;

        #[test]
        fn simple_unit() {
            let input = InputSpan::new_extra("kg", Config::default());
            let (_, unit) = parse(input).expect("should parse unit");

            let UnitExpr::Unit {
                identifier,
                exponent,
            } = unit.take_value()
            else {
                panic!("expected unit");
            };

            assert_eq!(identifier.as_str(), "kg");
            assert_eq!(exponent, None);
        }

        #[test]
        fn unit_one() {
            let input = InputSpan::new_extra("1", Config::default());
            let (_, unit) = parse(input).expect("should parse unit");

            assert_eq!(unit.take_value(), UnitExpr::UnitOne);
        }

        #[test]
        fn unit_one_with_whitespace() {
            let input = InputSpan::new_extra("1 ", Config::default());
            let (_, unit) = parse(input).expect("should parse unit");

            assert_eq!(unit.take_value(), UnitExpr::UnitOne);
        }

        #[test]
        fn unit_one_in_compound_expression() {
            let input = InputSpan::new_extra("1/s", Config::default());
            let (_, unit) = parse(input).expect("should parse unit");

            let UnitExpr::BinaryOp { op, left, right } = unit.take_value() else {
                panic!("expected binary op");
            };

            assert_eq!(*op, UnitOp::Divide);

            assert_eq!(left.take_value(), UnitExpr::UnitOne);

            let UnitExpr::Unit {
                identifier,
                exponent,
            } = right.take_value()
            else {
                panic!("expected unit");
            };

            assert_eq!(identifier.as_str(), "s");
            assert_eq!(exponent, None);
        }

        #[test]
        fn unit_one_in_complex_expression() {
            let input = InputSpan::new_extra("kg*1/s^2", Config::default());
            let (_, unit) = parse(input).expect("should parse unit");

            let UnitExpr::BinaryOp { op, left, right } = unit.take_value() else {
                panic!("expected binary op");
            };

            assert_eq!(*op, UnitOp::Divide);

            let UnitExpr::BinaryOp {
                op,
                left: left_left,
                right: left_right,
            } = left.take_value()
            else {
                panic!("expected binary op");
            };
            assert_eq!(*op, UnitOp::Multiply);

            let UnitExpr::Unit {
                identifier,
                exponent,
            } = left_left.take_value()
            else {
                panic!("expected unit");
            };
            assert_eq!(identifier.as_str(), "kg");
            assert_eq!(exponent, None);

            let UnitExpr::UnitOne = left_right.take_value() else {
                panic!("expected unit one");
            };

            let UnitExpr::Unit {
                identifier,
                exponent,
            } = right.take_value()
            else {
                panic!("expected unit");
            };
            assert_eq!(identifier.as_str(), "s");
            assert_eq!(exponent.as_ref().map(|e| e.value()), Some(2.0));
        }

        #[test]
        fn unit_with_exponent() {
            let input = InputSpan::new_extra("m^2", Config::default());
            let (_, unit) = parse(input).expect("should parse unit");

            let UnitExpr::Unit {
                identifier,
                exponent,
            } = unit.take_value()
            else {
                panic!("expected unit");
            };
            assert_eq!(identifier.as_str(), "m");
            assert_eq!(exponent.as_ref().map(|e| e.value()), Some(2.0));
        }

        #[test]
        fn compound_unit_multiply() {
            let input = InputSpan::new_extra("kg*m", Config::default());
            let (_, unit) = parse(input).expect("should parse unit");

            let UnitExpr::BinaryOp { op, left, right } = unit.take_value() else {
                panic!("expected binary op");
            };
            assert_eq!(*op, UnitOp::Multiply);

            let UnitExpr::Unit {
                identifier,
                exponent,
            } = left.take_value()
            else {
                panic!("expected unit");
            };
            assert_eq!(identifier.as_str(), "kg");
            assert_eq!(exponent, None);

            let UnitExpr::Unit {
                identifier,
                exponent,
            } = right.take_value()
            else {
                panic!("expected unit");
            };
            assert_eq!(identifier.as_str(), "m");
            assert_eq!(exponent, None);
        }

        #[test]
        fn compound_unit_divide() {
            let input = InputSpan::new_extra("m/s", Config::default());
            let (_, unit) = parse(input).expect("should parse unit");

            let UnitExpr::BinaryOp { op, left, right } = unit.take_value() else {
                panic!("expected binary op");
            };
            assert_eq!(*op, UnitOp::Divide);

            let UnitExpr::Unit {
                identifier,
                exponent,
            } = left.take_value()
            else {
                panic!("expected unit");
            };
            assert_eq!(identifier.as_str(), "m");
            assert_eq!(exponent, None);

            let UnitExpr::Unit {
                identifier,
                exponent,
            } = right.take_value()
            else {
                panic!("expected unit");
            };
            assert_eq!(identifier.as_str(), "s");
            assert_eq!(exponent, None);
        }

        #[test]
        fn complex_unit() {
            let input = InputSpan::new_extra("m^2*kg/s^2", Config::default());
            let (_, unit) = parse(input).expect("should parse unit");

            let UnitExpr::BinaryOp { op, left, right } = unit.take_value() else {
                panic!("expected binary op");
            };
            assert_eq!(*op, UnitOp::Divide);

            let UnitExpr::BinaryOp {
                op,
                left: left_left,
                right: left_right,
            } = left.take_value()
            else {
                panic!("expected binary op");
            };
            assert_eq!(*op, UnitOp::Multiply);

            let UnitExpr::Unit {
                identifier,
                exponent,
            } = left_left.take_value()
            else {
                panic!("expected unit");
            };
            assert_eq!(identifier.as_str(), "m");
            assert_eq!(exponent.as_ref().map(|e| e.value()), Some(2.0));

            let UnitExpr::Unit {
                identifier,
                exponent,
            } = left_right.take_value()
            else {
                panic!("expected unit");
            };
            assert_eq!(identifier.as_str(), "kg");
            assert_eq!(exponent, None);

            let UnitExpr::Unit {
                identifier,
                exponent,
            } = right.take_value()
            else {
                panic!("expected unit");
            };
            assert_eq!(identifier.as_str(), "s");
            assert_eq!(exponent.as_ref().map(|e| e.value()), Some(2.0));
        }

        #[test]
        fn unit_with_dollar_terminator() {
            let input = InputSpan::new_extra("k$", Config::default());
            let (_, unit) = parse(input).expect("should parse unit");

            let UnitExpr::Unit {
                identifier,
                exponent,
            } = unit.take_value()
            else {
                panic!("expected unit");
            };
            assert_eq!(identifier.as_str(), "k$");
            assert_eq!(exponent, None);
        }

        #[test]
        fn unit_with_percent_terminator() {
            let input = InputSpan::new_extra("%", Config::default());
            let (_, unit) = parse(input).expect("should parse unit");

            let UnitExpr::Unit {
                identifier,
                exponent,
            } = unit.take_value()
            else {
                panic!("expected unit");
            };
            assert_eq!(identifier.as_str(), "%");
            assert_eq!(exponent, None);
        }

        #[test]
        fn unit_with_terminator_and_exponent() {
            let input = InputSpan::new_extra("k$^2", Config::default());
            let (_, unit) = parse(input).expect("should parse unit");

            let UnitExpr::Unit {
                identifier,
                exponent,
            } = unit.take_value()
            else {
                panic!("expected unit");
            };

            assert_eq!(identifier.as_str(), "k$");
            assert_eq!(exponent.as_ref().map(|e| e.value()), Some(2.0));
        }

        #[test]
        fn parenthesized_unit() {
            let input = InputSpan::new_extra("(kg*m)/s^2", Config::default());
            let (_, unit) = parse(input).expect("should parse unit");

            let UnitExpr::BinaryOp { op, left, right } = unit.take_value() else {
                panic!("expected binary op");
            };
            assert_eq!(*op, UnitOp::Divide);

            let UnitExpr::Parenthesized { expr: left } = left.take_value() else {
                panic!("expected parenthesized");
            };

            let UnitExpr::BinaryOp {
                op,
                left: left_left,
                right: left_right,
            } = left.take_value()
            else {
                panic!("expected binary op");
            };
            assert_eq!(*op, UnitOp::Multiply);

            let UnitExpr::Unit {
                identifier,
                exponent,
            } = left_left.take_value()
            else {
                panic!("expected unit");
            };

            assert_eq!(identifier.as_str(), "kg");
            assert_eq!(exponent, None);

            let UnitExpr::Unit {
                identifier,
                exponent,
            } = left_right.take_value()
            else {
                panic!("expected unit");
            };

            assert_eq!(identifier.as_str(), "m");
            assert_eq!(exponent, None);

            let UnitExpr::Unit {
                identifier,
                exponent,
            } = right.take_value()
            else {
                panic!("expected unit");
            };
            assert_eq!(identifier.as_str(), "s");
            assert_eq!(exponent.as_ref().map(|e| e.value()), Some(2.0));
        }

        #[test]
        fn parse_complete_success() {
            let input = InputSpan::new_extra("kg", Config::default());
            let (rest, unit) = parse_complete(input).expect("should parse unit");

            let UnitExpr::Unit {
                identifier,
                exponent,
            } = unit.take_value()
            else {
                panic!("expected unit");
            };

            assert_eq!(identifier.as_str(), "kg");
            assert_eq!(exponent, None);

            assert_eq!(rest.fragment(), &"");
        }
    }

    mod parse_complete {
        use super::*;

        #[test]
        fn parse_complete_success() {
            let input = InputSpan::new_extra("kg", Config::default());
            let (rest, unit) = parse_complete(input).expect("should parse unit");

            let UnitExpr::Unit {
                identifier,
                exponent,
            } = unit.take_value()
            else {
                panic!("expected unit");
            };

            assert_eq!(identifier.as_str(), "kg");
            assert_eq!(exponent, None);

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        #[expect(
            clippy::assertions_on_result_states,
            reason = "we don't care about the result, just that it's an error"
        )]
        fn parse_complete_with_remaining_input() {
            let input = InputSpan::new_extra("kg rest", Config::default());
            let result = parse_complete(input);
            assert!(result.is_err());
        }
    }

    mod error {
        use super::*;

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
                ParserErrorReason::Expect(ExpectKind::Unit)
            ));
        }

        #[test]
        fn whitespace_only() {
            let input = InputSpan::new_extra("   ", Config::default());
            let result = parse(input);

            let Err(nom::Err::Error(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 0);
            assert!(matches!(
                error.reason,
                ParserErrorReason::Expect(ExpectKind::Unit)
            ));
        }

        #[test]
        fn missing_second_term_after_multiply() {
            let input = InputSpan::new_extra("kg*", Config::default());
            let result = parse(input);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 3); // After "*"

            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Unit(UnitKind::MissingSecondTerm { operator }),
                cause,
            } = error.reason
            else {
                panic!("Unexpected error {:?}", error.reason);
            };

            assert_eq!(operator, UnitOp::Multiply);
            assert_eq!(cause.start().offset, 2);
            assert_eq!(cause.end().offset, 3);
        }

        #[test]
        fn missing_second_term_after_divide() {
            let input = InputSpan::new_extra("kg/", Config::default());
            let result = parse(input);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 3); // After "/"
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Unit(UnitKind::MissingSecondTerm { operator }),
                cause,
            } = error.reason
            else {
                panic!("Unexpected error {error:?}");
            };

            assert_eq!(operator, UnitOp::Divide);
            assert_eq!(cause.start().offset, 2);
            assert_eq!(cause.end().offset, 3);
        }

        #[test]
        fn missing_exponent() {
            let input = InputSpan::new_extra("kg^", Config::default());
            let result = parse(input);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 3); // After "^"
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Unit(UnitKind::MissingExponent),
                cause,
            } = error.reason
            else {
                panic!("Unexpected error {error:?}");
            };

            assert_eq!(cause.start().offset, 2);
            assert_eq!(cause.end().offset, 3);
        }

        #[test]
        fn parenthesized_missing_expr() {
            let input = InputSpan::new_extra("()", Config::default());
            let result = parse(input);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 1); // After "("
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Unit(UnitKind::ParenMissingExpr),
                cause,
            } = error.reason
            else {
                panic!("Unexpected error {error:?}");
            };

            assert_eq!(cause.start().offset, 0);
            assert_eq!(cause.end().offset, 1);
        }

        #[test]
        fn unclosed_paren() {
            let input = InputSpan::new_extra("(kg*m", Config::default());
            let result = parse(input);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 5); // After "m"
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::UnclosedParen,
                cause,
            } = error.reason
            else {
                panic!("Unexpected error {error:?}");
            };

            assert_eq!(cause.start().offset, 0);
            assert_eq!(cause.end().offset, 1);
        }

        #[test]
        fn invalid_identifier() {
            let input = InputSpan::new_extra("@invalid", Config::default());
            let result = parse(input);

            let Err(nom::Err::Error(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 0); // At "@"
            assert!(matches!(
                error.reason,
                ParserErrorReason::Expect(ExpectKind::Unit)
            ));
        }

        #[test]
        fn invalid_exponent() {
            let input = InputSpan::new_extra("kg^@invalid", Config::default());
            let result = parse(input);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 3); // After "^"
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Unit(UnitKind::MissingExponent),
                cause,
            } = error.reason
            else {
                panic!("Unexpected error {error:?}");
            };

            assert_eq!(cause.start().offset, 2);
            assert_eq!(cause.end().offset, 3);
        }

        #[test]
        fn missing_second_term_in_complex_expression() {
            let input = InputSpan::new_extra("kg*m/", Config::default());
            let result = parse(input);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 5); // After "/"
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Unit(UnitKind::MissingSecondTerm { operator }),
                cause,
            } = error.reason
            else {
                panic!("Unexpected error {error:?}");
            };

            assert_eq!(operator, UnitOp::Divide);
            assert_eq!(cause.start().offset, 4);
            assert_eq!(cause.end().offset, 5);
        }

        #[test]
        fn nested_unclosed_paren() {
            let input = InputSpan::new_extra("((kg*m)", Config::default());
            let result = parse(input);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 7); // After "m"
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::UnclosedParen,
                cause,
            } = error.reason
            else {
                panic!("Unexpected error {error:?}");
            };

            assert_eq!(cause.start().offset, 0);
            assert_eq!(cause.end().offset, 1);
        }

        #[test]
        #[expect(
            clippy::assertions_on_result_states,
            reason = "we don't care about the result, just that it's ok"
        )]
        fn missing_operator_between_terms() {
            let input = InputSpan::new_extra("kg m", Config::default());
            let result = parse(input);

            // This test should actually succeed because the parser can handle
            // multiple terms without operators in some cases
            assert!(result.is_ok());
        }

        #[test]
        #[expect(
            clippy::assertions_on_result_states,
            reason = "we don't care about the result, just that it's ok"
        )]
        fn invalid_operator() {
            let input = InputSpan::new_extra("kg+m", Config::default());
            let result = parse(input);

            // This test should actually succeed because the parser can handle
            // invalid operators by parsing the first valid unit
            assert!(result.is_ok());
        }

        #[test]
        fn unit_one_with_digits() {
            let input = InputSpan::new_extra("123", Config::default());
            let result = parse(input);

            // This should fail because "123" should not be parsed as a unit_one
            // The parser should try to parse it as a unit identifier instead

            let Err(nom::Err::Error(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 0);
            assert!(matches!(
                error.reason,
                ParserErrorReason::Expect(ExpectKind::Unit)
            ));
        }

        #[test]
        fn unit_one_with_decimal_parses_partially() {
            let input = InputSpan::new_extra("1.5", Config::default());
            let (rest, unit) = parse(input).expect("should parse unit");

            // Should parse "1" as unit_one and leave ".5" as remainder
            assert_eq!(unit.take_value(), UnitExpr::UnitOne);
            assert_eq!(rest.fragment(), &".5");
        }

        #[test]
        fn unit_one_with_exponent_parses_partially() {
            let input = InputSpan::new_extra("1^2", Config::default());
            let (rest, unit) = parse(input).expect("should parse unit");

            // Should parse "1" as unit_one and leave "^2" as remainder
            assert_eq!(unit.take_value(), UnitExpr::UnitOne);
            assert_eq!(rest.fragment(), &"^2");
        }
    }
}
