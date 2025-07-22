use crate::util::get_span_from_ast_span;
use oneil_ast as ast;

/// Resolves an AST unit expression into a composite unit representation.
///
/// This function takes a parsed unit expression from the AST and converts it
/// into a `CompositeUnit` that represents the same unit in a flattened form.
/// The resolution process handles multiplication, division, and exponents
/// by recursively traversing the expression tree.
///
/// # Algorithm
///
/// The resolution process works by:
/// 1. Recursively traversing the unit expression tree
/// 2. For multiplication operations: process both operands with the same inverse flag
/// 3. For division operations: process the left operand normally, right operand with inverted flag
/// 4. For unit leaves: create a Unit with the identifier and exponent (negated if inverse)
/// 5. Collect all units into a flat list that represents the composite unit
///
/// # Arguments
///
/// * `unit` - The AST unit expression to resolve
///
/// # Returns
///
/// A `CompositeUnit` containing the flattened representation of the unit expression
pub fn resolve_unit(unit: &ast::unit::UnitExprNode) -> oneil_ir::unit::CompositeUnit {
    let units = resolve_unit_recursive(unit, false, Vec::new());
    oneil_ir::unit::CompositeUnit::new(units)
}

fn resolve_unit_recursive(
    unit: &ast::unit::UnitExprNode,
    is_inverse: bool,
    mut units: Vec<oneil_ir::unit::Unit>,
) -> Vec<oneil_ir::unit::Unit> {
    match unit.node_value() {
        ast::UnitExpr::BinaryOp { op, left, right } => {
            let units = resolve_unit_recursive(left, is_inverse, units);

            let units = match op.node_value() {
                ast::unit::UnitOp::Multiply => resolve_unit_recursive(right, is_inverse, units),
                ast::unit::UnitOp::Divide => resolve_unit_recursive(right, !is_inverse, units),
            };
            units
        }
        ast::UnitExpr::Unit {
            identifier,
            exponent,
        } => {
            let exponent_value = exponent
                .as_ref()
                .map(|e| e.node_value().value())
                .unwrap_or(1.0);
            let exponent_value = if is_inverse {
                -exponent_value
            } else {
                exponent_value
            };

            let name_span = get_span_from_ast_span(identifier.node_span());
            let exponent_span = match &exponent {
                Some(exp) => get_span_from_ast_span(exp.node_span()),
                None => oneil_ir::span::Span::new(identifier.node_span().end(), 0),
            };
            let unit = oneil_ir::unit::Unit::new(
                identifier.as_str().to_string(),
                name_span,
                exponent_value,
                exponent_span,
            );
            units.push(unit);
            units
        }
        ast::UnitExpr::Parenthesized { expr } => resolve_unit_recursive(expr, is_inverse, units),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oneil_ast::unit::{UnitExpr, UnitOp};

    /// Helper function to create a test span
    fn test_span(start: usize, end: usize) -> ast::Span {
        ast::Span::new(start, end - start, 0)
    }

    /// Helper function to create an identifier node
    fn create_identifier_node(name: &str, start: usize) -> ast::naming::IdentifierNode {
        let identifier = ast::naming::Identifier::new(name.to_string());
        ast::node::Node::new(test_span(start, start + name.len()), identifier)
    }

    /// Helper function to create a unit exponent node
    fn create_unit_exponent_node(
        value: f64,
        start: usize,
        end: usize,
    ) -> ast::unit::UnitExponentNode {
        let exponent = ast::unit::UnitExponent::new(value);
        ast::node::Node::new(test_span(start, end), exponent)
    }

    /// Helper function to create a simple unit expression node
    fn create_unit_node(
        name: &str,
        exponent: Option<f64>,
        start: usize,
        end: usize,
    ) -> ast::unit::UnitExprNode {
        let identifier_node = create_identifier_node(name, start);
        let exponent_node = exponent.map(|exp| create_unit_exponent_node(exp, start, end));

        let unit_expr = UnitExpr::Unit {
            identifier: identifier_node,
            exponent: exponent_node,
        };
        ast::node::Node::new(test_span(start, end), unit_expr)
    }

    /// Helper function to create a binary operation node
    fn create_binary_op_node(
        op: UnitOp,
        left: ast::unit::UnitExprNode,
        right: ast::unit::UnitExprNode,
        start: usize,
        end: usize,
    ) -> ast::unit::UnitExprNode {
        let op_node = ast::node::Node::new(test_span(start, end), op);
        let unit_expr = UnitExpr::BinaryOp {
            op: op_node,
            left: Box::new(left),
            right: Box::new(right),
        };
        ast::node::Node::new(test_span(start, end), unit_expr)
    }

    /// Helper function to create a parenthesized expression node
    fn create_parenthesized_node(
        expr: ast::unit::UnitExprNode,
        start: usize,
        end: usize,
    ) -> ast::unit::UnitExprNode {
        let unit_expr = UnitExpr::Parenthesized {
            expr: Box::new(expr),
        };
        ast::node::Node::new(test_span(start, end), unit_expr)
    }

    /// Helper function to check if a unit with the given name and exponent exists
    fn assert_unit_exists(units: &[oneil_ir::unit::Unit], name: &str, exponent: f64) {
        let found = units
            .iter()
            .any(|u| u.name() == name && u.exponent() == exponent);
        assert!(
            found,
            "Expected unit '{}' with exponent {} not found. Available units: {:?}",
            name,
            exponent,
            units
                .iter()
                .map(|u| format!("{}^{}", u.name(), u.exponent()))
                .collect::<Vec<_>>()
        );
    }

    /// Helper function to assert that exactly the expected units exist
    fn assert_units_match(units: &[oneil_ir::unit::Unit], expected: &[(&str, f64)]) {
        assert_eq!(
            units.len(),
            expected.len(),
            "Expected {} units, got {}. Available units: {:?}",
            expected.len(),
            units.len(),
            units
                .iter()
                .map(|u| format!("{}^{}", u.name(), u.exponent()))
                .collect::<Vec<_>>()
        );

        for (name, exponent) in expected {
            assert_unit_exists(units, name, *exponent);
        }
    }

    #[test]
    fn test_simple_unit() {
        let unit_expr = create_unit_node("m", Some(1.0), 0, 1);
        let composite = resolve_unit(&unit_expr);

        assert_units_match(composite.units(), &[("m", 1.0)]);
    }

    #[test]
    fn test_unit_with_default_exponent() {
        let unit_expr = create_unit_node("kg", None, 0, 2);
        let composite = resolve_unit(&unit_expr);

        assert_units_match(composite.units(), &[("kg", 1.0)]);
    }

    #[test]
    fn test_unit_with_custom_exponent() {
        let unit_expr = create_unit_node("m", Some(2.0), 0, 1);
        let composite = resolve_unit(&unit_expr);

        assert_units_match(composite.units(), &[("m", 2.0)]);
    }

    #[test]
    fn test_multiplication() {
        // m * kg
        let unit_expr = create_binary_op_node(
            UnitOp::Multiply,
            create_unit_node("m", Some(1.0), 0, 1),
            create_unit_node("kg", Some(1.0), 4, 6),
            0,
            6,
        );
        let composite = resolve_unit(&unit_expr);

        assert_units_match(composite.units(), &[("m", 1.0), ("kg", 1.0)]);
    }

    #[test]
    fn test_division() {
        // m / s
        let unit_expr = create_binary_op_node(
            UnitOp::Divide,
            create_unit_node("m", Some(1.0), 0, 1),
            create_unit_node("s", Some(1.0), 4, 5),
            0,
            5,
        );
        let composite = resolve_unit(&unit_expr);

        assert_units_match(composite.units(), &[("m", 1.0), ("s", -1.0)]);
    }

    #[test]
    fn test_complex_expression() {
        // (m * kg) / (s * K)
        let unit_expr = create_binary_op_node(
            UnitOp::Divide,
            create_binary_op_node(
                UnitOp::Multiply,
                create_unit_node("m", Some(1.0), 1, 2),
                create_unit_node("kg", Some(1.0), 5, 7),
                1,
                7,
            ),
            create_binary_op_node(
                UnitOp::Multiply,
                create_unit_node("s", Some(1.0), 11, 12),
                create_unit_node("K", Some(1.0), 15, 16),
                11,
                16,
            ),
            0,
            16,
        );
        let composite = resolve_unit(&unit_expr);

        assert_units_match(
            composite.units(),
            &[("m", 1.0), ("kg", 1.0), ("s", -1.0), ("K", -1.0)],
        );
    }

    #[test]
    fn test_nested_division() {
        // m / (s / kg)
        let unit_expr = create_binary_op_node(
            UnitOp::Divide,
            create_unit_node("m", Some(1.0), 0, 1),
            create_binary_op_node(
                UnitOp::Divide,
                create_unit_node("s", Some(1.0), 5, 6),
                create_unit_node("kg", Some(1.0), 9, 11),
                5,
                11,
            ),
            0,
            11,
        );
        let composite = resolve_unit(&unit_expr);

        assert_units_match(composite.units(), &[("m", 1.0), ("s", -1.0), ("kg", 1.0)]);
    }

    #[test]
    fn test_units_with_exponents() {
        // m^2 * kg^3 / s^-1
        let unit_expr = create_binary_op_node(
            UnitOp::Multiply,
            create_binary_op_node(
                UnitOp::Multiply,
                create_unit_node("m", Some(2.0), 0, 1),
                create_unit_node("kg", Some(3.0), 5, 7),
                0,
                7,
            ),
            create_binary_op_node(
                UnitOp::Divide,
                create_unit_node("s", Some(-1.0), 11, 12),
                create_unit_node("K", Some(1.0), 16, 17),
                11,
                17,
            ),
            0,
            17,
        );
        let composite = resolve_unit(&unit_expr);

        assert_units_match(
            composite.units(),
            &[("m", 2.0), ("kg", 3.0), ("s", -1.0), ("K", -1.0)],
        );
    }

    #[test]
    fn test_negative_exponents_in_division() {
        // m^-2 / s^-3
        let unit_expr = create_binary_op_node(
            UnitOp::Divide,
            create_unit_node("m", Some(-2.0), 0, 1),
            create_unit_node("s", Some(-3.0), 5, 6),
            0,
            6,
        );
        let composite = resolve_unit(&unit_expr);

        assert_units_match(composite.units(), &[("m", -2.0), ("s", 3.0)]);
    }

    #[test]
    fn test_deeply_nested_expression() {
        // ((m * kg) / s) * (N / m^2)
        let unit_expr = create_binary_op_node(
            UnitOp::Multiply,
            create_binary_op_node(
                UnitOp::Divide,
                create_binary_op_node(
                    UnitOp::Multiply,
                    create_unit_node("m", Some(1.0), 2, 3),
                    create_unit_node("kg", Some(1.0), 6, 8),
                    2,
                    8,
                ),
                create_unit_node("s", Some(1.0), 12, 13),
                2,
                13,
            ),
            create_binary_op_node(
                UnitOp::Divide,
                create_unit_node("N", Some(1.0), 17, 18),
                create_unit_node("m", Some(2.0), 22, 23),
                17,
                23,
            ),
            0,
            23,
        );
        let composite = resolve_unit(&unit_expr);

        assert_units_match(
            composite.units(),
            &[
                ("m", 1.0),
                ("kg", 1.0),
                ("s", -1.0),
                ("N", 1.0),
                ("m", -2.0),
            ],
        );
    }

    #[test]
    fn test_empty_unit_identifier() {
        let unit_expr = create_unit_node("", Some(1.0), 0, 0);
        let composite = resolve_unit(&unit_expr);

        assert_units_match(composite.units(), &[("", 1.0)]);
    }

    #[test]
    fn test_zero_exponent() {
        let unit_expr = create_unit_node("m", Some(0.0), 0, 1);
        let composite = resolve_unit(&unit_expr);

        assert_units_match(composite.units(), &[("m", 0.0)]);
    }

    #[test]
    fn test_fractional_exponents() {
        let unit_expr = create_unit_node("m", Some(0.5), 0, 1);
        let composite = resolve_unit(&unit_expr);

        assert_units_match(composite.units(), &[("m", 0.5)]);
    }

    #[test]
    fn test_order_insensitive_assertions() {
        // This test demonstrates that our assertions are order-insensitive
        // by testing a complex expression where the order of units might vary
        // (m * kg * s) / (N * K)
        let unit_expr = create_binary_op_node(
            UnitOp::Divide,
            create_binary_op_node(
                UnitOp::Multiply,
                create_binary_op_node(
                    UnitOp::Multiply,
                    create_unit_node("m", Some(1.0), 1, 2),
                    create_unit_node("kg", Some(1.0), 5, 7),
                    1,
                    7,
                ),
                create_unit_node("s", Some(1.0), 11, 12),
                1,
                12,
            ),
            create_binary_op_node(
                UnitOp::Multiply,
                create_unit_node("N", Some(1.0), 16, 17),
                create_unit_node("K", Some(1.0), 20, 21),
                16,
                21,
            ),
            0,
            21,
        );
        let composite = resolve_unit(&unit_expr);

        // The expected units are: m^1, kg^1, s^1, N^-1, K^-1
        // The order doesn't matter, we just check that all expected units exist
        assert_units_match(
            composite.units(),
            &[
                ("m", 1.0),
                ("kg", 1.0),
                ("s", 1.0),
                ("N", -1.0),
                ("K", -1.0),
            ],
        );

        // We can also test the same assertion with a different order - it should still pass
        assert_units_match(
            composite.units(),
            &[
                ("kg", 1.0),
                ("N", -1.0),
                ("m", 1.0),
                ("K", -1.0),
                ("s", 1.0),
            ],
        );
    }

    #[test]
    fn test_parenthesized_expression() {
        // Test a simple parenthesized expression: (m * kg)
        let inner_expr = create_binary_op_node(
            UnitOp::Multiply,
            create_unit_node("m", Some(1.0), 1, 2),
            create_unit_node("kg", Some(1.0), 5, 7),
            1,
            7,
        );
        let unit_expr = create_parenthesized_node(inner_expr, 0, 8);
        let composite = resolve_unit(&unit_expr);

        assert_units_match(composite.units(), &[("m", 1.0), ("kg", 1.0)]);
    }

    #[test]
    fn test_nested_parenthesized_expression() {
        // Test nested parentheses: ((m * kg) / s)
        let inner_mult = create_binary_op_node(
            UnitOp::Multiply,
            create_unit_node("m", Some(1.0), 2, 3),
            create_unit_node("kg", Some(1.0), 6, 8),
            2,
            8,
        );
        let inner_parenthesized = create_parenthesized_node(inner_mult, 1, 9);
        let division_expr = create_binary_op_node(
            UnitOp::Divide,
            inner_parenthesized,
            create_unit_node("s", Some(1.0), 12, 13),
            1,
            13,
        );
        let unit_expr = create_parenthesized_node(division_expr, 0, 14);
        let composite = resolve_unit(&unit_expr);

        assert_units_match(composite.units(), &[("m", 1.0), ("kg", 1.0), ("s", -1.0)]);
    }

    #[test]
    fn test_single_unit_multiple_parentheses() {
        // Test a single unit wrapped in multiple parentheses: ((m))
        let inner_unit = create_unit_node("m", Some(1.0), 2, 3);
        let first_parentheses = create_parenthesized_node(inner_unit, 1, 4);
        let unit_expr = create_parenthesized_node(first_parentheses, 0, 5);
        let composite = resolve_unit(&unit_expr);

        // The unit should be resolved correctly regardless of the number of parentheses
        assert_units_match(composite.units(), &[("m", 1.0)]);
    }

    #[test]
    fn test_single_unit_deep_nested_parentheses() {
        // Test a single unit with deeply nested parentheses: (((kg)))
        let inner_unit = create_unit_node("kg", Some(1.0), 3, 5);
        let third_level = create_parenthesized_node(inner_unit, 2, 6);
        let second_level = create_parenthesized_node(third_level, 1, 7);
        let unit_expr = create_parenthesized_node(second_level, 0, 8);
        let composite = resolve_unit(&unit_expr);

        // The unit should be resolved correctly even with deeply nested parentheses
        assert_units_match(composite.units(), &[("kg", 1.0)]);
    }

    #[test]
    fn test_single_unit_with_exponent_multiple_parentheses() {
        // Test a single unit with exponent wrapped in multiple parentheses: ((m^2))
        let inner_unit = create_unit_node("m", Some(2.0), 2, 3);
        let first_parentheses = create_parenthesized_node(inner_unit, 1, 4);
        let unit_expr = create_parenthesized_node(first_parentheses, 0, 5);
        let composite = resolve_unit(&unit_expr);

        // The unit with exponent should be resolved correctly regardless of parentheses
        assert_units_match(composite.units(), &[("m", 2.0)]);
    }
}
