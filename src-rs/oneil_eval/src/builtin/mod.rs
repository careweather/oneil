use ::std::{collections::HashMap, rc::Rc};

use crate::{
    error::EvalError,
    value::{SizedUnit, Value},
};

pub mod std;

pub trait BuiltinMap<F: BuiltinFunction> {
    fn builtin_values(&self) -> HashMap<String, Value>;
    fn builtin_functions(&self) -> HashMap<String, F>;
    // The units are stored as Rc<SizedUnit> so that multiple names
    // can point to the same unit (eg. "in", "inch", "inches")
    fn builtin_units(&self) -> HashMap<String, Rc<SizedUnit>>;
    fn builtin_prefixes(&self) -> HashMap<String, f64>;
}

pub trait BuiltinFunction {
    /// Calls the builtin function with the given arguments and returns the result.
    ///
    /// # Errors
    ///
    /// Returns an error if the builtin function fails to evaluate.
    fn call(&self, args: Vec<Value>) -> Result<Value, Vec<EvalError>>;
}

impl<F: Fn(Vec<Value>) -> Result<Value, Vec<EvalError>>> BuiltinFunction for F {
    fn call(&self, args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        self(args)
    }
}
