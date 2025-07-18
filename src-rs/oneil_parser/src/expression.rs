//! Expression parsing for the Oneil language.
//!
//! These expressions can be used in a variety of contexts, including:
//! - Parameter values
//! - Parameter limits
//! - Piecewise conditions
//! - Tests
//! - Model inputs

use nom::{
    Parser as _,
    branch::alt,
    combinator::{all_consuming, map, opt},
    multi::{many0, separated_list0},
};

use oneil_ast::{
    Span as AstSpan,
    expression::{BinaryOp, BinaryOpNode, Expr, ExprNode, Literal, UnaryOp, Variable},
    naming::Identifier,
    node::Node,
};

use crate::{
    error::{ErrorHandlingParser, ParserError},
    token::{
        keyword::{and, false_, not, or, true_},
        literal::{number, string},
        naming::identifier,
        symbol::{
            bang_equals, bar, caret, comma, dot, equals_equals, greater_than, greater_than_equals,
            less_than, less_than_equals, minus, minus_minus, paren_left, paren_right, percent,
            plus, slash, slash_slash, star,
        },
    },
    util::{Parser, Result, Span},
};

/// Creates a left-associative binary operator parser.
///
/// This function constructs a parser that handles left-associative binary operations
/// like addition, multiplication, etc. It parses a sequence of operands separated
/// by operators and builds the AST with proper left associativity.
///
/// For example, `a + b + c` is parsed as `(a + b) + c` rather than `a + (b + c)`.
///
/// # Arguments
///
/// * `operand` - Parser for the operands (e.g., expressions of higher precedence)
/// * `operator` - Parser for the binary operators
///
/// # Returns
///
/// A parser that handles left-associative binary operations with proper error handling.
fn left_associative_binary_op<'a>(
    mut operand: impl Parser<'a, ExprNode, ParserError> + Copy,
    mut operator: impl Parser<'a, BinaryOpNode, ParserError>,
) -> impl Parser<'a, ExprNode, ParserError> {
    move |input| {
        let (rest, first_operand) = operand.parse(input)?;
        let (rest, rest_operands) = many0(|input| {
            let (rest, operator) = operator.parse(input)?;
            let (rest, operand) = operand
                .or_fail_with(ParserError::expr_binary_op_missing_second_operand(
                    &operator,
                ))
                .parse(rest)?;
            Ok((rest, (operator, operand)))
        })
        .parse(rest)?;

        let expr = rest_operands
            .into_iter()
            .fold(first_operand, |acc, (op, expr)| {
                let left = acc;
                let right = expr;
                let span = AstSpan::calc_span(&left, &right);
                Node::new(span, Expr::binary_op(op, left, right))
            });

        Ok((rest, expr))
    }
}

/// Parses an expression
///
/// This function **may not consume the complete input**.
pub fn parse(input: Span) -> Result<ExprNode, ParserError> {
    expr(input)
}

/// Parses an expression
///
/// This function **fails if the complete input is not consumed**.
pub fn parse_complete(input: Span) -> Result<ExprNode, ParserError> {
    all_consuming(expr).parse(input)
}

/// Parses an expression with proper operator precedence.
///
/// This function is the entry point for expression parsing and delegates
/// to the highest precedence level (OR expressions). The precedence hierarchy
/// from lowest to highest is:
///
/// 1. OR (`or`)
/// 2. AND (`and`)
/// 3. NOT (`not`)
/// 4. Comparison (`==`, `!=`, `<`, `<=`, `>`, `>=`)
/// 5. Min/Max (`|`)
/// 6. Addition/Subtraction (`+`, `-`, `--`)
/// 7. Multiplication/Division (`*`, `/`, `//`, `%`)
/// 8. Exponentiation (`^`)
/// 9. Negation (`-`)
/// 10. Primary expressions (literals, variables, function calls, parentheses)
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns an expression node with proper operator precedence.
fn expr(input: Span) -> Result<ExprNode, ParserError> {
    or_expr
        .convert_error_to(ParserError::expect_expr)
        .parse(input)
}

/// Parses an OR expression (lowest precedence)
///
/// OR expressions have the lowest precedence in the expression hierarchy.
/// They are left-associative, meaning `a or b or c` is parsed as `(a or b) or c`.
///
/// Examples:
/// - `true or false`
/// - `x > 0 or y < 10`
/// - `a or b or c or d`
///
/// The parser uses the `left_associative_binary_op` helper to ensure proper
/// left associativity and error handling.
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns an expression node representing the OR expression with proper associativity.
fn or_expr(input: Span) -> Result<ExprNode, ParserError> {
    let or = or
        .map(|token| Node::new(token, BinaryOp::Or))
        .convert_errors();
    left_associative_binary_op(and_expr, or).parse(input)
}

/// Parses an AND expression
///
/// AND expressions have higher precedence than OR expressions but lower than NOT expressions.
/// They are left-associative, meaning `a and b and c` is parsed as `(a and b) and c`.
///
/// Examples:
/// - `true and false`
/// - `x > 0 and y < 10`
/// - `a and b and c and d`
///
/// The parser uses the `left_associative_binary_op` helper to ensure proper
/// left associativity and error handling.
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns an expression node representing the AND expression with proper associativity.
fn and_expr(input: Span) -> Result<ExprNode, ParserError> {
    let and = and
        .map(|token| Node::new(token, BinaryOp::And))
        .convert_errors();
    left_associative_binary_op(not_expr, and).parse(input)
}

