//! Parser for parameter declarations in an Oneil program.

use nom::Parser;
use nom::branch::alt;
use nom::combinator::{all_consuming, cut, opt};
use nom::multi::{many0, separated_list1};

use super::error::{ErrorHandlingParser, ParserError};
use super::expression::parse as parse_expr;
use super::note::parse as parse_note;
use super::token::{
    keyword::if_,
    naming::{identifier, label},
    structure::end_of_line,
    symbol::{
        brace_left, bracket_left, bracket_right, colon, comma, dollar, equals, paren_left,
        paren_right, star, star_star,
    },
};
use super::unit::parse as parse_unit;
use super::util::{Result, Span};
use crate::ast::parameter::{
    Limits, Parameter, ParameterValue, PiecewiseExpr, PiecewisePart, TraceLevel,
};

/// Parse a parameter declaration, e.g. `$ * x(0,100): y = 2*z : kg`.
///
/// This function **may not consume the complete input**.
///
/// # Examples
///
/// ```
/// use oneil::parser::parameter::parse;
/// use oneil::parser::{Config, Span};
///
/// let input = Span::new_extra("x: y = 42\n", Config::default());
/// let (rest, param) = parse(input).unwrap();
/// assert_eq!(rest.fragment(), &"");
/// ```
///
/// ```
/// use oneil::parser::parameter::parse;
/// use oneil::parser::{Config, Span};
///
/// let input = Span::new_extra("x: y = 42\nrest", Config::default());
/// let (rest, param) = parse(input).unwrap();
/// assert_eq!(rest.fragment(), &"rest");
/// ```
pub fn parse(input: Span) -> Result<Parameter, ParserError> {
    parameter_decl(input)
}

/// Parse a parameter declaration
///
/// This function **fails if the complete input is not consumed**.
///
/// # Examples
///
/// ```
/// use oneil::parser::parameter::parse_complete;
/// use oneil::parser::{Config, Span};
///
/// let input = Span::new_extra("x: y = 42\n", Config::default());
/// let (rest, param) = parse_complete(input).unwrap();
/// assert_eq!(rest.fragment(), &"");
/// ```
///
/// ```
/// use oneil::parser::parameter::parse_complete;
/// use oneil::parser::{Config, Span};
///
/// let input = Span::new_extra("x: y = 42\nrest", Config::default());
/// let result = parse_complete(input);
/// assert_eq!(result.is_err(), true);
/// ```
pub fn parse_complete(input: Span) -> Result<Parameter, ParserError> {
    all_consuming(parameter_decl).parse(input)
}

// TODO: the "preamble" (performance, trace level, name) needs to be optional,
//       but only in design files. Maybe we can use a config flag to indicate
//       whether we're parsing a design file or a model file and adjust from there.
//       For now, we'll just require the preamble in all cases.
fn parameter_decl(input: Span) -> Result<Parameter, ParserError> {
    let (rest, performance) = opt(performance).parse(input)?;

    let (rest, trace_level) = opt(trace_level).parse(rest)?;

    let (rest, name) = label.map_error(ParserError::expect_parameter).parse(rest)?;

    let (rest, limits) = opt(limits).parse(rest)?;

    let (rest, _) = colon.map_error(ParserError::expect_parameter).parse(rest)?;

    let (rest, ident) =
        cut(identifier.map_error(ParserError::parameter_missing_identifier)).parse(rest)?;

    let (rest, _) =
        cut(equals.map_error(ParserError::parameter_missing_equals_sign(ident))).parse(rest)?;

    let (rest, value) =
        cut(parameter_value.map_error(ParserError::parameter_missing_value(ident))).parse(rest)?;

    let (rest, _) = cut(end_of_line.map_error(ParserError::parameter_missing_end_of_line(ident)))
        .parse(rest)?;

    let (rest, note) = opt(parse_note).parse(rest)?;

    let param = Parameter {
        name: name.lexeme().to_string(),
        ident: ident.lexeme().to_string(),
        value,
        limits,
        is_performance: performance.is_some(),
        trace_level: trace_level.unwrap_or(TraceLevel::None),
        note,
    };

    Ok((rest, param))
}

/// Parse a performance indicator (`$`).
fn performance(input: Span) -> Result<bool, ParserError> {
    dollar.convert_errors().map(|_| true).parse(input)
}

/// Parse a trace level indicator (`*` or `**`).
fn trace_level(input: Span) -> Result<TraceLevel, ParserError> {
    let single_star = star.map(|_| TraceLevel::Trace);
    let double_star = star_star.map(|_| TraceLevel::Debug);

    double_star.or(single_star).convert_errors().parse(input)
}

/// Parse parameter limits (either continuous or discrete).
fn limits(input: Span) -> Result<Limits, ParserError> {
    alt((continuous_limits, discrete_limits)).parse(input)
}

