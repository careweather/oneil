//! Reference to the full set of standard builtins.

use indexmap::IndexMap;

use oneil_output::{Unit, Value};

use crate::function::{BuiltinFunctionFn, BuiltinFunction};
use crate::prefix::BuiltinPrefix;
use crate::unit::BuiltinUnit;
use crate::value::BuiltinValue;

use crate::function;
use crate::prefix;
use crate::unit;
use crate::value;

/// Reference to the standard builtin values, functions, units, and prefixes that come with Oneil.
#[derive(Debug, Clone)]
pub struct BuiltinRef {
    values: IndexMap<&'static str, BuiltinValue>,
    functions: IndexMap<&'static str, BuiltinFunction>,
    units: IndexMap<&'static str, BuiltinUnit>,
    prefixes: IndexMap<&'static str, BuiltinPrefix>,
}

impl BuiltinRef {
    /// Creates a new instance with all standard builtins.
    #[must_use]
    pub fn new() -> Self {
        Self {
            values: value::builtin_values_complete().collect(),
            functions: function::builtin_functions_complete().collect(),
            units: unit::builtin_units_complete().collect(),
            prefixes: prefix::builtin_prefixes_complete().collect(),
        }
    }

    /// Returns whether the given identifier names a builtin value.
    #[must_use]
    pub fn has_builtin_value(&self, identifier: &str) -> bool {
        self.values.contains_key(identifier)
    }

    /// Returns whether the given identifier names a builtin function.
    #[must_use]
    pub fn has_builtin_function(&self, identifier: &str) -> bool {
        self.functions.contains_key(identifier)
    }

    /// Returns the builtin value for the given identifier, if any.
    #[must_use]
    pub fn get_value(&self, identifier: &str) -> Option<&Value> {
        self.values.get(identifier).map(|v| &v.value)
    }

    /// Returns the builtin function for the given identifier, if any.
    #[must_use]
    pub fn get_function(&self, identifier: &str) -> Option<BuiltinFunctionFn> {
        self.functions
            .get(identifier)
            .map(|f| f.function)
    }

    /// Returns the builtin unit for the given name, if any.
    #[must_use]
    pub fn get_unit(&self, name: &str) -> Option<&Unit> {
        self.units.get(name).map(|u| &u.unit)
    }

    /// Returns an iterator over all builtin unit prefixes (name, multiplier).
    pub fn builtin_prefixes(&self) -> impl Iterator<Item = (&str, f64)> {
        self.prefixes
            .iter()
            .map(|(name, prefix)| (*name, prefix.value))
    }

    /// Returns documentation for all builtin units.
    ///
    /// Each item is the canonical unit name and a list of all aliases
    /// (which may not include the canonical name).
    pub fn builtin_units_docs(&self) -> impl Iterator<Item = (&'static str, Vec<&'static str>)> {
        let mut by_name: IndexMap<&'static str, Vec<&'static str>> = IndexMap::new();
        for unit in self.units.values() {
            by_name
                .entry(unit.readable_name)
                .or_default()
                .push(unit.alias);
        }
        by_name.into_iter()
    }

    /// Returns documentation for all builtin functions.
    ///
    /// Each item is the function name, its argument names, and its description.
    pub fn builtin_functions_docs(
        &self,
    ) -> impl Iterator<Item = (&'static str, (&'static [&'static str], &'static str))> + '_ {
        self.functions
            .iter()
            .map(|(name, f)| (*name, (f.args, f.description)))
    }

    /// Returns documentation for all builtin values.
    ///
    /// Each item is the value name, its description, and the value itself.
    pub fn builtin_values_docs(
        &self,
    ) -> impl Iterator<Item = (&'static str, (&'static str, Value))> + '_ {
        self.values
            .iter()
            .map(|(name, v)| (*name, (v.description, v.value.clone())))
    }

    /// Returns documentation for all builtin prefixes.
    ///
    /// Each item is the prefix name, its description, and its numeric value.
    pub fn builtin_prefixes_docs(
        &self,
    ) -> impl Iterator<Item = (&'static str, (&'static str, f64))> + '_ {
        self.prefixes
            .iter()
            .map(|(name, p)| (*name, (p.description, p.value)))
    }
}

impl Default for BuiltinRef {
    fn default() -> Self {
        Self::new()
    }
}
