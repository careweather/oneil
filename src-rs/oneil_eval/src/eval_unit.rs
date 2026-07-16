use oneil_ir as ir;
use oneil_shared::span::Span;

use oneil_output::{DisplayUnit, Unit};

use crate::context::{EvalContext, ExternalEvaluationContext};

/// Evaluates a composite unit and returns the resulting sized unit.
pub fn eval_unit<E: ExternalEvaluationContext>(
    unit: &ir::CompositeUnit,
    context: &EvalContext<'_, E>,
) -> (Unit, Span) {
    let unit_span = unit.span();

    let mut units = unit
        .units()
        .iter()
        .map(|unit| eval_unit_component(unit, context));

    // get the first unit
    let Some(first_unit) = units.next() else {
        return (Unit::one(), unit_span.clone());
    };

    // multiply the units together
    let mut result = first_unit;

    for unit in units {
        result = result * unit;
    }

    // evaluate the display unit based on the IR
    let display_info = eval_unit_display_expr(unit.display_unit());

    // construct the unit and return it
    let unit = result.with_unit_display_expr(display_info);
    (unit, unit_span.clone())
}

/// Evaluates a single unit component using its pre-resolved information.
///
/// The unit must have been resolved during the resolution phase. If the
/// resolved information is missing, this indicates an internal error
fn eval_unit_component<E: ExternalEvaluationContext>(
    unit: &ir::Unit,
    context: &EvalContext<'_, E>,
) -> Unit {
    let full_name = unit.name();
    let exponent = unit.exponent();

    let unit_display_expr = DisplayUnit::Unit {
        name: full_name.clone().into_string(),
        exponent,
    };

    let (prefix, base_name, is_db) = match unit.info() {
        ir::UnitInfo::Standard { prefix, base_name } => (prefix.as_ref(), Some(base_name), false),
        ir::UnitInfo::Db { prefix, base_name } => (prefix.as_ref(), base_name.as_ref(), true),
    };

    let base_unit = base_name.map_or_else(Unit::one, |name| {
        context
            .lookup_unit(name)
            .expect("base unit should exist in builtins")
    });

    let prefix_magnitude = prefix.map_or(1.0, |prefix| {
        context
            .lookup_prefix(prefix)
            .expect("prefix should exist in builtins")
    });

    base_unit
        .mul_magnitude(prefix_magnitude)
        .with_is_db_as(is_db)
        .pow(exponent)
        .with_unit_display_expr(unit_display_expr)
}

