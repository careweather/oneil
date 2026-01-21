//!  A dependency graph for the results of evaluating Oneil models.

use std::path::PathBuf;

use indexmap::{IndexMap, IndexSet};

/// A dependency graph for the results of evaluating Oneil models.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyGraph {
    depends_on: IndexMap<PathBuf, IndexMap<String, IndexSet<Dependency>>>,
    required_by: IndexMap<PathBuf, IndexMap<String, IndexSet<Requires>>>,
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

    /// Adds a dependency to the graph.
    pub fn add_depends_on(
        &mut self,
        param_path: PathBuf,
        param_name: String,
        dependency: Dependency,
    ) {
        self.depends_on
            .entry(param_path.clone())
            .or_default()
            .entry(param_name.clone())
            .or_default()
            .insert(dependency.clone());

        match dependency {
            Dependency::Builtin(_builtin) => { /* builtins don't track requirements */ }
            Dependency::Parameter(dependency_param_name) => {
                // in the same model, so the param path is the same
                self.required_by
                    .entry(param_path)
                    .or_default()
                    .entry(dependency_param_name)
                    .or_default()
                    .insert(Requires::Parameter(param_name));
            }
            Dependency::External(dependency_path, dependency_param_name) => {
                self.required_by
                    .entry(dependency_path)
                    .or_default()
                    .entry(dependency_param_name)
                    .or_default()
                    .insert(Requires::External(param_path, param_name));
            }
        }
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a variable that a parameter depends on.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Dependency {
    /// The parameter depends on a builtin variable.
    Builtin(String),
    /// The parameter depends on a parameter defined in the current model.
    Parameter(String),
    /// The parameter depends on a parameter defined in an external model.
    External(PathBuf, String),
}

/// Represents a parameter that requires the
/// parameter that this is being stored with.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Requires {
    /// The parameter is required by a parameter defined in the current model.
    Parameter(String),
    /// The parameter is required by a parameter defined in an external model.
    External(PathBuf, String),
}
