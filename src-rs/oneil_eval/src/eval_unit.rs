use oneil_ir as ir;

use crate::{context::EvalContext, error::EvalError, value::SizedUnit};

// TODO: figure out display units. for now, we just
//       discard the magnitude and return the dimensions
pub fn eval_unit(
    unit: &ir::CompositeUnit,
    context: &EvalContext,
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
        Ok(result)
    } else {
        Err(errors)
    }
}

fn eval_unit_component(unit: &ir::Unit, context: &EvalContext) -> Result<SizedUnit, EvalError> {
    let name = unit.name();
    let exponent = unit.exponent();

    // first check if the unit is a unit on its own
    let unit = context.lookup_unit(name);
    if let Some(unit) = unit {
        return Ok(unit.pow(exponent));
    }

    // then check if it's a unit with a prefix
    for (prefix, prefix_magnitude) in context.available_prefixes() {
        // check if the prefix matches the unit
        let Some(stripped_name) = name.strip_prefix(&prefix) else {
            continue;
        };

        let unit = context.lookup_unit(stripped_name);
        if let Some(unit) = unit {
            let unit = SizedUnit {
                magnitude: unit.magnitude * prefix_magnitude,
                unit: unit.unit,
            };
            return Ok(unit.pow(exponent));
        }
    }

    // if we get here, the unit is not found
    Err(EvalError::UnknownUnit)
}
