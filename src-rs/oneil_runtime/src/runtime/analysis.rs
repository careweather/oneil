//! Dependency and reference analysis for the runtime.
//!
//! Delegates to the [`oneil_analysis`] crate with the runtime as the context.

use indexmap::IndexMap;
use oneil_analysis::{
    self as analysis,
    output::Independents,
    output::error::{IndependentsErrors, ModelEvalHasErrors, TreeErrors},
};
use oneil_frontend::{CompilationUnit, InstancedModel};
use oneil_shared::{
    EvalInstanceKey,
    load_result::LoadResult,
    paths::ModelPath,
    symbols::{BuiltinValueName, ParameterName, TestIndex},
};

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
        model_path: &ModelPath,
        parameter_name: &ParameterName,
    ) -> (Option<tree::Tree<tree::DependencyTreeValue>>, RuntimeErrors) {
        let (tree, tree_errors) = self.get_dependency_tree_internal(model_path, parameter_name);

        // includes indirect errors because the tree is built from evaluated models
        let include_indirect_errors = true;

        let errors = tree_errors
            .model_paths()
            .fold(RuntimeErrors::new(), |mut acc, path| {
                acc.extend(self.get_model_diagnostics(path, include_indirect_errors));
                acc
            });

        (tree, errors)
    }

    #[must_use]
    fn get_dependency_tree_internal(
        &mut self,
        model_path: &ModelPath,
        parameter_name: &ParameterName,
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
        model_path: &ModelPath,
        parameter_name: &ParameterName,
    ) -> (Option<tree::Tree<tree::ReferenceTreeValue>>, RuntimeErrors) {
        let (tree, tree_errors) = self.get_reference_tree_internal(model_path, parameter_name);

        // includes indirect errors because the tree is built from evaluated models
        let include_indirect_errors = true;

        let errors = tree_errors
            .model_paths()
            .fold(RuntimeErrors::new(), |mut acc, path| {
                acc.extend(self.get_model_diagnostics(path, include_indirect_errors));
                acc
            });

        (tree, errors)
    }

    #[must_use]
    fn get_reference_tree_internal(
        &mut self,
        model_path: &ModelPath,
        parameter_name: &ParameterName,
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
    pub fn get_independents(&mut self, model_path: &ModelPath) -> (Independents, RuntimeErrors) {
        let (independents, independents_errors) = self.get_independents_internal(model_path);

        // includes indirect errors because the tree is built from evaluated models
        let include_indirect_errors = true;

        let errors = independents_errors
            .paths()
            .fold(RuntimeErrors::new(), |mut acc, path| {
                acc.extend(self.get_model_diagnostics(path, include_indirect_errors));
                acc
            });

        (independents, errors)
    }

    #[must_use]
    fn get_independents_internal(
        &mut self,
        model_path: &ModelPath,
    ) -> (Independents, IndependentsErrors) {
        let _ = self.eval_model(model_path);
        analysis::get_independents(model_path, self)
    }
}

impl analysis::ExternalAnalysisContext for Runtime {
    fn get_all_model_ir(&self) -> IndexMap<EvalInstanceKey, &InstancedModel> {
        self.unit_graph_cache
            .iter()
            .fold(IndexMap::new(), |mut instances, (unit, graph)| {
                if let CompilationUnit::Model(path) = unit {
                    insert_model_ir_subtree(
                        graph.root.as_ref(),
                        &EvalInstanceKey::root(path.clone()),
                        &mut instances,
                    );

                    for (path, instance) in &graph.reference_pool {
                        insert_model_ir_subtree(
                            instance.as_ref(),
                            &EvalInstanceKey::root(path.clone()),
                            &mut instances,
                        );
                    }
                }

                instances
            })
    }

    fn get_evaluated_model(
        &self,
        model_path: &ModelPath,
    ) -> Option<LoadResult<&oneil_output::Model, ModelEvalHasErrors>> {
        let entry = self.eval_cache.get_entry(model_path)?;
        let result = entry.as_ref().map_err(|_error| ModelEvalHasErrors);
        Some(result)
    }

    fn lookup_builtin_variable(
        &self,
        identifier: &BuiltinValueName,
    ) -> Option<&oneil_output::Value> {
        self.builtins.get_value(identifier)
    }

    fn lookup_parameter_value(
        &self,
        instance_key: &EvalInstanceKey,
        parameter_name: &ParameterName,
    ) -> Option<Result<&oneil_output::Parameter, oneil_analysis::output::error::GetValueError>>
    {
        let entry = self.eval_cache.get_entry_instance(instance_key)?;
        let parameter = entry.value().map_or_else(
            || Err(oneil_analysis::output::error::GetValueError::Model),
            |model| {
                model
                    .parameters
                    .get(parameter_name)
                    .ok_or(oneil_analysis::output::error::GetValueError::Parameter)
            },
        );

        Some(parameter)
    }

    fn lookup_test_value(
        &self,
        instance_key: &EvalInstanceKey,
        test_index: TestIndex,
    ) -> Option<Result<&oneil_output::Test, oneil_analysis::output::error::GetTestValueError>> {
        let entry = self.eval_cache.get_entry_instance(instance_key)?;
        let test = entry.value().map_or_else(
            || Err(oneil_analysis::output::error::GetTestValueError::Model),
            |model| {
                model
                    .tests
                    .get(&test_index)
                    .ok_or(oneil_analysis::output::error::GetTestValueError::Test)
            },
        );

        Some(test)
    }
}

/// Inserts an instanced model and its owned submodel descendants keyed by eval instance.
fn insert_model_ir_subtree<'model>(
    instance: &'model InstancedModel,
    instance_key: &EvalInstanceKey,
    instances: &mut IndexMap<EvalInstanceKey, &'model InstancedModel>,
) {
    instances.insert(instance_key.clone(), instance);

    for (alias, submodel) in instance.submodels() {
        insert_model_ir_subtree(
            submodel.instance.as_ref(),
            &EvalInstanceKey {
                model_path: submodel.instance.path().clone(),
                instance_path: instance_key.instance_path.clone().child(alias.clone()),
            },
            instances,
        );
    }
}
