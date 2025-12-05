use std::collections::HashMap;

use crate::{
    error::EvalError,
    value::{SizedUnit, Value, ValueType},
};

pub trait BuiltinMap<F: BuiltinFunction> {
    fn builtin_values(&self) -> HashMap<String, Value>;
    fn builtin_functions(&self) -> HashMap<String, F>;
    fn builtin_units(&self) -> HashMap<String, SizedUnit>;
    fn builtin_prefixes(&self) -> HashMap<String, f64>;
}

pub trait BuiltinFunction {
    /// Calls the builtin function with the given arguments and returns the result.
    ///
    /// # Errors
    ///
    /// Returns an error if the builtin function fails to evaluate.
    fn call(&self, args: Vec<Value>) -> Result<Value, Vec<EvalError>>;

    /// Returns the type of the arguments of the builtin function.
    fn argument_types(&self) -> Vec<ValueType>;

    /// Returns the type of the builtin function.
    fn return_type(&self) -> ValueType;
}
