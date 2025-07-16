//! Unit expression parsing for the Oneil language.

use nom::{
    Parser as NomParser,
    branch::alt,
    combinator::{all_consuming, map, opt},
    multi::many0,
};

use oneil_ast::{
    Span as AstSpan,
    naming::Identifier,
    node::Node,
    unit::{UnitExponent, UnitExpr, UnitExprNode, UnitOp},
};

use crate::{
    error::{ErrorHandlingParser, ParserError},
    token::{
        literal::number,
        naming::identifier,
        symbol::{caret, paren_left, paren_right, slash, star},
    },
    util::{Result, Span},
};

/// Parses a unit expression
///
/// This function **may not consume the complete input**.
pub fn parse(input: Span) -> Result<UnitExprNode, ParserError> {
    unit_expr(input)
}

/// Parses a unit expression
///
/// This function **fails if the complete input is not consumed**.
pub fn parse_complete(input: Span) -> Result<UnitExprNode, ParserError> {
    all_consuming(unit_expr).parse(input)
}

/// Parses a unit expression
fn unit_expr(input: Span) -> Result<UnitExprNode, ParserError> {
    let (rest, first_term) = unit_term
        .convert_error_to(ParserError::expect_unit)
        .parse(input)?;

    let (rest, rest_terms) = many0(|input| {
        let op = alt((
            map(star, |token| Node::new(token, UnitOp::Multiply)),
            map(slash, |token| Node::new(token, UnitOp::Divide)),
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

        Node::new(span, UnitExpr::binary_op(op, left, right))
    });

    Ok((rest, expr))
}

/// Parses a unit term
fn unit_term(input: Span) -> Result<UnitExprNode, ParserError> {
    let parse_unit = |input| {
        let (rest, id_token) = identifier.convert_errors().parse(input)?;
        let id_value = Identifier::new(id_token.lexeme().to_string());
        let id = Node::new(id_token, id_value);

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
            (n, parse_result.map_err(|_| ()))
        });

        let exp = match exp {
            Some((n, Ok(exp))) => Some(Node::new(n, UnitExponent::new(exp))),
            Some((n, Err(()))) => {
                return Err(nom::Err::Failure(ParserError::invalid_number(&n)));
            }
            None => None,
        };

        let span = match &exp {
            Some(n) => AstSpan::calc_span(&id, n),
            None => AstSpan::from(&id),
        };

        let expr = Node::new(span, UnitExpr::unit(id, exp));

        Ok((rest, expr))
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
        let expr = Node::new(span, UnitExpr::parenthesized(expr));

        Ok((rest, expr))
    };

    parse_unit.or(parse_parenthesized).parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;

    #[test]
    fn test_simple_unit() {
        let input = Span::new_extra("kg", Config::default());
        let (_, unit) = parse(input).unwrap();

        let expected_id = Node::new(AstSpan::new(0, 2, 2), Identifier::new("kg".to_string()));
        let expected_unit = Node::new(AstSpan::new(0, 2, 2), UnitExpr::unit(expected_id, None));

        assert_eq!(unit, expected_unit);
    }

    #[test]
    fn test_unit_with_exponent() {
        let input = Span::new_extra("m^2", Config::default());
        let (_, unit) = parse(input).unwrap();

        let expected_id = Node::new(AstSpan::new(0, 1, 1), Identifier::new("m".to_string()));
        let expected_exp = Node::new(AstSpan::new(2, 3, 3), UnitExponent::new(2.0));
        let expected_unit = Node::new(
            AstSpan::new(0, 3, 3),
            UnitExpr::unit(expected_id, Some(expected_exp)),
        );

        assert_eq!(unit, expected_unit);
    }

    #[test]
    fn test_compound_unit_multiply() {
        let input = Span::new_extra("kg*m", Config::default());
        let (_, unit) = parse(input).unwrap();

        let expected_kg_id = Node::new(AstSpan::new(0, 2, 2), Identifier::new("kg".to_string()));
        let expected_left = Node::new(AstSpan::new(0, 2, 2), UnitExpr::unit(expected_kg_id, None));

        let expected_m_id = Node::new(AstSpan::new(3, 4, 4), Identifier::new("m".to_string()));
        let expected_right = Node::new(AstSpan::new(3, 4, 4), UnitExpr::unit(expected_m_id, None));

        let expected_op = Node::new(AstSpan::new(2, 3, 3), UnitOp::Multiply);

        let expected_unit = Node::new(
            AstSpan::new(0, 4, 4),
            UnitExpr::binary_op(expected_op, expected_left, expected_right),
        );

        assert_eq!(unit, expected_unit);
    }

    #[test]
    fn test_compound_unit_divide() {
        let input = Span::new_extra("m/s", Config::default());
        let (_, unit) = parse(input).unwrap();

        let expected_m_id = Node::new(AstSpan::new(0, 1, 1), Identifier::new("m".to_string()));
        let expected_left = Node::new(AstSpan::new(0, 1, 1), UnitExpr::unit(expected_m_id, None));

        let expected_s_id = Node::new(AstSpan::new(2, 3, 3), Identifier::new("s".to_string()));
        let expected_right = Node::new(AstSpan::new(2, 3, 3), UnitExpr::unit(expected_s_id, None));

        let expected_op = Node::new(AstSpan::new(1, 2, 2), UnitOp::Divide);

        let expected_unit = Node::new(
            AstSpan::new(0, 3, 3),
            UnitExpr::binary_op(expected_op, expected_left, expected_right),
        );

        assert_eq!(unit, expected_unit);
    }

    #[test]
    fn test_complex_unit() {
        let input = Span::new_extra("m^2*kg/s^2", Config::default());
        let (_, unit) = parse(input).unwrap();

        // m^2
        let expected_m_id = Node::new(AstSpan::new(0, 1, 1), Identifier::new("m".to_string()));
        let expected_m_exp = Node::new(AstSpan::new(2, 3, 3), UnitExponent::new(2.0));
        let expected_m = Node::new(
            AstSpan::new(0, 3, 3),
            UnitExpr::unit(expected_m_id, Some(expected_m_exp)),
        );

        // kg
        let expected_kg_id = Node::new(AstSpan::new(4, 6, 6), Identifier::new("kg".to_string()));
        let expected_kg = Node::new(AstSpan::new(4, 6, 6), UnitExpr::unit(expected_kg_id, None));

        // m^2 * kg
        let expected_mult = Node::new(AstSpan::new(3, 4, 4), UnitOp::Multiply);
        let expected_left = Node::new(
            AstSpan::new(0, 6, 6),
            UnitExpr::binary_op(expected_mult, expected_m, expected_kg),
        );

        // s
        let expected_s_id = Node::new(AstSpan::new(7, 8, 8), Identifier::new("s".to_string()));
        let expected_s_exp = Node::new(AstSpan::new(9, 10, 10), UnitExponent::new(2.0));
        let expected_s = Node::new(
            AstSpan::new(7, 10, 10),
            UnitExpr::unit(expected_s_id, Some(expected_s_exp)),
        );

        // /
        let expected_div = Node::new(AstSpan::new(6, 7, 7), UnitOp::Divide);

        // (m^2*kg)/s^2
        let expected_unit = Node::new(
            AstSpan::new(0, 10, 10),
            UnitExpr::binary_op(expected_div, expected_left, expected_s),
        );

        assert_eq!(unit, expected_unit);
    }

    #[test]
    fn test_parenthesized_unit() {
        let input = Span::new_extra("(kg*m)/s^2", Config::default());
        let (_, unit) = parse(input).unwrap();

        // kg
        let expected_kg_id = Node::new(AstSpan::new(1, 3, 3), Identifier::new("kg".to_string()));
        let expected_kg = Node::new(AstSpan::new(1, 3, 3), UnitExpr::unit(expected_kg_id, None));

        // m
        let expected_m_id = Node::new(AstSpan::new(4, 5, 5), Identifier::new("m".to_string()));
        let expected_m = Node::new(AstSpan::new(4, 5, 5), UnitExpr::unit(expected_m_id, None));

        // *
        let expected_mult = Node::new(AstSpan::new(3, 4, 4), UnitOp::Multiply);

        // kg*m
        let expected_inner = Node::new(
            AstSpan::new(1, 5, 5),
            UnitExpr::binary_op(expected_mult, expected_kg, expected_m),
        );

        // (kg*m)
        let expected_paren = Node::new(
            AstSpan::new(0, 6, 6),
            UnitExpr::parenthesized(expected_inner),
        );

        // s
        let expected_s_id = Node::new(AstSpan::new(7, 8, 8), Identifier::new("s".to_string()));
        let expected_s_exp = Node::new(AstSpan::new(9, 10, 10), UnitExponent::new(2.0));
        let expected_s = Node::new(
            AstSpan::new(7, 10, 10),
            UnitExpr::unit(expected_s_id, Some(expected_s_exp)),
        );

        // /
        let expected_div = Node::new(AstSpan::new(6, 7, 7), UnitOp::Divide);

        // (kg*m)/s^2
        let expected_unit = Node::new(
            AstSpan::new(0, 10, 10),
            UnitExpr::binary_op(expected_div, expected_paren, expected_s),
        );

        assert_eq!(unit, expected_unit);
    }

    #[test]
    fn test_parse_complete_success() {
        let input = Span::new_extra("kg", Config::default());
        let (rest, unit) = parse_complete(input).unwrap();

        let expected_id = Node::new(AstSpan::new(0, 2, 2), Identifier::new("kg".to_string()));
        let expected_unit = Node::new(AstSpan::new(0, 2, 2), UnitExpr::unit(expected_id, None));

        assert_eq!(unit, expected_unit);
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_parse_complete_with_remaining_input() {
        let input = Span::new_extra("kg rest", Config::default());
        let result = parse_complete(input);
        assert!(result.is_err());
    }
}
