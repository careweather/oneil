//! Parser for parameter declarations in an Oneil program.

use nom::Parser;
use nom::branch::alt;
use nom::combinator::{all_consuming, opt};
use nom::multi::{many0, separated_list1};
use oneil_ast::debug_info::{TraceLevel, TraceLevelNode};
use oneil_ast::naming::{Identifier, Label};
use oneil_ast::node::Node;
use oneil_ast::parameter::{
    LimitsNode, ParameterValueNode, PerformanceMarker, PerformanceMarkerNode, PiecewisePartNode,
};
use oneil_ast::{
    Span as AstSpan,
    parameter::{Limits, Parameter, ParameterNode, ParameterValue, PiecewisePart},
};

use crate::error::{ErrorHandlingParser, ParserError};
use crate::expression::parse as parse_expr;
use crate::note::parse as parse_note;
use crate::token::{
    keyword::if_,
    naming::{identifier, label},
    structure::end_of_line,
    symbol::{
        brace_left, bracket_left, bracket_right, colon, comma, dollar, equals, paren_left,
        paren_right, star, star_star,
    },
};
use crate::unit::parse as parse_unit;
use crate::util::{Result, Span};

/// Parse a parameter declaration, e.g. `$ * x(0,100): y = 2*z : kg`.
///
/// This function **may not consume the complete input**.
pub fn parse(input: Span) -> Result<ParameterNode, ParserError> {
    parameter_decl(input)
}

/// Parse a parameter declaration
///
/// This function **fails if the complete input is not consumed**.
pub fn parse_complete(input: Span) -> Result<ParameterNode, ParserError> {
    all_consuming(parameter_decl).parse(input)
}

// TODO: the "preamble" (performance, trace level, name) needs to be optional,
//       but only in design files. Maybe we can use a config flag to indicate
//       whether we're parsing a design file or a model file and adjust from there.
//       For now, we'll just require the preamble in all cases.
fn parameter_decl(input: Span) -> Result<ParameterNode, ParserError> {
    let (rest, performance_marker) = opt(performance_marker).parse(input)?;

    let (rest, trace_level) = opt(trace_level).parse(rest)?;

    let (rest, label) = label
        .convert_error_to(ParserError::expect_parameter)
        .parse(rest)?;
    let label_span = AstSpan::from(&label);
    let label = Node::new(label_span, Label::new(label.lexeme().to_string()));

    let (rest, limits) = opt(limits).parse(rest)?;

    let (rest, colon_token) = colon
        .convert_error_to(ParserError::expect_parameter)
        .parse(rest)?;

    let (rest, ident) = identifier
        .or_fail_with(ParserError::parameter_missing_identifier(&colon_token))
        .parse(rest)?;
    let ident_span = AstSpan::from(&ident);
    let ident_node = Node::new(ident_span, Identifier::new(ident.lexeme().to_string()));

    let (rest, equals_token) = equals
        .or_fail_with(ParserError::parameter_missing_equals_sign(&ident_node))
        .parse(rest)?;

    let (rest, value) = parameter_value
        .or_fail_with(ParserError::parameter_missing_value(&equals_token))
        .parse(rest)?;

    let (rest, linebreak_token) = end_of_line
        .or_fail_with(ParserError::parameter_missing_end_of_line(&value))
        .parse(rest)?;

    let (rest, note) = opt(parse_note).parse(rest)?;

    // note that for the purposes of span calculation, the note is considered
    // "whitespace"
    let whitespace_span = match &note {
        Some(note) => AstSpan::calc_span(&linebreak_token, note),
        None => AstSpan::from(&linebreak_token),
    };

    let span = match (&performance_marker, &trace_level) {
        (Some(performance), _) => {
            AstSpan::calc_span_with_whitespace(performance, &value, &whitespace_span)
        }
        (None, Some(trace_level)) => {
            AstSpan::calc_span_with_whitespace(trace_level, &value, &whitespace_span)
        }
        (None, None) => AstSpan::calc_span_with_whitespace(&label, &value, &whitespace_span),
    };

    let param = Parameter::new(
        label,
        ident_node,
        value,
        limits,
        performance_marker,
        trace_level,
        note,
    );

    let node = Node::new(span, param);

    Ok((rest, node))
}