/// Parse continuous limits in parentheses, e.g. `(0, 100)`.
fn continuous_limits(input: Span) -> Result<Limits, ParserError> {
    let (rest, paren_left_token) = paren_left.convert_errors().parse(input)?;

    // TODO: in all parsers, change all `map_failure` to `map_error` and put
    //       them inside the call to `cut` as shown here, so that other failures
    //       don't get overwritten
    let (rest, min) = cut(parse_expr.map_error(ParserError::limit_missing_min)).parse(rest)?;

    let (rest, _) = cut(comma.map_error(ParserError::limit_missing_comma)).parse(rest)?;

    let (rest, max) = cut(parse_expr.map_error(ParserError::limit_missing_max)).parse(rest)?;

    let (rest, _) =
        cut(paren_right.map_error(ParserError::unclosed_paren(paren_left_token))).parse(rest)?;

    Ok((rest, Limits::Continuous { min, max }))
}

/// Parse discrete limits in square brackets, e.g. `[1, 2, 3]`.
fn discrete_limits(input: Span) -> Result<Limits, ParserError> {
    let (rest, bracket_left_token) = bracket_left.convert_errors().parse(input)?;

    let (rest, values) = cut(separated_list1(comma.convert_errors(), parse_expr)
        .map_error(ParserError::limit_missing_values))
    .parse(rest)?;

    let (rest, _) = cut(bracket_right.map_error(ParserError::unclosed_bracket(bracket_left_token)))
        .parse(rest)?;

    Ok((rest, Limits::Discrete { values }))
}

/// Parse a parameter value (either simple or piecewise).
fn parameter_value(input: Span) -> Result<ParameterValue, ParserError> {
    simple_value.or(piecewise_value).parse(input)
}

/// Parse a simple parameter value (expression with optional unit).
fn simple_value(input: Span) -> Result<ParameterValue, ParserError> {
    let (rest, expr) = parse_expr.parse(input)?;

    let (rest, unit) = opt(|input| {
        let (rest, colon_token) = colon.convert_errors().parse(input)?;

        let (rest, unit) =
            cut(parse_unit.map_error(ParserError::parameter_missing_unit(colon_token)))
                .parse(rest)?;

        Ok((rest, unit))
    })
    .parse(rest)?;

    let value = ParameterValue::Simple(expr, unit);

    Ok((rest, value))
}

