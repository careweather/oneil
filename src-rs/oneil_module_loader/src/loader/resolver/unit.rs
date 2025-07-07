use oneil_ast as ast;

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