/// Parse a performance indicator (`$`).
fn performance_marker(input: Span) -> Result<PerformanceMarkerNode, ParserError> {
    dollar
        .convert_errors()
        .map(|token| Node::new(token, PerformanceMarker::new()))
        .parse(input)
}

/// Parse a trace level indicator (`*` or `**`).
fn trace_level(input: Span) -> Result<TraceLevelNode, ParserError> {
    let single_star = star.map(|token| Node::new(token, TraceLevel::Trace));
    let double_star = star_star.map(|token| Node::new(token, TraceLevel::Debug));

    double_star.or(single_star).convert_errors().parse(input)
}

/// Parse parameter limits (either continuous or discrete).
fn limits(input: Span) -> Result<LimitsNode, ParserError> {
    alt((continuous_limits, discrete_limits)).parse(input)
}

/// Parse continuous limits in parentheses, e.g. `(0, 100)`.
fn continuous_limits(input: Span) -> Result<LimitsNode, ParserError> {
    let (rest, paren_left_token) = paren_left.convert_errors().parse(input)?;

    let (rest, min) = parse_expr
        .or_fail_with(ParserError::limit_missing_min(&paren_left_token))
        .parse(rest)?;

    let (rest, comma_token) = comma
        .or_fail_with(ParserError::limit_missing_comma(&min))
        .parse(rest)?;

    let (rest, max) = parse_expr
        .or_fail_with(ParserError::limit_missing_max(&comma_token))
        .parse(rest)?;

    let (rest, paren_right_token) = paren_right
        .or_fail_with(ParserError::unclosed_paren(&paren_left_token))
        .parse(rest)?;

    let span = AstSpan::calc_span(&paren_left_token, &paren_right_token);

    let node = Node::new(span, Limits::continuous(min, max));

    Ok((rest, node))
}

/// Parse discrete limits in square brackets, e.g. `[1, 2, 3]`.
fn discrete_limits(input: Span) -> Result<LimitsNode, ParserError> {
    let (rest, bracket_left_token) = bracket_left.convert_errors().parse(input)?;

    let (rest, values) = separated_list1(comma.convert_errors(), parse_expr)
        .or_fail_with(ParserError::limit_missing_values(&bracket_left_token))
        .parse(rest)?;

    let (rest, bracket_right_token) = bracket_right
        .or_fail_with(ParserError::unclosed_bracket(&bracket_left_token))
        .parse(rest)?;

    let span = AstSpan::calc_span(&bracket_left_token, &bracket_right_token);

    let node = Node::new(span, Limits::discrete(values));

    Ok((rest, node))
}

/// Parse a parameter value (either simple or piecewise).
fn parameter_value(input: Span) -> Result<ParameterValueNode, ParserError> {
    simple_value.or(piecewise_value).parse(input)
}

/// Parse a simple parameter value (expression with optional unit).
fn simple_value(input: Span) -> Result<ParameterValueNode, ParserError> {
    let (rest, expr) = parse_expr.parse(input)?;

    let (rest, unit) = opt(|input| {
        let (rest, colon_token) = colon.convert_errors().parse(input)?;

        let (rest, unit) = parse_unit
            .or_fail_with(ParserError::parameter_missing_unit(&colon_token))
            .parse(rest)?;

        Ok((rest, unit))
    })
    .parse(rest)?;

    let span = match &unit {
        Some(unit) => AstSpan::calc_span(&expr, unit),
        None => AstSpan::from(&expr),
    };

    let node = Node::new(span, ParameterValue::simple(expr, unit));

    Ok((rest, node))
}

