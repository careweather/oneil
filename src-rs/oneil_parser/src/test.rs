//! Parser for test declarations in an Oneil program.

use nom::Parser;
use nom::combinator::{all_consuming, opt};
use nom::multi::separated_list1;
use oneil_ast::Span as AstSpan;
use oneil_ast::debug_info::TraceLevelNode;
use oneil_ast::naming::Identifier;
use oneil_ast::node::Node;
use oneil_ast::test::{TestInputs, TestInputsNode, TestNode};

use crate::error::{ErrorHandlingParser, ParserError};
use crate::expression::parse as parse_expr;
use crate::token::{
    keyword::test as test_keyword,
    naming::identifier,
    structure::end_of_line,
    symbol::{brace_left, brace_right, colon, comma, star, star_star},
};
use crate::util::{Result, Span};
use oneil_ast::{debug_info::TraceLevel, test::Test};

/// Parse a test declaration, e.g. `* test {x, y}: x > y`.
///
/// This function **may not consume the complete input**.
pub fn parse(input: Span) -> Result<TestNode, ParserError> {
    test_decl(input)
}

/// Parse a test declaration
///
/// This function **fails if the complete input is not consumed**.
pub fn parse_complete(input: Span) -> Result<TestNode, ParserError> {
    all_consuming(test_decl).parse(input)
}

fn test_decl(input: Span) -> Result<TestNode, ParserError> {
    let (rest, trace_level) = opt(trace_level).parse(input)?;

    let (rest, test_keyword_token) = test_keyword
        .or_fail_with(ParserError::expect_test)
        .parse(rest)?;

    let (rest, inputs) = opt(test_inputs).parse(rest)?;

    // for error reporting
    let test_kw_or_inputs_span = inputs
        .as_ref()
        .map(AstSpan::from)
        .unwrap_or(AstSpan::from(&test_keyword_token));

    let (rest, colon_token) = colon
        .or_fail_with(ParserError::test_missing_colon(&test_kw_or_inputs_span))
        .parse(rest)?;

    let (rest, expr) = parse_expr
        .or_fail_with(ParserError::test_missing_expr(&colon_token))
        .parse(rest)?;

    let (rest, linebreak_token) = end_of_line
        .or_fail_with(ParserError::test_missing_end_of_line(&expr))
        .parse(rest)?;

    let span = match &trace_level {
        Some(trace_level) => {
            AstSpan::calc_span_with_whitespace(trace_level, &expr, &linebreak_token)
        }
        None => AstSpan::calc_span_with_whitespace(&test_keyword_token, &expr, &linebreak_token),
    };

    let test = Test::new(trace_level, inputs, expr);

    Ok((rest, Node::new(span, test)))
}

/// Parse a trace level indicator (`*` or `**`).
fn trace_level(input: Span) -> Result<TraceLevelNode, ParserError> {
    let single_star = star.map(|token| Node::new(token, TraceLevel::Trace));
    let double_star = star_star.map(|token| Node::new(token, TraceLevel::Debug));

    double_star.or(single_star).convert_errors().parse(input)
}

