//!  A dependency graph for the results of evaluating Oneil models.

use std::path::{Path, PathBuf};

use indexmap::{IndexMap, IndexSet};
use oneil_shared::span::Span;

use crate::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct DependencyTreeValue {
    pub reference_name: Option<String>,
    pub parameter_name: String,
    pub parameter_value: Value,
    pub display_info: Option<(PathBuf, Span)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RequiresTreeValue {
    pub model_path: PathBuf,
    pub parameter_name: String,
    pub parameter_value: Value,
    pub display_info: (PathBuf, Span),
}

/// A dependency graph for the results of evaluating Oneil models.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyGraph {
    depends_on: IndexMap<PathBuf, IndexMap<String, DependencySet>>,
    required_by: IndexMap<PathBuf, IndexMap<String, RequiresSet>>,
}

impl DependencyGraph {
    /// Creates a new dependency graph.
    #[must_use]
    pub fn new() -> Self {
        Self {
            depends_on: IndexMap::new(),
            required_by: IndexMap::new(),
        }
    }

    /// Adds a builtin dependency to the graph.
    pub fn add_depends_on_builtin(
        &mut self,
        param_path: PathBuf,
        param_name: String,
        dependency: BuiltinDependency,
    ) {
        self.depends_on
            .entry(param_path)
            .or_default()
            .entry(param_name)
            .or_default()
            .builtin_dependencies
            .insert(dependency);
    }

    /// Adds a parameter dependency to the graph.
    pub fn add_depends_on_parameter(
        &mut self,
        param_path: PathBuf,
        param_name: String,
        dependency: ParameterDependency,
    ) {
        self.depends_on
            .entry(param_path.clone())
            .or_default()
            .entry(param_name.clone())
            .or_default()
            .parameter_dependencies
            .insert(dependency.clone());

        let ParameterDependency {
            parameter_name: dependency_parameter_name,
        } = dependency;

        let requires = ParameterRequires {
            parameter_name: param_name,
        };

        self.required_by
            // the model path is the same for both parameters
            // because they are in the same model
            .entry(param_path)
            .or_default()
            .entry(dependency_parameter_name)
            .or_default()
            .parameter_requires
            .insert(requires);
    }

    /// Adds an external dependency to the graph.
    pub fn add_depends_on_external(
        &mut self,
        param_path: PathBuf,
        param_name: String,
        dependency: ExternalDependency,
    ) {
        self.depends_on
            .entry(param_path.clone())
            .or_default()
            .entry(param_name.clone())
            .or_default()
            .external_dependencies
            .insert(dependency.clone());

        let ExternalDependency {
            model_path: dependency_model_path,
            reference_name: dependency_reference_name,
            parameter_name: dependency_parameter_name,
        } = dependency;

        let requires = ExternalRequires {
            model_path: param_path,
            parameter_name: param_name,
            using_reference_name: dependency_reference_name,
        };

        self.required_by
            .entry(dependency_model_path)
            .or_default()
            .entry(dependency_parameter_name)
            .or_default()
            .external_requires
            .insert(requires);
    }

    pub fn depends_on(&self, model_path: &Path, parameter_name: &str) -> Option<&DependencySet> {
        self.depends_on
            .get(model_path)
            .and_then(|model| model.get(parameter_name))
    }

    pub fn requires(&self, model_path: &Path, parameter_name: &str) -> Option<&RequiresSet> {
        self.required_by
            .get(model_path)
            .and_then(|model| model.get(parameter_name))
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

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

/// A set of parameters that require a given parameter.
///
/// This structure tracks which other parameters or external models require
/// a given parameter. This is the reverse mapping of dependencies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequiresSet {
    /// Parameters within the same model that require this parameter.
    pub parameter_requires: IndexSet<ParameterRequires>,
    /// External models that require this parameter.
    pub external_requires: IndexSet<ExternalRequires>,
}

impl RequiresSet {
    /// Creates a new empty requires set.
    #[must_use]
    pub fn new() -> Self {
        Self {
            parameter_requires: IndexSet::new(),
            external_requires: IndexSet::new(),
        }
    }
}

impl Default for RequiresSet {
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

/// A requirement from another parameter within the same model.
///
/// This represents the reverse relationship of a `ParameterDependency`:
/// it indicates that another parameter in the same model requires this parameter.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParameterRequires {
    /// The name of the parameter that requires this parameter.
    pub parameter_name: String,
}

/// A requirement from an external model.
///
/// This represents the reverse relationship of an `ExternalDependency`:
/// it indicates that a parameter in another model requires this parameter.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternalRequires {
    /// The path to the model that requires this parameter.
    pub model_path: PathBuf,
    /// The name of the parameter in the external model that requires this parameter.
    pub parameter_name: String,
    /// The reference name used by the external model to access this model.
    pub using_reference_name: String,
}
