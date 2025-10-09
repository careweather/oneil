//! Parser for parameter declarations in an Oneil program.

use nom::Parser;
use nom::branch::alt;
use nom::combinator::{all_consuming, opt};
use nom::multi::{many0, separated_list1};
use oneil_ast::{
    IdentifierNode, LabelNode, Limits, LimitsNode, Node, Parameter, ParameterNode, ParameterValue,
    ParameterValueNode, PerformanceMarker, PerformanceMarkerNode, PiecewisePart, PiecewisePartNode,
    TraceLevel, TraceLevelNode,
};
use oneil_shared::span::Span;

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
    let (rest, performance_marker_node) = opt(performance_marker).parse(input)?;

    let (rest, trace_level_node) = opt(trace_level).parse(rest)?;

    let (rest, label_token) = label
        .convert_error_to(ParserError::expect_parameter)
        .parse(rest)?;
    let label_node = LabelNode::from(label_token);

    let (rest, limits_node) = opt(limits).parse(rest)?;

    let (rest, colon_token) = colon
        .convert_error_to(ParserError::expect_parameter)
        .parse(rest)?;

    let (rest, ident_token) = identifier
        .or_fail_with(ParserError::parameter_missing_identifier(
            colon_token.lexeme_span,
        ))
        .parse(rest)?;
    let ident_node = IdentifierNode::from(ident_token);

    let (rest, equals_token) = equals
        .or_fail_with(ParserError::parameter_missing_equals_sign(
            ident_node.span(),
        ))
        .parse(rest)?;

    let (rest, value_node) = parameter_value
        .or_fail_with(ParserError::parameter_missing_value(
            equals_token.lexeme_span,
        ))
        .parse(rest)?;

    let (rest, linebreak_token) = end_of_line
        .or_fail_with(ParserError::parameter_missing_end_of_line(
            value_node.span(),
        ))
        .parse(rest)?;

    let (rest, note_node) = opt(parse_note).parse(rest)?;

    let param_start_span = match (&performance_marker_node, &trace_level_node) {
        (Some(performance_marker_node), _) => performance_marker_node.span(),
        (None, Some(trace_level_node)) => trace_level_node.span(),
        (None, None) => label_token.lexeme_span,
    };

    let (param_end_span, param_whitespace_span) = match &note_node {
        Some(note_node) => (note_node.span(), note_node.whitespace_span()),
        None => (linebreak_token.lexeme_span, linebreak_token.whitespace_span),
    };

    let param_span = Span::from_start_and_end(&param_start_span, &param_end_span);

    let param = Parameter::new(
        label_node,
        ident_node,
        value_node,
        limits_node,
        performance_marker_node,
        trace_level_node,
        note_node,
    );

    let param_node = Node::new(param, param_span, param_whitespace_span);

    Ok((rest, param_node))
}

