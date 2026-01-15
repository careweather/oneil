use ::std::collections::HashMap;
use std::sync::Arc;

use oneil_eval::{
    builtin::{BuiltinFunction, BuiltinMap},
    value::{SizedUnit, Value},
};
use oneil_ir as ir;
use oneil_model_resolver::BuiltinRef;

pub struct Builtins<F: BuiltinFunction> {
    pub builtin_map: BuiltinMap<F>,
}

impl<F: BuiltinFunction> Builtins<F> {
    #[must_use]
    pub const fn new(
        values: HashMap<String, Value>,
        functions: HashMap<String, F>,
        units: HashMap<String, Arc<SizedUnit>>,
        prefixes: HashMap<String, f64>,
    ) -> Self {
        Self {
            builtin_map: BuiltinMap::new(values, functions, units, prefixes),
        }
    }
}

impl<F: BuiltinFunction> BuiltinRef for Builtins<F> {
    fn has_builtin_value(&self, identifier: &ir::Identifier) -> bool {
        self.builtin_map.values.contains_key(identifier.as_str())
    }

    fn has_builtin_function(&self, identifier: &ir::Identifier) -> bool {
        self.builtin_map.functions.contains_key(identifier.as_str())
    }
}
