//! Dependency and reference analysis for the runtime.
//!
//! Delegates to the [`oneil_analysis`] crate with the runtime as the context.

use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use oneil_analysis::{
    self as analysis,
    output::Independents,
    output::error::{IndependentsErrors, ModelEvalHasErrors, TreeErrors},
};
use oneil_ir as ir;
use oneil_shared::load_result::LoadResult;

use super::Runtime;
use crate::output::{error::RuntimeErrors, tree};

impl Runtime {
    /// Gets the dependency tree for a specific parameter.
    ///
    /// The tree shows all parameters, builtin values, and external dependencies
    /// that the specified parameter depends on, recursively.
    ///
    /// Evaluates the model (and its dependencies) first so that the tree can be
    /// built from cached results.
    #[must_use]
    pub fn get_dependency_tree(
        &mut self,
        model_path: &Path,
        parameter_name: &str,
    ) -> (Option<tree::Tree<tree::DependencyTreeValue>>, RuntimeErrors) {
        let (tree, tree_errors) = self.get_dependency_tree_internal(model_path, parameter_name);

        let errors = tree_errors
            .model_paths()
            .fold(RuntimeErrors::new(), |mut acc, path| {
                acc.extend(self.get_model_errors(path));
                acc
            });

        (tree, errors)
    }

    #[must_use]
    fn get_dependency_tree_internal(
        &mut self,
        model_path: &Path,
        parameter_name: &str,
    ) -> (Option<tree::Tree<tree::DependencyTreeValue>>, TreeErrors) {
        let _ = self.eval_model(model_path);
        analysis::get_dependency_tree(model_path, parameter_name, self)
    }

    /// Gets the reference tree for a specific parameter.
    ///
    /// The tree shows all parameters that depend on the specified parameter, recursively.
    /// This is the inverse of the dependency tree.
    ///
    /// Evaluates the model (and its dependencies) first so that the tree can be
    /// built from cached results.
    #[must_use]
    pub fn get_reference_tree(
        &mut self,
        model_path: &Path,
        parameter_name: &str,
    ) -> (Option<tree::Tree<tree::ReferenceTreeValue>>, RuntimeErrors) {
        let (tree, tree_errors) = self.get_reference_tree_internal(model_path, parameter_name);

        let errors = tree_errors
            .model_paths()
            .fold(RuntimeErrors::new(), |mut acc, path| {
                acc.extend(self.get_model_errors(path));
                acc
            });

        (tree, errors)
    }

    #[must_use]
    fn get_reference_tree_internal(
        &mut self,
        model_path: &Path,
        parameter_name: &str,
    ) -> (Option<tree::Tree<tree::ReferenceTreeValue>>, TreeErrors) {
        let _ = self.eval_model(model_path);
        analysis::get_reference_tree(self, model_path, parameter_name)
    }

    /// Gets independent parameters for a model and its referenced models.
    ///
    /// A parameter is independent if it has no parameter or external dependencies
    /// (it may still depend on builtin values). Evaluates the model first, then
    /// returns an [`Independents`] (model path → parameter name → value) and any errors.
    #[must_use]
    pub fn get_independents(&mut self, model_path: &Path) -> (Independents, RuntimeErrors) {
        let (independents, independents_errors) = self.get_independents_internal(model_path);

        let errors = independents_errors
            .paths()
            .fold(RuntimeErrors::new(), |mut acc, path| {
                acc.extend(self.get_model_errors(path));
                acc
            });

        (independents, errors)
    }

    #[must_use]
    fn get_independents_internal(
        &mut self,
        model_path: &Path,
    ) -> (Independents, IndependentsErrors) {
        let _ = self.eval_model(model_path);
        analysis::get_independents(model_path, self)
    }
}

impl analysis::ExternalAnalysisContext for Runtime {
    fn get_all_model_ir(&self) -> IndexMap<&PathBuf, &ir::Model> {
        self.ir_cache
            .iter()
            .filter_map(|(path, result)| result.value().map(|ir| (path, ir)))
            .collect()
    }

    fn get_evaluated_model(
        &self,
        model_path: &Path,
    ) -> Option<LoadResult<&oneil_output::Model, ModelEvalHasErrors>> {
        let entry = self.eval_cache.get_entry(model_path)?;
        let result = entry.as_ref().map_err(|_error| ModelEvalHasErrors);
        Some(result)
    }

    fn lookup_builtin_variable(
        &self,
        identifier: &oneil_ir::Identifier,
    ) -> Option<&oneil_output::Value> {
        self.builtins.get_value(identifier.as_str())
    }

    fn lookup_parameter_value(
        &self,
        model_path: &Path,
        parameter_name: &str,
    ) -> Option<Result<oneil_output::Parameter, oneil_analysis::output::error::GetValueError>> {
        let entry = self.eval_cache.get_entry(model_path)?;
        let parameter = entry.value().map_or_else(
            || Err(oneil_analysis::output::error::GetValueError::Model),
            |model| {
                model
                    .parameters
                    .get(parameter_name)
                    .cloned()
                    .ok_or(oneil_analysis::output::error::GetValueError::Parameter)
            },
        );

        Some(parameter)
    }
}
