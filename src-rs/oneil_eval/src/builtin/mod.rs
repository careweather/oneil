//! Functionality for Oneil's builtin values, functions, units,
//! and unit prefixes

use ::std::collections::HashMap;

use crate::{
    error::EvalError,
    value::{SizedUnit, Value},
};

pub mod std;

/// Represents a map of builtin values, functions, units, and prefixes.
///
/// For the standard maps that come with Oneil, see the `std` module.
#[derive(Debug, Clone)]
pub struct BuiltinMap<F: BuiltinFunction> {
    /// A map of builtin values
    pub values: HashMap<String, Value>,
    /// A map of builtin functions
    pub functions: HashMap<String, F>,
    /// A map of builtin units
    pub units: HashMap<String, SizedUnit>,
    /// A map of builtin unit prefixes
    pub prefixes: HashMap<String, f64>,
}

impl<F: BuiltinFunction> BuiltinMap<F> {
    /// Creates a new builtin map.
    #[must_use]
    pub const fn new(
        values: HashMap<String, Value>,
        functions: HashMap<String, F>,
        units: HashMap<String, SizedUnit>,
        prefixes: HashMap<String, f64>,
    ) -> Self {
        Self {
            values,
            functions,
            units,
            prefixes,
        }
    }
}

/// Represents a builtin function.
///
/// This is already implemented for any function that takes a `Vec<Value>`
/// and returns a `Result<Value, Vec<EvalError>>`.
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