/// Parse a piecewise parameter value.
fn piecewise_value(input: Span) -> Result<ParameterValueNode, ParserError> {
    let (rest, first_part) = piecewise_part.parse(input)?;

    let (rest, unit) = opt(|input| {
        let (rest, colon_token) = colon.convert_errors().parse(input)?;

        let (rest, unit) = parse_unit
            .or_fail_with(ParserError::parameter_missing_unit(&colon_token))
            .parse(rest)?;

        Ok((rest, unit))
    })
    .parse(rest)?;

    let (rest, rest_parts) = many0(|input| {
        let (rest, _) = end_of_line.convert_errors().parse(input)?;
        let (rest, part) = piecewise_part.parse(rest)?;
        Ok((rest, part))
    })
    .parse(rest)?;

    let span = match (rest_parts.last(), &unit) {
        (Some(part), _) => AstSpan::calc_span(&first_part, part),
        (None, Some(unit)) => AstSpan::calc_span(&first_part, unit),
        (None, None) => AstSpan::from(&first_part),
    };

    let mut parts = Vec::with_capacity(1 + rest_parts.len());
    parts.push(first_part);
    parts.extend(rest_parts);

    let node = Node::new(span, ParameterValue::piecewise(parts, unit));

    Ok((rest, node))
}

/// Parse a single piece of a piecewise expression, e.g. `{2*x if x > 0`.
fn piecewise_part(input: Span) -> Result<PiecewisePartNode, ParserError> {
    let (rest, brace_left_token) = brace_left.convert_errors().parse(input)?;

    let (rest, expr) = parse_expr
        .or_fail_with(ParserError::piecewise_missing_expr(&brace_left_token))
        .parse(rest)?;

    let (rest, if_token) = if_
        .or_fail_with(ParserError::piecewise_missing_if(&expr))
        .parse(rest)?;

    let (rest, if_expr) = parse_expr
        .or_fail_with(ParserError::piecewise_missing_if_expr(&if_token))
        .parse(rest)?;

    let node = Node::new(
        AstSpan::calc_span(&brace_left_token, &if_expr),
        PiecewisePart::new(expr, if_expr),
    );

    Ok((rest, node))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;
    use oneil_ast::{
        expression::{Expr, Literal},
        unit::{UnitExpr, UnitOp},
    };

    #[test]
    fn test_parse_simple_parameter() {
        let input = Span::new_extra("x: y = 42", Config::default());
        let (_, param) = parse(input).unwrap();

        assert_eq!(param.label().node_value().as_str(), "x");
        assert_eq!(param.ident().node_value().as_str(), "y");
        assert!(param.performance_marker().is_none());
        assert!(param.trace_level().is_none());

        match param.value().node_value() {
            ParameterValue::Simple(expr, unit) => {
                let expected_value = Node::new(AstSpan::new(7, 9, 9), Literal::number(42.0));
                let expected_expr = Node::new(AstSpan::new(7, 9, 9), Expr::literal(expected_value));

                assert_eq!(expr, &expected_expr);
                assert!(unit.is_none());
            }
            _ => panic!("Expected simple parameter value"),
        }
    }

    #[test]
    fn test_parse_parameter_with_multiword_label() {
        let input = Span::new_extra("Value of x: x = 42", Config::default());
        let (_, param) = parse(input).expect("Parameter should parse");
        assert_eq!(param.label().node_value().as_str(), "Value of x");
    }

    #[test]
    fn test_parse_parameter_with_continuous_limits() {
        let input = Span::new_extra("x(0, 100): y = 42", Config::default());
        let (_, param) = parse(input).unwrap();

        match param.limits().map(|limits| limits.node_value()) {
            Some(Limits::Continuous { min, max }) => {
                let expected_min_literal = Node::new(AstSpan::new(2, 3, 3), Literal::number(0.0));
                let expected_min =
                    Node::new(AstSpan::new(2, 3, 3), Expr::literal(expected_min_literal));
                let expected_max_literal = Node::new(AstSpan::new(5, 8, 8), Literal::number(100.0));
                let expected_max =
                    Node::new(AstSpan::new(5, 8, 8), Expr::literal(expected_max_literal));

                assert_eq!(min, &expected_min);
                assert_eq!(max, &expected_max);
            }
            _ => panic!("Expected continuous limits"),
        }
    }

    #[test]
    fn test_parse_parameter_with_discrete_limits() {
        let input = Span::new_extra("x[1, 2, 3]: y = 42", Config::default());
        let (_, param) = parse(input).unwrap();

        match param.limits().map(|limits| limits.node_value()) {
            Some(Limits::Discrete { values }) => {
                let expected_literals = vec![
                    Node::new(AstSpan::new(2, 3, 3), Literal::number(1.0)),
                    Node::new(AstSpan::new(5, 6, 6), Literal::number(2.0)),
                    Node::new(AstSpan::new(8, 9, 9), Literal::number(3.0)),
                ];

                let expected_exprs = expected_literals
                    .iter()
                    .map(|literal| {
                        let literal_span = AstSpan::from(literal);
                        Node::new(literal_span, Expr::literal(literal.clone()))
                    })
                    .collect::<Vec<_>>();

                assert_eq!(values.len(), 3);
                assert_eq!(values[0], expected_exprs[0]);
                assert_eq!(values[1], expected_exprs[1]);
                assert_eq!(values[2], expected_exprs[2]);
            }
            _ => panic!("Expected discrete limits"),
        }
    }

    #[test]
    fn test_parse_parameter_with_performance() {
        let input = Span::new_extra("$ x: y = 42", Config::default());
        let (_, param) = parse(input).unwrap();

        assert!(param.performance_marker().is_some());
    }

    #[test]
    fn test_parse_parameter_with_trace() {
        let input = Span::new_extra("* x: y = 42", Config::default());
        let (_, param) = parse(input).unwrap();

        assert_eq!(
            param
                .trace_level()
                .map(|trace_level| trace_level.node_value()),
            Some(&TraceLevel::Trace)
        );
    }

    #[test]
    fn test_parse_parameter_with_debug() {
        let input = Span::new_extra("** x: y = 42", Config::default());
        let (_, param) = parse(input).unwrap();

        assert_eq!(
            param
                .trace_level()
                .map(|trace_level| trace_level.node_value()),
            Some(&TraceLevel::Debug)
        );
    }

    #[test]
    fn test_parse_parameter_with_simple_units() {
        let input = Span::new_extra("x: y = 42 : kg", Config::default());
        let (_, param) = parse(input).unwrap();
        match param.value().node_value() {
            ParameterValue::Simple(expr, unit) => {
                let expected_value = Node::new(AstSpan::new(7, 9, 10), Literal::number(42.0));
                let expected_expr =
                    Node::new(AstSpan::new(7, 9, 10), Expr::literal(expected_value));

                assert_eq!(expr, &expected_expr);

                let expected_unit_identifier =
                    Node::new(AstSpan::new(12, 14, 14), Identifier::new("kg".to_string()));
                let expected_unit = Node::new(
                    AstSpan::new(12, 14, 14),
                    UnitExpr::unit(expected_unit_identifier, None),
                );

                assert_eq!(unit, &Some(expected_unit));
            }
            _ => panic!("Expected simple parameter value"),
        }
    }

    #[test]
    fn test_parse_parameter_with_compound_units() {
        let input = Span::new_extra("x: y = 42 : m/s^2", Config::default());
        let (_, param) = parse(input).unwrap();
        match param.value().node_value() {
            ParameterValue::Simple(_expr, unit) => {
                let unit = unit.clone().unwrap();

                match unit.node_value() {
                    UnitExpr::BinaryOp { op, .. } => {
                        assert_eq!(op.node_value(), &UnitOp::Divide);
                    }
                    _ => panic!("Expected binary unit expression"),
                }
            }
            _ => panic!("Expected simple parameter value"),
        }
    }

    #[test]
    fn test_parse_piecewise_parameter() {
        let input = Span::new_extra("x: y = {2*z if z > 0 \n {0 if z <= 0", Config::default());
        let (_, param) = parse(input).unwrap();
        match param.node_value().value().node_value() {
            ParameterValue::Piecewise(piecewise, unit) => {
                assert_eq!(piecewise.len(), 2);

                // First piece: 2*z if z > 0
                let first = &piecewise[0];
                assert!(matches!(
                    first.node_value().expr().node_value(),
                    Expr::BinaryOp { .. }
                ));
                assert!(matches!(
                    first.node_value().if_expr().node_value(),
                    Expr::BinaryOp { .. }
                ));

                // Second piece: 0 if z <= 0
                let second = &piecewise[1];
                assert!(matches!(
                    second.node_value().expr().node_value(),
                    Expr::Literal(_)
                ));
                assert!(matches!(
                    second.node_value().if_expr().node_value(),
                    Expr::BinaryOp { .. }
                ));

                assert!(unit.is_none());
            }
            _ => panic!("Expected piecewise parameter value"),
        }
    }

    #[test]
    fn test_parse_piecewise_parameter_with_units() {
        let input = Span::new_extra(
            "x: y = {2*z if z > 0 : m/s \n {0 if z <= 0 ",
            Config::default(),
        );
        let (_, param) = parse(input).unwrap();
        match param.node_value().value().node_value() {
            ParameterValue::Piecewise(_, unit) => {
                assert!(unit.is_some());
            }
            _ => panic!("Expected piecewise parameter value"),
        }
    }

    #[test]
    fn test_parse_parameter_with_all_features() {
        let input = Span::new_extra(
            "$ ** x(0, 100): y = {2*z if z > 0 : kg/m^2 \n {-z if z <= 0",
            Config::default(),
        );
        let (_, param) = parse(input).unwrap();

        assert!(param.node_value().performance_marker().is_some());
        assert_eq!(
            param.node_value().trace_level().unwrap().node_value(),
            &TraceLevel::Debug
        );
        assert_eq!(param.node_value().label().node_value().as_str(), "x");
        assert_eq!(param.node_value().ident().node_value().as_str(), "y");

        match param.node_value().limits() {
            Some(limits) => match limits.node_value() {
                Limits::Continuous { min, max } => {
                    assert!(matches!(min.node_value(), Expr::Literal(_)));
                    assert!(matches!(max.node_value(), Expr::Literal(_)));
                }
                _ => panic!("Expected continuous limits"),
            },
            None => panic!("Expected limits"),
        }

        match param.node_value().value().node_value() {
            ParameterValue::Piecewise(piecewise, unit) => {
                assert_eq!(piecewise.len(), 2);

                // Check unit
                assert!(matches!(
                    unit.as_ref().map(|u| u.node_value()),
                    Some(UnitExpr::BinaryOp { .. })
                ));
            }
            _ => panic!("Expected piecewise parameter value"),
        }
    }

    #[test]
    fn test_parse_complete_success() {
        let input = Span::new_extra("x: y = 42\n", Config::default());
        let (rest, param) = parse_complete(input).unwrap();
        assert_eq!(param.node_value().label().node_value().as_str(), "x");
        assert_eq!(param.node_value().ident().node_value().as_str(), "y");
        assert!(param.node_value().performance_marker().is_none());
        assert!(param.node_value().trace_level().is_none());
        match param.node_value().value().node_value() {
            ParameterValue::Simple(expr, unit) => {
                assert!(matches!(expr.node_value(), Expr::Literal(_)));
                assert!(unit.is_none());
            }
            _ => panic!("Expected simple parameter value"),
        }
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_parse_complete_with_remaining_input() {
        let input = Span::new_extra("x: y = 42\nrest", Config::default());
        let result = parse_complete(input);
        assert!(result.is_err());
    }
}
