use oneil_ast as ast;
use oneil_ir as ir;
use oneil_shared::symbols::{UnitBaseName, UnitName, UnitPrefix};

use crate::{ExternalResolutionContext, ResolutionContext, error::UnitResolutionError};

/// Resolves an AST unit expression into a composite unit representation.
///
/// This function takes a parsed unit expression from the AST and converts it
/// into a `CompositeUnit` that represents the same unit in a flattened form.
/// The resolution process handles multiplication, division, and exponents
/// by recursively traversing the expression tree. It also performs name
/// resolution for each unit, determining the prefix, stripped name, and
/// whether the unit is a decibel unit.
///
/// # Errors
///
/// Returns a vector of errors if any unit names cannot be resolved.
pub fn resolve_unit<E: ExternalResolutionContext>(
    unit: &ast::UnitExprNode,
    context: &ResolutionContext<'_, E>,
) -> Result<ir::CompositeUnit, Vec<UnitResolutionError>> {
    let (units, errors) = resolve_unit_recursive(unit, false, context, Vec::new(), Vec::new());
    let display_unit = resolve_display_unit(unit);

    let dimension = ir::compute_dimension_map(&units, |name| context.lookup_unit(name).cloned());
    let composite = ir::CompositeUnit::new(units, display_unit, unit.span().clone(), dimension);

    if errors.is_empty() {
        Ok(composite)
    } else {
        Err(errors)
    }
}

fn resolve_unit_recursive<E: ExternalResolutionContext>(
    unit: &ast::UnitExprNode,
    is_inverse: bool,
    context: &ResolutionContext<'_, E>,
    mut units: Vec<ir::Unit>,
    mut errors: Vec<UnitResolutionError>,
) -> (Vec<ir::Unit>, Vec<UnitResolutionError>) {
    match &**unit {
        ast::UnitExpr::BinaryOp { op, left, right } => {
            let (units, errors) = resolve_unit_recursive(left, is_inverse, context, units, errors);

            match &**op {
                ast::UnitOp::Multiply => {
                    resolve_unit_recursive(right, is_inverse, context, units, errors)
                }
                ast::UnitOp::Divide => {
                    resolve_unit_recursive(right, !is_inverse, context, units, errors)
                }
            }
        }

        ast::UnitExpr::Unit {
            identifier,
            exponent,
        } => {
            let unit_span = unit.span().clone();
            let name_span = identifier.span().clone();
            let exponent_span = exponent.as_ref().map(|n| n.span().clone());

            let exponent_value = exponent.as_ref().map_or(1.0, |e| e.value());
            let exponent_value = if is_inverse {
                -exponent_value
            } else {
                exponent_value
            };

            match resolve_unit_info(identifier.as_str(), context) {
                Some(info) => {
                    let full_name = UnitName::new(identifier.as_str().to_string());
                    let ir_unit = ir::Unit::new(
                        unit_span,
                        full_name,
                        name_span,
                        exponent_value,
                        exponent_span,
                        info,
                    );

                    units.push(ir_unit);
                }

                None => {
                    errors.push(UnitResolutionError::new(
                        identifier.clone().take_value(),
                        unit_span,
                    ));
                }
            }

            (units, errors)
        }

        ast::UnitExpr::UnitOne => (units, errors),

        ast::UnitExpr::Parenthesized { expr } => {
            resolve_unit_recursive(expr, is_inverse, context, units, errors)
        }
    }
}

