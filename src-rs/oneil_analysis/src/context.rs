//! Context types for tree traversal and analysis.

use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use oneil_ir as ir;
use oneil_output::{DependencySet, Model, Parameter, Value};
use oneil_shared::load_result::LoadResult;

use crate::dep_graph::{DependencyGraph, ReferenceSet};
use crate::output::error::{GetValueError, ModelEvalHasErrors};

/// External context provided to tree operations.
pub trait ExternalAnalysisContext {
    /// Returns the full map of model paths to their IR models.
    fn get_all_model_ir(&self) -> IndexMap<&PathBuf, &ir::Model>;

    /// Returns the value of a builtin variable by identifier, if defined.
    fn lookup_builtin_variable(&self, identifier: &ir::Identifier) -> Option<&Value>;

    /// Looks up the evaluated model at the given path.
    ///
    /// Returns `None` if the model is not in the context. Otherwise returns a
    /// [`LoadResult`]: success with the model reference, partial with the model and
    /// [`ModelEvalHasErrors`], or failure.
    fn get_evaluated_model(
        &self,
        model_path: &Path,
    ) -> Option<LoadResult<&Model, ModelEvalHasErrors>>;

    /// Looks up an evaluated parameter by model path and parameter name.
    ///
    /// Returns `None` if the model is not in the context, `Some(Err(...))` on value errors,
    /// or `Some(Ok(parameter))` when the parameter is found.
    fn lookup_parameter_value(
        &self,
        model_path: &Path,
        parameter_name: &str,
    ) -> Option<Result<Parameter, GetValueError>>;
}

/// Context for tree operations that holds a mutable reference to an [`ExternalAnalysisContext`].
pub struct TreeContext<'external, E: ExternalAnalysisContext> {
    /// Mutable reference to the external tree context.
    external: &'external mut E,
    /// The dependency graph for the tree.
    dependency_graph: DependencyGraph,
}

impl<'external, E: ExternalAnalysisContext> TreeContext<'external, E> {
    /// Creates a new tree context with the given mutable reference to an external context.
    #[must_use]
    pub const fn new(external: &'external mut E, dependency_graph: DependencyGraph) -> Self {
        Self {
            external,
            dependency_graph,
        }
    }

    /// Returns the value of a builtin variable by identifier, delegating to the external context.
    #[must_use]
    pub fn lookup_builtin_variable(&self, identifier: &str) -> Option<&Value> {
        let identifier = ir::Identifier::new(identifier.to_string());
        self.external.lookup_builtin_variable(&identifier)
    }

    /// Returns the dependents of the given parameter, from the dependency graph.
    ///
    /// If the parameter is not found, it is assumed that there are no dependents, so
    /// an empty [`DependencySet`] is returned.
    #[must_use]
    pub fn dependents(&self, model_path: &Path, parameter_name: &str) -> DependencySet {
        self.dependency_graph
            .dependents(model_path, parameter_name)
            .cloned()
            .unwrap_or_default()
    }

    /// Returns the parameters that reference the given parameter, from the dependency graph.
    #[must_use]
    pub fn references(&self, model_path: &Path, parameter_name: &str) -> ReferenceSet {
        self.dependency_graph
            .references(model_path, parameter_name)
            .cloned()
            .unwrap_or_default()
    }

    /// Looks up an evaluated parameter by model path and parameter name, delegating to the external context.
    #[must_use]
    pub fn lookup_parameter_value(
        &self,
        model_path: &Path,
        parameter_name: &str,
    ) -> Option<Result<Parameter, GetValueError>> {
        self.external
            .lookup_parameter_value(model_path, parameter_name)
    }
}