fn eval_unit_display_expr(unit: &ir::DisplayCompositeUnit) -> DisplayUnit {
    match unit {
        ir::DisplayCompositeUnit::BaseUnit(unit) => {
            let name = unit.name.clone();
            let exponent = unit.exponent;
            DisplayUnit::Unit { name, exponent }
        }
        ir::DisplayCompositeUnit::One => DisplayUnit::One,
        ir::DisplayCompositeUnit::Multiply(left, right) => {
            let left = eval_unit_display_expr(left);
            let right = eval_unit_display_expr(right);
            left * right
        }
        ir::DisplayCompositeUnit::Divide(left, right) => {
            let left = eval_unit_display_expr(left);
            let right = eval_unit_display_expr(right);
            left / right
        }
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use oneil_ir::test_helpers::unit::{UnitSpec, ir_composite_unit};
    use oneil_output::{Dimension, Unit};

    use crate::{
        check_is_close, check_unit_eq, context::EvalContext, test_context::TestExternalContext,
    };

    use super::*;

    /// Evaluates `specs` in a fresh context.
    fn eval_specs(specs: impl IntoIterator<Item = UnitSpec>) -> Unit {
        let mut external = TestExternalContext::new();
        let context = EvalContext::new(&mut external);
        eval_unit(&ir_composite_unit(specs), &context).0
    }

    #[track_caller]
    fn assert_eval_unit(
        name: &str,
        specs: &[UnitSpec],
        magnitude: f64,
        dims: &[(Dimension, f64)],
        is_db: bool,
    ) {
        let unit = eval_specs(specs.iter().copied());
        check_unit_eq(&unit, magnitude, dims, is_db).assert_with_name(name);
    }

    #[track_caller]
    fn assert_units_equivalent(name: &str, left: &[UnitSpec], right: &[UnitSpec]) {
        let left_unit = eval_specs(left.iter().copied());
        let right_unit = eval_specs(right.iter().copied());
        assert!(
            left_unit.numerically_eq(&right_unit),
            "{name}: units should be numerically equal\nleft={left_unit:?}\nright={right_unit:?}"
        );
    }

    #[test]
    fn eval_unitless() {
        let unit = eval_specs([]);
        assert!(unit.is_dimensionless(), "unit should be dimensionless");
    }

    #[test]
    #[expect(clippy::too_many_lines, clippy::type_complexity)]
    fn eval_units_table() {
        let cases: &[(&str, &[UnitSpec], f64, &[(Dimension, f64)], bool)] = &[
            (
                "simple",
                &[UnitSpec::new(None, Some("s"), false, 1.0)],
                1.0,
                &[(Dimension::Time, 1.0)],
                false,
            ),
            (
                "simple_with_prefix",
                &[UnitSpec::new(Some("m"), Some("s"), false, 1.0)],
                0.001,
                &[(Dimension::Time, 1.0)],
                false,
            ),
            (
                "simple_with_prefix_and_exponent",
                &[UnitSpec::new(Some("m"), Some("s"), false, 2.0)],
                0.001_f64.powi(2),
                &[(Dimension::Time, 2.0)],
                false,
            ),
            (
                "db",
                &[UnitSpec::new(None, None, true, 1.0)],
                1.0,
                &[],
                true,
            ),
            (
                "db_watts",
                &[UnitSpec::new(None, Some("W"), true, 1.0)],
                1.0,
                &[
                    (Dimension::Mass, 1.0),
                    (Dimension::Distance, 2.0),
                    (Dimension::Time, -3.0),
                ],
                true,
            ),
            (
                "db_watts_per_meter_squared_per_hertz",
                &[
                    UnitSpec::new(None, Some("W"), true, 1.0),
                    UnitSpec::new(None, Some("m"), false, -2.0),
                    UnitSpec::new(None, Some("Hz"), false, -1.0),
                ],
                1.0 / (2.0 * PI),
                &[(Dimension::Mass, 1.0), (Dimension::Time, -2.0)],
                true,
            ),
            (
                "kilometers",
                &[UnitSpec::new(Some("k"), Some("m"), false, 1.0)],
                1000.0,
                &[(Dimension::Distance, 1.0)],
                false,
            ),
            (
                "square_kilometers",
                &[UnitSpec::new(Some("k"), Some("m"), false, 2.0)],
                1000.0_f64.powi(2),
                &[(Dimension::Distance, 2.0)],
                false,
            ),
            (
                "gigahertz",
                &[UnitSpec::new(Some("G"), Some("Hz"), false, 1.0)],
                1e9 * (2.0 * PI),
                &[(Dimension::Time, -1.0)],
                false,
            ),
            (
                "kilohertz",
                &[UnitSpec::new(Some("k"), Some("Hz"), false, 1.0)],
                1e3 * (2.0 * PI),
                &[(Dimension::Time, -1.0)],
                false,
            ),
            (
                "megahertz",
                &[UnitSpec::new(Some("M"), Some("Hz"), false, 1.0)],
                1e6 * (2.0 * PI),
                &[(Dimension::Time, -1.0)],
                false,
            ),
            (
                "microseconds",
                &[UnitSpec::new(Some("u"), Some("s"), false, 1.0)],
                1e-6,
                &[(Dimension::Time, 1.0)],
                false,
            ),
            (
                "volts",
                &[UnitSpec::new(None, Some("V"), false, 1.0)],
                1.0,
                &[
                    (Dimension::Mass, 1.0),
                    (Dimension::Distance, 2.0),
                    (Dimension::Time, -3.0),
                    (Dimension::Current, -1.0),
                ],
                false,
            ),
            (
                "millivolts",
                &[UnitSpec::new(Some("m"), Some("V"), false, 1.0)],
                0.001,
                &[
                    (Dimension::Mass, 1.0),
                    (Dimension::Distance, 2.0),
                    (Dimension::Time, -3.0),
                    (Dimension::Current, -1.0),
                ],
                false,
            ),
            (
                "ohms",
                &[UnitSpec::new(None, Some("Ohm"), false, 1.0)],
                1.0,
                &[
                    (Dimension::Mass, 1.0),
                    (Dimension::Distance, 2.0),
                    (Dimension::Time, -3.0),
                    (Dimension::Current, -2.0),
                ],
                false,
            ),
            (
                "watts",
                &[UnitSpec::new(None, Some("W"), false, 1.0)],
                1.0,
                &[
                    (Dimension::Mass, 1.0),
                    (Dimension::Distance, 2.0),
                    (Dimension::Time, -3.0),
                ],
                false,
            ),
            (
                "watts_per_square_meter",
                &[
                    UnitSpec::new(None, Some("W"), false, 1.0),
                    UnitSpec::new(None, Some("m"), false, -2.0),
                ],
                1.0,
                &[(Dimension::Mass, 1.0), (Dimension::Time, -3.0)],
                false,
            ),
            (
                "kelvin",
                &[UnitSpec::new(None, Some("K"), false, 1.0)],
                1.0,
                &[(Dimension::Temperature, 1.0)],
                false,
            ),
            (
                "amperes",
                &[UnitSpec::new(None, Some("A"), false, 1.0)],
                1.0,
                &[(Dimension::Current, 1.0)],
                false,
            ),
            (
                "milliampere_hours",
                &[UnitSpec::new(Some("m"), Some("Ah"), false, 1.0)],
                3.6,
                &[(Dimension::Current, 1.0), (Dimension::Time, 1.0)],
                false,
            ),
            (
                "joules",
                &[UnitSpec::new(None, Some("J"), false, 1.0)],
                1.0,
                &[
                    (Dimension::Mass, 1.0),
                    (Dimension::Distance, 2.0),
                    (Dimension::Time, -2.0),
                ],
                false,
            ),
            (
                "hours",
                &[UnitSpec::new(None, Some("hr"), false, 1.0)],
                3600.0,
                &[(Dimension::Time, 1.0)],
                false,
            ),
            (
                "minutes",
                &[UnitSpec::new(None, Some("min"), false, 1.0)],
                60.0,
                &[(Dimension::Time, 1.0)],
                false,
            ),
            (
                "revolutions_per_minute",
                &[UnitSpec::new(None, Some("rpm"), false, 1.0)],
                2.0 * PI / 60.0,
                &[(Dimension::Time, -1.0)],
                false,
            ),
            (
                "degrees",
                &[UnitSpec::new(None, Some("deg"), false, 1.0)],
                PI / 180.0,
                &[],
                false,
            ),
            (
                "percent",
                &[UnitSpec::new(None, Some("%"), false, 1.0)],
                0.01,
                &[],
                false,
            ),
            (
                "megabits_per_second",
                &[UnitSpec::new(Some("M"), Some("bps"), false, 1.0)],
                1e6,
                &[(Dimension::Information, 1.0), (Dimension::Time, -1.0)],
                false,
            ),
            (
                "kilobytes",
                &[UnitSpec::new(Some("k"), Some("B"), false, 1.0)],
                8000.0,
                &[(Dimension::Information, 1.0)],
                false,
            ),
            (
                "boltzmann_constant_unit",
                &[
                    UnitSpec::new(None, Some("m"), false, 2.0),
                    UnitSpec::new(Some("k"), Some("g"), false, 1.0),
                    UnitSpec::new(None, Some("s"), false, -2.0),
                    UnitSpec::new(None, Some("K"), false, -1.0),
                ],
                1.0,
                &[
                    (Dimension::Distance, 2.0),
                    (Dimension::Mass, 1.0),
                    (Dimension::Time, -2.0),
                    (Dimension::Temperature, -1.0),
                ],
                false,
            ),
            (
                "meters_per_second",
                &[
                    UnitSpec::new(None, Some("m"), false, 1.0),
                    UnitSpec::new(None, Some("s"), false, -1.0),
                ],
                1.0,
                &[(Dimension::Distance, 1.0), (Dimension::Time, -1.0)],
                false,
            ),
            (
                "meters_per_second_squared",
                &[
                    UnitSpec::new(None, Some("m"), false, 1.0),
                    UnitSpec::new(None, Some("s"), false, -2.0),
                ],
                1.0,
                &[(Dimension::Distance, 1.0), (Dimension::Time, -2.0)],
                false,
            ),
        ];

        for (name, specs, magnitude, dims, is_db) in cases {
            assert_eval_unit(name, specs, *magnitude, dims, *is_db);
        }
    }

    #[test]
    #[expect(clippy::too_many_lines)]
    fn eval_unit_equivalence_table() {
        let cases: &[(&str, &[UnitSpec], &[UnitSpec])] = &[
            (
                "newtons_are_kg_m_s_2",
                &[UnitSpec::new(None, Some("N"), false, 1.0)],
                &[
                    UnitSpec::new(Some("k"), Some("g"), false, 1.0),
                    UnitSpec::new(None, Some("m"), false, 1.0),
                    UnitSpec::new(None, Some("s"), false, -2.0),
                ],
            ),
            (
                "joules_are_newton_meters",
                &[UnitSpec::new(None, Some("J"), false, 1.0)],
                &[
                    UnitSpec::new(None, Some("N"), false, 1.0),
                    UnitSpec::new(None, Some("m"), false, 1.0),
                ],
            ),
            (
                "joules_are_kg_m2_s2",
                &[UnitSpec::new(None, Some("J"), false, 1.0)],
                &[
                    UnitSpec::new(Some("k"), Some("g"), false, 1.0),
                    UnitSpec::new(None, Some("m"), false, 2.0),
                    UnitSpec::new(None, Some("s"), false, -2.0),
                ],
            ),
            (
                "watts_are_joules_per_second",
                &[UnitSpec::new(None, Some("W"), false, 1.0)],
                &[
                    UnitSpec::new(None, Some("J"), false, 1.0),
                    UnitSpec::new(None, Some("s"), false, -1.0),
                ],
            ),
            (
                "watts_are_newton_meters_per_second",
                &[UnitSpec::new(None, Some("W"), false, 1.0)],
                &[
                    UnitSpec::new(None, Some("N"), false, 1.0),
                    UnitSpec::new(None, Some("m"), false, 1.0),
                    UnitSpec::new(None, Some("s"), false, -1.0),
                ],
            ),
            (
                "watts_are_kg_m2_s3",
                &[UnitSpec::new(None, Some("W"), false, 1.0)],
                &[
                    UnitSpec::new(Some("k"), Some("g"), false, 1.0),
                    UnitSpec::new(None, Some("m"), false, 2.0),
                    UnitSpec::new(None, Some("s"), false, -3.0),
                ],
            ),
            (
                "volts_are_watts_per_ampere",
                &[UnitSpec::new(None, Some("V"), false, 1.0)],
                &[
                    UnitSpec::new(None, Some("W"), false, 1.0),
                    UnitSpec::new(None, Some("A"), false, -1.0),
                ],
            ),
            (
                "volts_are_kg_m2_s3_a",
                &[UnitSpec::new(None, Some("V"), false, 1.0)],
                &[
                    UnitSpec::new(Some("k"), Some("g"), false, 1.0),
                    UnitSpec::new(None, Some("m"), false, 2.0),
                    UnitSpec::new(None, Some("s"), false, -3.0),
                    UnitSpec::new(None, Some("A"), false, -1.0),
                ],
            ),
            (
                "ohms_are_volts_per_ampere",
                &[UnitSpec::new(None, Some("Ohm"), false, 1.0)],
                &[
                    UnitSpec::new(None, Some("V"), false, 1.0),
                    UnitSpec::new(None, Some("A"), false, -1.0),
                ],
            ),
            (
                "ohms_are_kg_m2_s3_a2",
                &[UnitSpec::new(None, Some("Ohm"), false, 1.0)],
                &[
                    UnitSpec::new(Some("k"), Some("g"), false, 1.0),
                    UnitSpec::new(None, Some("m"), false, 2.0),
                    UnitSpec::new(None, Some("s"), false, -3.0),
                    UnitSpec::new(None, Some("A"), false, -2.0),
                ],
            ),
            (
                "pascals_are_newtons_per_square_meter",
                &[UnitSpec::new(None, Some("Pa"), false, 1.0)],
                &[
                    UnitSpec::new(None, Some("N"), false, 1.0),
                    UnitSpec::new(None, Some("m"), false, -2.0),
                ],
            ),
            (
                "pascals_are_kg_m_s2",
                &[UnitSpec::new(None, Some("Pa"), false, 1.0)],
                &[
                    UnitSpec::new(Some("k"), Some("g"), false, 1.0),
                    UnitSpec::new(None, Some("m"), false, -1.0),
                    UnitSpec::new(None, Some("s"), false, -2.0),
                ],
            ),
            (
                "watt_hours_are_watts_times_hours",
                &[UnitSpec::new(None, Some("Wh"), false, 1.0)],
                &[
                    UnitSpec::new(None, Some("W"), false, 1.0),
                    UnitSpec::new(None, Some("hr"), false, 1.0),
                ],
            ),
            (
                "amp_hours_are_amperes_times_hours",
                &[UnitSpec::new(None, Some("Ah"), false, 1.0)],
                &[
                    UnitSpec::new(None, Some("A"), false, 1.0),
                    UnitSpec::new(None, Some("hr"), false, 1.0),
                ],
            ),
            (
                "tesla_are_kg_s2_a",
                &[UnitSpec::new(None, Some("T"), false, 1.0)],
                &[
                    UnitSpec::new(Some("k"), Some("g"), false, 1.0),
                    UnitSpec::new(None, Some("s"), false, -2.0),
                    UnitSpec::new(None, Some("A"), false, -1.0),
                ],
            ),
        ];

        for (name, left, right) in cases {
            assert_units_equivalent(name, left, right);
        }
    }

    #[test]
    fn eval_hertz_are_per_second() {
        let hertz_unit = eval_specs([UnitSpec::new(None, Some("Hz"), false, 1.0)]);
        let per_second_unit = eval_specs([UnitSpec::new(None, Some("s"), false, -1.0)]);

        check_is_close(per_second_unit.magnitude, hertz_unit.magnitude / (2.0 * PI)).assert();
        assert!(hertz_unit.dimensionally_eq(&per_second_unit));
        assert!(!hertz_unit.is_db);
        assert!(!per_second_unit.is_db);
    }
}
