use std::collections::HashMap;

use crate::{
    error::EvalError,
    value::{SizedUnit, Value},
};

pub trait BuiltinMap<F: BuiltinFunction> {
    fn builtin_values(&self) -> HashMap<String, Value>;
    fn builtin_functions(&self) -> HashMap<String, F>;
    fn builtin_units(&self) -> HashMap<String, SizedUnit>;
    fn builtin_prefixes(&self) -> HashMap<String, f64>;
}

pub trait BuiltinFunction {
    fn call(&self, args: Vec<Value>) -> Result<Value, Vec<EvalError>>;
}

impl<F: Fn(Vec<Value>) -> Result<Value, Vec<EvalError>>> BuiltinFunction for F {
    fn call(&self, args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        self(args)
    }
}