/// Parses a NOT expression
///
/// NOT expressions have higher precedence than AND expressions but lower than comparison expressions.
/// The NOT operator is a unary operator that negates its operand.
///
/// Examples:
/// - `not true`
/// - `not (x > 0)`
/// - `not not false` (double negation)
///
/// The parser handles both:
/// 1. NOT expressions: `not` followed by an expression
/// 2. Comparison expressions: expressions without NOT
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns an expression node representing either a NOT expression or a comparison expression.
fn not_expr(input: Span) -> Result<ExprNode, ParserError> {
    alt((
        |input| {
            let (rest, not_op) = not
                .map(|token| Node::new(token, UnaryOp::Not))
                .convert_errors()
                .parse(input)?;

            let (rest, expr) = not_expr
                .or_fail_with(ParserError::unary_op_missing_operand(&not_op))
                .parse(rest)?;

            let span = AstSpan::calc_span(&not_op, &expr);

            Ok((rest, Node::new(span, Expr::unary_op(not_op, expr))))
        },
        comparison_expr,
    ))
    .parse(input)
}

/// Parses a comparison expression
///
/// Comparison expressions have higher precedence than NOT expressions but lower than min/max expressions.
/// They support the standard comparison operators: `==`, `!=`, `<`, `<=`, `>`, `>=`.
///
/// Examples:
/// - `x == y`
/// - `a < b`
/// - `value >= 10`
/// - `name != 'John'`
/// - `42` (single operand, no comparison)
///
/// The parser handles both:
/// 1. Binary comparisons: `operand operator operand`
/// 2. Single operands: just an expression without comparison
///
/// Supported operators (in order of precedence):
/// - `<=` (less than or equal)
/// - `>=` (greater than or equal)
/// - `<` (less than)
/// - `>` (greater than)
/// - `==` (equal)
/// - `!=` (not equal)
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns an expression node representing either a comparison expression or a single operand.
fn comparison_expr(input: Span) -> Result<ExprNode, ParserError> {
    let mut op = alt((
        less_than_equals.map(|token| Node::new(token, BinaryOp::LessThanEq)),
        greater_than_equals.map(|token| Node::new(token, BinaryOp::GreaterThanEq)),
        less_than.map(|token| Node::new(token, BinaryOp::LessThan)),
        greater_than.map(|token| Node::new(token, BinaryOp::GreaterThan)),
        equals_equals.map(|token| Node::new(token, BinaryOp::Eq)),
        bang_equals.map(|token| Node::new(token, BinaryOp::NotEq)),
    ))
    .convert_errors();

    let (rest, first_operand) = minmax_expr.parse(input)?;
    let (rest, second_operand) = opt(|input| {
        let (rest, operator) = op.parse(input)?;
        let (rest, operand) = minmax_expr
            .or_fail_with(ParserError::expr_binary_op_missing_second_operand(
                &operator,
            ))
            .parse(rest)?;
        Ok((rest, (operator, operand)))
    })
    .parse(rest)?;

    let expr = match second_operand {
        Some((op, second_operand)) => {
            let left = first_operand;
            let right = second_operand;
            let span = AstSpan::calc_span(&left, &right);
            Node::new(span, Expr::binary_op(op, left, right))
        }
        None => first_operand,
    };

    Ok((rest, expr))
}

/// Parses a min/max expression
///
/// Min/max expressions have higher precedence than comparison expressions but lower than additive expressions.
/// They use the `|` operator to represent min/max operations between two values.
///
/// Examples:
/// - `min_weight | max_weight`
/// - `x | y`
///
/// The parser handles both:
/// 1. Binary min/max: `operand | operand`
/// 2. Single operands: just an expression without min/max
///
/// The `|` operator is used to represent min/max operations, where the result
/// depends on the context and the specific implementation of the min/max function.
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns an expression node representing either a min/max expression or a single operand.
fn minmax_expr(input: Span) -> Result<ExprNode, ParserError> {
    let (rest, first_operand) = additive_expr.parse(input)?;
    let (rest, second_operand) = opt(|input| {
        let (rest, operator) = bar
            .map(|token| Node::new(token, BinaryOp::MinMax))
            .convert_errors()
            .parse(input)?;

        let (rest, operand) = additive_expr
            .or_fail_with(ParserError::expr_binary_op_missing_second_operand(
                &operator,
            ))
            .parse(rest)?;

        Ok((rest, (operator, operand)))
    })
    .parse(rest)?;

    let expr = match second_operand {
        Some((op, second_operand)) => {
            let left = first_operand;
            let right = second_operand;
            let span = AstSpan::calc_span(&left, &right);
            Node::new(span, Expr::binary_op(op, left, right))
        }
        None => first_operand,
    };

    Ok((rest, expr))
}

/// Parses an additive expression
///
/// Additive expressions have higher precedence than min/max expressions but lower than multiplicative expressions.
/// They support addition (`+`), subtraction (`-`), and true subtraction (`--`).
///
/// Examples:
/// - `a + b`
/// - `x - y`
/// - `a -- b` (true subtraction)
/// - `1 + 2 + 3` (chained addition)
/// - `10 - 5 - 2` (chained subtraction)
///
/// The parser uses the `left_associative_binary_op` helper to ensure proper
/// left associativity and error handling.
///
/// Supported operators:
/// - `+` (addition)
/// - `-` (subtraction)
/// - `--` (true subtraction, different from regular subtraction)
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns an expression node representing the additive expression with proper associativity.
fn additive_expr(input: Span) -> Result<ExprNode, ParserError> {
    let op = alt((
        plus.map(|token| Node::new(token, BinaryOp::Add)),
        minus.map(|token| Node::new(token, BinaryOp::Sub)),
        minus_minus.map(|token| Node::new(token, BinaryOp::TrueSub)),
    ))
    .convert_errors();

    left_associative_binary_op(multiplicative_expr, op).parse(input)
}

