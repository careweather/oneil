//! Parser for parameter declarations in an Oneil program.

use nom::Parser;
use nom::branch::alt;
use nom::combinator::{cut, opt};
use nom::multi::{many0, separated_list1};

use super::expression::parse as parse_expr;
use super::note::parse as parse_note;
use super::token::{
    keyword::if_,
    naming::identifier,
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
pub fn parse(input: Span) -> Result<Parameter> {
    parameter_decl(input)
}

fn parameter_decl(input: Span) -> Result<Parameter> {
    (
        opt(performance),
        opt(trace_level),
        identifier,
        opt(limits),
        cut((
            colon,
            identifier,
            equals,
            parameter_value,
            end_of_line,
            opt(parse_note),
        )),
    )
        .map(
            |(performance, trace_level, name, limits, (_, ident, _, value, _, note))| Parameter {
                name: name.to_string(),
                ident: ident.to_string(),
                value,
                limits,
                is_performance: performance.is_some(),
                trace_level: trace_level.unwrap_or(TraceLevel::None),
                note,
            },
        )
        .parse(input)
}

/// Parse a performance indicator (`$`).
fn performance(input: Span) -> Result<bool> {
    dollar.map(|_| true).parse(input)
}

/// Parse a trace level indicator (`*` or `**`).
fn trace_level(input: Span) -> Result<TraceLevel> {
    let single_star = star.map(|_| TraceLevel::Trace);
    let double_star = star_star.map(|_| TraceLevel::Debug);

    double_star.or(single_star).parse(input)
}

/// Parse parameter limits (either continuous or discrete).
fn limits(input: Span) -> Result<Limits> {
    alt((continuous_limits, discrete_limits)).parse(input)
}

/// Parse continuous limits in parentheses, e.g. `(0, 100)`.
fn continuous_limits(input: Span) -> Result<Limits> {
    (
        paren_left,
        cut((parse_expr, comma, parse_expr, paren_right)),
    )
        .map(|(_, (min, _, max, _))| Limits::Continuous { min, max })
        .parse(input)
}

/// Parse discrete limits in square brackets, e.g. `[1, 2, 3]`.
fn discrete_limits(input: Span) -> Result<Limits> {
    (
        bracket_left,
        cut((separated_list1(comma, parse_expr), bracket_right)),
    )
        .map(|(_, (values, _))| Limits::Discrete { values })
        .parse(input)
}

/// Parse a parameter value (either simple or piecewise).
fn parameter_value(input: Span) -> Result<ParameterValue> {
    simple_value.or(piecewise_value).parse(input)
}

/// Parse a simple parameter value (expression with optional unit).
fn simple_value(input: Span) -> Result<ParameterValue> {
    (parse_expr, opt((colon, cut(parse_unit))))
        .map(|(expr, unit)| ParameterValue::Simple(expr, unit.map(|(_, u)| u)))
        .parse(input)
}

/// Parse a piecewise parameter value.
fn piecewise_value(input: Span) -> Result<ParameterValue> {
    (
        piecewise_part,
        opt((colon, cut(parse_unit))),
        many0((end_of_line, piecewise_part)),
    )
        .map(|(first, unit, rest)| {
            let mut parts = Vec::with_capacity(1 + rest.len());
            parts.push(first);
            parts.extend(rest.into_iter().map(|(_, part)| part));
            ParameterValue::Piecewise(PiecewiseExpr { parts }, unit.map(|(_, u)| u))
        })
        .parse(input)
}

/// Parse a single piece of a piecewise expression, e.g. `{2*x if x > 0}`.
fn piecewise_part(input: Span) -> Result<PiecewisePart> {
    (brace_left, cut((parse_expr, if_, parse_expr)))
        .map(|(_, (expr, _, if_expr))| PiecewisePart { expr, if_expr })
        .parse(input)
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
        let (_, param) = parameter_decl(input).unwrap();
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
    fn test_parse_parameter_with_continuous_limits() {
        let input = Span::new_extra("x(0, 100): y = 42", Config::default());
        let (_, param) = parameter_decl(input).unwrap();
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
        let (_, param) = parameter_decl(input).unwrap();
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
        let (_, param) = parameter_decl(input).unwrap();
        assert!(param.is_performance);
    }

    #[test]
    fn test_parse_parameter_with_trace() {
        let input = Span::new_extra("* x: y = 42", Config::default());
        let (_, param) = parameter_decl(input).unwrap();
        assert_eq!(param.trace_level, TraceLevel::Trace);
    }

    #[test]
    fn test_parse_parameter_with_debug() {
        let input = Span::new_extra("** x: y = 42", Config::default());
        let (_, param) = parameter_decl(input).unwrap();
        assert_eq!(param.trace_level, TraceLevel::Debug);
    }

    #[test]
    fn test_parse_parameter_with_simple_units() {
        let input = Span::new_extra("x: y = 42 : kg", Config::default());
        let (_, param) = parameter_decl(input).unwrap();
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
        // Test compound unit
        let input = Span::new_extra("x: y = 42 : m/s^2", Config::default());
        let (_, param) = parameter_decl(input).unwrap();
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
        let (rest, param) = parameter_decl(input).unwrap();
        println!("{:?}", rest);
        println!("{:?}", param);
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
        let (_, param) = parameter_decl(input).unwrap();
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
        let (_, param) = parameter_decl(input).unwrap();

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
}
