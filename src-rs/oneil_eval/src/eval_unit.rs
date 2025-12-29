use oneil_ir as ir;

use crate::{
    builtin::BuiltinFunction,
    context::EvalContext,
    error::EvalError,
    value::{DisplayUnit, SizedUnit, Unit},
};

// TODO: figure out display units. for now, we just
//       discard the magnitude and return the dimensions
/// Evaluates a composite unit and returns the resulting sized unit.
///
/// # Errors
///
/// Returns an error if the unit is not found.
pub fn eval_unit<F: BuiltinFunction>(
    unit: &ir::CompositeUnit,
    context: &EvalContext<F>,
) -> Result<SizedUnit, Vec<EvalError>> {
    let units = unit
        .units()
        .iter()
        .map(|unit| eval_unit_component(unit, context));

    let mut result = SizedUnit::unitless();
    let mut errors = Vec::new();

    for unit in units {
        match unit {
            Ok(unit) => {
                result = result * unit;
            }
            Err(error) => {
                errors.push(error);
            }
        }
    }

    if errors.is_empty() {
        // evaluate the display unit based on the IR
        let display_unit = eval_display_unit(unit.display_unit());

        let unit = SizedUnit {
            display_unit: Some(display_unit),
            ..result
        };

        Ok(unit)
    } else {
        Err(errors)
    }
}

fn eval_unit_component<F: BuiltinFunction>(
    unit: &ir::Unit,
    context: &EvalContext<F>,
) -> Result<SizedUnit, EvalError> {
    let full_name = unit.name();
    let exponent = unit.exponent();

    let (name, is_db) = full_name
        .strip_prefix("dB")
        .map_or((full_name, false), |stripped_name| (stripped_name, true));

    // handle dB units with no other unit
    if is_db && name.is_empty() {
        return Ok(SizedUnit {
            magnitude: 1.0,
            unit: Unit::unitless(),
            is_db,
            display_unit: Some(DisplayUnit::Unit(full_name.to_string(), None)),
        });
    }

    // first check if the unit is a unit on its own
    let unit = context.lookup_unit(name);
    if let Some(unit) = unit {
        let unit = unit.pow(exponent).set_is_db(is_db);
        return Ok(unit);
    }

    // then check if it's a unit with a prefix
    for (prefix, prefix_magnitude) in context.available_prefixes() {
        // check if the prefix matches the unit
        let Some(stripped_name) = name.strip_prefix(prefix) else {
            continue;
        };

        let unit = context.lookup_unit(stripped_name);
        if let Some(unit) = unit {
            let unit = SizedUnit {
                magnitude: unit.magnitude * prefix_magnitude,
                unit: unit.unit,
                is_db,
                display_unit: unit.display_unit,
            };
            return Ok(unit.pow(exponent));
        }
    }

    // if we get here, the unit is not found
    Err(EvalError::UnknownUnit)
}

fn eval_display_unit(unit: &ir::DisplayCompositeUnit) -> DisplayUnit {
    match unit {
        ir::DisplayCompositeUnit::BaseUnit(unit) => {
            let name = unit.name();
            let exponent = unit.exponent();
            DisplayUnit::Unit(name.to_string(), Some(exponent))
        }
        ir::DisplayCompositeUnit::Unitless => DisplayUnit::Unitless,
        ir::DisplayCompositeUnit::Multiply(left, right) => {
            let left = eval_display_unit(left);
            let right = eval_display_unit(right);
            left * right
        }
        ir::DisplayCompositeUnit::Divide(left, right) => {
            let left = eval_display_unit(left);
            let right = eval_display_unit(right);
            left / right
        }
    }
}

#[cfg(test)]
mod test {
    use std::f64::consts::PI;

    use crate::{
        assert_is_close, assert_units_eq,
        builtin::{self, BuiltinMap, std::StdBuiltinFunction},
        value::Dimension,
    };

    use super::*;

    fn create_eval_context() -> EvalContext<StdBuiltinFunction> {
        let builtins = BuiltinMap::new(
            builtin::std::builtin_values(),
            builtin::std::builtin_functions(),
            builtin::std::builtin_units(),
            builtin::std::builtin_prefixes(),
        );
        EvalContext::new(builtins)
    }

    /// Returns a display unit that isn't intended to be tested.
    fn unimportant_display_unit() -> ir::DisplayCompositeUnit {
        ir::DisplayCompositeUnit::BaseUnit(ir::Unit::new("unimportant".to_string(), 1.0))
    }

