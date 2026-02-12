//! Builtin documentation and lookup for the runtime.

use oneil_output::Value;

use super::Runtime;

impl Runtime {
    /// Returns documentation for all builtin units.
    pub fn builtin_units_docs(&self) -> impl Iterator<Item = (&'static str, Vec<&'static str>)> {
        self.builtins.builtin_units_docs()
    }

    /// Returns documentation for all builtin functions.
    pub fn builtin_functions_docs(
        &self,
    ) -> impl Iterator<Item = (&'static str, (&'static [&'static str], &'static str))> + '_ {
        self.builtins.builtin_functions_docs()
    }

    /// Returns documentation for all builtin values.
    pub fn builtin_values_docs(
        &self,
    ) -> impl Iterator<Item = (&'static str, (&'static str, Value))> + '_ {
        self.builtins.builtin_values_docs()
    }

    /// Returns documentation for all builtin prefixes.
    pub fn builtin_prefixes_docs(
        &self,
    ) -> impl Iterator<Item = (&'static str, (&'static str, f64))> + '_ {
        self.builtins.builtin_prefixes_docs()
    }
}
