use ::std::{collections::HashMap, rc::Rc};

use oneil_eval::{
    builtin::{BuiltinFunction, BuiltinMap},
    value::{MeasuredNumber, Number, SizedUnit, Unit, Value},
};
use oneil_ir as ir;
use oneil_model_resolver::BuiltinRef;

pub struct Builtins<F: BuiltinFunction> {
    values: HashMap<&'static str, f64>,
    functions: HashMap<&'static str, F>,
    units: HashMap<&'static str, Rc<SizedUnit>>,
    prefixes: HashMap<&'static str, f64>,
}

impl<F: BuiltinFunction> Builtins<F> {
    pub fn new(
        values: impl IntoIterator<Item = (&'static str, f64)>,
        functions: impl IntoIterator<Item = (&'static str, F)>,
        units: impl IntoIterator<Item = (&'static str, Rc<SizedUnit>)>,
        prefixes: impl IntoIterator<Item = (&'static str, f64)>,
    ) -> Self {
        Self {
            values: values.into_iter().collect(),
            functions: functions.into_iter().collect(),
            units: units.into_iter().collect(),
            prefixes: prefixes.into_iter().collect(),
        }
    }
}

impl<F: BuiltinFunction> BuiltinRef for Builtins<F> {
    fn has_builtin_value(&self, identifier: &ir::Identifier) -> bool {
        self.values.contains_key(identifier.as_str())
    }

    fn has_builtin_function(&self, identifier: &ir::Identifier) -> bool {
        self.functions.contains_key(identifier.as_str())
        // matches!(
        //     identifier.as_str(),
        // )
    }
}

impl<F: BuiltinFunction + Clone> BuiltinMap<F> for Builtins<F> {
    fn builtin_values(&self) -> HashMap<String, Value> {
        self.values
            .iter()
            .map(|(name, value)| {
                (
                    (*name).to_string(),
                    Value::Number(MeasuredNumber::new(
                        Number::Scalar(*value),
                        Unit::unitless(),
                    )),
                )
            })
            .collect()
    }

    fn builtin_functions(&self) -> HashMap<String, F> {
        self.functions
            .iter()
            .map(|(name, function)| ((*name).to_string(), function.clone()))
            .collect()
    }

    fn builtin_units(&self) -> HashMap<String, Rc<SizedUnit>> {
        self.units
            .iter()
            .map(|(name, unit)| ((*name).to_string(), unit.clone()))
            .collect()
    }

    fn builtin_prefixes(&self) -> HashMap<String, f64> {
        self.prefixes
            .iter()
            .map(|(name, magnitude)| ((*name).to_string(), *magnitude))
            .collect()
    }
}