/// Resolves a unit name to its components: prefix, stripped name, and dB flag.
fn resolve_unit_info<E: ExternalResolutionContext>(
    full_name: &str,
    context: &ResolutionContext<'_, E>,
) -> Option<ir::UnitInfo> {
    // check if the unit is a decibel unit
    let (name, is_db) = full_name
        .strip_prefix("dB")
        .map_or((full_name, false), |stripped_name| (stripped_name, true));

    // if the unit is a decibel unit and the name is empty,
    // return a decibel unit with no prefix and no base name
    if is_db && name.is_empty() {
        return Some(ir::UnitInfo::Db {
            prefix: None,
            base_name: None,
        });
    }

    let make_unit_info = if is_db {
        |prefix: Option<UnitPrefix>, base_name: UnitBaseName| ir::UnitInfo::Db {
            prefix,
            base_name: Some(base_name),
        }
    } else {
        |prefix: Option<UnitPrefix>, base_name: UnitBaseName| ir::UnitInfo::Standard {
            prefix,
            base_name,
        }
    };

    // if the name matches a builtin unit, return appropriate unit type
    if context.has_builtin_unit(name) {
        let base_name = UnitBaseName::from(name);
        let unit_info = make_unit_info(None, base_name);
        return Some(unit_info);
    }

    // try to match a prefix and look up the stripped unit
    for (prefix, _magnitude) in context.available_prefixes() {
        let Some(stripped_name) = name.strip_prefix(prefix.as_str()) else {
            continue;
        };

        let base_name = UnitBaseName::from(stripped_name);

        if !context.unit_supports_si_prefixes(&base_name) {
            continue;
        }

        if context.has_builtin_unit(base_name.as_str()) {
            let unit_info = make_unit_info(Some(prefix.clone()), base_name);
            return Some(unit_info);
        }
    }

    None
}

fn resolve_display_unit(unit: &ast::UnitExprNode) -> ir::DisplayCompositeUnit {
    match &**unit {
        ast::UnitExpr::BinaryOp { op, left, right } => match &**op {
            ast::UnitOp::Multiply => ir::DisplayCompositeUnit::Multiply(
                Box::new(resolve_display_unit(left)),
                Box::new(resolve_display_unit(right)),
            ),
            ast::UnitOp::Divide => ir::DisplayCompositeUnit::Divide(
                Box::new(resolve_display_unit(left)),
                Box::new(resolve_display_unit(right)),
            ),
        },
        ast::UnitExpr::Unit {
            identifier,
            exponent,
        } => {
            let display_unit = ir::DisplayUnit::new(
                identifier.as_str().to_string(),
                exponent.as_ref().map_or(1.0, |e| e.value()),
            );
            ir::DisplayCompositeUnit::BaseUnit(display_unit)
        }
        ast::UnitExpr::UnitOne => ir::DisplayCompositeUnit::One,
        ast::UnitExpr::Parenthesized { expr } => resolve_display_unit(expr),
    }
}

#[cfg(test)]
mod tests {
    use crate::test::{
        external_context::{TestBuiltinUnit, TestExternalContext},
        resolution_context::ResolutionContextBuilder,
        test_ast, test_model_path,
    };

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

    /// Asserts that resolved unit components match the expected name/exponent pairs.
    ///
    /// # Panics
    ///
    /// Panics if the actual units do not match the expected units.
    #[track_caller]
    fn assert_units_match(actual_units: &[ir::Unit], expected_units: &[(&str, f64)]) {
        let mut actual_units: Vec<(&str, f64)> = actual_units
            .iter()
            .map(|unit| (unit.name().as_str(), unit.exponent()))
            .collect();

        let mut expected_units: Vec<(&str, f64)> = expected_units.to_vec();

        actual_units.sort_by(unit_tuple_cmp);
        expected_units.sort_by(unit_tuple_cmp);

        assert_eq!(
            actual_units, expected_units,
            "actual units do not match expected units"
        );
    }

    fn test_external_context_with_common_units() -> TestExternalContext {
        TestExternalContext::new()
            .with_builtin_units([
                TestBuiltinUnit {
                    name: "m",
                    supports_si_prefixes: true,
                },
                TestBuiltinUnit {
                    name: "s",
                    supports_si_prefixes: true,
                },
                TestBuiltinUnit {
                    name: "g",
                    supports_si_prefixes: true,
                },
                TestBuiltinUnit {
                    name: "kg",
                    supports_si_prefixes: true,
                },
                TestBuiltinUnit {
                    name: "K",
                    supports_si_prefixes: true,
                },
                TestBuiltinUnit {
                    name: "N",
                    supports_si_prefixes: true,
                },
                TestBuiltinUnit {
                    name: "A",
                    supports_si_prefixes: true,
                },
                TestBuiltinUnit {
                    name: "mol",
                    supports_si_prefixes: true,
                },
                TestBuiltinUnit {
                    name: "cd",
                    supports_si_prefixes: true,
                },
            ])
            .with_builtin_prefixes([
                ("k", 1e3),
                ("M", 1e6),
                ("G", 1e9),
                ("m", 1e-3),
                ("u", 1e-6),
                ("n", 1e-9),
            ])
    }