/// Parse a performance indicator (`$`).
fn performance_marker(input: InputSpan<'_>) -> Result<'_, PerformanceMarkerNode, ParserError> {
    dollar
        .convert_errors()
        .map(|token| token.into_node_with_value(PerformanceMarker::new()))
        .parse(input)
}

/// Parse a trace level indicator (`*` or `**`).
fn trace_level(input: InputSpan<'_>) -> Result<'_, TraceLevelNode, ParserError> {
    let single_star = star.map(|token| token.into_node_with_value(TraceLevel::Trace));
    let double_star = star_star.map(|token| token.into_node_with_value(TraceLevel::Debug));

    double_star.or(single_star).convert_errors().parse(input)
}

/// Parse parameter limits (either continuous or discrete).
fn limits(input: InputSpan<'_>) -> Result<'_, LimitsNode, ParserError> {
    alt((continuous_limits, discrete_limits)).parse(input)
}

/// Parse continuous limits in parentheses, e.g. `(0, 100)`.
fn continuous_limits(input: InputSpan<'_>) -> Result<'_, LimitsNode, ParserError> {
    let (rest, paren_left_token) = paren_left.convert_errors().parse(input)?;

    let (rest, min_node) = parse_expr
        .or_fail_with(ParserError::limit_missing_min(paren_left_token.lexeme_span))
        .parse(rest)?;

    let (rest, comma_token) = comma
        .or_fail_with(ParserError::limit_missing_comma(min_node.span()))
        .parse(rest)?;

    let (rest, max_node) = parse_expr
        .or_fail_with(ParserError::limit_missing_max(comma_token.lexeme_span))
        .parse(rest)?;

    let (rest, paren_right_token) = paren_right
        .or_fail_with(ParserError::unclosed_paren(paren_left_token.lexeme_span))
        .parse(rest)?;

    let span = Span::from_start_and_end(
        &paren_left_token.lexeme_span,
        &paren_right_token.lexeme_span,
    );
    let whitespace_span = paren_right_token.whitespace_span;

    let node = Node::new(
        Limits::continuous(min_node, max_node),
        span,
        whitespace_span,
    );

    Ok((rest, node))
}

/// Parse discrete limits in square brackets, e.g. `[1, 2, 3]`.
fn discrete_limits(input: InputSpan<'_>) -> Result<'_, LimitsNode, ParserError> {
    let (rest, bracket_left_token) = bracket_left.convert_errors().parse(input)?;

    let (rest, value_nodes) = separated_list1(comma.convert_errors(), parse_expr)
        .or_fail_with(ParserError::limit_missing_values(
            bracket_left_token.lexeme_span,
        ))
        .parse(rest)?;

    let (rest, bracket_right_token) = bracket_right
        .or_fail_with(ParserError::unclosed_bracket(
            bracket_left_token.lexeme_span,
        ))
        .parse(rest)?;

    let span = Span::from_start_and_end(
        &bracket_left_token.lexeme_span,
        &bracket_right_token.lexeme_span,
    );
    let whitespace_span = bracket_right_token.whitespace_span;

    let node = Node::new(Limits::discrete(value_nodes), span, whitespace_span);

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
            .or_fail_with(ParserError::parameter_missing_unit(colon_token.lexeme_span))
            .parse(rest)?;

        Ok((rest, unit))
    })
    .parse(rest)?;

    let start_span = expr.span();
    let end_span = unit
        .as_ref()
        .map_or_else(|| expr.span(), |unit| unit.span());

    let span = Span::from_start_and_end(&start_span, &end_span);
    let whitespace_span = unit
        .as_ref()
        .map_or_else(|| expr.whitespace_span(), |unit| unit.whitespace_span());

    let node = Node::new(ParameterValue::simple(expr, unit), span, whitespace_span);

    Ok((rest, node))
}

/// Parse a piecewise parameter value.
fn piecewise_value(input: InputSpan<'_>) -> Result<'_, ParameterValueNode, ParserError> {
    let (rest, first_part) = piecewise_part.parse(input)?;

    let (rest, unit) = opt(|input| {
        let (rest, colon_token) = colon.convert_errors().parse(input)?;

        let (rest, unit) = parse_unit
            .or_fail_with(ParserError::parameter_missing_unit(colon_token.lexeme_span))
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

    let start_span = first_part.span();
    let (end_span, whitespace_span) = match (rest_parts.last(), &unit) {
        (Some(part), _) => (part.span(), part.whitespace_span()),
        (None, Some(unit)) => (unit.span(), unit.whitespace_span()),
        (None, None) => (first_part.span(), first_part.whitespace_span()),
    };
    let span = Span::from_start_and_end(&start_span, &end_span);

    let mut parts = Vec::with_capacity(1 + rest_parts.len());
    parts.push(first_part);
    parts.extend(rest_parts);

    let node = Node::new(
        ParameterValue::piecewise(parts, unit),
        span,
        whitespace_span,
    );

    Ok((rest, node))
}

/// Parse a single piece of a piecewise expression, e.g. `{2*x if x > 0`.
fn piecewise_part(input: InputSpan<'_>) -> Result<'_, PiecewisePartNode, ParserError> {
    let (rest, brace_left_token) = brace_left.convert_errors().parse(input)?;

    let (rest, expr_node) = parse_expr
        .or_fail_with(ParserError::piecewise_missing_expr(
            brace_left_token.lexeme_span,
        ))
        .parse(rest)?;

    let (rest, if_token) = if_
        .or_fail_with(ParserError::piecewise_missing_if(expr_node.span()))
        .parse(rest)?;

    let (rest, if_expr) = parse_expr
        .or_fail_with(ParserError::piecewise_missing_if_expr(if_token.lexeme_span))
        .parse(rest)?;

    let start_span = brace_left_token.lexeme_span;
    let end_span = if_expr.span();
    let span = Span::from_start_and_end(&start_span, &end_span);
    let whitespace_span = if_expr.whitespace_span();

    let node = Node::new(
        PiecewisePart::new(expr_node, if_expr),
        span,
        whitespace_span,
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
        use std::ops::Deref;

        use super::*;

        #[test]
        fn parse_simple_parameter() {
            let input = InputSpan::new_extra("x: y = 42", Config::default());
            let (_, param) = parse(input).expect("should parse simple parameter");

            assert_eq!(param.label().as_str(), "x");
            assert_eq!(param.ident().as_str(), "y");
            assert!(param.performance_marker().is_none());
            assert!(param.trace_level().is_none());

            let ParameterValue::Simple(expr, unit) = param.value().clone().take_value() else {
                panic!("Expected simple parameter value, got {:?}", param.value());
            };

            let Expr::Literal(value) = expr.take_value() else {
                panic!("Expected literal");
            };

            let Literal::Number(value) = value.take_value() else {
                panic!("Expected literal");
            };

            assert_eq!(value, 42.0);

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

            let Some(Limits::Continuous { min, max }) = param.limits().map(Node::deref).cloned()
            else {
                panic!("Expected continuous limits");
            };

            let Expr::Literal(value) = min.take_value() else {
                panic!("Expected literal");
            };
            let Literal::Number(value) = value.take_value() else {
                panic!("Expected literal");
            };
            assert_eq!(value, 0.0);

            let Expr::Literal(value) = max.take_value() else {
                panic!("Expected literal");
            };
            let Literal::Number(value) = value.take_value() else {
                panic!("Expected literal");
            };
            assert_eq!(value, 100.0);
        }

        #[test]
        fn parse_parameter_with_discrete_limits() {
            let input = InputSpan::new_extra("x[1, 2, 3]: y = 42", Config::default());
            let (_, param) = parse(input).expect("should parse parameter with discrete limits");

            let Some(Limits::Discrete { mut values }) = param.limits().map(Node::deref).cloned()
            else {
                panic!("Expected discrete limits");
            };

            assert_eq!(values.len(), 3);

            let value1 = values.remove(0);
            let value2 = values.remove(0);
            let value3 = values.remove(0);

            let Expr::Literal(value) = value1.take_value() else {
                panic!("Expected literal");
            };
            let Literal::Number(value) = value.take_value() else {
                panic!("Expected literal");
            };
            assert_eq!(value, 1.0);

            let Expr::Literal(value) = value2.take_value() else {
                panic!("Expected literal");
            };
            let Literal::Number(value) = value.take_value() else {
                panic!("Expected literal");
            };
            assert_eq!(value, 2.0);

            let Expr::Literal(value) = value3.take_value() else {
                panic!("Expected literal");
            };
            let Literal::Number(value) = value.take_value() else {
                panic!("Expected literal");
            };
            assert_eq!(value, 3.0);
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
                param.trace_level().map(Node::deref).cloned(),
                Some(TraceLevel::Trace)
            );
        }

        #[test]
        fn parse_parameter_with_debug() {
            let input = InputSpan::new_extra("** x: y = 42", Config::default());
            let (_, param) = parse(input).expect("should parse parameter with debug");

            assert_eq!(
                param.trace_level().map(Node::deref).cloned(),
                Some(TraceLevel::Debug)
            );
        }

        #[test]
        fn parse_parameter_with_simple_units() {
            let input = InputSpan::new_extra("x: y = 42 : kg", Config::default());
            let (_, param) = parse(input).expect("should parse parameter with simple units");
            let ParameterValue::Simple(expr, unit) = param.value().clone().take_value() else {
                panic!("Expected simple parameter value, got {:?}", param.value());
            };

            let Expr::Literal(value) = expr.take_value() else {
                panic!("Expected literal");
            };
            let Literal::Number(value) = value.take_value() else {
                panic!("Expected literal");
            };
            assert_eq!(value, 42.0);

            let Some(UnitExpr::Unit {
                identifier,
                exponent,
            }) = unit.as_ref().map(Node::deref).cloned()
            else {
                panic!("Expected unit");
            };
            assert_eq!(identifier.as_str(), "kg");

            assert_eq!(exponent, None);
        }

        #[test]
        fn parse_parameter_with_compound_units() {
            let input = InputSpan::new_extra("x: y = 42 : m/s^2", Config::default());
            let (_, param) = parse(input).expect("should parse parameter with compound units");
            let ParameterValue::Simple(_expr, unit) = param.value().clone().take_value() else {
                panic!("Expected simple parameter value, got {:?}", param.value());
            };

            let unit = unit.clone().expect("should have unit");

            let UnitExpr::BinaryOp { op, .. } = unit.take_value() else {
                panic!("Expected binary unit expression");
            };

            assert_eq!(*op, UnitOp::Divide);
        }

        #[test]
        fn parse_piecewise_parameter() {
            let input =
                InputSpan::new_extra("x: y = {2*z if z > 0 \n {0 if z <= 0", Config::default());
            let (_, param) = parse(input).expect("should parse piecewise parameter");

            let param_value = param.value().clone().take_value();
            let ParameterValue::Piecewise(piecewise, unit) = param_value else {
                panic!("Expected piecewise parameter value, got {param_value:?}");
            };

            assert_eq!(piecewise.len(), 2);

            // First piece: 2*z if z > 0
            let first = &piecewise[0];
            assert!(matches!(
                first.expr().clone().take_value(),
                Expr::BinaryOp { .. }
            ));
            assert!(matches!(
                first.if_expr().clone().take_value(),
                Expr::ComparisonOp { .. }
            ));

            // Second piece: 0 if z <= 0
            let second = &piecewise[1];
            assert!(matches!(
                second.expr().clone().take_value(),
                Expr::Literal(_)
            ));
            assert!(matches!(
                second.if_expr().clone().take_value(),
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

            let param_value = param.value().clone().take_value();
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
                param.trace_level().map(Node::deref).cloned(),
                Some(TraceLevel::Debug)
            );
            assert_eq!(param.label().as_str(), "x");
            assert_eq!(param.ident().as_str(), "y");

            let Some(Limits::Continuous { min, max }) = param.limits().map(Node::deref).cloned()
            else {
                panic!("Expected continuous limits, got {:?}", param.limits());
            };

            assert!(matches!(min.take_value(), Expr::Literal(_)));
            assert!(matches!(max.take_value(), Expr::Literal(_)));

            let ParameterValue::Piecewise(piecewise, unit) = param.value().clone().take_value()
            else {
                panic!(
                    "Expected piecewise parameter value, got {:?}",
                    param.value()
                );
            };

            assert_eq!(piecewise.len(), 2);

            // Check unit
            assert!(matches!(
                unit.as_ref().map(Node::deref).cloned(),
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

            let param_value = param.value();
            let ParameterValue::Simple(expr, unit) = param_value.clone().take_value() else {
                panic!("Expected simple parameter value, got {param_value:?}");
            };

            assert!(matches!(expr.take_value(), Expr::Literal(_)));
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

            let param_value = param.value().clone().take_value();
            let ParameterValue::Simple(expr, unit) = param_value else {
                panic!("Expected simple parameter value, got {param_value:?}");
            };

            assert!(matches!(expr.take_value(), Expr::Literal(_)));
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

            assert_eq!(cause.start().offset, 1);
            assert_eq!(cause.end().offset, 2);
        }

        #[test]
        fn missing_equals_sign() {
            let input = InputSpan::new_extra("x: y 42\n", Config::default());
            let result = parse(input);

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

            assert_eq!(cause.start().offset, 3);
            assert_eq!(cause.end().offset, 4);
        }

        #[test]
        fn missing_value() {
            let input = InputSpan::new_extra("x: y =\n", Config::default());
            let result = parse(input);

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

            assert_eq!(cause.start().offset, 5);
            assert_eq!(cause.end().offset, 6);
        }

        #[test]
        fn missing_unit_after_colon() {
            let input = InputSpan::new_extra("x: y = 42 :\n", Config::default());
            let result = parse(input);

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

            assert_eq!(cause.start().offset, 10);
            assert_eq!(cause.end().offset, 11);
        }

        #[test]
        fn continuous_limits_missing_min() {
            let input = InputSpan::new_extra("x(, 100): y = 42\n", Config::default());
            let result = parse(input);

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

            assert_eq!(cause.start().offset, 1);
            assert_eq!(cause.end().offset, 2);
        }

        #[test]
        fn continuous_limits_missing_comma() {
            let input = InputSpan::new_extra("x(0 100): y = 42\n", Config::default());
            let result = parse(input);

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

            assert_eq!(cause.start().offset, 2);
            assert_eq!(cause.end().offset, 3);
        }

        #[test]
        fn continuous_limits_missing_max() {
            let input = InputSpan::new_extra("x(0,): y = 42\n", Config::default());
            let result = parse(input);

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

            assert_eq!(cause.start().offset, 3);
            assert_eq!(cause.end().offset, 4);
        }

        #[test]
        fn continuous_limits_unclosed_paren() {
            let input = InputSpan::new_extra("x(0, 100: y = 42\n", Config::default());
            let result = parse(input);

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

            assert_eq!(cause.start().offset, 1);
            assert_eq!(cause.end().offset, 2);
        }

        #[test]
        fn discrete_limits_missing_values() {
            let input = InputSpan::new_extra("x[]: y = 42\n", Config::default());
            let result = parse(input);

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

            assert_eq!(cause.start().offset, 1);
            assert_eq!(cause.end().offset, 2);
        }

        #[test]
        fn discrete_limits_unclosed_bracket() {
            let input = InputSpan::new_extra("x[1, 2, 3: y = 42\n", Config::default());
            let result = parse(input);

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

            assert_eq!(cause.start().offset, 1);
            assert_eq!(cause.end().offset, 2);
        }

        #[test]
        fn piecewise_missing_expr() {
            let input = InputSpan::new_extra("x: y = { if z > 0\n", Config::default());
            let result = parse(input);

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

            assert_eq!(cause.start().offset, 7);
            assert_eq!(cause.end().offset, 8);
        }

        #[test]
        fn piecewise_missing_if() {
            let input = InputSpan::new_extra("x: y = {2*z z > 0\n", Config::default());
            let result = parse(input);

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

            assert_eq!(cause.start().offset, 8);
            assert_eq!(cause.end().offset, 11);
        }

        #[test]
        fn piecewise_missing_if_expr() {
            let input = InputSpan::new_extra("x: y = {2*z if\n", Config::default());
            let result = parse(input);

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

            assert_eq!(cause.start().offset, 12);
            assert_eq!(cause.end().offset, 14);
        }

        #[test]
        fn piecewise_missing_unit_after_colon() {
            let input = InputSpan::new_extra("x: y = {2*z if z > 0 :\n", Config::default());
            let result = parse(input);

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

            assert_eq!(cause.start().offset, 21);
            assert_eq!(cause.end().offset, 22);
        }

        #[test]
        fn invalid_expression() {
            let input = InputSpan::new_extra("x: y = @invalid\n", Config::default());
            let result = parse(input);

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

            assert_eq!(cause.start().offset, 5);
            assert_eq!(cause.end().offset, 6);
        }

        #[test]
        fn invalid_unit() {
            let input = InputSpan::new_extra("x: y = 42 : @invalid\n", Config::default());
            let result = parse(input);

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

            assert_eq!(cause.start().offset, 10);
            assert_eq!(cause.end().offset, 11);
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

            assert_eq!(cause.start().offset, 1);
            assert_eq!(cause.end().offset, 2);
        }

        #[test]
        fn piecewise_missing_newline_between_parts() {
            let input =
                InputSpan::new_extra("x: y = {2*z if z > 0 {0 if z <= 0\n", Config::default());
            let result = parse(input);

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

            assert_eq!(cause.start().offset, 7);
            assert_eq!(cause.end().offset, 20);
        }

        #[test]
        fn continuous_limits_with_extra_comma() {
            let input = InputSpan::new_extra("x(0, 100,): y = 42\n", Config::default());
            let result = parse(input);

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

            assert_eq!(cause.start().offset, 1);
            assert_eq!(cause.end().offset, 2);
        }

        #[test]
        fn discrete_limits_with_trailing_comma() {
            let input = InputSpan::new_extra("x[1, 2, 3,]: y = 42\n", Config::default());
            let result = parse(input);

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

            assert_eq!(cause.start().offset, 1);
            assert_eq!(cause.end().offset, 2);
        }

        #[test]
        fn piecewise_with_unit_on_wrong_line() {
            let input = InputSpan::new_extra(
                "x: y = {2*z if z > 0\n{0 if z <= 0 : m/s\n",
                Config::default(),
            );
            let result = parse(input);

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

            assert_eq!(cause.start().offset, 7);
            assert_eq!(cause.end().offset, 33);
        }
    }
}
