use std::io::Error as IoError;
use std::path::{Path, PathBuf};

use indexmap::{IndexMap, IndexSet};
use oneil_eval::value::{Unit, Value};
use oneil_eval::{self as eval, EvalError, ExternalEvaluationContext, IrLoadError};
use oneil_model_resolver as model_resolver;
use oneil_parser as parser;
use oneil_shared::error::{AsOneilError, OneilError};
use oneil_shared::span::Span;

use crate::{
    cache::{AstCache, EvalCache, IrCache, SourceCache},
    output::{self, ast, ir},
    std_builtin::StdBuiltins,
};

/// Runtime for the Oneil programming language.
///
/// The runtime manages caches for source files, ASTs, and IR, and provides
/// methods to load and process Oneil models.
#[derive(Debug)]
pub struct Runtime {
    source_cache: SourceCache,
    ast_cache: AstCache,
    ir_cache: IrCache,
    eval_cache: EvalCache,
    builtins: StdBuiltins,
}

impl Runtime {
    /// Creates a new runtime instance with empty caches.
    #[must_use]
    pub fn new() -> Self {
        Self {
            source_cache: SourceCache::new(),
            ast_cache: AstCache::new(),
            ir_cache: IrCache::new(),
            eval_cache: EvalCache::new(),
            builtins: StdBuiltins::new(),
        }
    }

    /// Gets the paths to files that the runtime relies on.
    #[must_use]
    pub fn get_watch_paths(&self) -> IndexSet<PathBuf> {
        self.source_cache.get_paths()
    }

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

