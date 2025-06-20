//! Parser for test declarations in an Oneil program.

use nom::Parser;
use nom::combinator::{all_consuming, cut, opt};
use nom::multi::separated_list1;

use super::error::{ErrorHandlingParser, ParserError};
use super::expression::parse as parse_expr;
use super::token::{
    keyword::test as test_keyword,
    naming::identifier,
    structure::end_of_line,
    symbol::{brace_left, brace_right, colon, comma, star, star_star},
};
use super::util::{Result, Span};
use crate::ast::{parameter::TraceLevel, test::Test};

/// Parse a test declaration, e.g. `* test {x, y}: x > y`.
///
/// This function **may not consume the complete input**.
///
/// # Examples
///
/// ```
/// use oneil::parser::test::parse;
/// use oneil::parser::{Config, Span};
///
/// let input = Span::new_extra("test: true\n", Config::default());
/// let (rest, test) = parse(input).unwrap();
/// assert_eq!(rest.fragment(), &"");
/// ```
///
/// ```
/// use oneil::parser::test::parse;
/// use oneil::parser::{Config, Span};
///
/// let input = Span::new_extra("test: true\nrest", Config::default());
/// let (rest, test) = parse(input).unwrap();
/// assert_eq!(rest.fragment(), &"rest");
/// ```
pub fn parse(input: Span) -> Result<Test, ParserError> {
    test_decl(input)
}

/// Parse a test declaration
///
/// This function **fails if the complete input is not consumed**.
///
/// # Examples
///
/// ```
/// use oneil::parser::test::parse_complete;
/// use oneil::parser::{Config, Span};
///
/// let input = Span::new_extra("test: true\n", Config::default());
/// let (rest, test) = parse_complete(input).unwrap();
/// assert_eq!(rest.fragment(), &"");
/// ```
///
/// ```
/// use oneil::parser::test::parse_complete;
/// use oneil::parser::{Config, Span};
///
/// let input = Span::new_extra("test: true\nrest", Config::default());
/// let result = parse_complete(input);
/// assert_eq!(result.is_err(), true);
/// ```
pub fn parse_complete(input: Span) -> Result<Test, ParserError> {
    all_consuming(test_decl).parse(input)
}

fn test_decl(input: Span) -> Result<Test, ParserError> {
    let (rest, trace_level) = opt(trace_level).parse(input)?;

    let (rest, test_keyword_token) = test_keyword
        .map_error(ParserError::expect_test)
        .parse(rest)?;

    let (rest, inputs) = opt(test_inputs).parse(rest)?;

    let (rest, _) =
        cut(colon.map_error(ParserError::test_missing_colon(test_keyword_token))).parse(rest)?;

    let (rest, expr) =
        cut(parse_expr.map_error(ParserError::test_missing_expr(test_keyword_token)))
            .parse(rest)?;

    let (rest, _) =
        cut(end_of_line.map_error(ParserError::test_missing_end_of_line(test_keyword_token)))
            .parse(rest)?;

    let test = Test {
        trace_level: trace_level.unwrap_or(TraceLevel::None),
        inputs: inputs.unwrap_or_default(),
        expr,
    };

    Ok((rest, test))
}

/// Parse a trace level indicator (`*` or `**`).
fn trace_level(input: Span) -> Result<TraceLevel, ParserError> {
    let single_star = star.map(|_| TraceLevel::Trace);
    let double_star = star_star.map(|_| TraceLevel::Debug);

    double_star.or(single_star).convert_errors().parse(input)
}

/// Parse test inputs in curly braces, e.g. `{x, y, z}`.
fn test_inputs(input: Span) -> Result<Vec<String>, ParserError> {
    let (rest, brace_left_token) = brace_left.convert_errors().parse(input)?;

    let (rest, inputs) = cut(separated_list1(comma, identifier)
        .map_error(ParserError::test_missing_inputs(brace_left_token)))
    .parse(rest)?;

    let (rest, _) =
        cut(brace_right.map_error(ParserError::unclosed_brace(brace_left_token))).parse(rest)?;

    let inputs = inputs
        .into_iter()
        .map(|id| id.lexeme().to_string())
        .collect();

    Ok((rest, inputs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::expression::{Expr, Literal};
    use crate::parser::Config;

    #[test]
    fn test_decl_basic() {
        let input = Span::new_extra("test: true\n", Config::default());
        let (rest, test) = parse(input).unwrap();
        assert_eq!(test.trace_level, TraceLevel::None);
        assert!(test.inputs.is_empty());
        assert_eq!(test.expr, Expr::Literal(Literal::Boolean(true)));
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_decl_at_eof() {
        let input = Span::new_extra("test: true", Config::default());
        let (_, test) = parse(input).unwrap();
        assert_eq!(test.trace_level, TraceLevel::None);
        assert!(test.inputs.is_empty());
        assert_eq!(test.expr, Expr::Literal(Literal::Boolean(true)));
    }

    #[test]
    fn test_decl_with_trace() {
        let input = Span::new_extra("* test: true\n", Config::default());
        let (rest, test) = parse(input).unwrap();
        assert_eq!(test.trace_level, TraceLevel::Trace);
        assert!(test.inputs.is_empty());
        assert_eq!(test.expr, Expr::Literal(Literal::Boolean(true)));
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_decl_with_debug() {
        let input = Span::new_extra("** test: true\n", Config::default());
        let (_, test) = parse(input).unwrap();
        assert_eq!(test.trace_level, TraceLevel::Debug);
        assert!(test.inputs.is_empty());
        assert_eq!(test.expr, Expr::Literal(Literal::Boolean(true)));
    }

    #[test]
    fn test_decl_with_inputs() {
        let input = Span::new_extra("test {x, y}: x > y\n", Config::default());
        let (rest, test) = parse(input).unwrap();
        assert_eq!(test.trace_level, TraceLevel::None);
        assert_eq!(test.inputs, vec!["x", "y"]);
        assert!(matches!(test.expr, Expr::BinaryOp { .. }));
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_decl_full() {
        let input = Span::new_extra("** test {x, y, z}: x > y and y > z\n", Config::default());
        let (rest, test) = parse(input).unwrap();
        assert_eq!(test.trace_level, TraceLevel::Debug);
        assert_eq!(test.inputs, vec!["x", "y", "z"]);
        assert!(matches!(test.expr, Expr::BinaryOp { .. }));
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_parse_complete_success() {
        let input = Span::new_extra("test: true\n", Config::default());
        let (rest, test) = parse_complete(input).unwrap();
        assert_eq!(test.trace_level, TraceLevel::None);
        assert!(test.inputs.is_empty());
        assert_eq!(test.expr, Expr::Literal(Literal::Boolean(true)));
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_parse_complete_with_remaining_input() {
        let input = Span::new_extra("test: true\nrest", Config::default());
        let result = parse_complete(input);
        assert!(result.is_err());
    }
}
