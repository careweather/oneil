//!  A dependency graph for the results of evaluating Oneil models.

use std::path::PathBuf;

use indexmap::{IndexMap, IndexSet};

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
            .entry(param_path.clone())
            .or_default()
            .entry(param_name.clone())
            .or_default()
            .builtin_dependencies
            .insert(dependency.clone());
    }

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

        self.required_by
            // the model path is the same for both parameters
            // because they are in the same model
            .entry(param_path.clone())
            .or_default()
            .entry(dependency.parameter_name.clone())
            .or_default()
            .parameter_requires
            .insert(ParameterRequires {
                parameter_name: param_name.clone(),
            });
    }

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

        self.required_by
            .entry(dependency.model_path.clone())
            .or_default()
            .entry(dependency.parameter_name.clone())
            .or_default()
            .external_requires
            .insert(ExternalRequires {
                model_path: param_path.clone(),
                parameter_name: param_name.clone(),
                using_reference_name: dependency.reference_name.clone(),
            });
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencySet {
    pub builtin_dependencies: IndexSet<BuiltinDependency>,
    pub parameter_dependencies: IndexSet<ParameterDependency>,
    pub external_dependencies: IndexSet<ExternalDependency>,
}

impl DependencySet {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequiresSet {
    parameter_requires: IndexSet<ParameterRequires>,
    external_requires: IndexSet<ExternalRequires>,
}

impl RequiresSet {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BuiltinDependency {
    pub ident: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParameterDependency {
    pub parameter_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternalDependency {
    pub model_path: PathBuf,
    pub reference_name: String,
    pub parameter_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParameterRequires {
    pub parameter_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternalRequires {
    pub model_path: PathBuf,
    pub parameter_name: String,
    pub using_reference_name: String,
}