    /// Evaluates a model and returns the result.
    ///
    /// # Errors
    ///
    /// Returns a [`EvalErrorReference`](output::reference::EvalErrorReference) if the model could not be evaluated.
    #[expect(
        clippy::missing_panics_doc,
        reason = "the panic only happens if an internal invariant is violated"
    )]
    pub fn eval_model(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<output::reference::ModelReference<'_>, output::reference::EvalErrorReference<'_>>
    {
        // make sure the IR is loaded for the model and its dependencies
        // TODO: once caching works, evaluating the model should load the IR as it goes
        let _ir_results = self.load_ir(&path);

        // evaluate the model and its dependencies
        let eval_result = eval::eval_model(&path, self);

        for (model_path, result) in eval_result {
            let source = self.source_cache.get(&model_path).unwrap_or("");

            match result {
                Ok(model) => {
                    self.eval_cache.insert_ok(model_path, model);
                }
                Err(eval_errors) if eval_errors.had_resolution_errors => {
                    let resolution_errors = self
                        .ir_cache
                        .get_error(&model_path)
                        .expect("should have resolution errors")
                        .clone();

                    self.eval_cache.insert_err(
                        model_path,
                        output::error::EvalError::Resolution(resolution_errors),
                    );
                }
                Err(eval_errors) => {
                    let parameter_errors = eval_errors
                        .parameters
                        .into_iter()
                        .map(|(name, errs)| {
                            (
                                name,
                                errs.into_iter()
                                    .map(|e| {
                                        OneilError::from_error_with_source(
                                            &e,
                                            model_path.clone(),
                                            source,
                                        )
                                    })
                                    .collect(),
                            )
                        })
                        .collect();

                    let test_errors = eval_errors
                        .tests
                        .into_iter()
                        .map(|e| OneilError::from_error_with_source(&e, model_path.clone(), source))
                        .collect();

                    self.eval_cache.insert_err(
                        model_path,
                        output::error::EvalError::EvalErrors {
                            partial_result: Box::new(eval_errors.partial_result),
                            parameter_errors,
                            test_errors,
                        },
                    );
                }
            }
        }

        let model = self
            .eval_cache
            .get_entry(path.as_ref())
            .expect("eval_model populates cache for requested path and dependencies");

        match model {
            Ok(model) => {
                let model_ref = output::reference::ModelReference::new(model, &self.eval_cache);
                Ok(model_ref)
            }
            Err(err) => {
                let err_ref = output::reference::EvalErrorReference::new(err, &self.eval_cache);
                Err(err_ref)
            }
        }
    }

    /// Loads the IR for a model and all of its dependencies.
    ///
    /// # Errors
    ///
    /// Returns a
    /// [`ResolutionErrorReference`](output::reference::ResolutionErrorReference) if that
    /// model had parse or resolution errors.
    #[expect(
        clippy::missing_panics_doc,
        reason = "the panic only happens if an internal invariant is violated"
    )]
    pub fn load_ir(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<
        output::reference::ModelIrReference<'_>,
        output::reference::ResolutionErrorReference<'_>,
    > {
        let results = model_resolver::load_model(&path, self);

        for (model_path, result) in results {
            let model_path = model_path.as_ref().to_path_buf();

            let (model, model_errors, ast_loaded) = result.into_parts();

            // If the AST failed to load during resolution, we insert
            // the parse error that caused it to fail
            if !ast_loaded {
                let parse_err = self
                    .ast_cache
                    .get_errors(&model_path)
                    .expect("should have ast error");

                self.ir_cache.insert_err(
                    model_path.clone(),
                    output::error::ResolutionError::Parse(parse_err.clone()),
                );
                continue;
            }

            let source = self
                .source_cache
                .get(&model_path)
                .expect("it has already been loaded previously");

            let (
                circular_dependency_errors,
                python_import_errors,
                model_import_errors,
                parameter_errors,
                test_errors,
            ) = model_errors.into_parts();

            let circular_dependency_oneil: Vec<OneilError> = circular_dependency_errors
                .into_iter()
                .map(|e| OneilError::from_error_with_source(&e, model_path.clone(), source))
                .collect();

            let python_import_oneil: Vec<OneilError> = python_import_errors
                .into_values()
                .map(|e| OneilError::from_error_with_source(&e, model_path.clone(), source))
                .collect();

            let model_import_oneil: Vec<OneilError> = model_import_errors
                .into_values()
                .map(|(_, e)| OneilError::from_error_with_source(&e, model_path.clone(), source))
                .collect();

            let parameter_errors_oneil: IndexMap<String, Vec<OneilError>> = parameter_errors
                .into_iter()
                .map(|(name, errs)| {
                    (
                        name.as_str().to_string(),
                        errs.into_iter()
                            .map(|e| {
                                OneilError::from_error_with_source(&e, model_path.clone(), source)
                            })
                            .collect(),
                    )
                })
                .collect();

            let test_errors_oneil: Vec<OneilError> = test_errors
                .into_iter()
                .flat_map(|(_test_index, errors)| {
                    errors
                        .into_iter()
                        .map(|e| OneilError::from_error_with_source(&e, model_path.clone(), source))
                })
                .collect();

            let has_errors = !circular_dependency_oneil.is_empty()
                || !python_import_oneil.is_empty()
                || !model_import_oneil.is_empty()
                || !parameter_errors_oneil.is_empty()
                || !test_errors_oneil.is_empty();

            if has_errors {
                let python_map = IndexMap::from_iter([(model_path.clone(), python_import_oneil)]);
                let model_import_map =
                    IndexMap::from_iter([(model_path.clone(), model_import_oneil)]);

                self.ir_cache.insert_err(
                    model_path.clone(),
                    output::error::ResolutionError::ResolutionErrors {
                        partial_ir: Box::new(model),
                        circular_dependency_errors: circular_dependency_oneil,
                        python_import_errors: python_map,
                        model_import_errors: model_import_map,
                        parameter_errors: parameter_errors_oneil,
                        test_errors: test_errors_oneil,
                    },
                );
            } else {
                self.ir_cache.insert_ok(model_path.clone(), model);
            }
        }

        let entry = self
            .ir_cache
            .get_entry(path.as_ref())
            .expect("entry was inserted in this function for the requested path");

        match entry.as_ref() {
            Ok(model) => Ok(output::reference::ModelIrReference::new(
                model,
                &self.ir_cache,
            )),

            Err(resolution_error) => Err(output::reference::ResolutionErrorReference::new(
                resolution_error,
                &self.ir_cache,
            )),
        }
    }

    /// Loads AST for a model.
    ///
    /// # Errors
    ///
    /// Returns a [`ParseError`](output::error::ParseError) if the AST could not be loaded.
    #[expect(
        clippy::missing_panics_doc,
        reason = "the panic only happens if an internal invariant is violated"
    )]
    pub fn load_ast(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<&ast::Model, output::error::ParseError> {
        let path = path.as_ref();
        let source = self
            .load_source(path)
            .map_err(output::error::ParseError::File)?;

        // parse the model and return an error if it fails
        match parser::parse_model(source, None) {
            Ok(ast) => {
                self.ast_cache
                    .insert_ok(path.to_path_buf(), ast.take_value());
                let ast = self.ast_cache.get(path).expect("it was just inserted");

                Ok(ast)
            }
            Err(e) => {
                // need to reload the source for lifetime reasons
                // TODO: maybe another call to `load_source` once caching works would make more sense?
                let source = self
                    .source_cache
                    .get(path)
                    .expect("it has already been loaded previously");
                let errors = e
                    .errors
                    .into_iter()
                    .map(|err| OneilError::from_error_with_source(&err, path.to_path_buf(), source))
                    .collect::<Vec<OneilError>>();

                let partial_ast = *e.partial_result;
                let partial_ast_for_error = partial_ast.clone();
                self.ast_cache
                    .insert_err(path.to_path_buf(), partial_ast, errors.clone());

                Err(output::error::ParseError::ParseErrors {
                    errors,
                    partial_ast: Box::new(partial_ast_for_error),
                })
            }
        }
    }

    /// Loads source code from a file.
    ///
    /// # Errors
    ///
    /// Returns a [`FileError`](output::error::FileError) if the file could not be read.
    #[expect(
        clippy::missing_panics_doc,
        reason = "the panic only happens if an internal invariant is violated"
    )]
    pub fn load_source(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<&str, output::error::FileError> {
        let path = path.as_ref();

        // Read the source code from the file
        match std::fs::read_to_string(path) {
            Ok(source) => {
                // If it's found, insert it into the loaded models map
                // and return it
                self.source_cache.insert_ok(path.to_path_buf(), source);

                let source = self.source_cache.get(path).expect("it was just inserted");

                Ok(source)
            }
            Err(e) => {
                let error = InternalIoError::new(path, e);
                let error = OneilError::from_error(&error, path.to_path_buf());

                self.source_cache
                    .insert_err(path.to_path_buf(), error.clone());

                Err(output::error::FileError {
                    error: Box::new(error),
                })
            }
        }
    }

    /// Gets the dependency graph for all models in the evaluation cache.
    ///
    /// The graph is built from the cached evaluation results. The cache must
    /// have been populated by a prior call to [`eval_model`](Self::eval_model).
    #[must_use]
    pub fn get_dependency_graph(&self) -> output::DependencyGraph {
        let mut dependency_graph = output::DependencyGraph::new();

        for (model_path, model) in self.eval_cache.models_iter() {
            for parameter in model.parameters.values() {
                let dependencies = &parameter.dependencies;

                for dependency in &dependencies.builtin_dependencies {
                    dependency_graph.add_depends_on_builtin(
                        model_path.clone(),
                        parameter.ident.clone(),
                        output::dependency::BuiltinDependency {
                            ident: dependency.ident.clone(),
                        },
                    );
                }

                for dependency in &dependencies.parameter_dependencies {
                    dependency_graph.add_depends_on_parameter(
                        model_path.clone(),
                        parameter.ident.clone(),
                        output::dependency::ParameterDependency {
                            parameter_name: dependency.parameter_name.clone(),
                        },
                    );
                }

                for dependency in &dependencies.external_dependencies {
                    dependency_graph.add_depends_on_external(
                        model_path.clone(),
                        parameter.ident.clone(),
                        output::dependency::ExternalDependency {
                            model_path: dependency.model_path.clone(),
                            reference_name: dependency.reference_name.clone(),
                            parameter_name: dependency.parameter_name.clone(),
                        },
                    );
                }
            }
        }

        dependency_graph
    }

    /// Gets the dependency tree for a specific parameter.
    ///
    /// The tree shows all parameters, builtin values, and external dependencies
    /// that the specified parameter depends on, recursively.
    #[must_use]
    pub fn get_dependency_tree(
        &mut self,
        model_path: &Path,
        parameter_name: &str,
    ) -> (
        Option<output::Tree<output::dependency::DependencyTreeValue>>,
        IndexMap<PathBuf, output::error::TreeError>,
    ) {
        let location = TreeValueLocation {
            model_path: model_path.to_path_buf(),
            reference_name: None,
            parameter_name: parameter_name.to_string(),
        };

        self.get_parameter_tree(
            &location,
            |runtime, location| {
                runtime.get_dependency_tree_value(
                    &location.model_path,
                    location.reference_name.as_deref(),
                    &location.parameter_name,
                )
            },
            |runtime, dependency_graph, location| {
                Self::get_dependency_tree_children(
                    runtime,
                    dependency_graph,
                    &location.model_path,
                    location.reference_name.as_deref(),
                    &location.parameter_name,
                )
            },
        )
    }

    /// Gets the reference tree for a specific parameter.
    ///
    /// The tree shows all parameters that depend on the specified parameter, recursively.
    /// This is the inverse of the dependency tree.
    #[must_use]
    pub fn get_reference_tree(
        &mut self,
        model_path: &Path,
        parameter_name: &str,
    ) -> (
        Option<output::Tree<output::dependency::ReferenceTreeValue>>,
        IndexMap<PathBuf, output::error::TreeError>,
    ) {
        let location = TreeValueLocation {
            model_path: model_path.to_path_buf(),
            reference_name: None,
            parameter_name: parameter_name.to_string(),
        };

        self.get_parameter_tree(
            &location,
            |runtime, location| {
                runtime.get_reference_tree_value(&location.model_path, &location.parameter_name)
            },
            |_runtime, dependency_graph, location| {
                Self::get_reference_tree_children(
                    dependency_graph,
                    &location.model_path,
                    &location.parameter_name,
                )
            },
        )
    }

    /// Unified implementation for dependency and reference trees.
    ///
    /// Recursively builds a tree of parameter values, using `get_value` to resolve
    /// each node and `get_children` to determine the values for the children.
    fn get_parameter_tree<V, GetVal, GetChildren>(
        &mut self,
        location: &TreeValueLocation,
        get_value: GetVal,
        get_children: GetChildren,
    ) -> (
        Option<output::Tree<V>>,
        IndexMap<PathBuf, output::error::TreeError>,
    )
    where
        GetVal: Fn(&Self, &TreeValueLocation) -> Option<GetValueResult<V>>,
        GetChildren:
            Fn(&Self, &output::DependencyGraph, &TreeValueLocation) -> GetChildrenResult<V>,
    {
        let _ = self.eval_model(&location.model_path);
        let dependency_graph = self.get_dependency_graph();

        return recurse(self, location, &dependency_graph, &get_value, &get_children);

        #[expect(
            clippy::items_after_statements,
            reason = "this is an internal recursive function, we keep it here for clarity"
        )]
        fn recurse<V, GetVal, GetChildren>(
            runtime: &Runtime,
            location: &TreeValueLocation,
            dependency_graph: &output::DependencyGraph,
            get_value: &GetVal,
            get_children: &GetChildren,
        ) -> (
            Option<output::Tree<V>>,
            IndexMap<PathBuf, output::error::TreeError>,
        )
        where
            GetVal: Fn(&Runtime, &TreeValueLocation) -> Option<GetValueResult<V>>,
            GetChildren:
                Fn(&Runtime, &output::DependencyGraph, &TreeValueLocation) -> GetChildrenResult<V>,
        {
            // get the value for the current location
            let Some(value) = get_value(runtime, location) else {
                // if it doesn't exist, return no tree and no errors
                return (None, IndexMap::new());
            };

            // get the children for the current location
            let GetChildrenResult {
                builtin_children,
                parameter_children,
            } = get_children(runtime, dependency_graph, location);

            // recurse on the parameter children
            let (parameter_children, errors): (Vec<_>, Vec<_>) = parameter_children
                .into_iter()
                .map(|location| {
                    recurse(
                        runtime,
                        &location,
                        dependency_graph,
                        get_value,
                        get_children,
                    )
                })
                .unzip();

            let parameter_children = parameter_children.into_iter().flatten();
            let mut errors = errors.into_iter().fold(
                IndexMap::<PathBuf, output::error::TreeError>::new(),
                |mut acc, error_map| {
                    for (path, tree_error) in error_map {
                        acc.entry(path).or_default().insert_all(tree_error);
                    }
                    acc
                },
            );

            let children = builtin_children
                .into_iter()
                .chain(parameter_children)
                .collect();

            match value {
                GetValueResult::ValidTree(value) => {
                    (Some(output::Tree::new(value, children)), errors)
                }

                GetValueResult::ParseError(parse_error) => {
                    let model_error = output::error::TreeError::Parse(parse_error);
                    errors.insert(location.model_path.clone(), model_error);
                    (None, errors)
                }

                GetValueResult::ParameterErrors(parameter_errors) => {
                    errors
                        .entry(location.model_path.clone())
                        .or_default()
                        .insert_parameter_errors(location.parameter_name.clone(), parameter_errors);
                    (None, errors)
                }
            }
        }
    }

    fn get_dependency_tree_children(
        &self,
        dependency_graph: &output::DependencyGraph,
        model_path: &Path,
        reference_name: Option<&str>,
        parameter_name: &str,
    ) -> GetChildrenResult<output::dependency::DependencyTreeValue> {
        let deps = dependency_graph
            .depends_on(model_path, parameter_name)
            .cloned()
            .unwrap_or_default();

        let builtin_children = deps
            .builtin_dependencies
            .iter()
            .map(|dep| {
                let parameter_value = self
                    .lookup_builtin_variable(&ir::Identifier::new(dep.ident.clone()))
                    .cloned()
                    .expect("the builtin value should be defined");

                let tree_value = output::dependency::DependencyTreeValue {
                    reference_name: None,
                    parameter_name: dep.ident.clone(),
                    parameter_value,
                    display_info: None,
                };

                output::Tree::new(tree_value, Vec::new())
            })
            .collect();

        let parameter_args = deps
            .parameter_dependencies
            .iter()
            .map(|dep| TreeValueLocation {
                model_path: model_path.to_path_buf(),
                reference_name: reference_name.map(String::from),
                parameter_name: dep.parameter_name.clone(),
            });

        let external_args = deps
            .external_dependencies
            .iter()
            .map(|dep| TreeValueLocation {
                model_path: dep.model_path.clone(),
                reference_name: Some(dep.reference_name.clone()),
                parameter_name: dep.parameter_name.clone(),
            });

        let parameter_children = parameter_args.chain(external_args).collect();

        GetChildrenResult {
            builtin_children,
            parameter_children,
        }
    }

    fn get_reference_tree_children(
        dependency_graph: &output::DependencyGraph,
        model_path: &Path,
        parameter_name: &str,
    ) -> GetChildrenResult<output::dependency::ReferenceTreeValue> {
        let deps = dependency_graph
            .references(model_path, parameter_name)
            .cloned()
            .unwrap_or_default();

        let parameter_args = deps
            .parameter_references
            .iter()
            .map(|dep| TreeValueLocation {
                model_path: model_path.to_path_buf(),
                reference_name: None,
                parameter_name: dep.parameter_name.clone(),
            });

        let external_args = deps
            .external_references
            .iter()
            .map(|dep| TreeValueLocation {
                model_path: dep.model_path.clone(),
                reference_name: None,
                parameter_name: dep.parameter_name.clone(),
            });

        let recurse_args = parameter_args.chain(external_args).collect();

        GetChildrenResult {
            // no builtins reference other parameters
            builtin_children: Vec::new(),
            parameter_children: recurse_args,
        }
    }

    fn get_dependency_tree_value(
        &self,
        model_path: &Path,
        reference_name: Option<&str>,
        parameter_name: &str,
    ) -> Option<GetValueResult<output::dependency::DependencyTreeValue>> {
        let eval_result = self.eval_cache.get_entry(model_path)?;

        // get the parameter from the model if it exists,
        // or return parameter errors if they exist
        let parameter = match eval_result {
            Ok(model) => model.parameters.get(parameter_name)?,
            Err(output::error::EvalError::Resolution(resolution_error)) => match resolution_error {
                output::error::ResolutionError::ResolutionErrors {
                    parameter_errors, ..
                } => {
                    return parameter_errors
                        .get(parameter_name)
                        .map(|errors| GetValueResult::ParameterErrors(errors.clone()));
                }
                output::error::ResolutionError::Parse(parse_error) => {
                    return Some(GetValueResult::ParseError(parse_error.clone()));
                }
            },
            // it may be possible to recover the parameter from the partial result
            Err(output::error::EvalError::EvalErrors {
                partial_result,
                parameter_errors,
                ..
            }) => {
                if let Some(errors) = parameter_errors.get(parameter_name) {
                    return Some(GetValueResult::ParameterErrors(errors.clone()));
                }

                partial_result.parameters.get(parameter_name)?
            }
        };

        let reference_name = reference_name.map(str::to_string);
        let parameter_name = parameter_name.to_string();

        let parameter_value = parameter.value.clone();

        let span = parameter.expr_span;
        let display_info = Some((model_path.to_path_buf(), span));

        let tree_value = output::dependency::DependencyTreeValue {
            reference_name,
            parameter_name,
            parameter_value,
            display_info,
        };

        Some(GetValueResult::ValidTree(tree_value))
    }

    fn get_reference_tree_value(
        &self,
        model_path: &Path,
        parameter_name: &str,
    ) -> Option<GetValueResult<output::dependency::ReferenceTreeValue>> {
        let eval_result = self.eval_cache.get_entry(model_path)?;

        // get the parameter from the model if it exists,
        // or return parameter errors if they exist
        let parameter = match eval_result {
            Ok(model) => model.parameters.get(parameter_name)?,
            Err(output::error::EvalError::Resolution(resolution_error)) => match resolution_error {
                output::error::ResolutionError::ResolutionErrors {
                    parameter_errors, ..
                } => {
                    return parameter_errors
                        .get(parameter_name)
                        .map(|errors| GetValueResult::ParameterErrors(errors.clone()));
                }
                output::error::ResolutionError::Parse(parse_error) => {
                    return Some(GetValueResult::ParseError(parse_error.clone()));
                }
            },
            // it may be possible to recover the parameter from the partial result
            Err(output::error::EvalError::EvalErrors {
                partial_result,
                parameter_errors,
                ..
            }) => {
                if let Some(errors) = parameter_errors.get(parameter_name) {
                    return Some(GetValueResult::ParameterErrors(errors.clone()));
                }

                partial_result.parameters.get(parameter_name)?
            }
        };

        let model_path = model_path.to_path_buf();

        let parameter_name = parameter_name.to_string();

        let parameter_value = parameter.value.clone();

        let span = parameter.expr_span;
        let display_info = (model_path.clone(), span);

        let tree_value = output::dependency::ReferenceTreeValue {
            model_path,
            parameter_name,
            parameter_value,
            display_info,
        };

        Some(GetValueResult::ValidTree(tree_value))
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl model_resolver::ExternalResolutionContext for Runtime {
    fn has_builtin_value(&self, identifier: &ir::Identifier) -> bool {
        self.builtins.has_builtin_value(identifier.as_str())
    }

    fn has_builtin_function(&self, identifier: &ir::Identifier) -> bool {
        self.builtins.has_builtin_function(identifier.as_str())
    }

    fn load_ast(
        &mut self,
        path: &ir::ModelPath,
    ) -> Result<&ast::Model, model_resolver::AstLoadingFailedError> {
        self.load_ast(path)
            .map_err(|_e| model_resolver::AstLoadingFailedError)
    }

    fn load_python_import(
        &mut self,
        python_path: &ir::PythonPath,
    ) -> Result<(), model_resolver::PythonImportLoadingFailedError> {
        // TODO: implement this
        Ok(())
    }
}

impl eval::ExternalEvaluationContext for Runtime {
    fn lookup_ir(&self, path: impl AsRef<Path>) -> Option<Result<&ir::Model, IrLoadError>> {
        let result = self.ir_cache.get_entry(path.as_ref())?;
        match result {
            Ok(ir) => Some(Ok(ir)),
            Err(_error) => Some(Err(IrLoadError)),
        }
    }

    fn lookup_builtin_variable(&self, identifier: &ir::Identifier) -> Option<&Value> {
        self.builtins.get_value(identifier.as_str())
    }

    fn evaluate_builtin_function(
        &self,
        identifier: &ir::Identifier,
        identifier_span: Span,
        args: Vec<(Value, Span)>,
    ) -> Option<Result<Value, Vec<EvalError>>> {
        let function = self.builtins.get_function(identifier.as_str())?;
        Some(function(identifier_span, args))
    }

    fn lookup_unit(&self, name: &str) -> Option<&Unit> {
        self.builtins.get_unit(name)
    }

    fn available_prefixes(&self) -> impl Iterator<Item = (&str, f64)> {
        self.builtins.builtin_prefixes()
    }
}

/// Error type for file loading failures.
struct InternalIoError<'a> {
    path: &'a Path,
    error: IoError,
}

impl<'a> InternalIoError<'a> {
    /// Creates a new file error from a path and I/O error.
    pub const fn new(path: &'a Path, error: IoError) -> Self {
        Self { path, error }
    }
}

impl AsOneilError for InternalIoError<'_> {
    fn message(&self) -> String {
        format!("couldn't read `{}` - {}", self.path.display(), self.error)
    }
}

struct TreeValueLocation {
    pub model_path: PathBuf,
    pub reference_name: Option<String>,
    pub parameter_name: String,
}

enum GetValueResult<T> {
    ValidTree(T),
    ParseError(output::error::ParseError),
    ParameterErrors(Vec<OneilError>),
}

struct GetChildrenResult<T> {
    builtin_children: Vec<output::Tree<T>>,
    parameter_children: Vec<TreeValueLocation>,
}
