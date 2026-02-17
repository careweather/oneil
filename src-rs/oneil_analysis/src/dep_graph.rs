//! A dependency graph for the results of evaluating Oneil models.

use std::path::{Path, PathBuf};

use indexmap::{IndexMap, IndexSet};
use oneil_output::{BuiltinDependency, DependencySet, ExternalDependency, ParameterDependency};

/// A dependency graph for the results of evaluating Oneil models.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyGraph {
    depends_on: IndexMap<PathBuf, IndexMap<String, DependencySet>>,
    referenced_by: IndexMap<PathBuf, IndexMap<String, ReferenceSet>>,
}

impl DependencyGraph {
    /// Creates a new dependency graph.
    #[must_use]
    pub fn new() -> Self {
        Self {
            depends_on: IndexMap::new(),
            referenced_by: IndexMap::new(),
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

        let reference = ParameterReference {
            parameter_name: param_name,
        };

        self.referenced_by
            .entry(param_path)
            .or_default()
            .entry(dependency_parameter_name)
            .or_default()
            .parameter_references
            .insert(reference);
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

        let reference = ExternalReference {
            model_path: param_path,
            parameter_name: param_name,
            using_reference_name: dependency_reference_name,
        };

        self.referenced_by
            .entry(dependency_model_path)
            .or_default()
            .entry(dependency_parameter_name)
            .or_default()
            .external_references
            .insert(reference);
    }

    /// Returns the parameters that a given parameter depends on.
    #[must_use]
    pub fn dependents(&self, model_path: &Path, parameter_name: &str) -> Option<&DependencySet> {
        let model = self.depends_on.get(model_path)?;
        model.get(parameter_name)
    }

    /// Returns the parameters that reference a given parameter.
    #[must_use]
    pub fn references(&self, model_path: &Path, parameter_name: &str) -> Option<&ReferenceSet> {
        let model = self.referenced_by.get(model_path)?;
        model.get(parameter_name)
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// A set of parameters that reference a given parameter.
///
/// This structure tracks which other parameters or external models reference
/// a given parameter. This is the reverse mapping of dependencies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceSet {
    /// Parameters within the same model that reference this parameter.
    pub parameter_references: IndexSet<ParameterReference>,
    /// External models that reference this parameter.
    pub external_references: IndexSet<ExternalReference>,
}

impl ReferenceSet {
    /// Creates a new empty reference set.
    #[must_use]
    pub fn new() -> Self {
        Self {
            parameter_references: IndexSet::new(),
            external_references: IndexSet::new(),
        }
    }
}

impl Default for ReferenceSet {
    fn default() -> Self {
        Self::new()
    }
}

/// A reference from another parameter within the same model.
///
/// This represents the reverse relationship of a `ParameterDependency`:
/// it indicates that another parameter in the same model references this parameter.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParameterReference {
    /// The name of the parameter that references this parameter.
    pub parameter_name: String,
}

/// A reference from an external model.
///
/// This represents the reverse relationship of an `ExternalDependency`:
/// it indicates that a parameter in another model references this parameter.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternalReference {
    /// The path to the model that references this parameter.
    pub model_path: PathBuf,
    /// The name of the parameter in the external model that references this parameter.
    pub parameter_name: String,
    /// The reference name used by the external model to access this model.
    pub using_reference_name: String,
}
