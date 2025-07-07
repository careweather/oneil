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
pub fn resolve_unit(unit: &ast::UnitExpr) -> oneil_module::unit::CompositeUnit {
    let units = resolve_unit_recursive(unit, false, Vec::new());
    oneil_module::unit::CompositeUnit::new(units)
}

fn resolve_unit_recursive(
    unit: &ast::UnitExpr,
    is_inverse: bool,
    mut units: Vec<oneil_module::unit::Unit>,
) -> Vec<oneil_module::unit::Unit> {
    match unit {
        ast::UnitExpr::BinaryOp { op, left, right } => {
            let units = resolve_unit_recursive(left, is_inverse, units);

            let units = match op {
                ast::unit::UnitOp::Multiply => resolve_unit_recursive(right, is_inverse, units),
                ast::unit::UnitOp::Divide => resolve_unit_recursive(right, !is_inverse, units),
            };
            units
        }
        ast::UnitExpr::Unit {
            identifier,
            exponent,
        } => {
            let exponent = exponent.unwrap_or(1.0);
            let exponent = if is_inverse { -exponent } else { exponent };

            let unit = oneil_module::unit::Unit::new(identifier.clone(), exponent);
            units.push(unit);
            units
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oneil_ast::unit::{UnitExpr, UnitOp};

    /// Helper function to create a simple unit expression
    fn unit(name: &str, exponent: Option<f64>) -> UnitExpr {
        UnitExpr::Unit {
            identifier: name.to_string(),
            exponent,
        }
    }

    /// Helper function to create a binary operation
    fn bin_op(op: UnitOp, left: UnitExpr, right: UnitExpr) -> UnitExpr {
        UnitExpr::BinaryOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    /// Helper function to check if a unit with the given name and exponent exists
    fn assert_unit_exists(units: &[oneil_module::unit::Unit], name: &str, exponent: f64) {
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
    fn assert_units_match(units: &[oneil_module::unit::Unit], expected: &[(&str, f64)]) {
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
        let unit_expr = unit("m", Some(1.0));
        let composite = resolve_unit(&unit_expr);

        assert_units_match(composite.units(), &[("m", 1.0)]);
    }

    #[test]
    fn test_unit_with_default_exponent() {
        let unit_expr = unit("kg", None);
        let composite = resolve_unit(&unit_expr);

        assert_units_match(composite.units(), &[("kg", 1.0)]);
    }

    #[test]
    fn test_unit_with_custom_exponent() {
        let unit_expr = unit("m", Some(2.0));
        let composite = resolve_unit(&unit_expr);

        assert_units_match(composite.units(), &[("m", 2.0)]);
    }

    #[test]
    fn test_multiplication() {
        // m * kg
        let unit_expr = bin_op(
            UnitOp::Multiply,
            unit("m", Some(1.0)),
            unit("kg", Some(1.0)),
        );
        let composite = resolve_unit(&unit_expr);

        assert_units_match(composite.units(), &[("m", 1.0), ("kg", 1.0)]);
    }

    #[test]
    fn test_division() {
        // m / s
        let unit_expr = bin_op(UnitOp::Divide, unit("m", Some(1.0)), unit("s", Some(1.0)));
        let composite = resolve_unit(&unit_expr);

        assert_units_match(composite.units(), &[("m", 1.0), ("s", -1.0)]);
    }

    #[test]
    fn test_complex_expression() {
        // (m * kg) / (s * K)
        let unit_expr = bin_op(
            UnitOp::Divide,
            bin_op(
                UnitOp::Multiply,
                unit("m", Some(1.0)),
                unit("kg", Some(1.0)),
            ),
            bin_op(UnitOp::Multiply, unit("s", Some(1.0)), unit("K", Some(1.0))),
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
        let unit_expr = bin_op(
            UnitOp::Divide,
            unit("m", Some(1.0)),
            bin_op(UnitOp::Divide, unit("s", Some(1.0)), unit("kg", Some(1.0))),
        );
        let composite = resolve_unit(&unit_expr);

        assert_units_match(composite.units(), &[("m", 1.0), ("s", -1.0), ("kg", 1.0)]);
    }

    #[test]
    fn test_units_with_exponents() {
        // m^2 * kg^3 / s^-1
        let unit_expr = bin_op(
            UnitOp::Multiply,
            bin_op(
                UnitOp::Multiply,
                unit("m", Some(2.0)),
                unit("kg", Some(3.0)),
            ),
            bin_op(UnitOp::Divide, unit("s", Some(-1.0)), unit("K", Some(1.0))),
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
        let unit_expr = bin_op(UnitOp::Divide, unit("m", Some(-2.0)), unit("s", Some(-3.0)));
        let composite = resolve_unit(&unit_expr);

        assert_units_match(composite.units(), &[("m", -2.0), ("s", 3.0)]);
    }

    #[test]
    fn test_deeply_nested_expression() {
        // ((m * kg) / s) * (N / m^2)
        let unit_expr = bin_op(
            UnitOp::Multiply,
            bin_op(
                UnitOp::Divide,
                bin_op(
                    UnitOp::Multiply,
                    unit("m", Some(1.0)),
                    unit("kg", Some(1.0)),
                ),
                unit("s", Some(1.0)),
            ),
            bin_op(UnitOp::Divide, unit("N", Some(1.0)), unit("m", Some(2.0))),
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
        let unit_expr = unit("", Some(1.0));
        let composite = resolve_unit(&unit_expr);

        assert_units_match(composite.units(), &[("", 1.0)]);
    }

    #[test]
    fn test_zero_exponent() {
        let unit_expr = unit("m", Some(0.0));
        let composite = resolve_unit(&unit_expr);

        assert_units_match(composite.units(), &[("m", 0.0)]);
    }

    #[test]
    fn test_fractional_exponents() {
        let unit_expr = unit("m", Some(0.5));
        let composite = resolve_unit(&unit_expr);

        assert_units_match(composite.units(), &[("m", 0.5)]);
    }

    #[test]
    fn test_order_insensitive_assertions() {
        // This test demonstrates that our assertions are order-insensitive
        // by testing a complex expression where the order of units might vary
        // (m * kg * s) / (N * K)
        let unit_expr = bin_op(
            UnitOp::Divide,
            bin_op(
                UnitOp::Multiply,
                bin_op(
                    UnitOp::Multiply,
                    unit("m", Some(1.0)),
                    unit("kg", Some(1.0)),
                ),
                unit("s", Some(1.0)),
            ),
            bin_op(UnitOp::Multiply, unit("N", Some(1.0)), unit("K", Some(1.0))),
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
}
