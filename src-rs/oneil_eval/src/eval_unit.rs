use oneil_ir as ir;
use oneil_shared::span::Span;

use oneil_output::{DisplayUnit, Unit};

use crate::context::{EvalContext, ExternalEvaluationContext};

/// Evaluates a composite unit and returns the resulting sized unit.
///
/// Built for uses outside of this crate, where an `EvalContext` does
/// not exist.
pub fn eval_unit_external<E: ExternalEvaluationContext>(
    unit: &ir::CompositeUnit,
    context: &mut E,
) -> Unit {
    // we don't need any pre-loaded models, so we can just use a new context
    let eval_context = EvalContext::new(context);

    eval_unit(unit, &eval_context).0
}

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
        return (Unit::one(), unit_span);
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
    (unit, unit_span)
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
        name: full_name.to_string(),
        exponent,
    };

    let (prefix, base_name, is_db) = match unit.info() {
        ir::UnitInfo::Standard { prefix, base_name } => {
            (prefix.as_deref(), Some(base_name.as_str()), false)
        }
        ir::UnitInfo::Db { prefix, base_name } => (prefix.as_deref(), base_name.as_deref(), true),
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
mod test {
    use std::f64::consts::PI;

    use oneil_output::Dimension;
    use oneil_shared::span::SourceLocation;

    use crate::{
        assert_is_close, assert_units_dimensionally_eq, context::EvalContext,
        test_context::TestExternalContext,
    };

    use super::*;

    /// Returns a dummy span for use in test parameters.
    ///
    /// This function creates a span with all fields set to zero.
    /// It is not intended to be directly tested, but rather used
    /// as a placeholder when constructing IR nodes for testing.
    fn random_span() -> Span {
        let start = SourceLocation {
            offset: 0,
            line: 0,
            column: 0,
        };
        let end = SourceLocation {
            offset: 0,
            line: 0,
            column: 0,
        };
        Span::new(start, end)
    }

    /// Returns a display unit that isn't intended to be tested.
    fn unimportant_display_unit() -> ir::DisplayCompositeUnit {
        ir::DisplayCompositeUnit::BaseUnit(ir::DisplayUnit::new("unimportant".to_string(), 1.0))
    }

    /// Specification for a unit in tests.
    #[derive(Debug, Clone, Copy)]
    struct UnitSpec {
        /// The base unit name (e.g., "m", "s", "W"). Use `None` for pure dB.
        base_name: Option<&'static str>,
        /// Optional SI prefix (e.g., "k" for kilo, "m" for milli).
        prefix: Option<&'static str>,
        /// Whether this is a decibel unit.
        is_db: bool,
        /// The exponent of the unit.
        exponent: f64,
    }

    impl UnitSpec {
        const fn new(
            base_name: Option<&'static str>,
            prefix: Option<&'static str>,
            is_db: bool,
            exponent: f64,
        ) -> Self {
            Self {
                base_name,
                prefix,
                is_db,
                exponent,
            }
        }
    }

    fn ir_composite_unit(unit_list: impl IntoIterator<Item = UnitSpec>) -> ir::CompositeUnit {
        let unit_vec = unit_list
            .into_iter()
            .map(|spec| {
                let full_name = build_full_name(spec.base_name, spec.prefix, spec.is_db);
                let info = build_unit_info(spec.base_name, spec.prefix, spec.is_db);
                ir::Unit::new(
                    random_span(),
                    full_name,
                    random_span(),
                    spec.exponent,
                    None,
                    info,
                )
            })
            .collect::<Vec<_>>();
        ir::CompositeUnit::new(unit_vec, unimportant_display_unit(), random_span())
    }

    fn build_full_name(base_name: Option<&str>, prefix: Option<&str>, is_db: bool) -> String {
        format!(
            "{}{}{}",
            if is_db { "dB" } else { "" },
            prefix.unwrap_or(""),
            base_name.unwrap_or("")
        )
    }

    fn build_unit_info(base_name: Option<&str>, prefix: Option<&str>, is_db: bool) -> ir::UnitInfo {
        if is_db {
            ir::UnitInfo::Db {
                prefix: prefix.map(String::from),
                base_name: base_name.map(String::from),
            }
        } else {
            ir::UnitInfo::Standard {
                prefix: prefix.map(String::from),
                base_name: base_name.unwrap_or("").to_string(),
            }
        }
    }

    mod unit_eval {

        use super::*;

        #[test]
        fn eval_unitless() {
            // setup unit and context
            let ir_unit = ir_composite_unit([]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            // evaluate unit
            let (unit, _unit_span) = eval_unit(&ir_unit, &context);

            assert!(unit.is_dimensionless(), "unit should be dimensionless");
        }

        #[test]
        fn eval_simple() {
            // setup unit and context
            let ir_unit = ir_composite_unit([UnitSpec::new(Some("s"), None, false, 1.0)]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            // evaluate unit
            let (unit, _unit_span) = eval_unit(&ir_unit, &context);

            // check unit
            assert_is_close!(1.0, unit.magnitude);
            assert_units_dimensionally_eq!([(Dimension::Time, 1.0)], unit);
            assert!(!unit.is_db);
        }

        #[test]
        fn eval_simple_with_prefix() {
            // setup unit and context
            let ir_unit = ir_composite_unit([UnitSpec::new(Some("s"), Some("m"), false, 1.0)]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            // evaluate unit
            let (unit, _unit_span) = eval_unit(&ir_unit, &context);

            // check sized unit
            assert_is_close!(0.001, unit.magnitude);
            assert_units_dimensionally_eq!([(Dimension::Time, 1.0)], unit);
            assert!(!unit.is_db);
        }

        #[test]
        fn eval_simple_with_prefix_and_exponent() {
            // setup unit and context
            let ir_unit = ir_composite_unit([UnitSpec::new(Some("s"), Some("m"), false, 2.0)]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            // evaluate unit
            let (unit, _unit_span) = eval_unit(&ir_unit, &context);

            // check sized unit
            assert_is_close!(0.001_f64.powi(2), unit.magnitude);
            assert_units_dimensionally_eq!([(Dimension::Time, 2.0)], unit);
            assert!(!unit.is_db);
        }

        #[test]
        fn eval_db() {
            // setup unit and context
            let ir_unit = ir_composite_unit([UnitSpec::new(None, None, true, 1.0)]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            // evaluate unit
            let (unit, _unit_span) = eval_unit(&ir_unit, &context);

            // check sized unit
            assert_is_close!(1.0, unit.magnitude);
            assert_units_dimensionally_eq!([], unit);
            assert!(unit.is_db);
        }

        #[test]
        fn eval_db_watts() {
            // setup unit and context
            let ir_unit = ir_composite_unit([UnitSpec::new(Some("W"), None, true, 1.0)]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            // evaluate unit
            let (unit, _unit_span) = eval_unit(&ir_unit, &context);

            // check sized unit
            assert_is_close!(1.0, unit.magnitude);
            assert_units_dimensionally_eq!(
                [
                    (Dimension::Mass, 1.0),
                    (Dimension::Distance, 2.0),
                    (Dimension::Time, -3.0)
                ],
                unit
            );
            assert!(unit.is_db);
        }

        #[test]
        fn eval_db_watts_per_meter_squared_per_hertz() {
            // setup unit and context
            let ir_unit = ir_composite_unit([
                UnitSpec::new(Some("W"), None, true, 1.0),
                UnitSpec::new(Some("m"), None, false, -2.0),
                UnitSpec::new(Some("Hz"), None, false, -1.0),
            ]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            // evaluate unit
            let (unit, _unit_span) = eval_unit(&ir_unit, &context);

            // check sized unit
            // dBW: Mass, Distance^2, Time^-3
            // m^-2: Distance^-2
            // Hz^-1: Time^1 (since Hz has Time^-1)
            // Result: Mass, Time^-2
            // Magnitude: 1 / (2π) because Hz has magnitude 2π, so Hz^-1 contributes 1/(2π)
            assert_is_close!(1.0 / (2.0 * PI), unit.magnitude);
            assert_units_dimensionally_eq!([(Dimension::Mass, 1.0), (Dimension::Time, -2.0)], unit);
            assert!(unit.is_db);
        }

        #[test]
        fn eval_kilometers() {
            // setup unit and context
            let ir_unit = ir_composite_unit([UnitSpec::new(Some("m"), Some("k"), false, 1.0)]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            // evaluate unit
            let (unit, _unit_span) = eval_unit(&ir_unit, &context);

            // check sized unit
            assert_is_close!(1000.0, unit.magnitude);
            assert_units_dimensionally_eq!([(Dimension::Distance, 1.0)], unit);
            assert!(!unit.is_db);
        }

        #[test]
        fn eval_square_kilometers() {
            // setup unit and context
            let ir_unit = ir_composite_unit([UnitSpec::new(Some("m"), Some("k"), false, 2.0)]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            // evaluate unit
            let (unit, _unit_span) = eval_unit(&ir_unit, &context);

            // check sized unit
            assert_is_close!(1000.0_f64.powi(2), unit.magnitude);
            assert_units_dimensionally_eq!([(Dimension::Distance, 2.0)], unit);
            assert!(!unit.is_db);
        }

        #[test]
        fn eval_gigahertz() {
            // setup unit and context
            let ir_unit = ir_composite_unit([UnitSpec::new(Some("Hz"), Some("G"), false, 1.0)]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            // evaluate unit
            let (unit, _unit_span) = eval_unit(&ir_unit, &context);

            // check sized unit
            // Hz has magnitude 2π, so GHz = 1e9 * 2π
            assert_is_close!(1e9 * (2.0 * PI), unit.magnitude);
            assert_units_dimensionally_eq!([(Dimension::Time, -1.0)], unit);
            assert!(!unit.is_db);
        }

        #[test]
        fn eval_kilohertz() {
            // setup unit and context
            let ir_unit = ir_composite_unit([UnitSpec::new(Some("Hz"), Some("k"), false, 1.0)]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            // evaluate unit
            let (unit, _unit_span) = eval_unit(&ir_unit, &context);

            // check sized unit
            // Hz has magnitude 2π, so kHz = 1e3 * 2π
            assert_is_close!(1e3 * (2.0 * PI), unit.magnitude);
            assert_units_dimensionally_eq!([(Dimension::Time, -1.0)], unit);
            assert!(!unit.is_db);
        }

        #[test]
        fn eval_megahertz() {
            // setup unit and context
            let ir_unit = ir_composite_unit([UnitSpec::new(Some("Hz"), Some("M"), false, 1.0)]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            // evaluate unit
            let (unit, _unit_span) = eval_unit(&ir_unit, &context);

            // check sized unit
            // Hz has magnitude 2π, so MHz = 1e6 * 2π
            assert_is_close!(1e6 * (2.0 * PI), unit.magnitude);
            assert_units_dimensionally_eq!([(Dimension::Time, -1.0)], unit);
            assert!(!unit.is_db);
        }

        #[test]
        fn eval_microseconds() {
            // setup unit and context
            let ir_unit = ir_composite_unit([UnitSpec::new(Some("s"), Some("u"), false, 1.0)]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            // evaluate unit
            let (unit, _unit_span) = eval_unit(&ir_unit, &context);

            // check sized unit
            assert_is_close!(1e-6, unit.magnitude);
            assert_units_dimensionally_eq!([(Dimension::Time, 1.0)], unit);
            assert!(!unit.is_db);
        }

        #[test]
        fn eval_volts() {
            // setup unit and context
            let ir_unit = ir_composite_unit([UnitSpec::new(Some("V"), None, false, 1.0)]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            // evaluate unit
            let (unit, _unit_span) = eval_unit(&ir_unit, &context);

            // check sized unit
            assert_is_close!(1.0, unit.magnitude);
            assert_units_dimensionally_eq!(
                [
                    (Dimension::Mass, 1.0),
                    (Dimension::Distance, 2.0),
                    (Dimension::Time, -3.0),
                    (Dimension::Current, -1.0)
                ],
                unit
            );
            assert!(!unit.is_db);
        }

        #[test]
        fn eval_millivolts() {
            // setup unit and context
            let ir_unit = ir_composite_unit([UnitSpec::new(Some("V"), Some("m"), false, 1.0)]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            // evaluate unit
            let (unit, _unit_span) = eval_unit(&ir_unit, &context);

            // check sized unit
            assert_is_close!(0.001, unit.magnitude);
            assert_units_dimensionally_eq!(
                [
                    (Dimension::Mass, 1.0),
                    (Dimension::Distance, 2.0),
                    (Dimension::Time, -3.0),
                    (Dimension::Current, -1.0)
                ],
                unit
            );
            assert!(!unit.is_db);
        }

        #[test]
        fn eval_ohms() {
            // setup unit and context
            let ir_unit = ir_composite_unit([UnitSpec::new(Some("Ohm"), None, false, 1.0)]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            // evaluate unit
            let (unit, _unit_span) = eval_unit(&ir_unit, &context);

            // check sized unit
            assert_is_close!(1.0, unit.magnitude);
            assert_units_dimensionally_eq!(
                [
                    (Dimension::Mass, 1.0),
                    (Dimension::Distance, 2.0),
                    (Dimension::Time, -3.0),
                    (Dimension::Current, -2.0)
                ],
                unit
            );
            assert!(!unit.is_db);
        }

        #[test]
        fn eval_watts() {
            // setup unit and context
            let ir_unit = ir_composite_unit([UnitSpec::new(Some("W"), None, false, 1.0)]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            // evaluate unit
            let (unit, _unit_span) = eval_unit(&ir_unit, &context);

            // check sized unit
            assert_is_close!(1.0, unit.magnitude);
            assert_units_dimensionally_eq!(
                [
                    (Dimension::Mass, 1.0),
                    (Dimension::Distance, 2.0),
                    (Dimension::Time, -3.0)
                ],
                unit
            );
            assert!(!unit.is_db);
        }

        #[test]
        fn eval_watts_per_square_meter() {
            // setup unit and context
            let ir_unit = ir_composite_unit([
                UnitSpec::new(Some("W"), None, false, 1.0),
                UnitSpec::new(Some("m"), None, false, -2.0),
            ]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            // evaluate unit
            let (unit, _unit_span) = eval_unit(&ir_unit, &context);

            // check sized unit
            assert_is_close!(1.0, unit.magnitude);
            assert_units_dimensionally_eq!([(Dimension::Mass, 1.0), (Dimension::Time, -3.0)], unit);
            assert!(!unit.is_db);
        }

        #[test]
        fn eval_kelvin() {
            // setup unit and context
            let ir_unit = ir_composite_unit([UnitSpec::new(Some("K"), None, false, 1.0)]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            // evaluate unit
            let (unit, _unit_span) = eval_unit(&ir_unit, &context);

            // check sized unit
            assert_is_close!(1.0, unit.magnitude);
            assert_units_dimensionally_eq!([(Dimension::Temperature, 1.0)], unit);
            assert!(!unit.is_db);
        }

        #[test]
        fn eval_amperes() {
            // setup unit and context
            let ir_unit = ir_composite_unit([UnitSpec::new(Some("A"), None, false, 1.0)]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            // evaluate unit
            let (unit, _unit_span) = eval_unit(&ir_unit, &context);

            // check sized unit
            assert_is_close!(1.0, unit.magnitude);
            assert_units_dimensionally_eq!([(Dimension::Current, 1.0)], unit);
            assert!(!unit.is_db);
        }

        #[test]
        fn eval_milliampere_hours() {
            // setup unit and context
            let ir_unit = ir_composite_unit([UnitSpec::new(Some("Ah"), Some("m"), false, 1.0)]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            // evaluate unit
            let (unit, _unit_span) = eval_unit(&ir_unit, &context);

            // check sized unit
            // mAh = 0.001 A * 3600 s = 3.6 A*s
            assert_is_close!(3.6, unit.magnitude);
            assert_units_dimensionally_eq!(
                [(Dimension::Current, 1.0), (Dimension::Time, 1.0)],
                unit
            );
            assert!(!unit.is_db);
        }

        #[test]
        fn eval_joules() {
            // setup unit and context
            let ir_unit = ir_composite_unit([UnitSpec::new(Some("J"), None, false, 1.0)]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            // evaluate unit
            let (unit, _unit_span) = eval_unit(&ir_unit, &context);

            // check sized unit
            assert_is_close!(1.0, unit.magnitude);
            assert_units_dimensionally_eq!(
                [
                    (Dimension::Mass, 1.0),
                    (Dimension::Distance, 2.0),
                    (Dimension::Time, -2.0)
                ],
                unit
            );
            assert!(!unit.is_db);
        }

        #[test]
        fn eval_hours() {
            // setup unit and context
            let ir_unit = ir_composite_unit([UnitSpec::new(Some("hr"), None, false, 1.0)]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            // evaluate unit
            let (unit, _unit_span) = eval_unit(&ir_unit, &context);

            // check sized unit
            // hr = 3600 s
            assert_is_close!(3600.0, unit.magnitude);
            assert_units_dimensionally_eq!([(Dimension::Time, 1.0)], unit);
            assert!(!unit.is_db);
        }

        #[test]
        fn eval_minutes() {
            // setup unit and context
            let ir_unit = ir_composite_unit([UnitSpec::new(Some("min"), None, false, 1.0)]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            // evaluate unit
            let (unit, _unit_span) = eval_unit(&ir_unit, &context);

            // check sized unit
            // min = 60 s
            assert_is_close!(60.0, unit.magnitude);
            assert_units_dimensionally_eq!([(Dimension::Time, 1.0)], unit);
            assert!(!unit.is_db);
        }

        #[test]
        fn eval_revolutions_per_minute() {
            // setup unit and context
            let ir_unit = ir_composite_unit([UnitSpec::new(Some("rpm"), None, false, 1.0)]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            // evaluate unit
            let (unit, _unit_span) = eval_unit(&ir_unit, &context);

            // check sized unit
            // rpm has magnitude 2π/60 (radians per second)
            assert_is_close!(2.0 * PI / 60.0, unit.magnitude);
            assert_units_dimensionally_eq!([(Dimension::Time, -1.0)], unit);
            assert!(!unit.is_db);
        }

        #[test]
        fn eval_degrees() {
            // setup unit and context
            let ir_unit = ir_composite_unit([UnitSpec::new(Some("deg"), None, false, 1.0)]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            // evaluate unit
            let (unit, _unit_span) = eval_unit(&ir_unit, &context);

            // check sized unit
            // deg is dimensionless with magnitude π/180 (conversion to radians)
            assert_is_close!(PI / 180.0, unit.magnitude);
            assert_units_dimensionally_eq!([], unit);
            assert!(!unit.is_db);
        }

        #[test]
        fn eval_percent() {
            // setup unit and context
            let ir_unit = ir_composite_unit([UnitSpec::new(Some("%"), None, false, 1.0)]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            // evaluate unit
            let (unit, _unit_span) = eval_unit(&ir_unit, &context);

            // check sized unit
            // % is dimensionless with magnitude 0.01
            assert_is_close!(0.01, unit.magnitude);
            assert_units_dimensionally_eq!([], unit);
            assert!(!unit.is_db);
        }

        #[test]
        fn eval_megabits_per_second() {
            // setup unit and context
            let ir_unit = ir_composite_unit([UnitSpec::new(Some("bps"), Some("M"), false, 1.0)]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            // evaluate unit
            let (unit, _unit_span) = eval_unit(&ir_unit, &context);

            // check sized unit
            // Mbps = 1e6 * bps, and bps has Information*Time^-1 dimension
            assert_is_close!(1e6, unit.magnitude);
            assert_units_dimensionally_eq!(
                [(Dimension::Information, 1.0), (Dimension::Time, -1.0)],
                unit
            );
            assert!(!unit.is_db);
        }

        #[test]
        fn eval_kilobytes() {
            // setup unit and context
            let ir_unit = ir_composite_unit([UnitSpec::new(Some("B"), Some("k"), false, 1.0)]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            // evaluate unit
            let (unit, _unit_span) = eval_unit(&ir_unit, &context);

            // check sized unit
            // B has magnitude 8 (bits), so kB = 1000 * 8 = 8000 bits
            assert_is_close!(8000.0, unit.magnitude);
            assert_units_dimensionally_eq!([(Dimension::Information, 1.0)], unit);
            assert!(!unit.is_db);
        }

        #[test]
        fn eval_boltzmann_constant_unit() {
            // setup unit and context
            // m^2*kg/s^2/K - the unit of Boltzmann's constant
            let ir_unit = ir_composite_unit([
                UnitSpec::new(Some("m"), None, false, 2.0),
                UnitSpec::new(Some("g"), Some("k"), false, 1.0),
                UnitSpec::new(Some("s"), None, false, -2.0),
                UnitSpec::new(Some("K"), None, false, -1.0),
            ]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            // evaluate unit
            let (unit, _unit_span) = eval_unit(&ir_unit, &context);

            // check sized unit
            // kg is the base unit (magnitude 1), not g
            assert_is_close!(1.0, unit.magnitude);
            assert_units_dimensionally_eq!(
                [
                    (Dimension::Distance, 2.0),
                    (Dimension::Mass, 1.0),
                    (Dimension::Time, -2.0),
                    (Dimension::Temperature, -1.0)
                ],
                unit
            );
            assert!(!unit.is_db);
        }

        #[test]
        fn eval_meters_per_second() {
            // setup unit and context
            let ir_unit = ir_composite_unit([
                UnitSpec::new(Some("m"), None, false, 1.0),
                UnitSpec::new(Some("s"), None, false, -1.0),
            ]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            // evaluate unit
            let (unit, _unit_span) = eval_unit(&ir_unit, &context);

            // check sized unit
            assert_is_close!(1.0, unit.magnitude);
            assert_units_dimensionally_eq!(
                [(Dimension::Distance, 1.0), (Dimension::Time, -1.0)],
                unit
            );
            assert!(!unit.is_db);
        }

        #[test]
        fn eval_meters_per_second_squared() {
            // setup unit and context
            let ir_unit = ir_composite_unit([
                UnitSpec::new(Some("m"), None, false, 1.0),
                UnitSpec::new(Some("s"), None, false, -2.0),
            ]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            // evaluate unit
            let (unit, _unit_span) = eval_unit(&ir_unit, &context);

            // check sized unit
            assert_is_close!(1.0, unit.magnitude);
            assert_units_dimensionally_eq!(
                [(Dimension::Distance, 1.0), (Dimension::Time, -2.0)],
                unit
            );
            assert!(!unit.is_db);
        }
    }

    mod unit_equivalence {
        use super::*;

        #[test]
        fn eval_newtons_are_kg_m_s_2() {
            let newton_unit = ir_composite_unit([UnitSpec::new(Some("N"), None, false, 1.0)]);
            let kg_m_s_2_unit = ir_composite_unit([
                UnitSpec::new(Some("g"), Some("k"), false, 1.0),
                UnitSpec::new(Some("m"), None, false, 1.0),
                UnitSpec::new(Some("s"), None, false, -2.0),
            ]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            let (newton_unit, _) = eval_unit(&newton_unit, &context);
            let (kg_m_s_2_unit, _) = eval_unit(&kg_m_s_2_unit, &context);

            assert!(newton_unit.numerically_eq(&kg_m_s_2_unit));
        }

        #[test]
        fn eval_joules_are_newton_meters() {
            let joule_unit = ir_composite_unit([UnitSpec::new(Some("J"), None, false, 1.0)]);
            let newton_meter_unit = ir_composite_unit([
                UnitSpec::new(Some("N"), None, false, 1.0),
                UnitSpec::new(Some("m"), None, false, 1.0),
            ]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            let (joule_unit, _) = eval_unit(&joule_unit, &context);
            let (newton_meter_unit, _) = eval_unit(&newton_meter_unit, &context);

            assert!(joule_unit.numerically_eq(&newton_meter_unit));
        }

        #[test]
        fn eval_joules_are_kg_m2_s2() {
            let joule_unit = ir_composite_unit([UnitSpec::new(Some("J"), None, false, 1.0)]);
            let kg_m2_s2_unit = ir_composite_unit([
                UnitSpec::new(Some("g"), Some("k"), false, 1.0),
                UnitSpec::new(Some("m"), None, false, 2.0),
                UnitSpec::new(Some("s"), None, false, -2.0),
            ]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            let (joule_unit, _) = eval_unit(&joule_unit, &context);
            let (kg_m2_s2_unit, _) = eval_unit(&kg_m2_s2_unit, &context);

            assert!(joule_unit.numerically_eq(&kg_m2_s2_unit));
        }

        #[test]
        fn eval_watts_are_joules_per_second() {
            let watt_unit = ir_composite_unit([UnitSpec::new(Some("W"), None, false, 1.0)]);
            let joule_per_second_unit = ir_composite_unit([
                UnitSpec::new(Some("J"), None, false, 1.0),
                UnitSpec::new(Some("s"), None, false, -1.0),
            ]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            let (watt_unit, _) = eval_unit(&watt_unit, &context);
            let (joule_per_second_unit, _) = eval_unit(&joule_per_second_unit, &context);

            assert!(watt_unit.numerically_eq(&joule_per_second_unit));
        }

        #[test]
        fn eval_watts_are_newton_meters_per_second() {
            let watt_unit = ir_composite_unit([UnitSpec::new(Some("W"), None, false, 1.0)]);
            let newton_meter_per_second_unit = ir_composite_unit([
                UnitSpec::new(Some("N"), None, false, 1.0),
                UnitSpec::new(Some("m"), None, false, 1.0),
                UnitSpec::new(Some("s"), None, false, -1.0),
            ]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            let (watt_unit, _) = eval_unit(&watt_unit, &context);
            let (newton_meter_per_second_unit, _) =
                eval_unit(&newton_meter_per_second_unit, &context);

            assert!(watt_unit.numerically_eq(&newton_meter_per_second_unit));
        }

        #[test]
        fn eval_watts_are_kg_m2_s3() {
            let watt_unit = ir_composite_unit([UnitSpec::new(Some("W"), None, false, 1.0)]);
            let kg_m2_s3_unit = ir_composite_unit([
                UnitSpec::new(Some("g"), Some("k"), false, 1.0),
                UnitSpec::new(Some("m"), None, false, 2.0),
                UnitSpec::new(Some("s"), None, false, -3.0),
            ]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            let (watt_unit, _) = eval_unit(&watt_unit, &context);
            let (kg_m2_s3_unit, _) = eval_unit(&kg_m2_s3_unit, &context);

            assert!(watt_unit.numerically_eq(&kg_m2_s3_unit));
        }

        #[test]
        fn eval_volts_are_watts_per_ampere() {
            let volt_unit = ir_composite_unit([UnitSpec::new(Some("V"), None, false, 1.0)]);
            let watt_per_ampere_unit = ir_composite_unit([
                UnitSpec::new(Some("W"), None, false, 1.0),
                UnitSpec::new(Some("A"), None, false, -1.0),
            ]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            let (volt_unit, _) = eval_unit(&volt_unit, &context);
            let (watt_per_ampere_unit, _) = eval_unit(&watt_per_ampere_unit, &context);

            assert!(volt_unit.numerically_eq(&watt_per_ampere_unit));
        }

        #[test]
        fn eval_volts_are_kg_m2_s3_a() {
            let volt_unit = ir_composite_unit([UnitSpec::new(Some("V"), None, false, 1.0)]);
            let kg_m2_s3_a_unit = ir_composite_unit([
                UnitSpec::new(Some("g"), Some("k"), false, 1.0),
                UnitSpec::new(Some("m"), None, false, 2.0),
                UnitSpec::new(Some("s"), None, false, -3.0),
                UnitSpec::new(Some("A"), None, false, -1.0),
            ]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            let (volt_unit, _) = eval_unit(&volt_unit, &context);
            let (kg_m2_s3_a_unit, _) = eval_unit(&kg_m2_s3_a_unit, &context);

            assert!(volt_unit.numerically_eq(&kg_m2_s3_a_unit));
        }

        #[test]
        fn eval_ohms_are_volts_per_ampere() {
            let ohm_unit = ir_composite_unit([UnitSpec::new(Some("Ohm"), None, false, 1.0)]);
            let volt_per_ampere_unit = ir_composite_unit([
                UnitSpec::new(Some("V"), None, false, 1.0),
                UnitSpec::new(Some("A"), None, false, -1.0),
            ]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            let (ohm_unit, _) = eval_unit(&ohm_unit, &context);
            let (volt_per_ampere_unit, _) = eval_unit(&volt_per_ampere_unit, &context);

            assert!(ohm_unit.numerically_eq(&volt_per_ampere_unit));
        }

        #[test]
        fn eval_ohms_are_kg_m2_s3_a2() {
            let ohm_unit = ir_composite_unit([UnitSpec::new(Some("Ohm"), None, false, 1.0)]);
            let kg_m2_s3_a2_unit = ir_composite_unit([
                UnitSpec::new(Some("g"), Some("k"), false, 1.0),
                UnitSpec::new(Some("m"), None, false, 2.0),
                UnitSpec::new(Some("s"), None, false, -3.0),
                UnitSpec::new(Some("A"), None, false, -2.0),
            ]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            let (ohm_unit, _) = eval_unit(&ohm_unit, &context);
            let (kg_m2_s3_a2_unit, _) = eval_unit(&kg_m2_s3_a2_unit, &context);

            assert!(ohm_unit.numerically_eq(&kg_m2_s3_a2_unit));
        }

        #[test]
        fn eval_pascals_are_newtons_per_square_meter() {
            let pascal_unit = ir_composite_unit([UnitSpec::new(Some("Pa"), None, false, 1.0)]);
            let newton_per_square_meter_unit = ir_composite_unit([
                UnitSpec::new(Some("N"), None, false, 1.0),
                UnitSpec::new(Some("m"), None, false, -2.0),
            ]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            let (pascal_unit, _) = eval_unit(&pascal_unit, &context);
            let (newton_per_square_meter_unit, _) =
                eval_unit(&newton_per_square_meter_unit, &context);

            assert!(pascal_unit.numerically_eq(&newton_per_square_meter_unit));
        }

        #[test]
        fn eval_pascals_are_kg_m_s2() {
            let pascal_unit = ir_composite_unit([UnitSpec::new(Some("Pa"), None, false, 1.0)]);
            let kg_m_s2_unit = ir_composite_unit([
                UnitSpec::new(Some("g"), Some("k"), false, 1.0),
                UnitSpec::new(Some("m"), None, false, -1.0),
                UnitSpec::new(Some("s"), None, false, -2.0),
            ]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            let (pascal_unit, _) = eval_unit(&pascal_unit, &context);
            let (kg_m_s2_unit, _) = eval_unit(&kg_m_s2_unit, &context);

            assert!(pascal_unit.numerically_eq(&kg_m_s2_unit));
        }

        #[test]
        fn eval_watt_hours_are_watts_times_hours() {
            let watt_hour_unit = ir_composite_unit([UnitSpec::new(Some("Wh"), None, false, 1.0)]);
            let watt_times_hour_unit = ir_composite_unit([
                UnitSpec::new(Some("W"), None, false, 1.0),
                UnitSpec::new(Some("hr"), None, false, 1.0),
            ]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            let (watt_hour_unit, _) = eval_unit(&watt_hour_unit, &context);
            let (watt_times_hour_unit, _) = eval_unit(&watt_times_hour_unit, &context);

            assert!(watt_hour_unit.numerically_eq(&watt_times_hour_unit));
        }

        #[test]
        fn eval_amp_hours_are_amperes_times_hours() {
            let amp_hour_unit = ir_composite_unit([UnitSpec::new(Some("Ah"), None, false, 1.0)]);
            let ampere_times_hour_unit = ir_composite_unit([
                UnitSpec::new(Some("A"), None, false, 1.0),
                UnitSpec::new(Some("hr"), None, false, 1.0),
            ]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            let (amp_hour_unit, _) = eval_unit(&amp_hour_unit, &context);
            let (ampere_times_hour_unit, _) = eval_unit(&ampere_times_hour_unit, &context);

            assert!(amp_hour_unit.numerically_eq(&ampere_times_hour_unit));
        }

        #[test]
        fn eval_tesla_are_kg_s2_a() {
            let tesla_unit = ir_composite_unit([UnitSpec::new(Some("T"), None, false, 1.0)]);
            let kg_s2_a_unit = ir_composite_unit([
                UnitSpec::new(Some("g"), Some("k"), false, 1.0),
                UnitSpec::new(Some("s"), None, false, -2.0),
                UnitSpec::new(Some("A"), None, false, -1.0),
            ]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            let (tesla_unit, _) = eval_unit(&tesla_unit, &context);
            let (kg_s2_a_unit, _) = eval_unit(&kg_s2_a_unit, &context);

            assert!(tesla_unit.numerically_eq(&kg_s2_a_unit));
        }

        #[test]
        fn eval_hertz_are_per_second() {
            let hertz_unit = ir_composite_unit([UnitSpec::new(Some("Hz"), None, false, 1.0)]);
            let per_second_unit = ir_composite_unit([UnitSpec::new(Some("s"), None, false, -1.0)]);
            let mut external = TestExternalContext::new();
            let context = EvalContext::new(&mut external);

            let (hertz_unit, _) = eval_unit(&hertz_unit, &context);
            let (per_second_unit, _) = eval_unit(&per_second_unit, &context);

            assert_is_close!(per_second_unit.magnitude, hertz_unit.magnitude / (2.0 * PI));
            assert!(hertz_unit.dimensionally_eq(&per_second_unit));
            assert!(!hertz_unit.is_db);
            assert!(!per_second_unit.is_db);
        }
    }
}