/// Parse test inputs in curly braces, e.g. `{x, y, z}`.
fn test_inputs(input: Span) -> Result<TestInputsNode, ParserError> {
    let (rest, brace_left_token) = brace_left.convert_errors().parse(input)?;

    let (rest, inputs) = separated_list1(comma, identifier)
        .or_fail_with(ParserError::test_missing_inputs(&brace_left_token))
        .parse(rest)?;

    let (rest, brace_right_token) = brace_right
        .or_fail_with(ParserError::unclosed_brace(&brace_left_token))
        .parse(rest)?;

    let inputs = inputs
        .into_iter()
        .map(|id| Node::new(id, Identifier::new(id.lexeme().to_string())))
        .collect();

    let span = AstSpan::calc_span(&brace_left_token, &brace_right_token);

    Ok((rest, Node::new(span, TestInputs::new(inputs))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;
    use oneil_ast::expression::{Expr, Literal};

    #[test]
    fn test_decl_basic() {
        let input = Span::new_extra("test: true\n", Config::default());
        let (rest, test) = parse(input).unwrap();

        let expected_span = AstSpan::new(0, 10, 11);

        let expected_test_expr = Node::new(AstSpan::new(6, 10, 10), Literal::boolean(true));
        let expected_test_expr =
            Node::new(AstSpan::new(6, 10, 10), Expr::literal(expected_test_expr));

        assert_eq!(test.node_span(), &expected_span);
        assert_eq!(test.trace_level(), None);
        assert_eq!(test.inputs(), None);
        assert_eq!(test.expr(), &expected_test_expr);

        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_decl_at_eof() {
        let input = Span::new_extra("test: true", Config::default());
        let (rest, test) = parse(input).unwrap();

        let expected_span = AstSpan::new(0, 10, 10);

        let expected_test_expr = Node::new(AstSpan::new(6, 10, 10), Literal::boolean(true));
        let expected_test_expr =
            Node::new(AstSpan::new(6, 10, 10), Expr::literal(expected_test_expr));

        assert_eq!(test.node_span(), &expected_span);
        assert_eq!(test.trace_level(), None);
        assert_eq!(test.inputs(), None);
        assert_eq!(test.expr(), &expected_test_expr);

        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_decl_with_trace() {
        let input = Span::new_extra("* test: true\n", Config::default());
        let (rest, test) = parse(input).unwrap();

        let expected_span = AstSpan::new(0, 12, 13);

        assert_eq!(test.node_span(), &expected_span);
        assert_eq!(
            test.trace_level().map(|t| t.node_value()),
            Some(&TraceLevel::Trace)
        );

        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_decl_with_debug() {
        let input = Span::new_extra("** test: true\n", Config::default());
        let (rest, test) = parse(input).unwrap();

        let expected_span = AstSpan::new(0, 13, 14);

        assert_eq!(test.node_span(), &expected_span);
        assert_eq!(
            test.trace_level().map(|t| t.node_value()),
            Some(&TraceLevel::Debug)
        );
        assert_eq!(test.inputs(), None);

        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_decl_with_inputs() {
        let input = Span::new_extra("test {x, y}: x > y\n", Config::default());
        let (rest, test) = parse(input).unwrap();

        let expected_span = AstSpan::new(0, 18, 19);

        let expected_test_inputs = vec![
            Node::new(AstSpan::new(6, 7, 7), Identifier::new("x".to_string())),
            Node::new(AstSpan::new(9, 10, 10), Identifier::new("y".to_string())),
        ];

        assert_eq!(test.node_span(), &expected_span);
        assert_eq!(test.trace_level(), None);
        assert_eq!(
            test.inputs().map(|i| i.node_value()),
            Some(&TestInputs::new(expected_test_inputs))
        );

        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_decl_full() {
        let input = Span::new_extra("** test {x, y, z}: x > y and y > z\n", Config::default());
        let (rest, test) = parse(input).unwrap();

        let expected_span = AstSpan::new(0, 34, 35);

        let expected_test_inputs = vec![
            Node::new(AstSpan::new(9, 10, 10), Identifier::new("x".to_string())),
            Node::new(AstSpan::new(12, 13, 13), Identifier::new("y".to_string())),
            Node::new(AstSpan::new(15, 16, 16), Identifier::new("z".to_string())),
        ];

        assert_eq!(test.node_span(), &expected_span);
        assert_eq!(
            test.trace_level().map(|t| t.node_value()),
            Some(&TraceLevel::Debug)
        );
        assert_eq!(
            test.inputs().map(|i| i.node_value()),
            Some(&TestInputs::new(expected_test_inputs))
        );

        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_parse_complete_success() {
        let input = Span::new_extra("test: true\n", Config::default());
        let (rest, test) = parse_complete(input).unwrap();

        let expected_span = AstSpan::new(0, 10, 11);

        let expected_test_expr = Node::new(AstSpan::new(6, 10, 10), Literal::boolean(true));
        let expected_test_expr =
            Node::new(AstSpan::new(6, 10, 10), Expr::literal(expected_test_expr));

        assert_eq!(test.node_span(), &expected_span);
        assert_eq!(test.trace_level(), None);
        assert_eq!(test.inputs(), None);
        assert_eq!(test.expr(), &expected_test_expr);

        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_parse_complete_with_remaining_input() {
        let input = Span::new_extra("test: true\nrest", Config::default());
        let result = parse_complete(input);
        assert!(result.is_err());
    }
}