/// Parses a multiplicative expression
///
/// Multiplicative expressions have higher precedence than additive expressions but lower than exponential expressions.
/// They support multiplication (`*`), division (`/`), true division (`//`), and modulo (`%`).
///
/// Examples:
/// - `a * b`
/// - `x / y`
/// - `a // b` (true division)
/// - `x % y` (modulo)
/// - `2 * 3 * 4` (chained multiplication)
/// - `10 / 2 / 5` (chained division)
///
/// The parser uses the `left_associative_binary_op` helper to ensure proper
/// left associativity and error handling.
///
/// Supported operators:
/// - `*` (multiplication)
/// - `/` (division)
/// - `//` (true division, different from regular division)
/// - `%` (modulo/remainder)
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns an expression node representing the multiplicative expression with proper associativity.
fn multiplicative_expr(input: Span) -> Result<ExprNode, ParserError> {
    let op = alt((
        star.map(|token| Node::new(token, BinaryOp::Mul)),
        slash.map(|token| Node::new(token, BinaryOp::Div)),
        slash_slash.map(|token| Node::new(token, BinaryOp::TrueDiv)),
        percent.map(|token| Node::new(token, BinaryOp::Mod)),
    ))
    .convert_errors();

    left_associative_binary_op(exponential_expr, op).parse(input)
}

/// Parses an exponential expression (right associative)
///
/// Exponential expressions have higher precedence than multiplicative expressions but lower than negation expressions.
/// They use the `^` operator and are right-associative, meaning `a^b^c` is parsed as `a^(b^c)`.
///
/// Examples:
/// - `a ^ b`
/// - `2 ^ 3`
/// - `x ^ y ^ z` (right associative: `x^(y^z)`)
/// - `(a + b) ^ 2`
///
/// The parser handles both:
/// 1. Binary exponentials: `operand ^ operand`
/// 2. Single operands: just an expression without exponentiation
///
/// Right associativity means that `2^3^2` is evaluated as `2^(3^2) = 2^9 = 512`,
/// not as `(2^3)^2 = 8^2 = 64`.
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns an expression node representing either an exponential expression or a single operand.
fn exponential_expr(input: Span) -> Result<ExprNode, ParserError> {
    let mut op = caret
        .map(|token| Node::new(token, BinaryOp::Pow))
        .convert_errors();

    let (rest, first_operand) = neg_expr.parse(input)?;
    let (rest, second_operand) = opt(|input| {
        let (rest, operator) = op.parse(input)?;
        let (rest, operand) = exponential_expr
            .or_fail_with(ParserError::expr_binary_op_missing_second_operand(
                &operator,
            ))
            .parse(rest)?;
        Ok((rest, (operator, operand)))
    })
    .parse(rest)?;

    let expr = match second_operand {
        Some((op, second_operand)) => {
            let left = first_operand;
            let right = second_operand;
            let span = AstSpan::calc_span(&left, &right);
            Node::new(span, Expr::binary_op(op, left, right))
        }
        None => first_operand,
    };

    Ok((rest, expr))
}

/// Parses a negation expression
///
/// Negation expressions have higher precedence than exponential expressions but lower than primary expressions.
/// The negation operator (`-`) is a unary operator that negates its operand.
///
/// Examples:
/// - `-42`
/// - `-x`
/// - `-(a + b)`
/// - `--5` (double negation)
///
/// The parser handles both:
/// 1. Negation expressions: `-` followed by an expression
/// 2. Primary expressions: expressions without negation
///
/// The negation operator can be applied to any expression, including
/// literals, variables, function calls, and parenthesized expressions.
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns an expression node representing either a negation expression or a primary expression.
fn neg_expr(input: Span) -> Result<ExprNode, ParserError> {
    alt((
        |input| {
            let (rest, minus_op) = minus
                .map(|token| Node::new(token, UnaryOp::Neg))
                .convert_errors()
                .parse(input)?;

            let (rest, expr) = neg_expr
                .or_fail_with(ParserError::unary_op_missing_operand(&minus_op))
                .parse(rest)?;

            let span = AstSpan::calc_span(&minus_op, &expr);

            Ok((rest, Node::new(span, Expr::unary_op(minus_op, expr))))
        },
        primary_expr,
    ))
    .parse(input)
}

/// Parses a primary expression (literals, identifiers, function calls, parenthesized expressions)
///
/// Primary expressions have the highest precedence in the expression hierarchy.
/// They represent the basic building blocks of expressions.
///
/// Examples:
/// - `42` (numeric literal)
/// - `'hello'` (string literal)
/// - `true`, `false` (boolean literals)
/// - `foo` (variable/identifier)
/// - `foo.bar.baz` (multi-word identifier)
/// - `func(1, 2, 3)` (function call)
/// - `(a + b)` (parenthesized expression)
///
/// The parser handles the following types of primary expressions:
/// 1. Numeric literals (integers and floating-point numbers)
/// 2. String literals (enclosed in single quotes)
/// 3. Boolean literals (`true` and `false`)
/// 4. Function calls (`name(arg1, arg2, ...)`)
/// 5. Variables (simple identifiers or dot-separated paths)
/// 6. Parenthesized expressions (`(expression)`)
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns an expression node representing the parsed primary expression.
fn primary_expr(input: Span) -> Result<ExprNode, ParserError> {
    alt((
        map(number.convert_errors(), |n| {
            let parse_result = n.lexeme().parse::<f64>();
            let parse_result = parse_result.expect("all valid numbers should parse correctly");

            let node = Node::new(n, Literal::number(parse_result));
            let node = Node::new(n, Expr::literal(node));
            node
        }),
        map(string.convert_errors(), |s| {
            // trim quotes from the string
            let s_contents = s.lexeme()[1..s.lexeme().len() - 1].to_string();
            let node = Node::new(s, Literal::string(s_contents));
            let node = Node::new(s, Expr::literal(node));
            node
        }),
        map(true_.convert_errors(), |t| {
            let node = Node::new(t, Literal::boolean(true));
            let node = Node::new(t, Expr::literal(node));
            node
        }),
        map(false_.convert_errors(), |t| {
            let node = Node::new(t, Literal::boolean(false));
            let node = Node::new(t, Expr::literal(node));
            node
        }),
        function_call,
        variable,
        parenthesized_expr,
    ))
    .parse(input)
}

