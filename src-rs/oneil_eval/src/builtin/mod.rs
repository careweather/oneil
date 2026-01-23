//! Functionality for Oneil's builtin values, functions, units,
//! and unit prefixes

use indexmap::IndexMap;

use oneil_shared::span::Span;

use crate::{
    error::EvalError,
    value::{Unit, Value},
};

pub mod std;

/// Represents a map of builtin values, functions, units, and prefixes.
///
/// For the standard maps that come with Oneil, see the `std` module.
#[derive(Debug, Clone)]
pub struct BuiltinMap<F: BuiltinFunction> {
    /// A map of builtin values
    pub values: IndexMap<String, Value>,
    /// A map of builtin functions
    pub functions: IndexMap<String, F>,
    /// A map of builtin units
    pub units: IndexMap<String, Unit>,
    /// A map of builtin unit prefixes
    pub prefixes: IndexMap<String, f64>,
}

impl<F: BuiltinFunction> BuiltinMap<F> {
    /// Creates a new builtin map.
    #[must_use]
    pub const fn new(
        values: IndexMap<String, Value>,
        functions: IndexMap<String, F>,
        units: IndexMap<String, Unit>,
        prefixes: IndexMap<String, f64>,
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
    fn call(
        &self,
        identifier_span: Span,
        args: Vec<(Value, Span)>,
    ) -> Result<Value, Vec<EvalError>>;
}

impl<F: Fn(Span, Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>>> BuiltinFunction for F {
    fn call(
        &self,
        identifier_span: Span,
        args: Vec<(Value, Span)>,
    ) -> Result<Value, Vec<EvalError>> {
        self(identifier_span, args)
    }
}
