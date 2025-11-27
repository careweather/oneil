use oneil_ir as ir;

use crate::{
    context::EvalContext,
    error::EvalError,
    eval_expr, eval_unit,
    value::{MeasuredNumber, SizedUnit, Unit, Value},
};

pub fn eval_parameter(
    parameter: &ir::Parameter,
    context: &EvalContext,
) -> Result<Value, Vec<EvalError>> {
    // TODO: this is about where we would use `trace_level`, but I'm not yet sure
    //       how to handle it.

    // evaluate the value and the unit
    let (value, unit_ir) = match parameter.value() {
        ir::ParameterValue::Simple(expr, unit) => {
            let value = eval_expr(expr, context)?;
            (value, unit)
        }
        ir::ParameterValue::Piecewise(piecewise, unit) => {
            let value = get_piecewise_result(piecewise, context)?;
            (value, unit)
        }
    };

    match value {
        Value::Boolean(_) => {
            if unit_ir.is_some() {
                Err(vec![EvalError::BooleanCannotHaveUnit])
            } else {
                Ok(value)
            }
        }
        Value::String(_) => {
            if unit_ir.is_some() {
                Err(vec![EvalError::StringCannotHaveUnit])
            } else {
                Ok(value)
            }
        }
        Value::Number(number) => {
            // evaluate the unit if it exists,
            // otherwise use the unitless unit
            let sized_unit = unit_ir
                .as_ref()
                .map(|unit| eval_unit(unit, context))
                .transpose()?
                .unwrap_or(SizedUnit::unitless());

            // if the value is unitless, assign it the given unit
            // otherwise, typecheck the value's unit against the given unit
            if number.unit.is_unitless() {
                let number_value = number.value * sized_unit.magnitude;
                let number_unit = sized_unit.unit;

                let number = MeasuredNumber {
                    value: number_value,
                    unit: number_unit,
                };

                Ok(Value::Number(number))
            } else {
                // TODO: is there anything that we need to do about the magnitude here?
                //       or is that only for displaying the value?
                if number.unit == sized_unit.unit {
                    Ok(Value::Number(number))
                } else {
                    Err(vec![EvalError::ParameterUnitMismatch])
                }
            }
        }
    }
}

fn get_piecewise_result(
    piecewise: &[ir::PiecewiseExpr],
    context: &EvalContext,
) -> Result<Value, Vec<EvalError>> {
    // evaluate each of the conditions and their bodies
    // TODO: this is slow, but we do it so that every branch is
    //       "typechecked". is there a better way to do this? Do we
    //       need to do better?
    let results = piecewise.iter().map(|piecewise_expr| {
        let if_result = eval_expr(piecewise_expr.if_expr(), context)?;
        let branch_result = eval_expr(piecewise_expr.expr(), context)?;

        match if_result {
            Value::Boolean(true) => Ok(Some(branch_result)),
            Value::Boolean(false) => Ok(None),
            Value::String(_) | Value::Number(_) => Err(vec![EvalError::InvalidIfExpressionType]),
        }
    });

    // find the branch that matches the condition
    // as well as any errors that occurred
    let mut result = None;
    let mut errors = Vec::new();

    for branch_result in results {
        match branch_result {
            Ok(maybe_branch_result) => {
                let Some(branch_result) = maybe_branch_result else {
                    continue;
                };

                if result.is_some() {
                    errors.push(EvalError::MultiplePiecewiseBranchesMatch);
                }

                result = Some(branch_result);
            }
            Err(e) => errors.extend(e),
        }
    }

    if !errors.is_empty() {
        Err(errors)
    } else if let Some(result) = result {
        Ok(result)
    } else {
        Err(vec![EvalError::NoPiecewiseBranchMatch])
    }
}