    fn ir_composite_unit(
        unit_list: impl IntoIterator<Item = (&'static str, f64)>,
    ) -> ir::CompositeUnit {
        let unit_vec = unit_list
            .into_iter()
            .map(|(name, exponent)| ir::Unit::new(name.to_string(), exponent))
            .collect::<Vec<_>>();
        ir::CompositeUnit::new(unit_vec, unimportant_display_unit())
    }

    mod unit_eval {

        use super::*;

        #[test]
        fn eval_unitless() {
            // setup unit and context
            let unit = ir_composite_unit([]);
            let context = create_eval_context();

            // evaluate unit
            let result = eval_unit(&unit, &context);

            // unwrap result
            let sized_unit = result.expect("should be able to eval unit");

            // check sized unit
            assert_is_close!(1.0, sized_unit.magnitude);
            assert_units_eq!([], sized_unit.unit);
            assert!(!sized_unit.is_db);
        }

        #[test]
        fn eval_simple() {
            // setup unit and context
            let unit = ir_composite_unit([("s", 1.0)]);
            let context = create_eval_context();

            // evaluate unit
            let result = eval_unit(&unit, &context);

            // unwrap result
            let sized_unit = result.expect("should be able to eval unit");

            // check sized unit
            assert_is_close!(1.0, sized_unit.magnitude);
            assert_units_eq!([(Dimension::Time, 1.0)], sized_unit.unit);
            assert!(!sized_unit.is_db);
        }

        #[test]
        fn eval_simple_with_prefix() {
            // setup unit and context
            let unit = ir_composite_unit([("ms", 1.0)]);
            let context = create_eval_context();

            // evaluate unit
            let result = eval_unit(&unit, &context);

            // unwrap result
            let sized_unit = result.expect("should be able to eval unit");

            // check sized unit
            assert_is_close!(0.001, sized_unit.magnitude);
            assert_units_eq!([(Dimension::Time, 1.0)], sized_unit.unit);
            assert!(!sized_unit.is_db);
        }

        #[test]
        fn eval_simple_with_prefix_and_exponent() {
            // setup unit and context
            let unit = ir_composite_unit([("ms", 2.0)]);
            let context = create_eval_context();

            // evaluate unit
            let result = eval_unit(&unit, &context);

            // unwrap result
            let sized_unit = result.expect("should be able to eval unit");

            // check sized unit
            assert_is_close!(0.001_f64.powi(2), sized_unit.magnitude);
            assert_units_eq!([(Dimension::Time, 2.0)], sized_unit.unit);
            assert!(!sized_unit.is_db);
        }

        #[test]
        fn eval_db() {
            // setup unit and context
            let unit = ir_composite_unit([("dB", 1.0)]);
            let context = create_eval_context();

            // evaluate unit
            let result = eval_unit(&unit, &context);

            // unwrap result
            let sized_unit = result.expect("should be able to eval unit");

            // check sized unit
            assert_is_close!(1.0, sized_unit.magnitude);
            assert_units_eq!([], sized_unit.unit);
            assert!(sized_unit.is_db);
        }

        #[test]
        fn eval_db_watts() {
            // setup unit and context
            let unit = ir_composite_unit([("dBW", 1.0)]);
            let context = create_eval_context();

            // evaluate unit
            let result = eval_unit(&unit, &context);

            // unwrap result
            let sized_unit = result.expect("should be able to eval unit");

            // check sized unit
            assert_is_close!(1.0, sized_unit.magnitude);
            assert_units_eq!(
                [
                    (Dimension::Mass, 1.0),
                    (Dimension::Distance, 2.0),
                    (Dimension::Time, -3.0)
                ],
                sized_unit.unit
            );
            assert!(sized_unit.is_db);
        }

        #[test]
        fn eval_db_watts_per_meter_squared_per_hertz() {
            // setup unit and context
            let unit = ir_composite_unit([("dBW", 1.0), ("m", -2.0), ("Hz", -1.0)]);
            let context = create_eval_context();

            // evaluate unit
            let result = eval_unit(&unit, &context);

            // unwrap result
            let sized_unit = result.expect("should be able to eval unit");

            // check sized unit
            // dBW: Mass, Distance^2, Time^-3
            // m^-2: Distance^-2
            // Hz^-1: Time^1 (since Hz has Time^-1)
            // Result: Mass, Time^-2
            // Magnitude: 1 / (2π) because Hz has magnitude 2π, so Hz^-1 contributes 1/(2π)
            assert_is_close!(1.0 / (2.0 * PI), sized_unit.magnitude);
            assert_units_eq!(
                [(Dimension::Mass, 1.0), (Dimension::Time, -2.0)],
                sized_unit.unit
            );
            assert!(sized_unit.is_db);
        }

        #[test]
        fn eval_kilometers() {
            // setup unit and context
            let unit = ir_composite_unit([("km", 1.0)]);
            let context = create_eval_context();

            // evaluate unit
            let result = eval_unit(&unit, &context);

            // unwrap result
            let sized_unit = result.expect("should be able to eval unit");

            // check sized unit
            assert_is_close!(1000.0, sized_unit.magnitude);
            assert_units_eq!([(Dimension::Distance, 1.0)], sized_unit.unit);
            assert!(!sized_unit.is_db);
        }

        #[test]
        fn eval_square_kilometers() {
            // setup unit and context
            let unit = ir_composite_unit([("km", 2.0)]);
            let context = create_eval_context();

            // evaluate unit
            let result = eval_unit(&unit, &context);

            // unwrap result
            let sized_unit = result.expect("should be able to eval unit");

            // check sized unit
            assert_is_close!(1000.0_f64.powi(2), sized_unit.magnitude);
            assert_units_eq!([(Dimension::Distance, 2.0)], sized_unit.unit);
            assert!(!sized_unit.is_db);
        }

        #[test]
        fn eval_gigahertz() {
            // setup unit and context
            let unit = ir_composite_unit([("GHz", 1.0)]);
            let context = create_eval_context();

            // evaluate unit
            let result = eval_unit(&unit, &context);

            // unwrap result
            let sized_unit = result.expect("should be able to eval unit");

            // check sized unit
            // Hz has magnitude 2π, so GHz = 1e9 * 2π
            assert_is_close!(1e9 * (2.0 * PI), sized_unit.magnitude);
            assert_units_eq!([(Dimension::Time, -1.0)], sized_unit.unit);
            assert!(!sized_unit.is_db);
        }

        #[test]
        fn eval_kilohertz() {
            // setup unit and context
            let unit = ir_composite_unit([("kHz", 1.0)]);
            let context = create_eval_context();

            // evaluate unit
            let result = eval_unit(&unit, &context);

            // unwrap result
            let sized_unit = result.expect("should be able to eval unit");

            // check sized unit
            // Hz has magnitude 2π, so kHz = 1e3 * 2π
            assert_is_close!(1e3 * (2.0 * PI), sized_unit.magnitude);
            assert_units_eq!([(Dimension::Time, -1.0)], sized_unit.unit);
            assert!(!sized_unit.is_db);
        }

        #[test]
        fn eval_megahertz() {
            // setup unit and context
            let unit = ir_composite_unit([("MHz", 1.0)]);
            let context = create_eval_context();

            // evaluate unit
            let result = eval_unit(&unit, &context);

            // unwrap result
            let sized_unit = result.expect("should be able to eval unit");

            // check sized unit
            // Hz has magnitude 2π, so MHz = 1e6 * 2π
            assert_is_close!(1e6 * (2.0 * PI), sized_unit.magnitude);
            assert_units_eq!([(Dimension::Time, -1.0)], sized_unit.unit);
            assert!(!sized_unit.is_db);
        }

        #[test]
        fn eval_microseconds() {
            // setup unit and context
            let unit = ir_composite_unit([("us", 1.0)]);
            let context = create_eval_context();

            // evaluate unit
            let result = eval_unit(&unit, &context);

            // unwrap result
            let sized_unit = result.expect("should be able to eval unit");

            // check sized unit
            assert_is_close!(1e-6, sized_unit.magnitude);
            assert_units_eq!([(Dimension::Time, 1.0)], sized_unit.unit);
            assert!(!sized_unit.is_db);
        }

        #[test]
        fn eval_volts() {
            // setup unit and context
            let unit = ir_composite_unit([("V", 1.0)]);
            let context = create_eval_context();

            // evaluate unit
            let result = eval_unit(&unit, &context);

            // unwrap result
            let sized_unit = result.expect("should be able to eval unit");

            // check sized unit
            assert_is_close!(1.0, sized_unit.magnitude);
            assert_units_eq!(
                [
                    (Dimension::Mass, 1.0),
                    (Dimension::Distance, 2.0),
                    (Dimension::Time, -3.0),
                    (Dimension::Current, -1.0)
                ],
                sized_unit.unit
            );
            assert!(!sized_unit.is_db);
        }

        #[test]
        fn eval_millivolts() {
            // setup unit and context
            let unit = ir_composite_unit([("mV", 1.0)]);
            let context = create_eval_context();

            // evaluate unit
            let result = eval_unit(&unit, &context);

            // unwrap result
            let sized_unit = result.expect("should be able to eval unit");

            // check sized unit
            assert_is_close!(0.001, sized_unit.magnitude);
            assert_units_eq!(
                [
                    (Dimension::Mass, 1.0),
                    (Dimension::Distance, 2.0),
                    (Dimension::Time, -3.0),
                    (Dimension::Current, -1.0)
                ],
                sized_unit.unit
            );
            assert!(!sized_unit.is_db);
        }

        #[test]
        fn eval_ohms() {
            // setup unit and context
            let unit = ir_composite_unit([("Ohm", 1.0)]);
            let context = create_eval_context();

            // evaluate unit
            let result = eval_unit(&unit, &context);

            // unwrap result
            let sized_unit = result.expect("should be able to eval unit");

            // check sized unit
            assert_is_close!(1.0, sized_unit.magnitude);
            assert_units_eq!(
                [
                    (Dimension::Mass, 1.0),
                    (Dimension::Distance, 2.0),
                    (Dimension::Time, -3.0),
                    (Dimension::Current, -2.0)
                ],
                sized_unit.unit
            );
            assert!(!sized_unit.is_db);
        }

        #[test]
        fn eval_watts() {
            // setup unit and context
            let unit = ir_composite_unit([("W", 1.0)]);
            let context = create_eval_context();

            // evaluate unit
            let result = eval_unit(&unit, &context);

            // unwrap result
            let sized_unit = result.expect("should be able to eval unit");

            // check sized unit
            assert_is_close!(1.0, sized_unit.magnitude);
            assert_units_eq!(
                [
                    (Dimension::Mass, 1.0),
                    (Dimension::Distance, 2.0),
                    (Dimension::Time, -3.0)
                ],
                sized_unit.unit
            );
            assert!(!sized_unit.is_db);
        }

        #[test]
        fn eval_watts_per_square_meter() {
            // setup unit and context
            let unit = ir_composite_unit([("W", 1.0), ("m", -2.0)]);
            let context = create_eval_context();

            // evaluate unit
            let result = eval_unit(&unit, &context);

            // unwrap result
            let sized_unit = result.expect("should be able to eval unit");

            // check sized unit
            assert_is_close!(1.0, sized_unit.magnitude);
            assert_units_eq!(
                [(Dimension::Mass, 1.0), (Dimension::Time, -3.0)],
                sized_unit.unit
            );
            assert!(!sized_unit.is_db);
        }

        #[test]
        fn eval_kelvin() {
            // setup unit and context
            let unit = ir_composite_unit([("K", 1.0)]);
            let context = create_eval_context();

            // evaluate unit
            let result = eval_unit(&unit, &context);

            // unwrap result
            let sized_unit = result.expect("should be able to eval unit");

            // check sized unit
            assert_is_close!(1.0, sized_unit.magnitude);
            assert_units_eq!([(Dimension::Temperature, 1.0)], sized_unit.unit);
            assert!(!sized_unit.is_db);
        }

        #[test]
        fn eval_amperes() {
            // setup unit and context
            let unit = ir_composite_unit([("A", 1.0)]);
            let context = create_eval_context();

            // evaluate unit
            let result = eval_unit(&unit, &context);

            // unwrap result
            let sized_unit = result.expect("should be able to eval unit");

            // check sized unit
            assert_is_close!(1.0, sized_unit.magnitude);
            assert_units_eq!([(Dimension::Current, 1.0)], sized_unit.unit);
            assert!(!sized_unit.is_db);
        }

        #[test]
        fn eval_milliampere_hours() {
            // setup unit and context
            let unit = ir_composite_unit([("mAh", 1.0)]);
            let context = create_eval_context();

            // evaluate unit
            let result = eval_unit(&unit, &context);

            // unwrap result
            let sized_unit = result.expect("should be able to eval unit");

            // check sized unit
            // mAh = 0.001 A * 3600 s = 3.6 A*s
            assert_is_close!(3.6, sized_unit.magnitude);
            assert_units_eq!(
                [(Dimension::Current, 1.0), (Dimension::Time, 1.0)],
                sized_unit.unit
            );
            assert!(!sized_unit.is_db);
        }

        #[test]
        fn eval_joules() {
            // setup unit and context
            let unit = ir_composite_unit([("J", 1.0)]);
            let context = create_eval_context();

            // evaluate unit
            let result = eval_unit(&unit, &context);

            // unwrap result
            let sized_unit = result.expect("should be able to eval unit");

            // check sized unit
            assert_is_close!(1.0, sized_unit.magnitude);
            assert_units_eq!(
                [
                    (Dimension::Mass, 1.0),
                    (Dimension::Distance, 2.0),
                    (Dimension::Time, -2.0)
                ],
                sized_unit.unit
            );
            assert!(!sized_unit.is_db);
        }

        #[test]
        fn eval_hours() {
            // setup unit and context
            let unit = ir_composite_unit([("hr", 1.0)]);
            let context = create_eval_context();

            // evaluate unit
            let result = eval_unit(&unit, &context);

            // unwrap result
            let sized_unit = result.expect("should be able to eval unit");

            // check sized unit
            // hr = 3600 s
            assert_is_close!(3600.0, sized_unit.magnitude);
            assert_units_eq!([(Dimension::Time, 1.0)], sized_unit.unit);
            assert!(!sized_unit.is_db);
        }

        #[test]
        fn eval_minutes() {
            // setup unit and context
            let unit = ir_composite_unit([("min", 1.0)]);
            let context = create_eval_context();

            // evaluate unit
            let result = eval_unit(&unit, &context);

            // unwrap result
            let sized_unit = result.expect("should be able to eval unit");

            // check sized unit
            // min = 60 s
            assert_is_close!(60.0, sized_unit.magnitude);
            assert_units_eq!([(Dimension::Time, 1.0)], sized_unit.unit);
            assert!(!sized_unit.is_db);
        }

        #[test]
        fn eval_revolutions_per_minute() {
            // setup unit and context
            let unit = ir_composite_unit([("rpm", 1.0)]);
            let context = create_eval_context();

            // evaluate unit
            let result = eval_unit(&unit, &context);

            // unwrap result
            let sized_unit = result.expect("should be able to eval unit");

            // check sized unit
            // rpm has magnitude 2π/60 (radians per second)
            assert_is_close!(2.0 * PI / 60.0, sized_unit.magnitude);
            assert_units_eq!([(Dimension::Time, -1.0)], sized_unit.unit);
            assert!(!sized_unit.is_db);
        }

        #[test]
        fn eval_degrees() {
            // setup unit and context
            let unit = ir_composite_unit([("deg", 1.0)]);
            let context = create_eval_context();

            // evaluate unit
            let result = eval_unit(&unit, &context);

            // unwrap result
            let sized_unit = result.expect("should be able to eval unit");

            // check sized unit
            // deg is dimensionless with magnitude π/180 (conversion to radians)
            assert_is_close!(PI / 180.0, sized_unit.magnitude);
            assert_units_eq!([], sized_unit.unit);
            assert!(!sized_unit.is_db);
        }

        #[test]
        fn eval_percent() {
            // setup unit and context
            let unit = ir_composite_unit([("%", 1.0)]);
            let context = create_eval_context();

            // evaluate unit
            let result = eval_unit(&unit, &context);

            // unwrap result
            let sized_unit = result.expect("should be able to eval unit");

            // check sized unit
            // % is dimensionless with magnitude 0.01
            assert_is_close!(0.01, sized_unit.magnitude);
            assert_units_eq!([], sized_unit.unit);
            assert!(!sized_unit.is_db);
        }

        #[test]
        fn eval_megabits_per_second() {
            // setup unit and context
            let unit = ir_composite_unit([("Mbps", 1.0)]);
            let context = create_eval_context();

            // evaluate unit
            let result = eval_unit(&unit, &context);

            // unwrap result
            let sized_unit = result.expect("should be able to eval unit");

            // check sized unit
            // Mbps = 1e6 * bps, and bps has Information*Time^-1 dimension
            assert_is_close!(1e6, sized_unit.magnitude);
            assert_units_eq!(
                [(Dimension::Information, 1.0), (Dimension::Time, -1.0)],
                sized_unit.unit
            );
            assert!(!sized_unit.is_db);
        }

        #[test]
        fn eval_kilobytes() {
            // setup unit and context
            let unit = ir_composite_unit([("kB", 1.0)]);
            let context = create_eval_context();

            // evaluate unit
            let result = eval_unit(&unit, &context);

            // unwrap result
            let sized_unit = result.expect("should be able to eval unit");

            // check sized unit
            // B has magnitude 8 (bits), so kB = 1000 * 8 = 8000 bits
            assert_is_close!(8000.0, sized_unit.magnitude);
            assert_units_eq!([(Dimension::Information, 1.0)], sized_unit.unit);
            assert!(!sized_unit.is_db);
        }

        #[test]
        fn eval_boltzmann_constant_unit() {
            // setup unit and context
            // m^2*kg/s^2/K - the unit of Boltzmann's constant
            let unit = ir_composite_unit([("m", 2.0), ("kg", 1.0), ("s", -2.0), ("K", -1.0)]);
            let context = create_eval_context();

            // evaluate unit
            let result = eval_unit(&unit, &context);

            // unwrap result
            let sized_unit = result.expect("should be able to eval unit");

            // check sized unit
            // kg is the base unit (magnitude 1), not g
            assert_is_close!(1.0, sized_unit.magnitude);
            assert_units_eq!(
                [
                    (Dimension::Distance, 2.0),
                    (Dimension::Mass, 1.0),
                    (Dimension::Time, -2.0),
                    (Dimension::Temperature, -1.0)
                ],
                sized_unit.unit
            );
            assert!(!sized_unit.is_db);
        }

        #[test]
        fn eval_meters_per_second() {
            // setup unit and context
            let unit = ir_composite_unit([("m", 1.0), ("s", -1.0)]);
            let context = create_eval_context();

            // evaluate unit
            let result = eval_unit(&unit, &context);

            // unwrap result
            let sized_unit = result.expect("should be able to eval unit");

            // check sized unit
            assert_is_close!(1.0, sized_unit.magnitude);
            assert_units_eq!(
                [(Dimension::Distance, 1.0), (Dimension::Time, -1.0)],
                sized_unit.unit
            );
            assert!(!sized_unit.is_db);
        }

        #[test]
        fn eval_meters_per_second_squared() {
            // setup unit and context
            let unit = ir_composite_unit([("m", 1.0), ("s", -2.0)]);
            let context = create_eval_context();

            // evaluate unit
            let result = eval_unit(&unit, &context);

            // unwrap result
            let sized_unit = result.expect("should be able to eval unit");

            // check sized unit
            assert_is_close!(1.0, sized_unit.magnitude);
            assert_units_eq!(
                [(Dimension::Distance, 1.0), (Dimension::Time, -2.0)],
                sized_unit.unit
            );
            assert!(!sized_unit.is_db);
        }
    }

    mod unit_equivalence {
        use super::*;

        #[test]
        fn eval_newtons_are_kg_m_s_2() {
            // setup unit and context
            let newton_unit = ir_composite_unit([("N", 1.0)]);
            let kg_m_s_2_unit = ir_composite_unit([("kg", 1.0), ("m", 1.0), ("s", -2.0)]);
            let context = create_eval_context();

            // evaluate newton unit
            let result = eval_unit(&newton_unit, &context);
            let newton_sized_unit = result.expect("should be able to eval unit");

            // evaluate kg_m_s_2 unit
            let result = eval_unit(&kg_m_s_2_unit, &context);
            let kg_m_s_2_sized_unit = result.expect("should be able to eval unit");

            // check if units are equal
            assert_eq!(newton_sized_unit, kg_m_s_2_sized_unit);
            assert!(!newton_sized_unit.is_db);
            assert!(!kg_m_s_2_sized_unit.is_db);
        }

        #[test]
        fn eval_joules_are_newton_meters() {
            // setup unit and context
            let joule_unit = ir_composite_unit([("J", 1.0)]);
            let newton_meter_unit = ir_composite_unit([("N", 1.0), ("m", 1.0)]);
            let context = create_eval_context();

            // evaluate joule unit
            let result = eval_unit(&joule_unit, &context);
            let joule_sized_unit = result.expect("should be able to eval unit");

            // evaluate newton_meter unit
            let result = eval_unit(&newton_meter_unit, &context);
            let newton_meter_sized_unit = result.expect("should be able to eval unit");

            // check if units are equal
            assert_eq!(joule_sized_unit, newton_meter_sized_unit);
            assert!(!joule_sized_unit.is_db);
            assert!(!newton_meter_sized_unit.is_db);
        }

        #[test]
        fn eval_joules_are_kg_m2_s2() {
            // setup unit and context
            let joule_unit = ir_composite_unit([("J", 1.0)]);
            let kg_m2_s2_unit = ir_composite_unit([("kg", 1.0), ("m", 2.0), ("s", -2.0)]);
            let context = create_eval_context();

            // evaluate joule unit
            let result = eval_unit(&joule_unit, &context);
            let joule_sized_unit = result.expect("should be able to eval unit");

            // evaluate kg_m2_s2 unit
            let result = eval_unit(&kg_m2_s2_unit, &context);
            let kg_m2_s2_sized_unit = result.expect("should be able to eval unit");

            // check if units are equal
            assert_eq!(joule_sized_unit, kg_m2_s2_sized_unit);
            assert!(!joule_sized_unit.is_db);
            assert!(!kg_m2_s2_sized_unit.is_db);
        }

        #[test]
        fn eval_watts_are_joules_per_second() {
            // setup unit and context
            let watt_unit = ir_composite_unit([("W", 1.0)]);
            let joule_per_second_unit = ir_composite_unit([("J", 1.0), ("s", -1.0)]);
            let context = create_eval_context();

            // evaluate watt unit
            let result = eval_unit(&watt_unit, &context);
            let watt_sized_unit = result.expect("should be able to eval unit");

            // evaluate joule_per_second unit
            let result = eval_unit(&joule_per_second_unit, &context);
            let joule_per_second_sized_unit = result.expect("should be able to eval unit");

            // check if units are equal
            assert_eq!(watt_sized_unit, joule_per_second_sized_unit);
            assert!(!watt_sized_unit.is_db);
            assert!(!joule_per_second_sized_unit.is_db);
        }

        #[test]
        fn eval_watts_are_newton_meters_per_second() {
            // setup unit and context
            let watt_unit = ir_composite_unit([("W", 1.0)]);
            let newton_meter_per_second_unit =
                ir_composite_unit([("N", 1.0), ("m", 1.0), ("s", -1.0)]);
            let context = create_eval_context();

            // evaluate watt unit
            let result = eval_unit(&watt_unit, &context);
            let watt_sized_unit = result.expect("should be able to eval unit");

            // evaluate newton_meter_per_second unit
            let result = eval_unit(&newton_meter_per_second_unit, &context);
            let newton_meter_per_second_sized_unit = result.expect("should be able to eval unit");

            // check if units are equal
            assert_eq!(watt_sized_unit, newton_meter_per_second_sized_unit);
            assert!(!watt_sized_unit.is_db);
            assert!(!newton_meter_per_second_sized_unit.is_db);
        }

        #[test]
        fn eval_watts_are_kg_m2_s3() {
            // setup unit and context
            let watt_unit = ir_composite_unit([("W", 1.0)]);
            let kg_m2_s3_unit = ir_composite_unit([("kg", 1.0), ("m", 2.0), ("s", -3.0)]);
            let context = create_eval_context();

            // evaluate watt unit
            let result = eval_unit(&watt_unit, &context);
            let watt_sized_unit = result.expect("should be able to eval unit");

            // evaluate kg_m2_s3 unit
            let result = eval_unit(&kg_m2_s3_unit, &context);
            let kg_m2_s3_sized_unit = result.expect("should be able to eval unit");

            // check if units are equal
            assert_eq!(watt_sized_unit, kg_m2_s3_sized_unit);
            assert!(!watt_sized_unit.is_db);
            assert!(!kg_m2_s3_sized_unit.is_db);
        }

        #[test]
        fn eval_volts_are_watts_per_ampere() {
            // setup unit and context
            let volt_unit = ir_composite_unit([("V", 1.0)]);
            let watt_per_ampere_unit = ir_composite_unit([("W", 1.0), ("A", -1.0)]);
            let context = create_eval_context();

            // evaluate volt unit
            let result = eval_unit(&volt_unit, &context);
            let volt_sized_unit = result.expect("should be able to eval unit");

            // evaluate watt_per_ampere unit
            let result = eval_unit(&watt_per_ampere_unit, &context);
            let watt_per_ampere_sized_unit = result.expect("should be able to eval unit");

            // check if units are equal
            assert_eq!(volt_sized_unit, watt_per_ampere_sized_unit);
            assert!(!volt_sized_unit.is_db);
            assert!(!watt_per_ampere_sized_unit.is_db);
        }

        #[test]
        fn eval_volts_are_kg_m2_s3_a() {
            // setup unit and context
            let volt_unit = ir_composite_unit([("V", 1.0)]);
            let kg_m2_s3_a_unit =
                ir_composite_unit([("kg", 1.0), ("m", 2.0), ("s", -3.0), ("A", -1.0)]);
            let context = create_eval_context();

            // evaluate volt unit
            let result = eval_unit(&volt_unit, &context);
            let volt_sized_unit = result.expect("should be able to eval unit");

            // evaluate kg_m2_s3_a unit
            let result = eval_unit(&kg_m2_s3_a_unit, &context);
            let kg_m2_s3_a_sized_unit = result.expect("should be able to eval unit");

            // check if units are equal
            assert_eq!(volt_sized_unit, kg_m2_s3_a_sized_unit);
            assert!(!volt_sized_unit.is_db);
            assert!(!kg_m2_s3_a_sized_unit.is_db);
        }

        #[test]
        fn eval_ohms_are_volts_per_ampere() {
            // setup unit and context
            let ohm_unit = ir_composite_unit([("Ohm", 1.0)]);
            let volt_per_ampere_unit = ir_composite_unit([("V", 1.0), ("A", -1.0)]);
            let context = create_eval_context();

            // evaluate ohm unit
            let result = eval_unit(&ohm_unit, &context);
            let ohm_sized_unit = result.expect("should be able to eval unit");

            // evaluate volt_per_ampere unit
            let result = eval_unit(&volt_per_ampere_unit, &context);
            let volt_per_ampere_sized_unit = result.expect("should be able to eval unit");

            // check if units are equal
            assert_eq!(ohm_sized_unit, volt_per_ampere_sized_unit);
            assert!(!ohm_sized_unit.is_db);
            assert!(!volt_per_ampere_sized_unit.is_db);
        }

        #[test]
        fn eval_ohms_are_kg_m2_s3_a2() {
            // setup unit and context
            let ohm_unit = ir_composite_unit([("Ohm", 1.0)]);
            let kg_m2_s3_a2_unit =
                ir_composite_unit([("kg", 1.0), ("m", 2.0), ("s", -3.0), ("A", -2.0)]);
            let context = create_eval_context();

            // evaluate ohm unit
            let result = eval_unit(&ohm_unit, &context);
            let ohm_sized_unit = result.expect("should be able to eval unit");

            // evaluate kg_m2_s3_a2 unit
            let result = eval_unit(&kg_m2_s3_a2_unit, &context);
            let kg_m2_s3_a2_sized_unit = result.expect("should be able to eval unit");

            // check if units are equal
            assert_eq!(ohm_sized_unit, kg_m2_s3_a2_sized_unit);
            assert!(!ohm_sized_unit.is_db);
            assert!(!kg_m2_s3_a2_sized_unit.is_db);
        }

        #[test]
        fn eval_pascals_are_newtons_per_square_meter() {
            // setup unit and context
            let pascal_unit = ir_composite_unit([("Pa", 1.0)]);
            let newton_per_square_meter_unit = ir_composite_unit([("N", 1.0), ("m", -2.0)]);
            let context = create_eval_context();

            // evaluate pascal unit
            let result = eval_unit(&pascal_unit, &context);
            let pascal_sized_unit = result.expect("should be able to eval unit");

            // evaluate newton_per_square_meter unit
            let result = eval_unit(&newton_per_square_meter_unit, &context);
            let newton_per_square_meter_sized_unit = result.expect("should be able to eval unit");

            // check if units are equal
            assert_eq!(pascal_sized_unit, newton_per_square_meter_sized_unit);
            assert!(!pascal_sized_unit.is_db);
            assert!(!newton_per_square_meter_sized_unit.is_db);
        }

        #[test]
        fn eval_pascals_are_kg_m_s2() {
            // setup unit and context
            let pascal_unit = ir_composite_unit([("Pa", 1.0)]);
            let kg_m_s2_unit = ir_composite_unit([("kg", 1.0), ("m", -1.0), ("s", -2.0)]);
            let context = create_eval_context();

            // evaluate pascal unit
            let result = eval_unit(&pascal_unit, &context);
            let pascal_sized_unit = result.expect("should be able to eval unit");

            // evaluate kg_m_s2 unit
            let result = eval_unit(&kg_m_s2_unit, &context);
            let kg_m_s2_sized_unit = result.expect("should be able to eval unit");

            // check if units are equal
            assert_eq!(pascal_sized_unit, kg_m_s2_sized_unit);
            assert!(!pascal_sized_unit.is_db);
            assert!(!kg_m_s2_sized_unit.is_db);
        }

        #[test]
        fn eval_watt_hours_are_watts_times_hours() {
            // setup unit and context
            // Wh = W * hr
            let watt_hour_unit = ir_composite_unit([("Wh", 1.0)]);
            let watt_times_hour_unit = ir_composite_unit([("W", 1.0), ("hr", 1.0)]);
            let context = create_eval_context();

            // evaluate watt_hour unit
            let result = eval_unit(&watt_hour_unit, &context);
            let watt_hour_sized_unit = result.expect("should be able to eval unit");

            // evaluate watt_times_hour unit
            let result = eval_unit(&watt_times_hour_unit, &context);
            let watt_times_hour_sized_unit = result.expect("should be able to eval unit");

            // check if units are equal (magnitude and dimensions)
            assert_eq!(watt_hour_sized_unit, watt_times_hour_sized_unit);
            assert!(!watt_hour_sized_unit.is_db);
            assert!(!watt_times_hour_sized_unit.is_db);
        }

        #[test]
        fn eval_amp_hours_are_amperes_times_hours() {
            // setup unit and context
            // Ah = A * hr
            let amp_hour_unit = ir_composite_unit([("Ah", 1.0)]);
            let ampere_times_hour_unit = ir_composite_unit([("A", 1.0), ("hr", 1.0)]);
            let context = create_eval_context();

            // evaluate amp_hour unit
            let result = eval_unit(&amp_hour_unit, &context);
            let amp_hour_sized_unit = result.expect("should be able to eval unit");

            // evaluate ampere_times_hour unit
            let result = eval_unit(&ampere_times_hour_unit, &context);
            let ampere_times_hour_sized_unit = result.expect("should be able to eval unit");

            // check if units are equal (magnitude and dimensions)
            assert_eq!(amp_hour_sized_unit, ampere_times_hour_sized_unit);
            assert!(!amp_hour_sized_unit.is_db);
            assert!(!ampere_times_hour_sized_unit.is_db);
        }

        #[test]
        fn eval_tesla_are_kg_s2_a() {
            // setup unit and context
            let tesla_unit = ir_composite_unit([("T", 1.0)]);
            let kg_s2_a_unit = ir_composite_unit([("kg", 1.0), ("s", -2.0), ("A", -1.0)]);
            let context = create_eval_context();

            // evaluate tesla unit
            let result = eval_unit(&tesla_unit, &context);
            let tesla_sized_unit = result.expect("should be able to eval unit");

            // evaluate kg_s2_a unit
            let result = eval_unit(&kg_s2_a_unit, &context);
            let kg_s2_a_sized_unit = result.expect("should be able to eval unit");

            // check if units are equal
            assert_eq!(tesla_sized_unit, kg_s2_a_sized_unit);
            assert!(!tesla_sized_unit.is_db);
            assert!(!kg_s2_a_sized_unit.is_db);
        }

        #[test]
        fn eval_hertz_are_per_second() {
            // setup unit and context
            // Hz has magnitude 2π, so we need to account for that
            let hertz_unit = ir_composite_unit([("Hz", 1.0)]);
            let per_second_unit = ir_composite_unit([("s", -1.0)]);
            let context = create_eval_context();

            // evaluate hertz unit
            let result = eval_unit(&hertz_unit, &context);
            let hertz_sized_unit = result.expect("should be able to eval unit");

            // evaluate per_second unit
            let result = eval_unit(&per_second_unit, &context);
            let per_second_sized_unit = result.expect("should be able to eval unit");

            // check dimensions are equal and magnitudes have the correct relationship
            assert_is_close!(
                per_second_sized_unit.magnitude,
                hertz_sized_unit.magnitude / (2.0 * PI)
            );
            assert_eq!(hertz_sized_unit.unit, per_second_sized_unit.unit);
            assert!(!hertz_sized_unit.is_db);
            assert!(!per_second_sized_unit.is_db);
        }
    }
}
