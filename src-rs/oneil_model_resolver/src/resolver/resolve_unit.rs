use crate::util::get_span_from_ast_span;

use oneil_ast as ast;
use oneil_ir as ir;

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
pub fn resolve_unit(unit: &ast::UnitExprNode) -> ir::CompositeUnit {
    let units = resolve_unit_recursive(unit, false, Vec::new());
    ir::CompositeUnit::new(units)
}

fn resolve_unit_recursive(
    unit: &ast::UnitExprNode,
    is_inverse: bool,
    mut units: Vec<ir::Unit>,
) -> Vec<ir::Unit> {
    match unit.node_value() {
        ast::UnitExpr::BinaryOp { op, left, right } => {
            let units = resolve_unit_recursive(left, is_inverse, units);

            match op.node_value() {
                ast::UnitOp::Multiply => resolve_unit_recursive(right, is_inverse, units),
                ast::UnitOp::Divide => resolve_unit_recursive(right, !is_inverse, units),
            }
        }
        ast::UnitExpr::Unit {
            identifier,
            exponent,
        } => {
            let exponent_value = exponent.as_ref().map_or(1.0, |e| e.value());
            let exponent_value = if is_inverse {
                -exponent_value
            } else {
                exponent_value
            };

            let name_span = get_span_from_ast_span(identifier.node_span());
            let exponent_span = exponent
                .as_ref()
                .map(|exp| get_span_from_ast_span(exp.node_span()));
            let unit = ir::Unit::new(
                identifier.as_str().to_string(),
                name_span,
                exponent_value,
                // TODO: should this be an Option rather than filling in with an arbitrary span?
                exponent_span,
            );
            units.push(unit);
            units
        }
        ast::UnitExpr::UnitOne => units,
        ast::UnitExpr::Parenthesized { expr } => resolve_unit_recursive(expr, is_inverse, units),
    }
}

#[cfg(test)]
mod tests {
    use crate::test::construct::test_ast;

    use super::*;
    use oneil_ast as ast;

    fn unit_tuple_cmp(
        (a_name, a_exponent): &(&str, f64),
        (b_name, b_exponent): &(&str, f64),
    ) -> std::cmp::Ordering {
        a_name.cmp(b_name).then(
            a_exponent
                .partial_cmp(b_exponent)
                .unwrap_or(std::cmp::Ordering::Equal),
        )
    }

    macro_rules! assert_units_match {
        ($actual_units:expr, $expected_units:expr $(,)?) => {
            let mut actual_units: Vec<(&str, f64)> = $actual_units
                .into_iter()
                .map(|u| (u.name(), u.exponent()))
                .collect();

            let mut expected_units: Vec<(&str, f64)> = $expected_units.into_iter().collect();

            actual_units.sort_by(unit_tuple_cmp);
            expected_units.sort_by(unit_tuple_cmp);

            assert_eq!(
                actual_units, expected_units,
                "actual units do not match expected units"
            );
        };
    }

    #[test]
    fn test_simple_unit() {
        // create a simple unit expression
        let unit_expr = test_ast::unit_node("m");

        // resolve the unit
        let composite = resolve_unit(&unit_expr);

        // check the result
        assert_units_match!(composite.units(), [("m", 1.0)]);
    }

    #[test]
    fn test_unit_with_default_exponent() {
        // create a unit expression without explicit exponent
        let unit_expr = test_ast::unit_node("kg");

        // resolve the unit
        let composite = resolve_unit(&unit_expr);

        // check the result
        assert_units_match!(composite.units(), [("kg", 1.0)]);
    }

    #[test]
    fn test_unit_with_custom_exponent() {
        // create a unit expression with custom exponent
        let unit_expr = test_ast::unit_with_exponent_node("m", 2.0);

        // resolve the unit
        let composite = resolve_unit(&unit_expr);

        // check the result
        assert_units_match!(composite.units(), [("m", 2.0)]);
    }

    #[test]
    fn test_multiplication() {
        // create a multiplication expression: m * kg
        let unit_expr = test_ast::unit_binary_op_node(
            ast::UnitOp::Multiply,
            test_ast::unit_node("m"),
            test_ast::unit_node("kg"),
        );

        // resolve the unit
        let composite = resolve_unit(&unit_expr);

        // check the result
        assert_units_match!(composite.units(), [("m", 1.0), ("kg", 1.0)]);
    }