/// Parses a function call
///
/// Function calls have the format `name(arg1, arg2, ...)` where `name` is an identifier
/// and the arguments are comma-separated expressions.
///
/// Examples:
/// - `foo()`
/// - `bar(42)`
/// - `func(x, y, z)`
/// - `calculate(a + b, c * d)`
/// - `nested(foo(1), bar(2))`
///
/// The parser requires:
/// - A valid identifier as the function name
/// - Opening parenthesis `(`
/// - Zero or more comma-separated expressions as arguments
/// - Closing parenthesis `)`
///
/// Function calls can be nested, and arguments can be any valid expression
/// including literals, variables, other function calls, and complex expressions.
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns an expression node representing the function call, or an error if
/// the function call is malformed (e.g., unclosed parentheses).
fn function_call(input: Span) -> Result<ExprNode, ParserError> {
    let (rest, name) = identifier.convert_errors().parse(input)?;
    let name_span = AstSpan::from(&name);
    let name = Node::new(name_span, Identifier::new(name.lexeme().to_string()));

    let (rest, paren_left_span) = paren_left.convert_errors().parse(rest)?;
    let (rest, args) = separated_list0(comma.convert_errors(), expr).parse(rest)?;
    let (rest, paren_right_span) = paren_right
        .or_fail_with(ParserError::unclosed_paren(&paren_left_span))
        .parse(rest)?;

    let span = AstSpan::calc_span(&name, &paren_right_span);

    Ok((rest, Node::new(span, Expr::function_call(name, args))))
}

/// Parses a variable name
///
/// Variables can be simple identifiers or dot-separated paths representing
/// nested object properties or module members.
///
/// Examples:
/// - `foo` (simple variable)
/// - `foo.bar` (nested property)
/// - `foo.bar.baz` (deeply nested)
/// - `module.function.parameter`
/// - `config.database.host`
///
/// The parser handles:
/// 1. Simple identifiers: single variable names
/// 2. Dot-separated paths: multiple identifiers connected by dots
///
/// Each component in a dot-separated path must be a valid identifier.
/// The parser builds a nested structure where each dot represents
/// an accessor operation on the previous component.
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns an expression node representing the variable, which can be either
/// a simple identifier or a nested accessor structure.
fn variable(input: Span) -> Result<ExprNode, ParserError> {
    let (rest, first_id) = identifier.convert_errors().parse(input)?;
    let first_id_span = AstSpan::from(&first_id);
    let first_id = Node::new(
        first_id_span,
        Identifier::new(first_id.lexeme().to_string()),
    );

    let (rest, rest_ids) = many0(|input| {
        let (rest, dot_token) = dot.convert_errors().parse(input)?;
        let (rest, id) = identifier
            .or_fail_with(ParserError::expr_variable_missing_parent(&dot_token))
            .parse(rest)?;
        let id_span = AstSpan::from(&id);
        let id = Node::new(id_span, Identifier::new(id.lexeme().to_string()));

        Ok((rest, id))
    })
    .parse(rest)?;

    let first_id = Node::new(first_id_span, Variable::identifier(first_id));

    let variable = rest_ids.into_iter().fold(first_id, |acc, id| {
        let start = &acc;
        let end = &id;
        let span = AstSpan::calc_span(start, end);

        Node::new(span, Variable::accessor(id, acc))
    });

    let span = AstSpan::from(&variable);
    let expr = Node::new(span, Expr::variable(variable));

    Ok((rest, expr))
}

