//! Builtin reference implementation for the model resolver
//!
//! This module provides a reference implementation of the `BuiltinRef` trait
//! required by `oneil_model_resolver`. It wraps a `BuiltinMap` and provides
//! methods to check for the existence of builtin values and functions.

use ::std::collections::HashMap;
use std::sync::Arc;

use oneil_eval::{
    builtin::{BuiltinFunction, BuiltinMap},
    value::{SizedUnit, Value},
};
use oneil_ir as ir;
use oneil_model_resolver::BuiltinRef;

/// Reference implementation of `BuiltinRef` for the model resolver
///
/// This struct wraps a `BuiltinMap` and implements the `BuiltinRef` trait,
/// allowing the model resolver to check for the existence of builtin values
/// and functions during resolution.
pub struct Builtins<F: BuiltinFunction> {
    /// The underlying builtin map containing values, functions, units, and prefixes
    pub builtin_map: BuiltinMap<F>,
}

impl<F: BuiltinFunction> Builtins<F> {
    /// Creates a new `Builtins` instance with the provided builtin collections
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
