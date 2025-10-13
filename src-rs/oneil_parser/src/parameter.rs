//! Parser for parameter declarations in an Oneil program.

use nom::Parser;
use nom::branch::alt;
use nom::combinator::{all_consuming, opt};
use nom::multi::{many0, separated_list1};
use oneil_ast::{
    AstSpan, Identifier, Label, Limits, LimitsNode, Node, Parameter, ParameterNode, ParameterValue,
    ParameterValueNode, PerformanceMarker, PerformanceMarkerNode, PiecewisePart, PiecewisePartNode,
    TraceLevel, TraceLevelNode,
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
use crate::util::{InputSpan, Result};

/// Parse a parameter declaration, e.g. `$ * x(0,100): y = 2*z : kg`.
///
/// This function **may not consume the complete input**.
pub fn parse(input: InputSpan<'_>) -> Result<'_, ParameterNode, ParserError> {
    parameter_decl(input)
}

/// Parse a parameter declaration
///
/// This function **fails if the complete input is not consumed**.
pub fn parse_complete(input: InputSpan<'_>) -> Result<'_, ParameterNode, ParserError> {
    all_consuming(parameter_decl).parse(input)
}

// TODO: the "preamble" (performance, trace level, name) needs to be optional,
//       but only in design files. Maybe we can use a config flag to indicate
//       whether we're parsing a design file or a model file and adjust from there.
//       For now, we'll just require the preamble in all cases.
/// Parses a complete parameter declaration with all optional components.
fn parameter_decl(input: InputSpan<'_>) -> Result<'_, ParameterNode, ParserError> {
    let (rest, performance_marker) = opt(performance_marker).parse(input)?;

    let (rest, trace_level) = opt(trace_level).parse(rest)?;

    let (rest, label) = label
        .convert_error_to(ParserError::expect_parameter)
        .parse(rest)?;
    let label_span = AstSpan::from(&label);
    let label = Node::new(&label_span, Label::new(label.lexeme().to_string()));

    let (rest, limits) = opt(limits).parse(rest)?;

    let (rest, colon_token) = colon
        .convert_error_to(ParserError::expect_parameter)
        .parse(rest)?;

    let (rest, ident) = identifier
        .or_fail_with(ParserError::parameter_missing_identifier(&colon_token))
        .parse(rest)?;
    let ident_span = AstSpan::from(&ident);
    let ident_node = Node::new(&ident_span, Identifier::new(ident.lexeme().to_string()));

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
    let whitespace_span = note.as_ref().map_or_else(
        || AstSpan::from(&linebreak_token),
        |note| AstSpan::calc_span(&linebreak_token, note),
    );

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

    let param_node = Node::new(&span, param);

    Ok((rest, param_node))
}

/// Parse a performance indicator (`$`).
fn performance_marker(input: InputSpan<'_>) -> Result<'_, PerformanceMarkerNode, ParserError> {
    dollar
        .convert_errors()
        .map(|token| Node::new(&token, PerformanceMarker::new()))
        .parse(input)
}

/// Parse a trace level indicator (`*` or `**`).
fn trace_level(input: InputSpan<'_>) -> Result<'_, TraceLevelNode, ParserError> {
    let single_star = star.map(|token| Node::new(&token, TraceLevel::Trace));
    let double_star = star_star.map(|token| Node::new(&token, TraceLevel::Debug));

    double_star.or(single_star).convert_errors().parse(input)
}

/// Parse parameter limits (either continuous or discrete).
fn limits(input: InputSpan<'_>) -> Result<'_, LimitsNode, ParserError> {
    alt((continuous_limits, discrete_limits)).parse(input)
}

/// Parse continuous limits in parentheses, e.g. `(0, 100)`.
fn continuous_limits(input: InputSpan<'_>) -> Result<'_, LimitsNode, ParserError> {
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

    let node = Node::new(&span, Limits::continuous(min, max));

    Ok((rest, node))
}

/// Parse discrete limits in square brackets, e.g. `[1, 2, 3]`.
fn discrete_limits(input: InputSpan<'_>) -> Result<'_, LimitsNode, ParserError> {
    let (rest, bracket_left_token) = bracket_left.convert_errors().parse(input)?;

    let (rest, values) = separated_list1(comma.convert_errors(), parse_expr)
        .or_fail_with(ParserError::limit_missing_values(&bracket_left_token))
        .parse(rest)?;

    let (rest, bracket_right_token) = bracket_right
        .or_fail_with(ParserError::unclosed_bracket(&bracket_left_token))
        .parse(rest)?;

    let span = AstSpan::calc_span(&bracket_left_token, &bracket_right_token);

    let node = Node::new(&span, Limits::discrete(values));

    Ok((rest, node))
}

/// Parse a parameter value (either simple or piecewise).
fn parameter_value(input: InputSpan<'_>) -> Result<'_, ParameterValueNode, ParserError> {
    simple_value.or(piecewise_value).parse(input)
}

/// Parse a simple parameter value (expression with optional unit).
fn simple_value(input: InputSpan<'_>) -> Result<'_, ParameterValueNode, ParserError> {
    let (rest, expr) = parse_expr.parse(input)?;

    let (rest, unit) = opt(|input| {
        let (rest, colon_token) = colon.convert_errors().parse(input)?;

        let (rest, unit) = parse_unit
            .or_fail_with(ParserError::parameter_missing_unit(&colon_token))
            .parse(rest)?;

        Ok((rest, unit))
    })
    .parse(rest)?;

    let span = unit.as_ref().map_or_else(
        || AstSpan::from(&expr),
        |unit| AstSpan::calc_span(&expr, unit),
    );

    let node = Node::new(&span, ParameterValue::simple(expr, unit));

    Ok((rest, node))
}

/// Parse a piecewise parameter value.
fn piecewise_value(input: InputSpan<'_>) -> Result<'_, ParameterValueNode, ParserError> {
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

    let node = Node::new(&span, ParameterValue::piecewise(parts, unit));

    Ok((rest, node))
}

/// Parse a single piece of a piecewise expression, e.g. `{2*x if x > 0`.
fn piecewise_part(input: InputSpan<'_>) -> Result<'_, PiecewisePartNode, ParserError> {
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
        &AstSpan::calc_span(&brace_left_token, &if_expr),
        PiecewisePart::new(expr, if_expr),
    );

    Ok((rest, node))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Config,
        error::reason::{ExpectKind, IncompleteKind, ParameterKind, ParserErrorReason},
    };
    use oneil_ast::{Expr, Literal, UnitExpr, UnitOp};

    mod success {
        use super::*;

        #[test]
        fn parse_simple_parameter() {
            let input = InputSpan::new_extra("x: y = 42", Config::default());
            let (_, param) = parse(input).expect("should parse simple parameter");

            assert_eq!(param.label().as_str(), "x");
            assert_eq!(param.ident().as_str(), "y");
            assert!(param.performance_marker().is_none());
            assert!(param.trace_level().is_none());

            let ParameterValue::Simple(expr, unit) = param.value().node_value() else {
                panic!(
                    "Expected simple parameter value, got {:?}",
                    param.value().node_value()
                );
            };

            let expected_value = Node::new(&AstSpan::new(7, 2, 0), Literal::number(42.0));
            let expected_expr = Node::new(&AstSpan::new(7, 2, 0), Expr::literal(expected_value));

            assert_eq!(expr, &expected_expr);
            assert!(unit.is_none());
        }

        #[test]
        fn parse_parameter_with_multiword_label() {
            let input = InputSpan::new_extra("Value of x: x = 42", Config::default());
            let (_, param) = parse(input).expect("Parameter should parse");
            assert_eq!(param.label().as_str(), "Value of x");
        }

        #[test]
        fn parse_parameter_with_continuous_limits() {
            let input = InputSpan::new_extra("x(0, 100): y = 42", Config::default());
            let (_, param) = parse(input).expect("should parse parameter with continuous limits");

            let Some(Limits::Continuous { min, max }) = param.limits().map(Node::node_value) else {
                panic!("Expected continuous limits");
            };

            let expected_min_literal = Node::new(&AstSpan::new(2, 1, 0), Literal::number(0.0));
            let expected_min =
                Node::new(&AstSpan::new(2, 1, 0), Expr::literal(expected_min_literal));
            let expected_max_literal = Node::new(&AstSpan::new(5, 3, 0), Literal::number(100.0));
            let expected_max =
                Node::new(&AstSpan::new(5, 3, 0), Expr::literal(expected_max_literal));

            assert_eq!(min, &expected_min);
            assert_eq!(max, &expected_max);
        }

        #[test]
        fn parse_parameter_with_discrete_limits() {
            let input = InputSpan::new_extra("x[1, 2, 3]: y = 42", Config::default());
            let (_, param) = parse(input).expect("should parse parameter with discrete limits");

            let Some(Limits::Discrete { values }) = param.limits().map(Node::node_value) else {
                panic!("Expected discrete limits");
            };

            let expected_literals = [
                Node::new(&AstSpan::new(2, 1, 0), Literal::number(1.0)),
                Node::new(&AstSpan::new(5, 1, 0), Literal::number(2.0)),
                Node::new(&AstSpan::new(8, 1, 0), Literal::number(3.0)),
            ];

            let expected_exprs = expected_literals
                .iter()
                .map(|literal| {
                    let literal_span = AstSpan::from(literal);
                    Node::new(&literal_span, Expr::literal(literal.clone()))
                })
                .collect::<Vec<_>>();

            assert_eq!(values.len(), 3);
            assert_eq!(values[0], expected_exprs[0]);
            assert_eq!(values[1], expected_exprs[1]);
            assert_eq!(values[2], expected_exprs[2]);
        }

        #[test]
        fn parse_parameter_with_performance() {
            let input = InputSpan::new_extra("$ x: y = 42", Config::default());
            let (_, param) = parse(input).expect("should parse parameter with performance");

            assert!(param.performance_marker().is_some());
        }

        #[test]
        fn parse_parameter_with_trace() {
            let input = InputSpan::new_extra("* x: y = 42", Config::default());
            let (_, param) = parse(input).expect("should parse parameter with trace");

            assert_eq!(
                param.trace_level().map(Node::node_value),
                Some(&TraceLevel::Trace)
            );
        }

        #[test]
        fn parse_parameter_with_debug() {
            let input = InputSpan::new_extra("** x: y = 42", Config::default());
            let (_, param) = parse(input).expect("should parse parameter with debug");

            assert_eq!(
                param.trace_level().map(Node::node_value),
                Some(&TraceLevel::Debug)
            );
        }

        #[test]
        fn parse_parameter_with_simple_units() {
            let input = InputSpan::new_extra("x: y = 42 : kg", Config::default());
            let (_, param) = parse(input).expect("should parse parameter with simple units");
            let ParameterValue::Simple(expr, unit) = param.value().node_value() else {
                panic!(
                    "Expected simple parameter value, got {:?}",
                    param.value().node_value()
                );
            };

            let expected_value = Node::new(&AstSpan::new(7, 2, 1), Literal::number(42.0));
            let expected_expr = Node::new(&AstSpan::new(7, 2, 1), Expr::literal(expected_value));

            assert_eq!(expr, &expected_expr);

            let expected_unit_identifier =
                Node::new(&AstSpan::new(12, 2, 0), Identifier::new("kg".to_string()));
            let expected_unit = Node::new(
                &AstSpan::new(12, 2, 0),
                UnitExpr::unit(expected_unit_identifier, None),
            );

            assert_eq!(unit, &Some(expected_unit));
        }

        #[test]
        fn parse_parameter_with_compound_units() {
            let input = InputSpan::new_extra("x: y = 42 : m/s^2", Config::default());
            let (_, param) = parse(input).expect("should parse parameter with compound units");
            let ParameterValue::Simple(_expr, unit) = param.value().node_value() else {
                panic!(
                    "Expected simple parameter value, got {:?}",
                    param.value().node_value()
                );
            };

            let unit = unit.clone().expect("should have unit");

            let UnitExpr::BinaryOp { op, .. } = unit.node_value() else {
                panic!("Expected binary unit expression");
            };

            assert_eq!(op.node_value(), &UnitOp::Divide);
        }

        #[test]
        fn parse_piecewise_parameter() {
            let input =
                InputSpan::new_extra("x: y = {2*z if z > 0 \n {0 if z <= 0", Config::default());
            let (_, param) = parse(input).expect("should parse piecewise parameter");

            let param_value = param.value().node_value();
            let ParameterValue::Piecewise(piecewise, unit) = param_value else {
                panic!("Expected piecewise parameter value, got {param_value:?}");
            };

            assert_eq!(piecewise.len(), 2);

            // First piece: 2*z if z > 0
            let first = &piecewise[0];
            assert!(matches!(
                first.node_value().expr().node_value(),
                Expr::BinaryOp { .. }
            ));
            assert!(matches!(
                first.node_value().if_expr().node_value(),
                Expr::ComparisonOp { .. }
            ));

            // Second piece: 0 if z <= 0
            let second = &piecewise[1];
            assert!(matches!(
                second.node_value().expr().node_value(),
                Expr::Literal(_)
            ));
            assert!(matches!(
                second.node_value().if_expr().node_value(),
                Expr::ComparisonOp { .. }
            ));

            assert!(unit.is_none());
        }

        #[test]
        fn parse_piecewise_parameter_with_units() {
            let input = InputSpan::new_extra(
                "x: y = {2*z if z > 0 : m/s \n {0 if z <= 0 ",
                Config::default(),
            );
            let (_, param) = parse(input).expect("should parse parameter with all features");

            let param_value = param.value().node_value();
            let ParameterValue::Piecewise(_, unit) = param_value else {
                panic!("Expected piecewise parameter value, got {param_value:?}");
            };

            assert!(unit.is_some());
        }

        #[test]
        fn parse_parameter_with_all_features() {
            let input = InputSpan::new_extra(
                "$ ** x(0, 100): y = {2*z if z > 0 : kg/m^2 \n {-z if z <= 0",
                Config::default(),
            );
            let (_, param) = parse(input).expect("should parse parameter with all features");

            assert!(param.performance_marker().is_some());
            assert_eq!(
                param.trace_level().map(Node::node_value),
                Some(&TraceLevel::Debug)
            );
            assert_eq!(param.label().as_str(), "x");
            assert_eq!(param.ident().as_str(), "y");

            let Some(Limits::Continuous { min, max }) = param.limits().map(Node::node_value) else {
                panic!(
                    "Expected continuous limits, got {:?}",
                    param.limits().map(Node::node_value)
                );
            };

            assert!(matches!(min.node_value(), Expr::Literal(_)));
            assert!(matches!(max.node_value(), Expr::Literal(_)));

            let ParameterValue::Piecewise(piecewise, unit) = param.value().node_value() else {
                panic!(
                    "Expected piecewise parameter value, got {:?}",
                    param.value().node_value()
                );
            };

            assert_eq!(piecewise.len(), 2);

            // Check unit
            assert!(matches!(
                unit.as_ref().map(Node::node_value),
                Some(UnitExpr::BinaryOp { .. })
            ));
        }

        #[test]
        fn parse_complete_success() {
            let input = InputSpan::new_extra("x: y = 42\n", Config::default());
            let (rest, param) = parse_complete(input).expect("should parse complete parameter");
            assert_eq!(param.label().as_str(), "x");
            assert_eq!(param.ident().as_str(), "y");
            assert!(param.performance_marker().is_none());
            assert!(param.trace_level().is_none());

            let param_value = param.value().node_value();
            let ParameterValue::Simple(expr, unit) = param_value else {
                panic!("Expected simple parameter value, got {param_value:?}");
            };

            assert!(matches!(expr.node_value(), Expr::Literal(_)));
            assert!(unit.is_none());
            assert_eq!(rest.fragment(), &"");
        }
    }

    mod parse_complete {
        use super::*;

        #[test]
        fn parse_complete_success() {
            let input = InputSpan::new_extra("x: y = 42\n", Config::default());
            let (rest, param) = parse_complete(input).expect("should parse complete parameter");
            assert_eq!(param.label().as_str(), "x");
            assert_eq!(param.ident().as_str(), "y");
            assert!(param.performance_marker().is_none());
            assert!(param.trace_level().is_none());

            let param_value = param.value().node_value();
            let ParameterValue::Simple(expr, unit) = param_value else {
                panic!("Expected simple parameter value, got {param_value:?}");
            };

            assert!(matches!(expr.node_value(), Expr::Literal(_)));
            assert!(unit.is_none());
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        #[expect(
            clippy::assertions_on_result_states,
            reason = "we don't care about the result, just that it's an error"
        )]
        fn parse_complete_with_remaining_input() {
            let input = InputSpan::new_extra("x: y = 42\nrest", Config::default());
            let result = parse_complete(input);
            assert!(result.is_err());
        }
    }

    mod error {
        use super::*;

        #[test]
        fn missing_label() {
            let input = InputSpan::new_extra(": y = 42\n", Config::default());
            let result = parse(input);

            let Err(nom::Err::Error(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 0);
            assert!(matches!(
                error.reason,
                ParserErrorReason::Expect(ExpectKind::Parameter)
            ));
        }

        #[test]
        fn missing_identifier() {
            let input = InputSpan::new_extra("x: = 42\n", Config::default());
            let result = parse(input);
            let expected_colon_span = AstSpan::new(1, 1, 1);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 3); // After ":"
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Parameter(ParameterKind::MissingIdentifier),
                cause,
            } = error.reason
            else {
                panic!("Unexpected error {:?}", error.reason);
            };

            assert_eq!(cause, expected_colon_span);
        }

        #[test]
        fn missing_equals_sign() {
            let input = InputSpan::new_extra("x: y 42\n", Config::default());
            let result = parse(input);
            let expected_ident_span = AstSpan::new(3, 1, 1);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 5); // After "y"
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Parameter(ParameterKind::MissingEqualsSign),
                cause,
            } = error.reason
            else {
                panic!("Unexpected error {error:?}");
            };

            assert_eq!(cause, expected_ident_span);
        }

        #[test]
        fn missing_value() {
            let input = InputSpan::new_extra("x: y =\n", Config::default());
            let result = parse(input);
            let expected_equals_span = AstSpan::new(5, 1, 0);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 6); // After "="
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Parameter(ParameterKind::MissingValue),
                cause,
            } = error.reason
            else {
                panic!("Unexpected error {error:?}");
            };

            assert_eq!(cause, expected_equals_span);
        }

        #[test]
        fn missing_unit_after_colon() {
            let input = InputSpan::new_extra("x: y = 42 :\n", Config::default());
            let result = parse(input);
            let expected_colon_span = AstSpan::new(10, 1, 0);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 11); // After ":"
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Parameter(ParameterKind::MissingUnit),
                cause,
            } = error.reason
            else {
                panic!("Unexpected error {error:?}");
            };

            assert_eq!(cause, expected_colon_span);
        }

        #[test]
        fn continuous_limits_missing_min() {
            let input = InputSpan::new_extra("x(, 100): y = 42\n", Config::default());
            let result = parse(input);
            let expected_paren_span = AstSpan::new(1, 1, 0);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 2); // After "("
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Parameter(ParameterKind::LimitMissingMin),
                cause,
            } = error.reason
            else {
                panic!("Unexpected error {error:?}");
            };

            assert_eq!(cause, expected_paren_span);
        }

        #[test]
        fn continuous_limits_missing_comma() {
            let input = InputSpan::new_extra("x(0 100): y = 42\n", Config::default());
            let result = parse(input);
            let expected_min_span = AstSpan::new(2, 1, 1);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 4); // After "0"
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Parameter(ParameterKind::LimitMissingComma),
                cause,
            } = error.reason
            else {
                panic!("Unexpected error {error:?}");
            };

            assert_eq!(cause, expected_min_span);
        }

        #[test]
        fn continuous_limits_missing_max() {
            let input = InputSpan::new_extra("x(0,): y = 42\n", Config::default());
            let result = parse(input);
            let expected_comma_span = AstSpan::new(3, 1, 0);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 4); // After ","
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Parameter(ParameterKind::LimitMissingMax),
                cause,
            } = error.reason
            else {
                panic!("Unexpected error {error:?}");
            };

            assert_eq!(cause, expected_comma_span);
        }

        #[test]
        fn continuous_limits_unclosed_paren() {
            let input = InputSpan::new_extra("x(0, 100: y = 42\n", Config::default());
            let result = parse(input);
            let expected_paren_span = AstSpan::new(1, 1, 0);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 8); // After "100"
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
        fn discrete_limits_missing_values() {
            let input = InputSpan::new_extra("x[]: y = 42\n", Config::default());
            let result = parse(input);
            let expected_bracket_span = AstSpan::new(1, 1, 0);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 2); // After "["
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Parameter(ParameterKind::LimitMissingValues),
                cause,
            } = error.reason
            else {
                panic!("Unexpected error {error:?}");
            };

            assert_eq!(cause, expected_bracket_span);
        }

        #[test]
        fn discrete_limits_unclosed_bracket() {
            let input = InputSpan::new_extra("x[1, 2, 3: y = 42\n", Config::default());
            let result = parse(input);
            let expected_bracket_span = AstSpan::new(1, 1, 0);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 9); // After "3"
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::UnclosedBracket,
                cause,
            } = error.reason
            else {
                panic!("Unexpected error {error:?}");
            };

            assert_eq!(cause, expected_bracket_span);
        }

        #[test]
        fn piecewise_missing_expr() {
            let input = InputSpan::new_extra("x: y = { if z > 0\n", Config::default());
            let result = parse(input);
            let expected_brace_span = AstSpan::new(7, 1, 1);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 9); // After "{"
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Parameter(ParameterKind::PiecewiseMissingExpr),
                cause,
            } = error.reason
            else {
                panic!("Unexpected error {error:?}");
            };

            assert_eq!(cause, expected_brace_span);
        }

        #[test]
        fn piecewise_missing_if() {
            let input = InputSpan::new_extra("x: y = {2*z z > 0\n", Config::default());
            let result = parse(input);
            let expected_expr_span = AstSpan::new(8, 3, 1);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 12); // After "2*z"
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Parameter(ParameterKind::PiecewiseMissingIf),
                cause,
            } = error.reason
            else {
                panic!("Unexpected error {error:?}");
            };

            assert_eq!(cause, expected_expr_span);
        }

        #[test]
        fn piecewise_missing_if_expr() {
            let input = InputSpan::new_extra("x: y = {2*z if\n", Config::default());
            let result = parse(input);
            let expected_if_span = AstSpan::new(12, 2, 0);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 14); // After "if"
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Parameter(ParameterKind::PiecewiseMissingIfExpr),
                cause,
            } = error.reason
            else {
                panic!("Unexpected error {error:?}");
            };

            assert_eq!(cause, expected_if_span);
        }

        #[test]
        fn piecewise_missing_unit_after_colon() {
            let input = InputSpan::new_extra("x: y = {2*z if z > 0 :\n", Config::default());
            let result = parse(input);
            let expected_colon_span = AstSpan::new(21, 1, 0);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 22); // After ":"
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Parameter(ParameterKind::MissingUnit),
                cause,
            } = error.reason
            else {
                panic!("Unexpected error {error:?}");
            };

            assert_eq!(cause, expected_colon_span);
        }

        #[test]
        fn invalid_expression() {
            let input = InputSpan::new_extra("x: y = @invalid\n", Config::default());
            let result = parse(input);
            let expected_equals_span = AstSpan::new(5, 1, 1);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 7); // At "@"
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Parameter(ParameterKind::MissingValue),
                cause,
            } = error.reason
            else {
                panic!("Unexpected error {error:?}");
            };

            assert_eq!(cause, expected_equals_span);
        }

        #[test]
        fn invalid_unit() {
            let input = InputSpan::new_extra("x: y = 42 : @invalid\n", Config::default());
            let result = parse(input);
            let expected_colon_span = AstSpan::new(10, 1, 1);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 12); // After ":"
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Parameter(ParameterKind::MissingUnit),
                cause,
            } = error.reason
            else {
                panic!("Unexpected error {error:?}");
            };

            assert_eq!(cause, expected_colon_span);
        }

        #[test]
        fn malformed_performance_marker() {
            let input = InputSpan::new_extra("$$ x: y = 42\n", Config::default());
            let result = parse(input);

            let Err(nom::Err::Error(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 1);
            assert!(matches!(
                error.reason,
                ParserErrorReason::Expect(ExpectKind::Parameter)
            ));
        }

        #[test]
        fn malformed_trace_level() {
            let input = InputSpan::new_extra("*** x: y = 42\n", Config::default());
            let result = parse(input);

            let Err(nom::Err::Error(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 2);
            assert!(matches!(
                error.reason,
                ParserErrorReason::Expect(ExpectKind::Parameter)
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
                ParserErrorReason::Expect(ExpectKind::Parameter)
            ));
        }

        #[test]
        fn whitespace_only() {
            let input = InputSpan::new_extra("   \n", Config::default());
            let result = parse(input);

            let Err(nom::Err::Error(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 0);
            assert!(matches!(
                error.reason,
                ParserErrorReason::Expect(ExpectKind::Parameter)
            ));
        }

        #[test]
        fn missing_colon_after_label() {
            let input = InputSpan::new_extra("x y = 42\n", Config::default());
            let result = parse(input);

            let Err(nom::Err::Error(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 4); // After "x"
            assert!(matches!(
                error.reason,
                ParserErrorReason::Expect(ExpectKind::Parameter)
            ));
        }

        #[test]
        fn missing_colon_after_limits() {
            let input = InputSpan::new_extra("x(0, 100) y = 42\n", Config::default());
            let result = parse(input);

            let Err(nom::Err::Error(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 10); // After ")"
            assert!(matches!(
                error.reason,
                ParserErrorReason::Expect(ExpectKind::Parameter)
            ));
        }

        #[test]
        fn mixed_limits_syntax() {
            let input = InputSpan::new_extra("x(0, 100][1, 2]: y = 42\n", Config::default());
            let result = parse(input);
            let expected_paren_span = AstSpan::new(1, 1, 0);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 8); // At "]"
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
        fn piecewise_missing_newline_between_parts() {
            let input =
                InputSpan::new_extra("x: y = {2*z if z > 0 {0 if z <= 0\n", Config::default());
            let result = parse(input);
            let expected_first_part_span = AstSpan::new(7, 13, 1);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 21); // After first part
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Parameter(ParameterKind::MissingEndOfLine),
                cause,
            } = error.reason
            else {
                panic!("Unexpected error {error:?}");
            };

            assert_eq!(cause, expected_first_part_span);
        }

        #[test]
        fn continuous_limits_with_extra_comma() {
            let input = InputSpan::new_extra("x(0, 100,): y = 42\n", Config::default());
            let result = parse(input);
            let expected_paren_span = AstSpan::new(1, 1, 0);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 8); // At extra ","
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
        fn discrete_limits_with_trailing_comma() {
            let input = InputSpan::new_extra("x[1, 2, 3,]: y = 42\n", Config::default());
            let result = parse(input);
            let expected_bracket_span = AstSpan::new(1, 1, 0);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 9); // At trailing ","
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::UnclosedBracket,
                cause,
            } = error.reason
            else {
                panic!("Unexpected error {error:?}");
            };

            assert_eq!(cause, expected_bracket_span);
        }

        #[test]
        fn piecewise_with_unit_on_wrong_line() {
            let input = InputSpan::new_extra(
                "x: y = {2*z if z > 0\n{0 if z <= 0 : m/s\n",
                Config::default(),
            );
            let result = parse(input);
            let expected_second_part_span = AstSpan::new(7, 26, 1);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 34); // After second part
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Parameter(ParameterKind::MissingEndOfLine),
                cause,
            } = error.reason
            else {
                panic!("Unexpected error {error:?}");
            };

            assert_eq!(cause, expected_second_part_span);
        }
    }
}
