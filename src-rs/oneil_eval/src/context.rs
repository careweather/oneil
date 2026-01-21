use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use indexmap::IndexMap;

use oneil_ir as ir;
use oneil_shared::span::Span;

use crate::{
    builtin::{BuiltinFunction, BuiltinMap},
    error::{EvalError, ModelError},
    output::{
        dependency::{DependencyGraph, DependencyTreeValue, RequiresTreeValue},
        eval_result,
        tree::Tree,
    },
    value::{Unit, Value},
};

#[derive(Debug, Clone)]
pub struct Model {
    parameters: IndexMap<String, Result<eval_result::Parameter, Vec<EvalError>>>,
    submodels: IndexMap<String, PathBuf>,
    references: IndexMap<String, PathBuf>,
    tests: Vec<Result<eval_result::Test, Vec<EvalError>>>,
}

impl Model {
    pub fn new() -> Self {
        Self {
            parameters: IndexMap::new(),
            submodels: IndexMap::new(),
            references: IndexMap::new(),
            tests: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EvalContext<F: BuiltinFunction> {
    models: IndexMap<PathBuf, Model>,
    // TODO: update this to hold the actual Python import results
    python_imports: IndexMap<PathBuf, Result<(), EvalError>>,
    current_model: Option<PathBuf>,
    active_python_imports: HashSet<PathBuf>,
    active_references: HashSet<PathBuf>,
    builtins: BuiltinMap<F>,
}

impl<F: BuiltinFunction> EvalContext<F> {
    pub fn new(builtins: BuiltinMap<F>) -> Self {
        Self {
            models: IndexMap::new(),
            python_imports: IndexMap::new(),
            current_model: None,
            active_python_imports: HashSet::new(),
            active_references: HashSet::new(),
            builtins,
        }
    }

    /// Looks up the given builtin variable and returns the corresponding value.
    ///
    /// # Panics
    ///
    /// Panics if the builtin value is not defined. This should never be the case.
    /// If it is, then there is a bug either in the model resolver when it resolves builtin variables
    /// or in the builtin map when it defines the builtin values.
    pub fn lookup_builtin_variable(&self, identifier: &ir::Identifier) -> Value {
        self.builtins
            .values
            .get(identifier.as_str())
            .expect("builtin value should be defined")
            .clone()
    }

    pub fn lookup_parameter_value(
        &self,
        parameter_name: &ir::ParameterName,
        variable_span: Span,
    ) -> Result<Value, Vec<EvalError>> {
        let current_model = self
            .current_model
            .as_ref()
            .expect("current model should be set when looking up a parameter");

        self.lookup_model_parameter_value_internal(current_model, parameter_name, variable_span)
    }

    pub fn lookup_model_parameter_value(
        &self,
        model: &ir::ModelPath,
        parameter_name: &ir::ParameterName,
        variable_span: Span,
    ) -> Result<Value, Vec<EvalError>> {
        self.lookup_model_parameter_value_internal(model.as_ref(), parameter_name, variable_span)
    }

    fn lookup_model_parameter_value_internal(
        &self,
        model_path: &Path,
        parameter_name: &ir::ParameterName,
        variable_span: Span,
    ) -> Result<Value, Vec<EvalError>> {
        let model = self
            .models
            .get(model_path)
            .expect("current model should be created when set");

        model
            .parameters
            .get(parameter_name.as_str())
            .expect("parameter should be defined")
            .clone()
            .map(|parameter| parameter.value)
            .map_err(|_errors| {
                vec![EvalError::ParameterHasError {
                    parameter_name: parameter_name.as_str().to_string(),
                    variable_span,
                }]
            })
    }

    // TODO: figure out what error this should actually be
    pub fn evaluate_builtin_function(
        &self,
        identifier: &ir::Identifier,
        identifier_span: Span,
        args: Vec<(Value, Span)>,
    ) -> Result<Value, Vec<EvalError>> {
        self.builtins
            .functions
            .get(identifier.as_str())
            .expect("builtin function should be defined")
            .call(identifier_span, args)
    }

    pub fn evaluate_imported_function(
        &self,
        identifier: &ir::Identifier,
        identifier_span: Span,
        args: Vec<(Value, Span)>,
    ) -> Result<Value, Vec<EvalError>> {
        let _ = (self, identifier, args);
        Err(vec![EvalError::Unsupported {
            relevant_span: identifier_span,
            feature_name: Some("imported function".to_string()),
            will_be_supported: true,
        }])
    }

    pub fn lookup_unit(&self, name: &str) -> Option<Unit> {
        self.builtins.units.get(name).cloned()
    }

    pub const fn available_prefixes(&self) -> &IndexMap<String, f64> {
        &self.builtins.prefixes
    }

    pub fn load_python_import(&mut self, python_path: PathBuf, python_import_span: Span) {
        self.python_imports.insert(
            python_path,
            Err(EvalError::Unsupported {
                relevant_span: python_import_span,
                feature_name: Some("Python import".to_string()),
                will_be_supported: true,
            }),
        );
    }

    pub fn set_active_model(&mut self, model_path: PathBuf) {
        self.models
            .entry(model_path.clone())
            .or_insert_with(Model::new);

        self.current_model = Some(model_path);
    }

    pub fn clear_active_model(&mut self) {
        self.current_model = None;
    }

    pub fn clear_active_python_imports(&mut self) {
        self.active_python_imports.clear();
    }

    pub fn activate_python_import(&mut self, python_import: PathBuf) {
        self.active_python_imports.insert(python_import);
    }

    pub fn add_parameter_result(
        &mut self,
        parameter_name: String,
        result: Result<eval_result::Parameter, Vec<EvalError>>,
    ) {
        // TODO: Maybe use type state pattern to enforce this?
        let Some(current_model) = self.current_model.as_ref() else {
            panic!("current model should be set when adding a parameter result");
        };

        let model = self
            .models
            .get_mut(current_model)
            .expect("current model should be created when set");

        model.parameters.insert(parameter_name, result);
    }

    pub fn add_submodel(&mut self, submodel_name: &str, submodel_import: &ir::ModelPath) {
        let Some(current_model) = self.current_model.as_ref() else {
            panic!("current model should be set when adding a submodel");
        };

        let model = self
            .models
            .get_mut(current_model)
            .expect("current model should be created when set");

        model.submodels.insert(
            submodel_name.to_string(),
            submodel_import.as_ref().to_path_buf(),
        );
    }

    pub fn add_reference(&mut self, reference_name: &str, reference_path: &ir::ModelPath) {
        let Some(current_model) = self.current_model.as_ref() else {
            panic!("current model should be set when adding a reference");
        };

        let model = self
            .models
            .get_mut(current_model)
            .expect("current model should be created when set");

        model.references.insert(
            reference_name.to_string(),
            reference_path.as_ref().to_path_buf(),
        );
    }

    pub fn add_test_result(&mut self, test_result: Result<eval_result::Test, Vec<EvalError>>) {
        let Some(current_model) = self.current_model.as_ref() else {
            panic!("current model should be set when adding a test result");
        };

        let model = self
            .models
            .get_mut(current_model)
            .expect("current model should be created when set");

        model.tests.push(test_result);
    }

    pub fn activate_reference(&mut self, reference: PathBuf) {
        self.active_references.insert(reference);
    }

    /// Gets the result of a model.
    ///
    /// Returns a model with all valid results, as well as
    /// a list of any errors that occurred during evaluation.
    ///
    /// If no errors occurred, the list of errors will be empty.
    pub fn get_model_result(&self, model_path: &Path) -> Option<eval_result::EvalResult> {
        if !self.models.contains_key(model_path) {
            return None;
        }

        let mut eval_result = eval_result::EvalResult::new(model_path.to_path_buf());

        self.collect_model_info(model_path, &mut eval_result);

        Some(eval_result)
    }

    fn collect_model_info(&self, model_path: &Path, eval_result: &mut eval_result::EvalResult) {
        if eval_result.model_is_visited(model_path) {
            return;
        }

        let Some(model) = self.models.get(model_path) else {
            panic!("model should exist");
        };

        // NOTE: we only need to do references because all submodels are also references
        for reference_path in model.references.values() {
            self.collect_model_info(reference_path, eval_result);
        }

        let submodels = model.submodels.clone();
        let references = model.references.clone();

        let (parameters, parameter_errors) = Self::collect_parameters(model, model_path);
        let (tests, test_errors) = Self::collect_tests(model, model_path);

        let model_result = eval_result::Model {
            path: model_path.to_path_buf(),
            submodels,
            references,
            parameters,
            tests,
        };

        let errors = [parameter_errors, test_errors].concat();

        eval_result.add_model(model_path.to_path_buf(), model_result, errors);
    }

    /// Collects parameter results.
    ///
    /// Returns a tuple of (successful parameters, errors).
    fn collect_parameters(
        model: &Model,
        model_path: &Path,
    ) -> (IndexMap<String, eval_result::Parameter>, Vec<ModelError>) {
        model
            .parameters
            .iter()
            .map(|(name, parameter_result)| {
                parameter_result
                    .clone()
                    .map(|parameter_result| (name.clone(), parameter_result))
            })
            .fold(
                (IndexMap::new(), Vec::new()),
                |(mut parameters, mut parameter_errors), result| match result {
                    Ok((name, parameter_result)) => {
                        parameters.insert(name, parameter_result);
                        (parameters, parameter_errors)
                    }
                    Err(errors) => {
                        let errors = errors.into_iter().map(|error| ModelError {
                            model_path: model_path.to_path_buf(),
                            error,
                        });

                        parameter_errors.extend(errors);
                        (parameters, parameter_errors)
                    }
                },
            )
    }

    /// Collects test results.
    ///
    /// Returns a tuple of (successful tests, errors).
    fn collect_tests(
        model: &Model,
        model_path: &Path,
    ) -> (Vec<eval_result::Test>, Vec<ModelError>) {
        model.tests.iter().fold(
            (Vec::new(), Vec::new()),
            |(mut tests, mut test_errors), test_result| match test_result.clone() {
                Ok(test_result) => {
                    tests.push(test_result);
                    (tests, test_errors)
                }
                Err(errors) => {
                    let errors = errors.into_iter().map(|error| ModelError {
                        model_path: model_path.to_path_buf(),
                        error,
                    });

                    test_errors.extend(errors);
                    (tests, test_errors)
                }
            },
        )
    }

    /// Gets the dependency graph for all models in the context.
    pub fn get_dependency_graph(&self) -> DependencyGraph {
        let mut dependency_graph = DependencyGraph::new();

        for (model_path, model) in &self.models {
            for parameter in model.parameters.values() {
                let Ok(parameter) = parameter else {
                    continue;
                };

                let dependencies = &parameter.dependencies;

                for dependency in &dependencies.builtin_dependencies {
                    dependency_graph.add_depends_on_builtin(
                        model_path.clone(),
                        parameter.ident.clone(),
                        dependency.clone(),
                    );
                }

                for dependency in &dependencies.parameter_dependencies {
                    dependency_graph.add_depends_on_parameter(
                        model_path.clone(),
                        parameter.ident.clone(),
                        dependency.clone(),
                    );
                }

                for dependency in &dependencies.external_dependencies {
                    dependency_graph.add_depends_on_external(
                        model_path.clone(),
                        parameter.ident.clone(),
                        dependency.clone(),
                    );
                }
            }
        }

        dependency_graph
    }

    pub fn get_dependency_parameter_tree(
        &self,
        model_path: &Path,
        reference_name: Option<&str>,
        parameter_name: &str,
    ) -> Option<Tree<DependencyTreeValue>> {
        let dependency_graph = self.get_dependency_graph();
        return recurse(
            self,
            model_path,
            reference_name,
            parameter_name,
            &dependency_graph,
        );

        #[expect(
            clippy::items_after_statements,
            reason = "this is an internal recursive function, we keep it here for clarity"
        )]
        fn recurse<F: BuiltinFunction>(
            context: &EvalContext<F>,
            model_path: &Path,
            reference_name: Option<&str>,
            parameter_name: &str,
            dependency_graph: &DependencyGraph,
        ) -> Option<Tree<DependencyTreeValue>> {
            let value =
                context.get_dependency_tree_value(model_path, reference_name, parameter_name)?;

            let deps = dependency_graph.depends_on(model_path, parameter_name)?;

            let builtin_deps = deps.builtin_dependencies.iter().map(|dep| {
                let parameter_value =
                    context.lookup_builtin_variable(&ir::Identifier::new(dep.ident.clone()));

                let tree_value = DependencyTreeValue {
                    reference_name: None,
                    parameter_name: dep.ident.clone(),
                    parameter_value,
                    display_info: None,
                };

                Tree::new(tree_value, Vec::new())
            });

            let parameter_deps = deps.parameter_dependencies.iter().filter_map(|dep| {
                recurse(
                    context,
                    model_path,
                    reference_name,
                    &dep.parameter_name,
                    dependency_graph,
                )
            });

            let external_deps = deps.external_dependencies.iter().filter_map(|dep| {
                recurse(
                    context,
                    &dep.model_path,
                    Some(&dep.reference_name),
                    &dep.parameter_name,
                    dependency_graph,
                )
            });

            let children = builtin_deps
                .chain(parameter_deps)
                .chain(external_deps)
                .collect::<Vec<_>>();

            Some(Tree::new(value, children))
        }
    }

    fn get_dependency_tree_value(
        &self,
        model_path: &Path,
        reference_name: Option<&str>,
        parameter_name: &str,
    ) -> Option<DependencyTreeValue> {
        let model = self.models.get(model_path)?;
        let parameter = model.parameters.get(parameter_name)?.as_ref().ok()?;

        let reference_name = reference_name.map(str::to_string);
        let parameter_name = parameter_name.to_string();

        let parameter_value = parameter.value.clone();

        let span = parameter.expr_span;
        let display_info = Some((model_path.to_path_buf(), span));

        let tree_value = DependencyTreeValue {
            reference_name,
            parameter_name,
            parameter_value,
            display_info,
        };

        Some(tree_value)
    }

    pub fn get_requires_tree(
        &self,
        model_path: &Path,
        parameter_name: &str,
    ) -> Option<Tree<RequiresTreeValue>> {
        let dependency_graph = self.get_dependency_graph();
        return recurse(self, model_path, parameter_name, &dependency_graph);

        #[expect(
            clippy::items_after_statements,
            reason = "this is an internal recursive function, we keep it here for clarity"
        )]
        fn recurse<F: BuiltinFunction>(
            context: &EvalContext<F>,
            model_path: &Path,
            parameter_name: &str,
            dependency_graph: &DependencyGraph,
        ) -> Option<Tree<RequiresTreeValue>> {
            let value = context.get_requires_tree_value(model_path, parameter_name)?;

            let deps = dependency_graph.requires(model_path, parameter_name)?;

            let parameter_deps = deps.parameter_requires.iter().filter_map(|dep| {
                recurse(context, model_path, &dep.parameter_name, dependency_graph)
            });

            let external_deps = deps.external_requires.iter().filter_map(|dep| {
                recurse(
                    context,
                    &dep.model_path,
                    &dep.parameter_name,
                    dependency_graph,
                )
            });

            let children = parameter_deps.chain(external_deps).collect::<Vec<_>>();

            Some(Tree::new(value, children))
        }
    }

    fn get_requires_tree_value(
        &self,
        model_path: &Path,
        parameter_name: &str,
    ) -> Option<RequiresTreeValue> {
        let model = self.models.get(model_path)?;
        let parameter = model.parameters.get(parameter_name)?.as_ref().ok()?;

        let model_path = model_path.to_path_buf();

        let parameter_name = parameter_name.to_string();

        let parameter_value = parameter.value.clone();

        let span = parameter.expr_span;
        let display_info = (model_path.clone(), span);

        let tree_value = RequiresTreeValue {
            model_path,
            parameter_name,
            parameter_value,
            display_info,
        };

        Some(tree_value)
    }
}
