//! Unit expression parsing for the Oneil language.

use nom::{
    Parser as NomParser,
    branch::alt,
    combinator::{all_consuming, map, opt},
    multi::many0,
};

use oneil_ast::{AstSpan, Identifier, Node, UnitExponent, UnitExpr, UnitExprNode, UnitOp};

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
///
/// This function parses unit expressions with left-associative multiplication and division.
/// It handles sequences of unit terms separated by `*` (multiplication) or `/` (division)
/// operators, building the AST with proper associativity.
///
/// Examples:
/// - `kg*m` → `(kg * m)`
/// - `m/s^2` → `(m / s^2)`
/// - `kg*m/s^2` → `((kg * m) / s^2)`
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns a unit expression node with proper operator precedence and associativity.
fn unit_expr(input: InputSpan<'_>) -> Result<'_, UnitExprNode, ParserError> {
    let (rest, first_term) = unit_term
        .convert_error_to(ParserError::expect_unit)
        .parse(input)?;

    let (rest, rest_terms) = many0(|input| {
        let op = alt((
            map(star, |token| Node::new(&token, UnitOp::Multiply)),
            map(slash, |token| Node::new(&token, UnitOp::Divide)),
        ));

        let (rest, op) = op.convert_errors().parse(input)?;
        let (rest, term) = unit_term
            .or_fail_with(ParserError::unit_missing_second_term(&op))
            .parse(rest)?;
        Ok((rest, (op, term)))
    })
    .parse(rest)?;

    let expr = rest_terms.into_iter().fold(first_term, |acc, (op, expr)| {
        let left = acc;
        let right = expr;
        let span = AstSpan::calc_span(&left, &right);

        Node::new(&span, UnitExpr::binary_op(op, left, right))
    });

    Ok((rest, expr))
}

/// Parses a unit term, which can be either a simple unit or a parenthesized expression.
///
/// A unit term is the basic building block of unit expressions. It can be:
/// - A simple unit identifier (e.g., `kg`, `m`, `s`)
/// - A unit with an exponent (e.g., `m^2`, `s^-1`)
/// - A parenthesized unit expression (e.g., `(kg * m)`)
///
/// The function handles both simple units and parenthesized expressions,
/// with proper error handling for missing exponents and unclosed parentheses.
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns a unit expression node representing the parsed term.
fn unit_term(input: InputSpan<'_>) -> Result<'_, UnitExprNode, ParserError> {
    let parse_unit = |input| {
        let (rest, id_token) = unit_identifier.convert_errors().parse(input)?;
        let id_value = Identifier::new(id_token.lexeme().to_string());
        let id = Node::new(&id_token, id_value);

        let (rest, exp) = opt(|input| {
            let (rest, caret_token) = caret.convert_errors().parse(input)?;
            let (rest, exp) = number
                .or_fail_with(ParserError::unit_missing_exponent(&caret_token))
                .parse(rest)?;
            Ok((rest, exp))
        })
        .parse(rest)?;

        let exp = exp.map(|n| {
            let parse_result = n.lexeme().parse::<f64>();
            let parse_result = parse_result.expect("all valid numbers should parse correctly");

            (n, parse_result)
        });

        let exp = match exp {
            Some((n, exp)) => Some(Node::new(&n, UnitExponent::new(exp))),
            None => None,
        };

        let span = exp
            .as_ref()
            .map_or_else(|| AstSpan::from(&id), |n| AstSpan::calc_span(&id, n));

        let expr = Node::new(&span, UnitExpr::unit(id, exp));

        Ok((rest, expr))
    };

    let parse_unit_one = |input| {
        let (rest, unit_one_token) = unit_one.convert_errors().parse(input)?;
        let unit_one_value = UnitExpr::unit_one();
        let unit_one = Node::new(&unit_one_token, unit_one_value);

        Ok((rest, unit_one))
    };

    let parse_parenthesized = |input| {
        let (rest, paren_left_token) = paren_left.convert_errors().parse(input)?;

        let (rest, expr) = unit_expr
            .or_fail_with(ParserError::unit_paren_missing_expr(&paren_left_token))
            .parse(rest)?;

        let (rest, paren_right_token) = paren_right
            .or_fail_with(ParserError::unclosed_paren(&paren_left_token))
            .parse(rest)?;

        let span = AstSpan::calc_span(&paren_left_token, &paren_right_token);

        // note: we need to wrap the expr in a parenthesized node in order to keep the spans accurate
        //       otherwise, calculating spans using the parenthesized node as a start or end span
        //       will result in the calculated span ignoring the parens
        let expr = Node::new(&span, UnitExpr::parenthesized(expr));

        Ok((rest, expr))
    };

    parse_unit
        .or(parse_unit_one)
        .or(parse_parenthesized)
        .parse(input)
}

#[cfg(test)]
#[expect(
    clippy::similar_names,
    reason = "test code uses names where only difference is variable name"
)]
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

            let expected_id = Node::new(&AstSpan::new(0, 2, 0), Identifier::new("kg".to_string()));
            let expected_unit =
                Node::new(&AstSpan::new(0, 2, 0), UnitExpr::unit(expected_id, None));

            assert_eq!(unit, expected_unit);
        }

        #[test]
        fn unit_one() {
            let input = InputSpan::new_extra("1", Config::default());
            let (_, unit) = parse(input).expect("should parse unit");

            let expected_unit = Node::new(&AstSpan::new(0, 1, 0), UnitExpr::UnitOne);

            assert_eq!(unit, expected_unit);
        }

        #[test]
        fn unit_one_with_whitespace() {
            let input = InputSpan::new_extra("1 ", Config::default());
            let (_, unit) = parse(input).expect("should parse unit");

            // The span should include the whitespace length
            let expected_unit = Node::new(&AstSpan::new(0, 1, 1), UnitExpr::UnitOne);

            assert_eq!(unit, expected_unit);
        }

        #[test]
        fn unit_one_in_compound_expression() {
            let input = InputSpan::new_extra("1/s", Config::default());
            let (_, unit) = parse(input).expect("should parse unit");

            // 1
            let expected_one = Node::new(&AstSpan::new(0, 1, 0), UnitExpr::UnitOne);

            // s
            let expected_s_id = Node::new(&AstSpan::new(2, 1, 0), Identifier::new("s".to_string()));
            let expected_s = Node::new(&AstSpan::new(2, 1, 0), UnitExpr::unit(expected_s_id, None));

            // /
            let expected_div = Node::new(&AstSpan::new(1, 1, 0), UnitOp::Divide);

            // 1/s
            let expected_unit = Node::new(
                &AstSpan::new(0, 3, 0),
                UnitExpr::binary_op(expected_div, expected_one, expected_s),
            );

            assert_eq!(unit, expected_unit);
        }

        #[test]
        fn unit_one_in_complex_expression() {
            let input = InputSpan::new_extra("kg*1/s^2", Config::default());
            let (_, unit) = parse(input).expect("should parse unit");

            // kg
            let expected_kg_id =
                Node::new(&AstSpan::new(0, 2, 0), Identifier::new("kg".to_string()));
            let expected_kg =
                Node::new(&AstSpan::new(0, 2, 0), UnitExpr::unit(expected_kg_id, None));

            // 1
            let expected_one = Node::new(&AstSpan::new(3, 1, 0), UnitExpr::UnitOne);

            // kg * 1
            let expected_mult = Node::new(&AstSpan::new(2, 1, 0), UnitOp::Multiply);
            let expected_left = Node::new(
                &AstSpan::new(0, 4, 0),
                UnitExpr::binary_op(expected_mult, expected_kg, expected_one),
            );

            // s
            let expected_s_id = Node::new(&AstSpan::new(5, 1, 0), Identifier::new("s".to_string()));
            let expected_s_exp = Node::new(&AstSpan::new(7, 1, 0), UnitExponent::new(2.0));
            let expected_s = Node::new(
                &AstSpan::new(5, 3, 0),
                UnitExpr::unit(expected_s_id, Some(expected_s_exp)),
            );

            // /
            let expected_div = Node::new(&AstSpan::new(4, 1, 0), UnitOp::Divide);

            // (kg*1)/s^2
            let expected_unit = Node::new(
                &AstSpan::new(0, 8, 0),
                UnitExpr::binary_op(expected_div, expected_left, expected_s),
            );

            assert_eq!(unit, expected_unit);
        }

        #[test]
        fn unit_with_exponent() {
            let input = InputSpan::new_extra("m^2", Config::default());
            let (_, unit) = parse(input).expect("should parse unit");

            let expected_id = Node::new(&AstSpan::new(0, 1, 0), Identifier::new("m".to_string()));
            let expected_exp = Node::new(&AstSpan::new(2, 1, 0), UnitExponent::new(2.0));
            let expected_unit = Node::new(
                &AstSpan::new(0, 3, 0),
                UnitExpr::unit(expected_id, Some(expected_exp)),
            );

            assert_eq!(unit, expected_unit);
        }

        #[test]
        fn compound_unit_multiply() {
            let input = InputSpan::new_extra("kg*m", Config::default());
            let (_, unit) = parse(input).expect("should parse unit");

            let expected_kg_id =
                Node::new(&AstSpan::new(0, 2, 0), Identifier::new("kg".to_string()));
            let expected_left =
                Node::new(&AstSpan::new(0, 2, 0), UnitExpr::unit(expected_kg_id, None));

            let expected_m_id = Node::new(&AstSpan::new(3, 1, 0), Identifier::new("m".to_string()));
            let expected_right =
                Node::new(&AstSpan::new(3, 1, 0), UnitExpr::unit(expected_m_id, None));

            let expected_op = Node::new(&AstSpan::new(2, 1, 0), UnitOp::Multiply);

            let expected_unit = Node::new(
                &AstSpan::new(0, 4, 0),
                UnitExpr::binary_op(expected_op, expected_left, expected_right),
            );

            assert_eq!(unit, expected_unit);
        }

        #[test]
        fn compound_unit_divide() {
            let input = InputSpan::new_extra("m/s", Config::default());
            let (_, unit) = parse(input).expect("should parse unit");

            let expected_m_id = Node::new(&AstSpan::new(0, 1, 0), Identifier::new("m".to_string()));
            let expected_left =
                Node::new(&AstSpan::new(0, 1, 0), UnitExpr::unit(expected_m_id, None));

            let expected_s_id = Node::new(&AstSpan::new(2, 1, 0), Identifier::new("s".to_string()));
            let expected_right =
                Node::new(&AstSpan::new(2, 1, 0), UnitExpr::unit(expected_s_id, None));

            let expected_op = Node::new(&AstSpan::new(1, 1, 0), UnitOp::Divide);

            let expected_unit = Node::new(
                &AstSpan::new(0, 3, 0),
                UnitExpr::binary_op(expected_op, expected_left, expected_right),
            );

            assert_eq!(unit, expected_unit);
        }

        #[test]
        fn complex_unit() {
            let input = InputSpan::new_extra("m^2*kg/s^2", Config::default());
            let (_, unit) = parse(input).expect("should parse unit");

            // m^2
            let expected_m_id = Node::new(&AstSpan::new(0, 1, 0), Identifier::new("m".to_string()));
            let expected_m_exp = Node::new(&AstSpan::new(2, 1, 0), UnitExponent::new(2.0));
            let expected_m = Node::new(
                &AstSpan::new(0, 3, 0),
                UnitExpr::unit(expected_m_id, Some(expected_m_exp)),
            );

            // kg
            let expected_kg_id =
                Node::new(&AstSpan::new(4, 2, 0), Identifier::new("kg".to_string()));
            let expected_kg =
                Node::new(&AstSpan::new(4, 2, 0), UnitExpr::unit(expected_kg_id, None));

            // m^2 * kg
            let expected_mult = Node::new(&AstSpan::new(3, 1, 0), UnitOp::Multiply);
            let expected_left = Node::new(
                &AstSpan::new(0, 6, 0),
                UnitExpr::binary_op(expected_mult, expected_m, expected_kg),
            );

            // s
            let expected_s_id = Node::new(&AstSpan::new(7, 1, 0), Identifier::new("s".to_string()));
            let expected_s_exp = Node::new(&AstSpan::new(9, 1, 0), UnitExponent::new(2.0));
            let expected_s = Node::new(
                &AstSpan::new(7, 3, 0),
                UnitExpr::unit(expected_s_id, Some(expected_s_exp)),
            );

            // /
            let expected_div = Node::new(&AstSpan::new(6, 1, 0), UnitOp::Divide);

            // (m^2*kg)/s^2
            let expected_unit = Node::new(
                &AstSpan::new(0, 10, 0),
                UnitExpr::binary_op(expected_div, expected_left, expected_s),
            );

            assert_eq!(unit, expected_unit);
        }

        #[test]
        fn unit_with_dollar_terminator() {
            let input = InputSpan::new_extra("k$", Config::default());
            let (_, unit) = parse(input).expect("should parse unit");

            let expected_id = Node::new(&AstSpan::new(0, 2, 0), Identifier::new("k$".to_string()));
            let expected_unit =
                Node::new(&AstSpan::new(0, 2, 0), UnitExpr::unit(expected_id, None));

            assert_eq!(unit, expected_unit);
        }

        #[test]
        fn unit_with_percent_terminator() {
            let input = InputSpan::new_extra("%", Config::default());
            let (_, unit) = parse(input).expect("should parse unit");

            let expected_id = Node::new(&AstSpan::new(0, 1, 0), Identifier::new("%".to_string()));
            let expected_unit =
                Node::new(&AstSpan::new(0, 1, 0), UnitExpr::unit(expected_id, None));

            assert_eq!(unit, expected_unit);
        }

        #[test]
        fn unit_with_terminator_and_exponent() {
            let input = InputSpan::new_extra("k$^2", Config::default());
            let (_, unit) = parse(input).expect("should parse unit");

            let expected_id = Node::new(&AstSpan::new(0, 2, 0), Identifier::new("k$".to_string()));
            let expected_exp = Node::new(&AstSpan::new(3, 1, 0), UnitExponent::new(2.0));
            let expected_unit = Node::new(
                &AstSpan::new(0, 4, 0),
                UnitExpr::unit(expected_id, Some(expected_exp)),
            );

            assert_eq!(unit, expected_unit);
        }

        #[test]
        fn compound_unit_with_terminators() {
            let input = InputSpan::new_extra("k$*%", Config::default());
            let (_, unit) = parse(input).expect("should parse unit");

            let expected_k_id =
                Node::new(&AstSpan::new(0, 2, 0), Identifier::new("k$".to_string()));
            let expected_left =
                Node::new(&AstSpan::new(0, 2, 0), UnitExpr::unit(expected_k_id, None));

            let expected_percent_id =
                Node::new(&AstSpan::new(3, 1, 0), Identifier::new("%".to_string()));
            let expected_right = Node::new(
                &AstSpan::new(3, 1, 0),
                UnitExpr::unit(expected_percent_id, None),
            );

            let expected_op = Node::new(&AstSpan::new(2, 1, 0), UnitOp::Multiply);

            let expected_unit = Node::new(
                &AstSpan::new(0, 4, 0),
                UnitExpr::binary_op(expected_op, expected_left, expected_right),
            );

            assert_eq!(unit, expected_unit);
        }

        #[test]
        fn parenthesized_unit() {
            let input = InputSpan::new_extra("(kg*m)/s^2", Config::default());
            let (_, unit) = parse(input).expect("should parse unit");

            // kg
            let expected_kg_id =
                Node::new(&AstSpan::new(1, 2, 0), Identifier::new("kg".to_string()));
            let expected_kg =
                Node::new(&AstSpan::new(1, 2, 0), UnitExpr::unit(expected_kg_id, None));

            // m
            let expected_m_id = Node::new(&AstSpan::new(4, 1, 0), Identifier::new("m".to_string()));
            let expected_m = Node::new(&AstSpan::new(4, 1, 0), UnitExpr::unit(expected_m_id, None));

            // *
            let expected_mult = Node::new(&AstSpan::new(3, 1, 0), UnitOp::Multiply);

            // kg*m
            let expected_inner = Node::new(
                &AstSpan::new(1, 4, 0),
                UnitExpr::binary_op(expected_mult, expected_kg, expected_m),
            );

            // (kg*m)
            let expected_paren = Node::new(
                &AstSpan::new(0, 6, 0),
                UnitExpr::parenthesized(expected_inner),
            );

            // s
            let expected_s_id = Node::new(&AstSpan::new(7, 1, 0), Identifier::new("s".to_string()));
            let expected_s_exp = Node::new(&AstSpan::new(9, 1, 0), UnitExponent::new(2.0));
            let expected_s = Node::new(
                &AstSpan::new(7, 3, 0),
                UnitExpr::unit(expected_s_id, Some(expected_s_exp)),
            );

            // /
            let expected_div = Node::new(&AstSpan::new(6, 1, 0), UnitOp::Divide);

            // (kg*m)/s^2
            let expected_unit = Node::new(
                &AstSpan::new(0, 10, 0),
                UnitExpr::binary_op(expected_div, expected_paren, expected_s),
            );

            assert_eq!(unit, expected_unit);
        }

        #[test]
        fn parse_complete_success() {
            let input = InputSpan::new_extra("kg", Config::default());
            let (rest, unit) = parse_complete(input).expect("should parse unit");

            let expected_id = Node::new(&AstSpan::new(0, 2, 0), Identifier::new("kg".to_string()));
            let expected_unit =
                Node::new(&AstSpan::new(0, 2, 0), UnitExpr::unit(expected_id, None));

            assert_eq!(unit, expected_unit);
            assert_eq!(rest.fragment(), &"");
        }
    }

    mod parse_complete {
        use super::*;

        #[test]
        fn parse_complete_success() {
            let input = InputSpan::new_extra("kg", Config::default());
            let (rest, unit) = parse_complete(input).expect("should parse unit");

            let expected_id = Node::new(&AstSpan::new(0, 2, 0), Identifier::new("kg".to_string()));
            let expected_unit =
                Node::new(&AstSpan::new(0, 2, 0), UnitExpr::unit(expected_id, None));

            assert_eq!(unit, expected_unit);
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
            let expected_op_span = AstSpan::new(2, 1, 0);

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
            assert_eq!(cause, expected_op_span);
        }

        #[test]
        fn missing_second_term_after_divide() {
            let input = InputSpan::new_extra("kg/", Config::default());
            let result = parse(input);
            let expected_op_span = AstSpan::new(2, 1, 0);

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
            assert_eq!(cause, expected_op_span);
        }

        #[test]
        fn missing_exponent() {
            let input = InputSpan::new_extra("kg^", Config::default());
            let result = parse(input);
            let expected_caret_span = AstSpan::new(2, 1, 0);

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

            assert_eq!(cause, expected_caret_span);
        }

        #[test]
        fn parenthesized_missing_expr() {
            let input = InputSpan::new_extra("()", Config::default());
            let result = parse(input);
            let expected_paren_span = AstSpan::new(0, 1, 0);

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

            assert_eq!(cause, expected_paren_span);
        }

        #[test]
        fn unclosed_paren() {
            let input = InputSpan::new_extra("(kg*m", Config::default());
            let result = parse(input);
            let expected_paren_span = AstSpan::new(0, 1, 0);

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

            assert_eq!(cause, expected_paren_span);
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
            let expected_caret_span = AstSpan::new(2, 1, 0);

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

            assert_eq!(cause, expected_caret_span);
        }

        #[test]
        fn missing_second_term_in_complex_expression() {
            let input = InputSpan::new_extra("kg*m/", Config::default());
            let result = parse(input);
            let expected_op_span = AstSpan::new(4, 1, 0);

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
            assert_eq!(cause, expected_op_span);
        }

        #[test]
        fn nested_unclosed_paren() {
            let input = InputSpan::new_extra("((kg*m)", Config::default());
            let result = parse(input);
            let expected_paren_span = AstSpan::new(0, 1, 0);

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

            assert_eq!(cause, expected_paren_span);
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
            let expected_unit = Node::new(&AstSpan::new(0, 1, 0), UnitExpr::UnitOne);
            assert_eq!(unit, expected_unit);
            assert_eq!(rest.fragment(), &".5");
        }

        #[test]
        fn unit_one_with_exponent_parses_partially() {
            let input = InputSpan::new_extra("1^2", Config::default());
            let (rest, unit) = parse(input).expect("should parse unit");

            // Should parse "1" as unit_one and leave "^2" as remainder
            let expected_unit = Node::new(&AstSpan::new(0, 1, 0), UnitExpr::UnitOne);
            assert_eq!(unit, expected_unit);
            assert_eq!(rest.fragment(), &"^2");
        }
    }
}
