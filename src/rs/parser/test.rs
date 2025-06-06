//! Parser for test declarations in an Oneil program.

use nom::Parser;
use nom::combinator::{cut, opt};
use nom::multi::separated_list1;

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
pub fn parse(input: Span) -> Result<Test> {
    test_decl(input)
}

fn test_decl(input: Span) -> Result<Test> {
    (
        opt(trace_level),
        test_keyword,
        cut((opt(test_inputs), colon, parse_expr, end_of_line)),
    )
        .map(|(trace_level, _, (inputs, _, expr, _))| Test {
            trace_level: trace_level.unwrap_or(TraceLevel::None),
            inputs: inputs.unwrap_or_default(),
            expr,
        })
        .parse(input)
}

/// Parse a trace level indicator (`*` or `**`).
fn trace_level(input: Span) -> Result<TraceLevel> {
    let single_star = star.map(|_| TraceLevel::Trace);
    let double_star = star_star.map(|_| TraceLevel::Debug);

    double_star.or(single_star).parse(input)
}

/// Parse test inputs in curly braces, e.g. `{x, y, z}`.
fn test_inputs(input: Span) -> Result<Vec<String>> {
    (
        brace_left,
        cut((separated_list1(comma, identifier), brace_right)),
    )
        .map(|(_, (inputs, _))| inputs.into_iter().map(|id| id.to_string()).collect())
        .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::expression::{Expr, Literal};

    #[test]
    fn test_decl_basic() {
        let input = Span::new("test: true\n");
        let (rest, test) = test_decl(input).unwrap();
        assert_eq!(test.trace_level, TraceLevel::None);
        assert!(test.inputs.is_empty());
        assert_eq!(test.expr, Expr::Literal(Literal::Boolean(true)));
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_decl_at_eof() {
        let input = Span::new("test: true");
        let (_, test) = test_decl(input).unwrap();
        assert_eq!(test.trace_level, TraceLevel::None);
        assert!(test.inputs.is_empty());
        assert_eq!(test.expr, Expr::Literal(Literal::Boolean(true)));
    }

    #[test]
    fn test_decl_with_trace() {
        let input = Span::new("* test: true\n");
        let (rest, test) = test_decl(input).unwrap();
        assert_eq!(test.trace_level, TraceLevel::Trace);
        assert!(test.inputs.is_empty());
        assert_eq!(test.expr, Expr::Literal(Literal::Boolean(true)));
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_decl_with_debug() {
        let input = Span::new("** test: true\n");
        let (_, test) = test_decl(input).unwrap();
        assert_eq!(test.trace_level, TraceLevel::Debug);
        assert!(test.inputs.is_empty());
        assert_eq!(test.expr, Expr::Literal(Literal::Boolean(true)));
    }

    #[test]
    fn test_decl_with_inputs() {
        let input = Span::new("test {x, y}: x > y\n");
        let (rest, test) = test_decl(input).unwrap();
        assert_eq!(test.trace_level, TraceLevel::None);
        assert_eq!(test.inputs, vec!["x", "y"]);
        assert!(matches!(test.expr, Expr::BinaryOp { .. }));
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_decl_full() {
        let input = Span::new("** test {x, y, z}: x > y and y > z\n");
        let (rest, test) = test_decl(input).unwrap();
        assert_eq!(test.trace_level, TraceLevel::Debug);
        assert_eq!(test.inputs, vec!["x", "y", "z"]);
        assert!(matches!(test.expr, Expr::BinaryOp { .. }));
        assert_eq!(rest.fragment(), &"");
    }
}
