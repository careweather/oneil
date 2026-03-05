//! Dependency sets for parameters.

use std::path::PathBuf;

use indexmap::IndexSet;

/// A set of dependencies for a parameter.
///
/// This structure groups all types of dependencies that a parameter may have:
/// builtin functions, other parameters in the same model, and parameters from
/// external models.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencySet {
    /// Dependencies on builtin values.
    pub builtin_dependencies: IndexSet<BuiltinDependency>,
    /// Dependencies on other parameters within the same model.
    pub parameter_dependencies: IndexSet<ParameterDependency>,
    /// Dependencies on parameters from external models.
    pub external_dependencies: IndexSet<ExternalDependency>,
}

impl DependencySet {
    /// Creates a new empty dependency set.
    #[must_use]
    pub fn new() -> Self {
        Self {
            builtin_dependencies: IndexSet::new(),
            parameter_dependencies: IndexSet::new(),
            external_dependencies: IndexSet::new(),
        }
    }
}

impl Default for DependencySet {
    fn default() -> Self {
        Self::new()
    }
}

/// A dependency on a builtin value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BuiltinDependency {
    /// The identifier (name) of the builtin value.
    pub ident: String,
}

/// A dependency on another parameter within the same model.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParameterDependency {
    /// The name of the parameter that this depends on.
    pub parameter_name: String,
}

/// A dependency on a parameter from an external model.
///
/// External dependencies represent references to parameters in other model files,
/// accessed through model references. These create cross-model dependency relationships.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternalDependency {
    /// The path to the external model containing the parameter.
    pub model_path: PathBuf,
    /// The reference name used to access the external model.
    pub reference_name: String,
    /// The name of the parameter in the external model.
    pub parameter_name: String,
}
