//! Functionality for Oneil's builtin values, functions, units,
//! and unit prefixes

use ::std::{collections::HashMap, rc::Rc};

use crate::{
    error::EvalError,
    value::{SizedUnit, Value},
};

pub mod std;

/// Represents a map of builtin values, functions, units, and prefixes.
///
/// The basic way to implement this is to create a struct that
/// holds each of the maps as fields.
///
/// For the standard maps that come with Oneil, see the `std` module.
pub trait BuiltinMap<F: BuiltinFunction> {
    /// Returns a map of builtin values.
    fn builtin_values(&self) -> HashMap<String, Value>;

    /// Returns a map of builtin functions.
    ///
    /// For more details about the function type, see
    /// the `BuiltinFunction` trait.
    fn builtin_functions(&self) -> HashMap<String, F>;

    /// Returns a map of builtin units.
    ///
    /// The units are stored as `Rc<SizedUnit>` so that multiple names
    /// can point to the same unit (eg. "in", "inch", "inches").
    fn builtin_units(&self) -> HashMap<String, Rc<SizedUnit>>;

    /// Returns a map of builtin unit prefixes.
    fn builtin_prefixes(&self) -> HashMap<String, f64>;
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