    #[test]
    fn test_division() {
        // create a division expression: m / s
        let unit_expr = test_ast::unit_binary_op_node(
            ast::UnitOp::Divide,
            test_ast::unit_node("m"),
            test_ast::unit_node("s"),
        );

        // resolve the unit
        let composite = resolve_unit(&unit_expr);

        // check the result
        assert_units_match!(composite.units(), [("m", 1.0), ("s", -1.0)]);
    }

    #[test]
    fn test_complex_expression() {
        // create a complex expression: (m * kg) / (s * K)
        let unit_expr = test_ast::unit_binary_op_node(
            ast::UnitOp::Divide,
            test_ast::unit_binary_op_node(
                ast::UnitOp::Multiply,
                test_ast::unit_node("m"),
                test_ast::unit_node("kg"),
            ),
            test_ast::unit_binary_op_node(
                ast::UnitOp::Multiply,
                test_ast::unit_node("s"),
                test_ast::unit_node("K"),
            ),
        );

        // resolve the unit
        let composite = resolve_unit(&unit_expr);

        // check the result
        assert_units_match!(
            composite.units(),
            [("m", 1.0), ("kg", 1.0), ("s", -1.0), ("K", -1.0)],
        );
    }

    #[test]
    fn test_nested_division() {
        // create a nested division expression: m / (s / kg)
        let unit_expr = test_ast::unit_binary_op_node(
            ast::UnitOp::Divide,
            test_ast::unit_node("m"),
            test_ast::unit_binary_op_node(
                ast::UnitOp::Divide,
                test_ast::unit_node("s"),
                test_ast::unit_node("kg"),
            ),
        );

        // resolve the unit
        let composite = resolve_unit(&unit_expr);

        // check the result
        assert_units_match!(composite.units(), [("m", 1.0), ("s", -1.0), ("kg", 1.0)]);
    }

    #[test]
    fn test_units_with_exponents() {
        // create an expression with exponents: m^2 * kg^3 * s^-1 / K^1
        let unit_expr = test_ast::unit_binary_op_node(
            ast::UnitOp::Multiply,
            test_ast::unit_binary_op_node(
                ast::UnitOp::Multiply,
                test_ast::unit_with_exponent_node("m", 2.0),
                test_ast::unit_with_exponent_node("kg", 3.0),
            ),
            test_ast::unit_binary_op_node(
                ast::UnitOp::Divide,
                test_ast::unit_with_exponent_node("s", -1.0),
                test_ast::unit_with_exponent_node("K", 1.0),
            ),
        );

        // resolve the unit
        let composite = resolve_unit(&unit_expr);

        // check the result
        assert_units_match!(
            composite.units(),
            [("m", 2.0), ("kg", 3.0), ("s", -1.0), ("K", -1.0)],
        );
    }

    #[test]
    fn test_negative_exponents_in_division() {
        // create an expression with negative exponents: m^-2 / s^-3
        let unit_expr = test_ast::unit_binary_op_node(
            ast::UnitOp::Divide,
            test_ast::unit_with_exponent_node("m", -2.0),
            test_ast::unit_with_exponent_node("s", -3.0),
        );

        // resolve the unit
        let composite = resolve_unit(&unit_expr);

        // check the result
        assert_units_match!(composite.units(), [("m", -2.0), ("s", 3.0)]);
    }

    #[test]
    fn test_deeply_nested_expression() {
        // create a deeply nested expression: ((m * kg) / s) * (N / m^2)
        let unit_expr = test_ast::unit_binary_op_node(
            ast::UnitOp::Multiply,
            test_ast::unit_binary_op_node(
                ast::UnitOp::Divide,
                test_ast::unit_binary_op_node(
                    ast::UnitOp::Multiply,
                    test_ast::unit_node("m"),
                    test_ast::unit_node("kg"),
                ),
                test_ast::unit_node("s"),
            ),
            test_ast::unit_binary_op_node(
                ast::UnitOp::Divide,
                test_ast::unit_node("N"),
                test_ast::unit_with_exponent_node("m", 2.0),
            ),
        );

        // resolve the unit
        let composite = resolve_unit(&unit_expr);

        // check the result
        assert_units_match!(
            composite.units(),
            [
                ("m", 1.0),
                ("kg", 1.0),
                ("s", -1.0),
                ("N", 1.0),
                ("m", -2.0),
            ],
        );
    }