/// Parse a piecewise parameter value.
fn piecewise_value(input: Span) -> Result<ParameterValue, ParserError> {
    let (rest, first_part) = piecewise_part.parse(input)?;

    let (rest, unit) = opt(|input| {
        let (rest, colon_token) = colon.convert_errors().parse(input)?;

        let (rest, unit) =
            cut(parse_unit.map_error(ParserError::parameter_missing_unit(colon_token)))
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

    let mut parts = Vec::with_capacity(1 + rest_parts.len());
    parts.push(first_part);
    parts.extend(rest_parts);

    let value = ParameterValue::Piecewise(PiecewiseExpr { parts }, unit);

    Ok((rest, value))
}

/// Parse a single piece of a piecewise expression, e.g. `{2*x if x > 0`.
fn piecewise_part(input: Span) -> Result<PiecewisePart, ParserError> {
    let (rest, brace_left_token) = brace_left.convert_errors().parse(input)?;

    let (rest, expr) =
        cut(parse_expr.map_error(ParserError::piecewise_missing_expr(brace_left_token)))
            .parse(rest)?;

    let (rest, _) =
        cut(if_.map_error(ParserError::piecewise_missing_if(brace_left_token))).parse(rest)?;

    let (rest, if_expr) =
        cut(parse_expr.map_error(ParserError::piecewise_missing_if_expr(brace_left_token)))
            .parse(rest)?;

    let part = PiecewisePart { expr, if_expr };

    Ok((rest, part))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{
        expression::{BinaryOp, Expr, Literal},
        unit::{UnitExpr, UnitOp},
    };
    use crate::parser::Config;

    #[test]
    fn test_parse_simple_parameter() {
        let input = Span::new_extra("x: y = 42", Config::default());
        let (_, param) = parse(input).unwrap();
        assert_eq!(param.name, "x");
        assert_eq!(param.ident, "y");
        assert!(!param.is_performance);
        assert_eq!(param.trace_level, TraceLevel::None);

        match param.value {
            ParameterValue::Simple(expr, unit) => {
                assert_eq!(expr, Expr::Literal(Literal::Number(42.0)));
                assert!(unit.is_none());
            }
            _ => panic!("Expected simple parameter value"),
        }
    }

    #[test]
    fn test_parse_parameter_with_multiword_label() {
        let input = Span::new_extra("Value of x: x = 42", Config::default());
        let (_, param) = parse(input).expect("Parameter should parse");
        assert_eq!(param.name, "Value of x");
    }

    #[test]
    fn test_parse_parameter_with_continuous_limits() {
        let input = Span::new_extra("x(0, 100): y = 42", Config::default());
        let (_, param) = parse(input).unwrap();
        match param.limits {
            Some(Limits::Continuous { min, max }) => {
                assert_eq!(min, Expr::Literal(Literal::Number(0.0)));
                assert_eq!(max, Expr::Literal(Literal::Number(100.0)));
            }
            _ => panic!("Expected continuous limits"),
        }
    }

    #[test]
    fn test_parse_parameter_with_discrete_limits() {
        let input = Span::new_extra("x[1, 2, 3]: y = 42", Config::default());
        let (_, param) = parse(input).unwrap();
        match param.limits {
            Some(Limits::Discrete { values }) => {
                assert_eq!(values.len(), 3);
                assert_eq!(values[0], Expr::Literal(Literal::Number(1.0)));
                assert_eq!(values[1], Expr::Literal(Literal::Number(2.0)));
                assert_eq!(values[2], Expr::Literal(Literal::Number(3.0)));
            }
            _ => panic!("Expected discrete limits"),
        }
    }

    #[test]
    fn test_parse_parameter_with_performance() {
        let input = Span::new_extra("$ x: y = 42", Config::default());
        let (_, param) = parse(input).unwrap();
        assert!(param.is_performance);
    }

    #[test]
    fn test_parse_parameter_with_trace() {
        let input = Span::new_extra("* x: y = 42", Config::default());
        let (_, param) = parse(input).unwrap();
        assert_eq!(param.trace_level, TraceLevel::Trace);
    }

    #[test]
    fn test_parse_parameter_with_debug() {
        let input = Span::new_extra("** x: y = 42", Config::default());
        let (_, param) = parse(input).unwrap();
        assert_eq!(param.trace_level, TraceLevel::Debug);
    }

    #[test]
    fn test_parse_parameter_with_simple_units() {
        let input = Span::new_extra("x: y = 42 : kg", Config::default());
        let (_, param) = parse(input).unwrap();
        match param.value {
            ParameterValue::Simple(expr, unit) => {
                assert_eq!(expr, Expr::Literal(Literal::Number(42.0)));
                assert_eq!(
                    unit,
                    Some(UnitExpr::Unit {
                        identifier: "kg".to_string(),
                        exponent: None,
                    })
                );
            }
            _ => panic!("Expected simple parameter value"),
        }
    }

    #[test]
    fn test_parse_parameter_with_compound_units() {
        let input = Span::new_extra("x: y = 42 : m/s^2", Config::default());
        let (_, param) = parse(input).unwrap();
        match param.value {
            ParameterValue::Simple(expr, unit) => {
                assert_eq!(expr, Expr::Literal(Literal::Number(42.0)));
                assert!(matches!(
                    unit,
                    Some(UnitExpr::BinaryOp {
                        op: UnitOp::Divide,
                        left: _,
                        right: _,
                    })
                ));
            }
            _ => panic!("Expected simple parameter value"),
        }
    }

    #[test]
    fn test_parse_piecewise_parameter() {
        let input = Span::new_extra("x: y = {2*z if z > 0 \n {0 if z <= 0", Config::default());
        let (_, param) = parse(input).unwrap();
        match param.value {
            ParameterValue::Piecewise(piecewise, unit) => {
                assert_eq!(piecewise.parts.len(), 2);

                // First piece: 2*z if z > 0
                let first = &piecewise.parts[0];
                assert!(matches!(
                    first.expr,
                    Expr::BinaryOp {
                        op: BinaryOp::Mul,
                        left: _,
                        right: _,
                    }
                ));
                assert!(matches!(
                    first.if_expr,
                    Expr::BinaryOp {
                        op: BinaryOp::GreaterThan,
                        left: _,
                        right: _,
                    }
                ));

                // Second piece: 0 if z <= 0
                let second = &piecewise.parts[1];
                assert!(matches!(second.expr, Expr::Literal(Literal::Number(0.0))));
                assert!(matches!(
                    second.if_expr,
                    Expr::BinaryOp {
                        op: BinaryOp::LessThanEq,
                        left: _,
                        right: _,
                    }
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
        match param.value {
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

        assert!(param.is_performance);
        assert_eq!(param.trace_level, TraceLevel::Debug);
        assert_eq!(param.name, "x");
        assert_eq!(param.ident, "y");

        match param.limits {
            Some(Limits::Continuous { min, max }) => {
                assert_eq!(min, Expr::Literal(Literal::Number(0.0)));
                assert_eq!(max, Expr::Literal(Literal::Number(100.0)));
            }
            _ => panic!("Expected continuous limits"),
        }

        match &param.value {
            ParameterValue::Piecewise(piecewise, unit) => {
                assert_eq!(piecewise.parts.len(), 2);

                // Check unit
                assert!(matches!(
                    unit,
                    Some(UnitExpr::BinaryOp {
                        op: UnitOp::Divide,
                        left: _,
                        right: _,
                    })
                ));
            }
            _ => panic!("Expected piecewise parameter value"),
        }
    }

    #[test]
    fn test_parse_complete_success() {
        let input = Span::new_extra("x: y = 42\n", Config::default());
        let (rest, param) = parse_complete(input).unwrap();
        assert_eq!(param.name, "x");
        assert_eq!(param.ident, "y");
        assert!(!param.is_performance);
        assert_eq!(param.trace_level, TraceLevel::None);
        match param.value {
            ParameterValue::Simple(expr, unit) => {
                assert_eq!(expr, Expr::Literal(Literal::Number(42.0)));
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