/// Parses a parenthesized expression
///
/// Parenthesized expressions allow grouping of expressions to control precedence
/// and associativity. They have the format `(expression)`.
///
/// Examples:
/// - `(42)`
/// - `(a + b)`
/// - `(x * y + z)`
/// - `((a + b) * c)`
/// - `(func(1, 2))`
///
/// The parser requires:
/// - Opening parenthesis `(`
/// - A valid expression (any expression type)
/// - Closing parenthesis `)`
///
/// Parenthesized expressions are wrapped in a special node type to preserve
/// the span information that includes the parentheses. This is important for
/// accurate error reporting and source location tracking.
///
/// Note: The parser ensures that parentheses are properly matched and will
/// fail with an error if the closing parenthesis is missing.
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns an expression node representing the parenthesized expression, or an error if
/// the parentheses are malformed (e.g., unclosed parentheses or missing expression).
fn parenthesized_expr(input: Span) -> Result<ExprNode, ParserError> {
    let (rest, paren_left_span) = paren_left.convert_errors().parse(input)?;

    let (rest, expr) = expr
        .or_fail_with(ParserError::expr_paren_missing_expression(&paren_left_span))
        .parse(rest)?;

    let (rest, paren_right_span) = paren_right
        .or_fail_with(ParserError::unclosed_paren(&paren_left_span))
        .parse(rest)?;

    // note: we need to wrap the expr in a parenthesized node in order to keep the spans accurate
    //       otherwise, calculating spans using the parenthesized node as a start or end span
    //       will result in the calculated span ignoring the parens
    let span = AstSpan::calc_span(&paren_left_span, &paren_right_span);
    let expr = Node::new(span, Expr::parenthesized(expr));

    Ok((rest, expr))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;

    #[test]
    fn test_primary_expr_number() {
        let input = Span::new_extra("42", Config::default());
        let (_, expr) = parse(input).unwrap();

        let expected_expr = Node::new(
            AstSpan::new(0, 2, 2),
            Expr::literal(Node::new(AstSpan::new(0, 2, 2), Literal::number(42.0))),
        );

        assert_eq!(expr, expected_expr);
    }

    #[test]
    fn test_primary_expr_string() {
        let input = Span::new_extra("'hello'", Config::default());
        let (_, expr) = parse(input).unwrap();

        let expected_expr = Node::new(
            AstSpan::new(0, 7, 7),
            Expr::literal(Node::new(
                AstSpan::new(0, 7, 7),
                Literal::string("hello".to_string()),
            )),
        );

        assert_eq!(expr, expected_expr);
    }

    #[test]
    fn test_primary_expr_boolean_true() {
        let input = Span::new_extra("true", Config::default());
        let (_, expr) = parse(input).unwrap();

        let expected_expr = Node::new(
            AstSpan::new(0, 4, 4),
            Expr::literal(Node::new(AstSpan::new(0, 4, 4), Literal::boolean(true))),
        );

        assert_eq!(expr, expected_expr);
    }

    #[test]
    fn test_primary_expr_boolean_false() {
        let input = Span::new_extra("false", Config::default());
        let (_, expr) = parse(input).unwrap();

        let expected_expr = Node::new(
            AstSpan::new(0, 5, 5),
            Expr::literal(Node::new(AstSpan::new(0, 5, 5), Literal::boolean(false))),
        );

        assert_eq!(expr, expected_expr);
    }

    #[test]
    fn test_primary_expr_simple_identifier() {
        let input = Span::new_extra("foo", Config::default());
        let (_, expr) = parse(input).unwrap();

        let expected_id = Node::new(AstSpan::new(0, 3, 3), Identifier::new("foo".to_string()));

        let expected_expr = Node::new(
            AstSpan::new(0, 3, 3),
            Expr::variable(Node::new(
                AstSpan::new(0, 3, 3),
                Variable::identifier(expected_id),
            )),
        );

        assert_eq!(expr, expected_expr);
    }

    #[test]
    fn test_primary_expr_multiword_identifier() {
        let input = Span::new_extra("foo.bar.baz", Config::default());
        let (_, expr) = parse(input).unwrap();

        let expected_id = Node::new(AstSpan::new(0, 3, 3), Identifier::new("foo".to_string()));
        let expected_id2 = Node::new(AstSpan::new(4, 7, 7), Identifier::new("bar".to_string()));
        let expected_id3 = Node::new(AstSpan::new(8, 11, 11), Identifier::new("baz".to_string()));

        let variable = Node::new(AstSpan::new(0, 3, 3), Variable::identifier(expected_id));
        let variable = Node::new(
            AstSpan::new(0, 7, 7),
            Variable::accessor(expected_id2, variable),
        );
        let variable = Node::new(
            AstSpan::new(0, 11, 11),
            Variable::accessor(expected_id3, variable),
        );

        let expected_expr = Node::new(AstSpan::new(0, 11, 11), Expr::variable(variable));
        assert_eq!(expr, expected_expr);
    }

    #[test]
    fn test_function_call() {
        let input = Span::new_extra("foo(1, 2)", Config::default());
        let (_, expr) = parse(input).unwrap();

        let expected_foo = Node::new(AstSpan::new(0, 3, 3), Identifier::new("foo".to_string()));

        let expected_1 = Node::new(AstSpan::new(4, 5, 5), Literal::number(1.0));
        let expected_1 = Node::new(AstSpan::new(4, 5, 5), Expr::literal(expected_1));

        let expected_2 = Node::new(AstSpan::new(7, 8, 8), Literal::number(2.0));
        let expected_2 = Node::new(AstSpan::new(7, 8, 8), Expr::literal(expected_2));

        let expected_expr = Node::new(
            AstSpan::new(0, 9, 9),
            Expr::function_call(expected_foo, vec![expected_1, expected_2]),
        );

        assert_eq!(expr, expected_expr);
    }

    #[test]
    fn test_neg_expr() {
        let input = Span::new_extra("-42", Config::default());
        let (_, expr) = parse(input).unwrap();

        let expected_42 = Node::new(AstSpan::new(1, 3, 3), Literal::number(42.0));
        let expected_42 = Node::new(AstSpan::new(1, 3, 3), Expr::literal(expected_42));

        let op = Node::new(AstSpan::new(0, 1, 1), UnaryOp::Neg);

        let expected_expr = Node::new(AstSpan::new(0, 3, 3), Expr::unary_op(op, expected_42));

        assert_eq!(expr, expected_expr);
    }

    #[test]
    fn test_exponential_expr() {
        let input = Span::new_extra("2^3", Config::default());
        let (_, expr) = parse(input).unwrap();

        let expected_2 = Node::new(AstSpan::new(0, 1, 1), Literal::number(2.0));
        let expected_2 = Node::new(AstSpan::new(0, 1, 1), Expr::literal(expected_2));

        let expected_3 = Node::new(AstSpan::new(2, 3, 3), Literal::number(3.0));
        let expected_3 = Node::new(AstSpan::new(2, 3, 3), Expr::literal(expected_3));

        let op = Node::new(AstSpan::new(1, 2, 2), BinaryOp::Pow);

        let expected_expr = Node::new(
            AstSpan::new(0, 3, 3),
            Expr::binary_op(op, expected_2, expected_3),
        );

        assert_eq!(expr, expected_expr);
    }

    #[test]
    fn test_multiplicative_expr() {
        let input = Span::new_extra("2*3", Config::default());
        let (_, expr) = parse(input).unwrap();

        let expected_2 = Node::new(AstSpan::new(0, 1, 1), Literal::number(2.0));
        let expected_2 = Node::new(AstSpan::new(0, 1, 1), Expr::literal(expected_2));

        let expected_3 = Node::new(AstSpan::new(2, 3, 3), Literal::number(3.0));
        let expected_3 = Node::new(AstSpan::new(2, 3, 3), Expr::literal(expected_3));

        let op = Node::new(AstSpan::new(1, 2, 2), BinaryOp::Mul);

        let expected_expr = Node::new(
            AstSpan::new(0, 3, 3),
            Expr::binary_op(op, expected_2, expected_3),
        );

        assert_eq!(expr, expected_expr);
    }

    #[test]
    fn test_additive_expr() {
        let input = Span::new_extra("2+3", Config::default());
        let (_, expr) = parse(input).unwrap();

        let expected_2 = Node::new(AstSpan::new(0, 1, 1), Literal::number(2.0));
        let expected_2 = Node::new(AstSpan::new(0, 1, 1), Expr::literal(expected_2));

        let expected_3 = Node::new(AstSpan::new(2, 3, 3), Literal::number(3.0));
        let expected_3 = Node::new(AstSpan::new(2, 3, 3), Expr::literal(expected_3));

        let op = Node::new(AstSpan::new(1, 2, 2), BinaryOp::Add);

        let expected_expr = Node::new(
            AstSpan::new(0, 3, 3),
            Expr::binary_op(op, expected_2, expected_3),
        );

        assert_eq!(expr, expected_expr);
    }

    #[test]
    fn test_minmax_expr() {
        let input = Span::new_extra("min_weight | max_weight", Config::default());
        let (_, expr) = parse(input).unwrap();

        let expected_min = Node::new(
            AstSpan::new(0, 10, 11),
            Identifier::new("min_weight".to_string()),
        );
        let expected_min = Node::new(AstSpan::new(0, 10, 11), Variable::identifier(expected_min));
        let expected_min = Node::new(AstSpan::new(0, 10, 11), Expr::variable(expected_min));

        let expected_max = Node::new(
            AstSpan::new(13, 23, 23),
            Identifier::new("max_weight".to_string()),
        );
        let expected_max = Node::new(AstSpan::new(13, 23, 23), Variable::identifier(expected_max));
        let expected_max = Node::new(AstSpan::new(13, 23, 23), Expr::variable(expected_max));

        let op = Node::new(AstSpan::new(11, 12, 13), BinaryOp::MinMax);

        let expected_expr = Node::new(
            AstSpan::new(0, 23, 23),
            Expr::binary_op(op, expected_min, expected_max),
        );

        assert_eq!(expr, expected_expr);
    }

    #[test]
    fn test_comparison_expr() {
        let input = Span::new_extra("2<3", Config::default());
        let (_, expr) = parse(input).unwrap();

        let expected_2 = Node::new(AstSpan::new(0, 1, 1), Literal::number(2.0));
        let expected_2 = Node::new(AstSpan::new(0, 1, 1), Expr::literal(expected_2));

        let expected_3 = Node::new(AstSpan::new(2, 3, 3), Literal::number(3.0));
        let expected_3 = Node::new(AstSpan::new(2, 3, 3), Expr::literal(expected_3));

        let op = Node::new(AstSpan::new(1, 2, 2), BinaryOp::LessThan);

        let expected_expr = Node::new(
            AstSpan::new(0, 3, 3),
            Expr::binary_op(op, expected_2, expected_3),
        );

        assert_eq!(expr, expected_expr);
    }

    #[test]
    fn test_not_expr() {
        let input = Span::new_extra("not true", Config::default());
        let (_, expr) = parse(input).unwrap();

        let expected_true = Node::new(AstSpan::new(4, 8, 8), Literal::boolean(true));
        let expected_true = Node::new(AstSpan::new(4, 8, 8), Expr::literal(expected_true));

        let op = Node::new(AstSpan::new(0, 3, 4), UnaryOp::Not);

        let expected_expr = Node::new(AstSpan::new(0, 8, 8), Expr::unary_op(op, expected_true));

        assert_eq!(expr, expected_expr);
    }

    #[test]
    fn test_and_expr() {
        let input = Span::new_extra("true and false", Config::default());
        let (_, expr) = parse(input).unwrap();

        let expected_true = Node::new(AstSpan::new(0, 4, 5), Literal::boolean(true));
        let expected_true = Node::new(AstSpan::new(0, 4, 5), Expr::literal(expected_true));

        let expected_false = Node::new(AstSpan::new(9, 14, 14), Literal::boolean(false));
        let expected_false = Node::new(AstSpan::new(9, 14, 14), Expr::literal(expected_false));

        let op = Node::new(AstSpan::new(5, 8, 9), BinaryOp::And);

        let expected_expr = Node::new(
            AstSpan::new(0, 14, 14),
            Expr::binary_op(op, expected_true, expected_false),
        );

        assert_eq!(expr, expected_expr);
    }

    #[test]
    fn test_or_expr() {
        let input = Span::new_extra("true or false", Config::default());
        let (_, expr) = parse(input).unwrap();

        let expected_true = Node::new(AstSpan::new(0, 4, 5), Literal::boolean(true));
        let expected_true = Node::new(AstSpan::new(0, 4, 5), Expr::literal(expected_true));

        let expected_false = Node::new(AstSpan::new(8, 13, 13), Literal::boolean(false));
        let expected_false = Node::new(AstSpan::new(8, 13, 13), Expr::literal(expected_false));

        let op = Node::new(AstSpan::new(5, 7, 8), BinaryOp::Or);

        let expected_expr = Node::new(
            AstSpan::new(0, 13, 13),
            Expr::binary_op(op, expected_true, expected_false),
        );

        assert_eq!(expr, expected_expr);
    }

    #[test]
    fn test_complex_expr() {
        let input = Span::new_extra("-(2 + 3*4^2) < foo(5, 6) and not bar", Config::default());
        let (_, expr) = parse(input).unwrap();
        // The exact structure is complex but we just verify it parses
        assert!(matches!(expr.node_value(), Expr::BinaryOp { .. }));
    }

    #[test]
    fn test_parse_complete_success() {
        let input = Span::new_extra("42", Config::default());
        let (rest, expr) = parse_complete(input).unwrap();

        let expected_42 = Node::new(AstSpan::new(0, 2, 2), Literal::number(42.0));
        let expected_42 = Node::new(AstSpan::new(0, 2, 2), Expr::literal(expected_42));

        assert_eq!(expr, expected_42);
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_parse_complete_with_remaining_input() {
        let input = Span::new_extra("42 rest", Config::default());
        let result = parse_complete(input);
        assert!(result.is_err());
    }

    mod error_tests {
        use super::*;
        use crate::error::reason::{ExpectKind, ExprKind, IncompleteKind, ParserErrorReason};

        mod general_error_tests {
            use super::*;

            #[test]
            fn test_empty_input() {
                let input = Span::new_extra("", Config::default());
                let result = parse(input);
                match result {
                    Err(nom::Err::Error(error)) => {
                        assert_eq!(error.error_offset, 0);
                        assert!(matches!(
                            error.reason,
                            ParserErrorReason::Expect(ExpectKind::Expr)
                        ));
                    }
                    _ => panic!("Expected error for empty input"),
                }
            }

            #[test]
            fn test_whitespace_only() {
                let input = Span::new_extra("   ", Config::default());
                let result = parse(input);
                match result {
                    Err(nom::Err::Error(error)) => {
                        assert_eq!(error.error_offset, 0);
                        assert!(matches!(
                            error.reason,
                            ParserErrorReason::Expect(ExpectKind::Expr)
                        ));
                    }
                    _ => panic!("Expected error for whitespace only"),
                }
            }

            #[test]
            fn test_symbols_only() {
                let input = Span::new_extra("+++", Config::default());
                let result = parse(input);
                match result {
                    Err(nom::Err::Error(error)) => {
                        assert_eq!(error.error_offset, 0);
                        assert!(matches!(
                            error.reason,
                            ParserErrorReason::Expect(ExpectKind::Expr)
                        ));
                    }
                    _ => panic!("Expected error for symbols only"),
                }
            }

            #[test]
            fn test_parse_complete_with_remaining_input() {
                let input = Span::new_extra("42 + 1 rest", Config::default());
                let result = parse_complete(input);
                match result {
                    Err(nom::Err::Error(error)) => {
                        assert_eq!(error.error_offset, 7);
                        assert_eq!(error.reason, ParserErrorReason::UnexpectedToken);
                    }
                    _ => panic!("Expected error for remaining input"),
                }
            }
        }

        mod unary_op_error_tests {
            use super::*;

            #[test]
            fn test_negation_missing_operand() {
                let input = Span::new_extra("-", Config::default());
                let result = parse(input);
                let expected_minus_span = AstSpan::new(0, 1, 1);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 1);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind:
                                    IncompleteKind::Expr(ExprKind::UnaryOpMissingOperand { operator }),
                                cause,
                            } => {
                                assert_eq!(operator, UnaryOp::Neg);
                                assert_eq!(cause, expected_minus_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }

            #[test]
            fn test_not_missing_operand() {
                let input = Span::new_extra("not", Config::default());
                let result = parse(input);
                let expected_not_span = AstSpan::new(0, 3, 3);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 3);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind:
                                    IncompleteKind::Expr(ExprKind::UnaryOpMissingOperand { operator }),
                                cause,
                            } => {
                                assert_eq!(operator, UnaryOp::Not);
                                assert_eq!(cause, expected_not_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }
        }

        mod binary_op_error_tests {
            use super::*;

            #[test]
            fn test_addition_missing_second_operand() {
                let input = Span::new_extra("1 +", Config::default());
                let result = parse(input);
                let expected_plus_span = AstSpan::new(2, 3, 3);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 3);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind:
                                    IncompleteKind::Expr(ExprKind::BinaryOpMissingSecondOperand {
                                        operator,
                                    }),
                                cause,
                            } => {
                                assert_eq!(operator, BinaryOp::Add);
                                assert_eq!(cause, expected_plus_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }

            #[test]
            fn test_multiplication_missing_second_operand() {
                let input = Span::new_extra("2 *", Config::default());
                let result = parse(input);
                let expected_star_span = AstSpan::new(2, 3, 3);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 3);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind:
                                    IncompleteKind::Expr(ExprKind::BinaryOpMissingSecondOperand {
                                        operator,
                                    }),
                                cause,
                            } => {
                                assert_eq!(operator, BinaryOp::Mul);
                                assert_eq!(cause, expected_star_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }

            #[test]
            fn test_exponentiation_missing_second_operand() {
                let input = Span::new_extra("2 ^", Config::default());
                let result = parse(input);
                let expected_caret_span = AstSpan::new(2, 3, 3);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 3);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind:
                                    IncompleteKind::Expr(ExprKind::BinaryOpMissingSecondOperand {
                                        operator,
                                    }),
                                cause,
                            } => {
                                assert_eq!(operator, BinaryOp::Pow);
                                assert_eq!(cause, expected_caret_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }

            #[test]
            fn test_comparison_missing_second_operand() {
                let input = Span::new_extra("x <", Config::default());
                let result = parse(input);
                let expected_less_span = AstSpan::new(2, 3, 3);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 3);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind:
                                    IncompleteKind::Expr(ExprKind::BinaryOpMissingSecondOperand {
                                        operator,
                                    }),
                                cause,
                            } => {
                                assert_eq!(operator, BinaryOp::LessThan);
                                assert_eq!(cause, expected_less_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }

            #[test]
            fn test_logical_and_missing_second_operand() {
                let input = Span::new_extra("true and", Config::default());
                let result = parse(input);
                let expected_and_span = AstSpan::new(5, 8, 8);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 8);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind:
                                    IncompleteKind::Expr(ExprKind::BinaryOpMissingSecondOperand {
                                        operator,
                                    }),
                                cause,
                            } => {
                                assert_eq!(operator, BinaryOp::And);
                                assert_eq!(cause, expected_and_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }

            #[test]
            fn test_logical_or_missing_second_operand() {
                let input = Span::new_extra("false or", Config::default());
                let result = parse(input);
                let expected_or_span = AstSpan::new(6, 8, 8);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 8);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind:
                                    IncompleteKind::Expr(ExprKind::BinaryOpMissingSecondOperand {
                                        operator,
                                    }),
                                cause,
                            } => {
                                assert_eq!(operator, BinaryOp::Or);
                                assert_eq!(cause, expected_or_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }

            #[test]
            fn test_minmax_missing_second_operand() {
                let input = Span::new_extra("min_weight |", Config::default());
                let result = parse(input);
                let expected_bar_span = AstSpan::new(11, 12, 12);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 12);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind:
                                    IncompleteKind::Expr(ExprKind::BinaryOpMissingSecondOperand {
                                        operator,
                                    }),
                                cause,
                            } => {
                                assert_eq!(operator, BinaryOp::MinMax);
                                assert_eq!(cause, expected_bar_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }
        }

        mod parenthesized_expr_error_tests {
            use super::*;

            #[test]
            fn test_missing_expression_in_parentheses() {
                let input = Span::new_extra("()", Config::default());
                let result = parse(input);
                let expected_paren_left_span = AstSpan::new(0, 1, 1);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 1);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::Expr(ExprKind::ParenMissingExpr),
                                cause,
                            } => {
                                assert_eq!(cause, expected_paren_left_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }

            #[test]
            fn test_unclosed_parentheses() {
                let input = Span::new_extra("(1 + 2", Config::default());
                let result = parse(input);
                let expected_paren_left_span = AstSpan::new(0, 1, 1);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 6);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::UnclosedParen,
                                cause,
                            } => {
                                assert_eq!(cause, expected_paren_left_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }

            #[test]
            fn test_nested_unclosed_parentheses() {
                let input = Span::new_extra("((1 + 2)", Config::default());
                let result = parse(input);
                let expected_paren_left_span = AstSpan::new(0, 1, 1);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 8);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::UnclosedParen,
                                cause,
                            } => {
                                assert_eq!(cause, expected_paren_left_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }
        }

        mod function_call_error_tests {
            use super::*;

            #[test]
            fn test_missing_opening_paren() {
                let input = Span::new_extra("foo", Config::default());
                let result = parse(input);
                // This should succeed as it's a valid variable
                assert!(result.is_ok());
            }

            #[test]
            fn test_missing_closing_paren() {
                let input = Span::new_extra("foo(1, 2", Config::default());
                let result = parse(input);
                let expected_paren_left_span = AstSpan::new(3, 4, 4);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 8);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::UnclosedParen,
                                cause,
                            } => {
                                assert_eq!(cause, expected_paren_left_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }

            #[test]
            fn test_empty_function_call() {
                let input = Span::new_extra("foo()", Config::default());
                let result = parse(input);
                // This should succeed as it's a valid function call with no arguments
                assert!(result.is_ok());
            }

            #[test]
            fn test_missing_argument_after_comma() {
                let input = Span::new_extra("foo(1,)", Config::default());
                let result = parse(input);
                let expected_paren_left_span = AstSpan::new(3, 4, 4);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 5);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::UnclosedParen,
                                cause,
                            } => {
                                assert_eq!(cause, expected_paren_left_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }
        }

        mod variable_error_tests {
            use super::*;

            #[test]
            fn test_missing_identifier_after_dot() {
                let input = Span::new_extra("foo.", Config::default());
                let result = parse(input);
                let expected_dot_span = AstSpan::new(3, 4, 4);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 4);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::Expr(ExprKind::VariableMissingParentModel),
                                cause,
                            } => {
                                assert_eq!(cause, expected_dot_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }

            #[test]
            fn test_consecutive_dots() {
                let input = Span::new_extra("foo..bar", Config::default());
                let result = parse(input);
                let expected_dot_span = AstSpan::new(3, 4, 4);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 4);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::Expr(ExprKind::VariableMissingParentModel),
                                cause,
                            } => {
                                assert_eq!(cause, expected_dot_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }

            #[test]
            fn test_dot_at_end() {
                let input = Span::new_extra("foo.bar.", Config::default());
                let result = parse(input);
                let expected_dot_span = AstSpan::new(7, 8, 8);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 8);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::Expr(ExprKind::VariableMissingParentModel),
                                cause,
                            } => {
                                assert_eq!(cause, expected_dot_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }
        }

        mod literal_error_tests {
            use super::*;

            #[test]
            fn test_unterminated_string() {
                let input = Span::new_extra("'hello", Config::default());
                let result = parse(input);
                // This should be a token error for unterminated string
                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 6);
                        assert!(matches!(error.reason, ParserErrorReason::TokenError(_)));
                    }
                    _ => panic!(
                        "Expected token error for unterminated string, got {:?}",
                        result
                    ),
                }
            }

            #[test]
            fn test_invalid_number() {
                let input = Span::new_extra("@", Config::default());
                let result = parse(input);
                // This should be an Expect(Expr) error since @ is not a valid expression start
                match result {
                    Err(nom::Err::Error(error)) => {
                        assert_eq!(error.error_offset, 0);
                        assert!(matches!(
                            error.reason,
                            ParserErrorReason::Expect(ExpectKind::Expr)
                        ));
                    }
                    _ => panic!(
                        "Expected Expect(Expr) error for invalid expression start, got {:?}",
                        result
                    ),
                }
            }
        }

        mod precedence_error_tests {
            use super::*;

            #[test]
            fn test_chained_binary_ops_missing_operand() {
                let input = Span::new_extra("1 + 2 *", Config::default());
                let result = parse(input);
                let expected_star_span = AstSpan::new(6, 7, 7);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 7);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind:
                                    IncompleteKind::Expr(ExprKind::BinaryOpMissingSecondOperand {
                                        operator,
                                    }),
                                cause,
                            } => {
                                assert_eq!(operator, BinaryOp::Mul);
                                assert_eq!(cause, expected_star_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }

            #[test]
            fn test_complex_expression_missing_operand() {
                let input = Span::new_extra("(1 + 2) * 3 +", Config::default());
                let result = parse(input);
                let expected_plus_span = AstSpan::new(12, 13, 13);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 13);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind:
                                    IncompleteKind::Expr(ExprKind::BinaryOpMissingSecondOperand {
                                        operator,
                                    }),
                                cause,
                            } => {
                                assert_eq!(operator, BinaryOp::Add);
                                assert_eq!(cause, expected_plus_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }
        }
    }
}
