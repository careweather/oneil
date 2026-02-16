//! Dependency and reference analysis for the runtime.

use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use oneil_ir as ir;

use crate::{
    context::{ExternalTreeContext, TreeContext},
    output::{
        self,
        error::{GetValueError, TreeErrors},
    },
};

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

/// Gets the dependency tree for a specific parameter.
///
/// The tree shows all parameters, builtin values, and external dependencies
/// that the specified parameter depends on, recursively.
#[must_use]
pub fn get_dependency_tree(
    model_ir: &IndexMap<PathBuf, ir::Model>,
    model_path: &Path,
    parameter_name: &str,
) -> (
    Option<output::Tree<output::DependencyTreeValue>>,
    TreeErrors,
) {
    let location = TreeValueLocation {
        model_path: model_path.to_path_buf(),
        reference_name: None,
        parameter_name: parameter_name.to_string(),
    };

    get_parameter_tree(
        &location,
        |r, location| {
            get_dependency_tree_value(
                r,
                &location.model_path,
                location.reference_name.as_deref(),
                &location.parameter_name,
            )
        },
        |r, dependency_graph, location| {
            get_dependency_tree_children(
                r,
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
    model_path: &Path,
    parameter_name: &str,
) -> (Option<output::Tree<output::ReferenceTreeValue>>, TreeErrors) {
    let location = TreeValueLocation {
        model_path: model_path.to_path_buf(),
        reference_name: None,
        parameter_name: parameter_name.to_string(),
    };

    get_parameter_tree(
        model_ir,
        &location,
        |r, location| get_reference_tree_value(r, &location.model_path, &location.parameter_name),
        |_runtime, dependency_graph, location| {
            get_reference_tree_children(
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
    location: &TreeValueLocation,
    tree_context: &TreeContext,
    get_value: GetVal,
    get_children: GetChildren,
) -> (Option<output::Tree<V>>, TreeErrors)
where
    GetVal: Fn(&TreeValueLocation, &TreeContext) -> Option<V>,
    GetChildren: Fn(
        &crate::dep_graph::DependencyGraph,
        &TreeValueLocation,
        &TreeContext,
    ) -> GetChildrenResult<V>,
{
    let dependency_graph = get_dependency_graph(tree_context);

    return recurse(
        runtime,
        location,
        &dependency_graph,
        &get_value,
        &get_children,
    );

    #[expect(
        clippy::items_after_statements,
        reason = "this is an internal recursive function, we keep it here for clarity"
    )]
    fn recurse<V: std::fmt::Debug, GetVal, GetChildren>(
        location: &TreeValueLocation,
        tree_context: &TreeContext,
        get_value: &GetVal,
        get_children: &GetChildren,
    ) -> (Option<output::Tree<V>>, TreeErrors)
    where
        GetVal: Fn(&TreeValueLocation, &TreeContext) -> Option<Result<V, GetValueError>>,
        GetChildren: Fn(&TreeValueLocation, &TreeContext) -> GetChildrenResult<V>,
    {
        // get the value for the current location
        let Some(value) = get_value(location, tree_context) else {
            // if it doesn't exist, return no tree and no errors
            return (None, TreeErrors::empty());
        };

        let value = match value {
            Ok(value) => value,
            Err(GetValueError::Model) => {
                let mut tree_errors = TreeErrors::empty();
                tree_errors.insert_model_error(location.model_path.clone());

                return (None, tree_errors);
            }
            Err(GetValueError::Parameter) => {
                let mut tree_errors = TreeErrors::empty();
                tree_errors.insert_parameter_error(
                    location.model_path.clone(),
                    location.parameter_name.clone(),
                );

                return (None, tree_errors);
            }
        };

        // get the children for the current location
        let GetChildrenResult {
            builtin_children,
            parameter_children,
        } = get_children(location, tree_context);

        // recurse on the parameter children
        let (parameter_children, tree_errors) = parameter_children
            .into_iter()
            .map(|location| recurse(&location, tree_context, get_value, get_children))
            .fold(
                (Vec::new(), TreeErrors::empty()),
                |(mut children, mut errors), (child, child_errors)| {
                    children.extend(child);
                    errors.extend(child_errors);
                    (children, errors)
                },
            );

        let children = builtin_children
            .into_iter()
            .chain(parameter_children)
            .collect();

        (Some(output::Tree::new(value, children)), tree_errors)
    }
}

fn get_dependency_tree_children(
    model_path: &Path,
    reference_name: Option<&str>,
    parameter_name: &str,
    tree_context: &TreeContext,
) -> GetChildrenResult<output::DependencyTreeValue> {
    let deps = dependency_graph
        .dependents(model_path, parameter_name)
        .cloned()
        .unwrap_or_default();

    let builtin_children = deps
        .builtin_dependencies
        .iter()
        .map(|dep| {
            let parameter_value = runtime
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
    dependency_graph: &crate::dep_graph::DependencyGraph,
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
    model_path: &Path,
    reference_name: Option<&str>,
    parameter_name: &str,
) -> Option<output::dependency::DependencyTreeValue> {
    let model_result = runtime.eval_cache.get_entry(model_path)?;
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
    model_path: &Path,
    parameter_name: &str,
) -> Option<output::dependency::ReferenceTreeValue> {
    let model_result = runtime.eval_cache.get_entry(model_path)?;
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

/// Gets the dependency graph for all models in the evaluation cache.
///
/// The graph is built from the cached evaluation results. The cache must
/// have been populated by a prior call to [`Runtime::load_ir`]. This
/// can be done indirectly by calling [`Runtime::eval_model`].
#[must_use]
fn get_dependency_graph<E: ExternalTreeContext>(
    tree_context: &TreeContext<'_, E>,
) -> crate::dep_graph::DependencyGraph {
    let mut dependency_graph = crate::dep_graph::DependencyGraph::new();

    for (model_path, model) in tree_context.get_all_model_ir() {
        for (parameter_name, parameter) in model.get_parameters() {
            let dependencies = parameter.dependencies();

            for builtin_dep in dependencies.builtin().keys() {
                dependency_graph.add_depends_on_builtin(
                    model_path.clone(),
                    parameter_name.as_str().to_string(),
                    oneil_output::BuiltinDependency {
                        ident: builtin_dep.as_str().to_string(),
                    },
                );
            }

            for parameter_dep in dependencies.parameter().keys() {
                dependency_graph.add_depends_on_parameter(
                    model_path.clone(),
                    parameter_name.as_str().to_string(),
                    oneil_output::ParameterDependency {
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
                    oneil_output::ExternalDependency {
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