    #[test]
    fn simple_unit() {
        // create a simple unit expression
        let unit_expr = test_ast::unit_node("m");

        // build the context
        let active_path = test_model_path("main");
        let mut external = test_external_context_with_common_units();
        let resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // resolve the unit
        let composite =
            resolve_unit(&unit_expr, &resolution_context).expect("resolve should succeed");

        // check the result
        assert_units_match(composite.units(), &[("m", 1.0)]);
    }

    #[test]
    fn unit_with_default_exponent() {
        // create a unit expression without explicit exponent
        let unit_expr = test_ast::unit_node("kg");

        // build the context
        let active_path = test_model_path("main");
        let mut external = test_external_context_with_common_units();
        let resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // resolve the unit
        let composite =
            resolve_unit(&unit_expr, &resolution_context).expect("resolve should succeed");

        // check the result
        assert_units_match(composite.units(), &[("kg", 1.0)]);
    }

    #[test]
    fn unit_with_custom_exponent() {
        // create a unit expression with custom exponent
        let unit_expr = test_ast::unit_with_exponent_node("m", 2.0);

        // build the context
        let active_path = test_model_path("main");
        let mut external = test_external_context_with_common_units();
        let resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // resolve the unit
        let composite =
            resolve_unit(&unit_expr, &resolution_context).expect("resolve should succeed");

        // check the result
        assert_units_match(composite.units(), &[("m", 2.0)]);
    }

    #[test]
    fn multiplication() {
        // create a multiplication expression: m * kg
        let unit_expr = test_ast::unit_binary_op_node(
            ast::UnitOp::Multiply,
            test_ast::unit_node("m"),
            test_ast::unit_node("kg"),
        );

        // build the context
        let active_path = test_model_path("main");
        let mut external = test_external_context_with_common_units();
        let resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // resolve the unit
        let composite =
            resolve_unit(&unit_expr, &resolution_context).expect("resolve should succeed");

        // check the result
        assert_units_match(composite.units(), &[("m", 1.0), ("kg", 1.0)]);
    }

    #[test]
    fn division() {
        // create a division expression: m / s
        let unit_expr = test_ast::unit_binary_op_node(
            ast::UnitOp::Divide,
            test_ast::unit_node("m"),
            test_ast::unit_node("s"),
        );

        // build the context
        let active_path = test_model_path("main");
        let mut external = test_external_context_with_common_units();
        let resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // resolve the unit
        let composite =
            resolve_unit(&unit_expr, &resolution_context).expect("resolve should succeed");

        // check the result
        assert_units_match(composite.units(), &[("m", 1.0), ("s", -1.0)]);
    }

    #[test]
    fn complex_expression() {
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

        // build the context
        let active_path = test_model_path("main");
        let mut external = test_external_context_with_common_units();
        let resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // resolve the unit
        let composite =
            resolve_unit(&unit_expr, &resolution_context).expect("resolve should succeed");

        // check the result
        assert_units_match(
            composite.units(),
            &[("m", 1.0), ("kg", 1.0), ("s", -1.0), ("K", -1.0)],
        );
    }

    #[test]
    fn nested_division() {
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

        // build the context
        let active_path = test_model_path("main");
        let mut external = test_external_context_with_common_units();
        let resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // resolve the unit
        let composite =
            resolve_unit(&unit_expr, &resolution_context).expect("resolve should succeed");

        // check the result
        assert_units_match(composite.units(), &[("m", 1.0), ("s", -1.0), ("kg", 1.0)]);
    }

    #[test]
    fn units_with_exponents() {
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

        // build the context
        let active_path = test_model_path("main");
        let mut external = test_external_context_with_common_units();
        let resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // resolve the unit
        let composite =
            resolve_unit(&unit_expr, &resolution_context).expect("resolve should succeed");

        // check the result
        assert_units_match(
            composite.units(),
            &[("m", 2.0), ("kg", 3.0), ("s", -1.0), ("K", -1.0)],
        );
    }