    #[test]
    fn test_fractional_exponents() {
        // create a unit expression with fractional exponent
        // m^0.5
        let unit_expr = test_ast::unit_with_exponent_node("m", 0.5);

        // resolve the unit
        let composite = resolve_unit(&unit_expr);

        // check the result
        assert_units_match!(composite.units(), [("m", 0.5)]);
    }

    #[test]
    fn test_parenthesized_expression() {
        // create a simple parenthesized expression: (m * kg)
        let inner_expr = test_ast::unit_binary_op_node(
            ast::UnitOp::Multiply,
            test_ast::unit_node("m"),
            test_ast::unit_node("kg"),
        );
        let unit_expr = test_ast::parenthesized_unit_node(inner_expr);

        // resolve the unit
        let composite = resolve_unit(&unit_expr);

        // check the result
        assert_units_match!(composite.units(), [("m", 1.0), ("kg", 1.0)]);
    }

    #[test]
    fn test_nested_parenthesized_expression() {
        // create nested parentheses: ((m * kg) / s)
        let inner_mult = test_ast::unit_binary_op_node(
            ast::UnitOp::Multiply,
            test_ast::unit_node("m"),
            test_ast::unit_node("kg"),
        );
        let inner_parenthesized = test_ast::parenthesized_unit_node(inner_mult);
        let division_expr = test_ast::unit_binary_op_node(
            ast::UnitOp::Divide,
            inner_parenthesized,
            test_ast::unit_node("s"),
        );
        let unit_expr = test_ast::parenthesized_unit_node(division_expr);

        // resolve the unit
        let composite = resolve_unit(&unit_expr);

        // check the result
        assert_units_match!(composite.units(), [("m", 1.0), ("kg", 1.0), ("s", -1.0)]);
    }

    #[test]
    fn test_single_unit_multiple_parentheses() {
        // create a single unit wrapped in multiple parentheses: ((m))
        let inner_unit = test_ast::unit_node("m");
        let first_parentheses = test_ast::parenthesized_unit_node(inner_unit);
        let unit_expr = test_ast::parenthesized_unit_node(first_parentheses);

        // resolve the unit
        let composite = resolve_unit(&unit_expr);

        // check the result - the unit should be resolved correctly regardless of parentheses
        assert_units_match!(composite.units(), [("m", 1.0)]);
    }

    #[test]
    fn test_unit_one() {
        // create a UnitOne expression
        let unit_expr = test_ast::unit_one_node();

        // resolve the unit
        let composite = resolve_unit(&unit_expr);

        // check the result - UnitOne should result in an empty composite unit
        assert_eq!(
            composite.units().len(),
            0,
            "UnitOne should result in no units"
        );
    }

    #[test]
    fn test_unit_one_in_multiplication() {
        // create a multiplication expression with UnitOne: 1 * m
        let unit_expr = test_ast::unit_binary_op_node(
            ast::UnitOp::Multiply,
            test_ast::unit_one_node(),
            test_ast::unit_node("m"),
        );

        // resolve the unit
        let composite = resolve_unit(&unit_expr);

        // check the result - UnitOne should be ignored in multiplication
        assert_units_match!(composite.units(), [("m", 1.0)]);
    }

    #[test]
    fn test_unit_one_in_division() {
        // create a division expression with UnitOne: m / 1
        let unit_expr = test_ast::unit_binary_op_node(
            ast::UnitOp::Divide,
            test_ast::unit_node("m"),
            test_ast::unit_one_node(),
        );

        // resolve the unit
        let composite = resolve_unit(&unit_expr);

        // check the result - UnitOne should be ignored in division
        assert_units_match!(composite.units(), [("m", 1.0)]);
    }
}
