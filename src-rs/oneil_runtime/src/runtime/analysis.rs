//! Dependency and reference analysis for the runtime.

use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use oneil_shared::partial::MaybePartialResult;

use super::Runtime;
use crate::output;

#[derive(Debug)]
struct TreeValueLocation {
    pub model_path: PathBuf,
    pub reference_name: Option<String>,
    pub parameter_name: String,
}

#[derive(Debug)]
struct GetChildrenResult<T> {
    builtin_children: Vec<output::Tree<T>>,
    parameter_children: Vec<TreeValueLocation>,
}

impl Runtime {
    /// Gets the dependency graph for all models in the evaluation cache.
    ///
    /// The graph is built from the cached evaluation results. The cache must
    /// have been populated by a prior call to [`load_ir`](Self::load_ir). This
    /// can be done indirectly by calling [`eval_model`](Self::eval_model).
    #[must_use]
    fn get_dependency_graph(&self) -> output::DependencyGraph {
        todo!();
        let mut dependency_graph = output::DependencyGraph::new();

        let model_ir = self
            .ir_cache
            .iter()
            .filter_map(|(path, result)| {
                let ir = result.value();
                ir.map(|ir| (path, ir))
            })
            .collect::<IndexMap<_, _>>();

        for (model_path, model) in model_ir {
            for (parameter_name, parameter) in model.get_parameters() {
                let dependencies = parameter.dependencies();

                for builtin_dep in dependencies.builtin().keys() {
                    dependency_graph.add_depends_on_builtin(
                        model_path.clone(),
                        parameter_name.as_str().to_string(),
                        output::dependency::BuiltinDependency {
                            ident: builtin_dep.as_str().to_string(),
                        },
                    );
                }

                for parameter_dep in dependencies.parameter().keys() {
                    dependency_graph.add_depends_on_parameter(
                        model_path.clone(),
                        parameter_name.as_str().to_string(),
                        output::dependency::ParameterDependency {
                            parameter_name: parameter_dep.as_str().to_string(),
                        },
                    );
                }

                for ((reference_dep_name, parameter_dep_name), (external_model_path, _)) in
                    dependencies.external()
                {
                    dependency_graph.add_depends_on_external(
                        external_model_path.as_ref().to_path_buf(),
                        parameter_name.as_str().to_string(),
                        output::dependency::ExternalDependency {
                            model_path: external_model_path.as_ref().to_path_buf(),
                            reference_name: reference_dep_name.as_str().to_string(),
                            parameter_name: parameter_dep_name.as_str().to_string(),
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
    ) -> Option<
        MaybePartialResult<
            output::Tree<output::dependency::DependencyTreeValue>,
            output::error::TreeError,
        >,
    > {
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
    ) -> Option<
        MaybePartialResult<
            output::Tree<output::dependency::ReferenceTreeValue>,
            output::error::TreeError,
        >,
    > {
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
    fn get_parameter_tree<V: std::fmt::Debug, GetVal, GetChildren>(
        &mut self,
        location: &TreeValueLocation,
        get_value: GetVal,
        get_children: GetChildren,
    ) -> Option<MaybePartialResult<output::Tree<V>, output::error::TreeError>>
    where
        GetVal: Fn(&Self, &TreeValueLocation) -> Option<V>,
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
        fn recurse<V: std::fmt::Debug, GetVal, GetChildren>(
            runtime: &Runtime,
            location: &TreeValueLocation,
            dependency_graph: &output::DependencyGraph,
            get_value: &GetVal,
            get_children: &GetChildren,
        ) -> Option<output::Tree<Result<V, output::error::TreeError>>>
        where
            GetVal: Fn(&Runtime, &TreeValueLocation) -> Option<V>,
            GetChildren:
                Fn(&Runtime, &output::DependencyGraph, &TreeValueLocation) -> GetChildrenResult<V>,
        {
            // get the value for the current location
            let Some(value) = get_value(runtime, location) else {
                // if it doesn't exist, return no tree and no errors
                return None;
            };

            // get the children for the current location
            let GetChildrenResult {
                builtin_children,
                parameter_children,
            } = get_children(runtime, dependency_graph, location);

            // recurse on the parameter children
            let parameter_children: Vec<_> = parameter_children
                .into_iter()
                .filter_map(|location| {
                    recurse(
                        runtime,
                        &location,
                        dependency_graph,
                        get_value,
                        get_children,
                    )
                })
                .collect();

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
                    // add the parameter errors to the errors map
                    errors
                        .entry(location.model_path.clone())
                        .or_default()
                        .insert_parameter_errors(location.parameter_name.clone(), parameter_errors);

                    // get the errors for the dependent parameters
                    let dependent_parameter_errors = runtime.get_dependent_parameter_errors(
                        &location.model_path,
                        &location.parameter_name,
                        dependency_graph,
                    );

                    // add the dependent parameter errors to the errors map
                    for (model_path, model_errors) in dependent_parameter_errors {
                        errors
                            .entry(model_path)
                            .or_default()
                            .insert_all(model_errors);
                    }

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
            .dependents(model_path, parameter_name)
            .cloned()
            .unwrap_or_default();

        let builtin_children = deps
            .builtin_dependencies
            .iter()
            .map(|dep| {
                let parameter_value = self
                    .lookup_builtin_variable(&oneil_ir::Identifier::new(dep.ident.clone()))
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
    ) -> Option<output::dependency::DependencyTreeValue> {
        let model_result = self.eval_cache.get_entry(model_path)?;
        let model = model_result.value()?;
        let parameter = model.parameters.get(parameter_name)?;

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

        Some(tree_value)
    }

    fn get_reference_tree_value(
        &self,
        model_path: &Path,
        parameter_name: &str,
    ) -> Option<output::dependency::ReferenceTreeValue> {
        let model_result = self.eval_cache.get_entry(model_path)?;
        let model = model_result.value()?;
        let parameter = model.parameters.get(parameter_name)?;

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

        Some(tree_value)
    }
}