    #[test]
    fn negative_exponents_in_division() {
        // create an expression with negative exponents: m^-2 / s^-3
        let unit_expr = test_ast::unit_binary_op_node(
            ast::UnitOp::Divide,
            test_ast::unit_with_exponent_node("m", -2.0),
            test_ast::unit_with_exponent_node("s", -3.0),
        );

        // build the context
        let active_path = test_model_path("main");
        let mut external = test_external_context_with_common_units();
        let resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // resolve the unit
        let composite =
            resolve_unit(&unit_expr, &resolution_context).expect("resolve should succeed");

        // check the result
        assert_units_match(composite.units(), &[("m", -2.0), ("s", 3.0)]);
    }

    #[test]
    fn deeply_nested_expression() {
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

        // build the context
        let active_path = test_model_path("main");
        let mut external = test_external_context_with_common_units();
        let resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // resolve the unit
        let composite =
            resolve_unit(&unit_expr, &resolution_context).expect("resolve should succeed");

        // check the result
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
    fn fractional_exponents() {
        // create a unit expression with fractional exponent
        // m^0.5
        let unit_expr = test_ast::unit_with_exponent_node("m", 0.5);

        // build the context
        let active_path = test_model_path("main");
        let mut external = test_external_context_with_common_units();
        let resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // resolve the unit
        let composite =
            resolve_unit(&unit_expr, &resolution_context).expect("resolve should succeed");

        // check the result
        assert_units_match(composite.units(), &[("m", 0.5)]);
    }

    #[test]
    fn parenthesized_expression() {
        // create a simple parenthesized expression: (m * kg)
        let inner_expr = test_ast::unit_binary_op_node(
            ast::UnitOp::Multiply,
            test_ast::unit_node("m"),
            test_ast::unit_node("kg"),
        );
        let unit_expr = test_ast::parenthesized_unit_node(inner_expr);

        // build the context
        let active_path = test_model_path("main");
        let mut external = test_external_context_with_common_units();
        let resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // resolve the unit
        let composite =
            resolve_unit(&unit_expr, &resolution_context).expect("resolve should succeed");

        // check the result
        assert_units_match(composite.units(), &[("m", 1.0), ("kg", 1.0)]);
    }

    #[test]
    fn nested_parenthesized_expression() {
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

        // build the context
        let active_path = test_model_path("main");
        let mut external = test_external_context_with_common_units();
        let resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // resolve the unit
        let composite =
            resolve_unit(&unit_expr, &resolution_context).expect("resolve should succeed");

        // check the result
        assert_units_match(composite.units(), &[("m", 1.0), ("kg", 1.0), ("s", -1.0)]);
    }

    #[test]
    fn single_unit_multiple_parentheses() {
        // create a single unit wrapped in multiple parentheses: ((m))
        let inner_unit = test_ast::unit_node("m");
        let first_parentheses = test_ast::parenthesized_unit_node(inner_unit);
        let unit_expr = test_ast::parenthesized_unit_node(first_parentheses);

        // build the context
        let active_path = test_model_path("main");
        let mut external = test_external_context_with_common_units();
        let resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // resolve the unit
        let composite =
            resolve_unit(&unit_expr, &resolution_context).expect("resolve should succeed");

        // check the result - the unit should be resolved correctly regardless of parentheses
        assert_units_match(composite.units(), &[("m", 1.0)]);
    }

    #[test]
    fn unit_one() {
        // create a UnitOne expression
        let unit_expr = test_ast::unit_one_node();

        // build the context
        let active_path = test_model_path("main");
        let mut external = test_external_context_with_common_units();
        let resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // resolve the unit
        let composite =
            resolve_unit(&unit_expr, &resolution_context).expect("resolve should succeed");

        // check the result - UnitOne should result in an empty composite unit
        assert_eq!(
            composite.units().len(),
            0,
            "UnitOne should result in no units"
        );
    }

    #[test]
    fn unit_one_in_multiplication() {
        // create a multiplication expression with UnitOne: 1 * m
        let unit_expr = test_ast::unit_binary_op_node(
            ast::UnitOp::Multiply,
            test_ast::unit_one_node(),
            test_ast::unit_node("m"),
        );

        // build the context
        let active_path = test_model_path("main");
        let mut external = test_external_context_with_common_units();
        let resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // resolve the unit
        let composite =
            resolve_unit(&unit_expr, &resolution_context).expect("resolve should succeed");

        // check the result - UnitOne should be ignored in multiplication
        assert_units_match(composite.units(), &[("m", 1.0)]);
    }

    #[test]
    fn unit_one_in_division() {
        // create a division expression with UnitOne: m / 1
        let unit_expr = test_ast::unit_binary_op_node(
            ast::UnitOp::Divide,
            test_ast::unit_node("m"),
            test_ast::unit_one_node(),
        );

        // build the context
        let active_path = test_model_path("main");
        let mut external = test_external_context_with_common_units();
        let resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // resolve the unit
        let composite =
            resolve_unit(&unit_expr, &resolution_context).expect("resolve should succeed");

        // check the result - UnitOne should be ignored in division
        assert_units_match(composite.units(), &[("m", 1.0)]);
    }

    #[test]
    fn unknown_unit_returns_error() {
        // create a unit expression with an unknown unit
        let unit_expr = test_ast::unit_node("unknown_unit");

        // build the context
        let active_path = test_model_path("main");
        let mut external = test_external_context_with_common_units();
        let resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // resolve the unit
        let errors =
            resolve_unit(&unit_expr, &resolution_context).expect_err("resolve should fail");

        // check the result - should have an error
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].unit_name().as_str(), "unknown_unit");
    }

    #[test]
    fn unit_with_prefix_is_resolved() {
        // create a unit expression with a known prefix
        let unit_expr = test_ast::unit_node("km");

        // build the context
        let active_path = test_model_path("main");
        let mut external = test_external_context_with_common_units();
        let resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // resolve the unit
        let composite =
            resolve_unit(&unit_expr, &resolution_context).expect("resolve should succeed");

        // check the result
        assert_units_match(composite.units(), &[("km", 1.0)]);

        // check that the resolved info has the correct prefix
        let unit = &composite.units()[0];
        let ir::UnitInfo::Standard { prefix, base_name } = unit.info() else {
            panic!("unit should be a standard unit");
        };

        assert_eq!(prefix.as_ref().map(UnitPrefix::as_str), Some("k"));
        assert_eq!(base_name.as_str(), "m");
    }

    #[test]
    fn db_unit_is_resolved() {
        // create a unit expression for dB
        let unit_expr = test_ast::unit_node("dB");

        // build the context
        let active_path = test_model_path("main");
        let mut external = test_external_context_with_common_units();
        let resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // resolve the unit
        let composite =
            resolve_unit(&unit_expr, &resolution_context).expect("resolve should succeed");

        // check the result
        assert_units_match(composite.units(), &[("dB", 1.0)]);

        // check that the resolved info has is_db set
        let unit = &composite.units()[0];
        let ir::UnitInfo::Db { prefix, base_name } = unit.info() else {
            panic!("unit should be a decibel unit");
        };

        assert_eq!(prefix.as_ref(), None);
        assert_eq!(base_name.as_ref(), None);
    }

    #[test]
    fn db_unit_with_base_unit_is_resolved() {
        // create a unit expression for dBm (dB with meter base)
        let unit_expr = test_ast::unit_node("dBm");

        // build the context
        let active_path = test_model_path("main");
        let mut external = test_external_context_with_common_units();
        let resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // resolve the unit
        let composite =
            resolve_unit(&unit_expr, &resolution_context).expect("resolve should succeed");

        // check the result
        assert_units_match(composite.units(), &[("dBm", 1.0)]);

        // check that the resolved info has is_db set and the correct stripped name
        let unit = &composite.units()[0];
        let ir::UnitInfo::Db { prefix, base_name } = unit.info() else {
            panic!("unit should be a decibel unit");
        };

        assert_eq!(prefix.as_ref(), None);
        assert_eq!(base_name.as_ref().map(UnitBaseName::as_str), Some("m"));
    }
}
