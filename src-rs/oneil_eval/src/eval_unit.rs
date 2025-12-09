use oneil_ir as ir;

use crate::{
    builtin::BuiltinFunction,
    context::EvalContext,
    error::EvalError,
    value::{SizedUnit, Unit},
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
) -> Result<(SizedUnit, bool), Vec<EvalError>> {
    let units = unit
        .units()
        .iter()
        .map(|unit| eval_unit_component(unit, context));

    let mut is_db = false;
    let mut result = SizedUnit::unitless();
    let mut errors = Vec::new();

    for unit in units {
        match unit {
            Ok((unit, is_db_unit)) => {
                is_db |= is_db_unit;
                result = result * unit;
            }
            Err(error) => {
                errors.push(error);
            }
        }
    }

    if errors.is_empty() {
        Ok((result, is_db))
    } else {
        Err(errors)
    }
}

fn eval_unit_component<F: BuiltinFunction>(
    unit: &ir::Unit,
    context: &EvalContext<F>,
) -> Result<(SizedUnit, bool), EvalError> {
    let name = unit.name();
    let exponent = unit.exponent();

    let (name, is_db) = name
        .strip_prefix("dB")
        .map_or((name, false), |stripped_name| (stripped_name, true));

    // handle dB units with no other unit
    if is_db && name.is_empty() {
        return Ok((
            SizedUnit {
                magnitude: 1.0,
                unit: Unit::unitless(),
            },
            is_db,
        ));
    }

    // first check if the unit is a unit on its own
    let unit = context.lookup_unit(name);
    if let Some(unit) = unit {
        return Ok((unit.pow(exponent), is_db));
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
            };
            return Ok((unit.pow(exponent), is_db));
        }
    }

    // if we get here, the unit is not found
    Err(EvalError::UnknownUnit)
}
