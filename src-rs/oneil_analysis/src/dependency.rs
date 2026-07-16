//! Dependency and reference analysis for the runtime.

use oneil_output::DependencySet;
use oneil_shared::{
    EvalInstanceKey,
    paths::ModelPath,
    symbols::{ParameterName, ReferenceName, TestIndex},
};

use crate::{
    context::{ExternalAnalysisContext, TreeContext},
    output::{
        self,
        error::{GetTestValueError, GetValueError, TreeErrors},
    },
};

#[derive(Debug)]
struct TreeValueLocation {
    pub instance_key: EvalInstanceKey,
    pub reference_name: Option<ReferenceName>,
    pub parameter_name: ParameterName,
}

#[derive(Debug)]
struct GetChildrenResult<T> {
    builtin_children: Vec<output::Tree<T>>,
    parameter_children: Vec<TreeValueLocation>,
    test_children: Vec<output::Tree<T>>,
    test_errors: TreeErrors,
}

/// Gets the dependency tree for a specific parameter.
///
/// The tree shows all parameters, builtin values, and external dependencies
/// that the specified parameter depends on, recursively.
#[must_use]
pub fn get_dependency_tree<E: ExternalAnalysisContext>(
    model_path: &ModelPath,
    parameter_name: &ParameterName,
    external_context: &mut E,
) -> (
    Option<output::Tree<output::DependencyTreeValue>>,
    TreeErrors,
) {
    let location = TreeValueLocation {
        instance_key: EvalInstanceKey::root(model_path.clone()),
        reference_name: None,
        parameter_name: parameter_name.clone(),
    };

    get_parameter_tree(
        &location,
        external_context,
        get_dependency_value,
        |location, tree_context| {
            get_dependency_tree_children(
                &location.instance_key,
                location.reference_name.as_ref(),
                &location.parameter_name,
                tree_context,
            )
        },
    )
}

fn get_dependency_value<E: ExternalAnalysisContext>(
    location: &TreeValueLocation,
    tree_context: &TreeContext<'_, E>,
) -> Option<Result<output::DependencyTreeValue, GetValueError>> {
    let parameter =
        tree_context.lookup_parameter_value(&location.instance_key, &location.parameter_name)?;

    let result = parameter.map(|parameter| {
        let dependency_name = location.reference_name.as_ref().map_or_else(
            || output::DependencyName::Parameter(location.parameter_name.clone()),
            |reference_name| {
                output::DependencyName::External(
                    reference_name.clone(),
                    location.parameter_name.clone(),
                )
            },
        );

        let parameter_value = parameter.value;
        let display_info = Some((
            location.instance_key.model_path.clone(),
            parameter.expr_span,
        ));

        output::DependencyTreeValue {
            dependency_name,
            parameter_value,
            display_info,
        }
    });

    Some(result)
}

fn get_dependency_tree_children(
    instance_key: &EvalInstanceKey,
    reference_name: Option<&ReferenceName>,
    parameter_name: &ParameterName,
    tree_context: &TreeContext<'_, impl ExternalAnalysisContext>,
) -> GetChildrenResult<output::DependencyTreeValue> {
    let DependencySet {
        builtin_dependencies,
        parameter_dependencies,
        external_dependencies,
    } = tree_context.dependents(instance_key, parameter_name);

    let builtin_children = builtin_dependencies
        .into_iter()
        .map(|dep| {
            let parameter_value = tree_context
                .lookup_builtin_variable(&dep.name)
                .cloned()
                .expect("the builtin value should be defined");

            let tree_value = output::DependencyTreeValue {
                dependency_name: output::DependencyName::Builtin(dep.name),
                parameter_value,
                display_info: None,
            };

            output::Tree::new(tree_value, Vec::new())
        })
        .collect();

    let parameter_args = parameter_dependencies
        .into_iter()
        .map(|dep| TreeValueLocation {
            instance_key: instance_key.clone(),
            reference_name: reference_name.cloned(),
            parameter_name: dep.parameter_name,
        });

    let external_args = external_dependencies
        .into_iter()
        .map(|dep| TreeValueLocation {
            instance_key: dep.instance_key.clone(),
            reference_name: Some(dep.reference_name.clone()),
            parameter_name: dep.parameter_name,
        });

    let parameter_children = parameter_args.chain(external_args).collect();

    GetChildrenResult {
        builtin_children,
        parameter_children,
        test_children: Vec::new(),
        test_errors: TreeErrors::empty(),
    }
}

/// Gets the reference tree for a specific parameter.
///
/// The tree shows all parameters that depend on the specified parameter, recursively.
/// This is the inverse of the dependency tree.
#[must_use]
pub fn get_reference_tree<E: ExternalAnalysisContext>(
    external_context: &mut E,
    model_path: &ModelPath,
    parameter_name: &ParameterName,
) -> (Option<output::Tree<output::ReferenceTreeValue>>, TreeErrors) {
    let location = TreeValueLocation {
        instance_key: EvalInstanceKey::root(model_path.clone()),
        reference_name: None,
        parameter_name: parameter_name.clone(),
    };

    get_parameter_tree(
        &location,
        external_context,
        get_reference_value,
        |location, tree_context| {
            get_reference_tree_children(
                &location.instance_key,
                &location.parameter_name,
                tree_context,
            )
        },
    )
}

fn get_reference_value<E: ExternalAnalysisContext>(
    location: &TreeValueLocation,
    tree_context: &TreeContext<'_, E>,
) -> Option<Result<output::ReferenceTreeValue, GetValueError>> {
    let parameter =
        tree_context.lookup_parameter_value(&location.instance_key, &location.parameter_name)?;

    let result = parameter.map(|parameter| {
        let model_path = location.instance_key.model_path.clone();
        let parameter_name = location.parameter_name.clone();
        let parameter_value = parameter.value;
        let display_info = (model_path.clone(), parameter.expr_span);

        output::ReferenceTreeValue::Parameter {
            model_path,
            parameter_name,
            parameter_value,
            display_info,
        }
    });

    Some(result)
}

fn get_reference_tree_children(
    instance_key: &EvalInstanceKey,
    parameter_name: &ParameterName,
    tree_context: &TreeContext<'_, impl ExternalAnalysisContext>,
) -> GetChildrenResult<output::ReferenceTreeValue> {
    enum GetTestError {
        Model(ModelPath),
        Test(ModelPath, TestIndex),
    }

    let deps = tree_context.references(instance_key, parameter_name);
    let model_path = &instance_key.model_path;

    let parameter_children = deps.parameter.into_iter().map(|dep| TreeValueLocation {
        instance_key: instance_key.clone(),
        reference_name: None,
        parameter_name: dep.parameter_name,
    });

    let external_children = deps
        .external_parameter
        .into_iter()
        .map(|dep| TreeValueLocation {
            instance_key: dep.instance_key,
            reference_name: None,
            parameter_name: dep.parameter_name,
        });

    let all_param_children = parameter_children.chain(external_children).collect();

    let test_children = deps.test.iter().filter_map(|dep| {
        let test = tree_context.lookup_test_value(instance_key, dep.test_index)?;

        let result = test
            .map(|test| {
                let test_passed = test.passed();
                let display_info = (model_path.clone(), test.expr_span);

                output::ReferenceTreeValue::Test {
                    model_path: model_path.clone(),
                    test_index: dep.test_index,
                    test_passed,
                    display_info,
                }
            })
            .map_err(|error| match error {
                GetTestValueError::Model => GetTestError::Model(model_path.clone()),
                GetTestValueError::Test => GetTestError::Test(model_path.clone(), dep.test_index),
            });

        Some(result)
    });

    let test_external_children = deps.external_test.iter().filter_map(|dep| {
        let test = tree_context.lookup_test_value(&dep.instance_key, dep.test_index)?;

        let result = test
            .map(|test| {
                let test_passed = test.passed();
                let model_path = dep.instance_key.model_path.clone();
                let display_info = (model_path.clone(), test.expr_span);

                output::ReferenceTreeValue::Test {
                    model_path,
                    test_index: dep.test_index,
                    test_passed,
                    display_info,
                }
            })
            .map_err(|error| match error {
                GetTestValueError::Model => {
                    GetTestError::Model(dep.instance_key.model_path.clone())
                }
                GetTestValueError::Test => {
                    GetTestError::Test(dep.instance_key.model_path.clone(), dep.test_index)
                }
            });

        Some(result)
    });

    let (all_test_children, test_errors) = test_children.chain(test_external_children).fold(
        (Vec::new(), TreeErrors::empty()),
        |(mut children, mut errors), result| {
            match result {
                Ok(child) => {
                    children.push(output::Tree::new(child, Vec::new()));
                }
                Err(GetTestError::Model(model_path)) => {
                    errors.insert_model_error(model_path);
                }
                Err(GetTestError::Test(model_path, test_index)) => {
                    errors.insert_test_error(model_path, test_index);
                }
            }

            (children, errors)
        },
    );

    GetChildrenResult {
        // no builtins reference other parameters
        builtin_children: Vec::new(),
        parameter_children: all_param_children,
        test_children: all_test_children,
        test_errors,
    }
}

/// Unified implementation for dependency and reference trees.
///
/// Recursively builds a tree of parameter values, using `get_value` to resolve
/// each node and `get_children` to determine the values for the children.
fn get_parameter_tree<V: std::fmt::Debug, E: ExternalAnalysisContext, GetVal, GetChildren>(
    location: &TreeValueLocation,
    external_context: &mut E,
    get_value: GetVal,
    get_children: GetChildren,
) -> (Option<output::Tree<V>>, TreeErrors)
where
    GetVal: Fn(&TreeValueLocation, &TreeContext<'_, E>) -> Option<Result<V, GetValueError>>,
    GetChildren: Fn(&TreeValueLocation, &TreeContext<'_, E>) -> GetChildrenResult<V>,
{
    let dependency_graph = get_dependency_graph(external_context);

    let tree_context = TreeContext::new(external_context, dependency_graph);

    return recurse(location, &tree_context, &get_value, &get_children);

    #[expect(
        clippy::items_after_statements,
        reason = "this is an internal recursive function, we keep it here for clarity"
    )]
    fn recurse<V: std::fmt::Debug, E: ExternalAnalysisContext, GetVal, GetChildren>(
        location: &TreeValueLocation,
        tree_context: &TreeContext<'_, E>,
        get_value: &GetVal,
        get_children: &GetChildren,
    ) -> (Option<output::Tree<V>>, TreeErrors)
    where
        GetVal: Fn(&TreeValueLocation, &TreeContext<'_, E>) -> Option<Result<V, GetValueError>>,
        GetChildren: Fn(&TreeValueLocation, &TreeContext<'_, E>) -> GetChildrenResult<V>,
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
                tree_errors.insert_model_error(location.instance_key.model_path.clone());

                return (None, tree_errors);
            }
            Err(GetValueError::Parameter) => {
                let mut tree_errors = TreeErrors::empty();
                tree_errors.insert_parameter_error(
                    location.instance_key.model_path.clone(),
                    location.parameter_name.clone(),
                );

                return (None, tree_errors);
            }
        };

        // get the children for the current location
        let GetChildrenResult {
            builtin_children,
            parameter_children,
            test_children,
            test_errors,
        } = get_children(location, tree_context);

        // recurse on the parameter children
        let (parameter_children, mut tree_errors) = parameter_children
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
            .chain(test_children)
            .collect();

        tree_errors.extend(test_errors);

        (Some(output::Tree::new(value, children)), tree_errors)
    }
}

/// Gets the dependency graph for all models in the evaluation cache.
///
/// The graph is built from the cached evaluation results. The cache must
/// have been populated by a prior call to [`Runtime::load_ir`]. This
/// can be done indirectly by calling [`Runtime::eval_model`].
#[must_use]
fn get_dependency_graph<E: ExternalAnalysisContext>(
    external_context: &E,
) -> crate::dep_graph::DependencyGraph {
    let mut dependency_graph = crate::dep_graph::DependencyGraph::new();

    let all_model_ir = external_context.get_all_model_ir();

    for (instance_key, model) in &all_model_ir {
        let resolve_external_key =
            |reference_name: &oneil_shared::symbols::ReferenceName| -> Option<EvalInstanceKey> {
                resolve_external_instance_key(instance_key, model, &all_model_ir, reference_name)
            };

        for (parameter_name, parameter) in model.parameters() {
            let dependencies = parameter.dependencies();

            for builtin_dep in dependencies.builtin().keys() {
                dependency_graph.add_depends_on_builtin(
                    instance_key.clone(),
                    parameter_name.clone(),
                    oneil_output::BuiltinDependency {
                        name: builtin_dep.clone(),
                    },
                );
            }

            for parameter_dep in dependencies.parameter().keys() {
                dependency_graph.add_depends_on_parameter(
                    instance_key.clone(),
                    parameter_name.clone(),
                    oneil_output::ParameterDependency {
                        parameter_name: parameter_dep.clone(),
                    },
                );
            }

            for ((reference_dep_name, parameter_dep_name), _span) in dependencies.external() {
                let Some(external_instance_key) = resolve_external_key(reference_dep_name) else {
                    continue;
                };
                dependency_graph.add_depends_on_external(
                    instance_key.clone(),
                    parameter_name.clone(),
                    oneil_output::ExternalDependency {
                        instance_key: external_instance_key,
                        reference_name: reference_dep_name.clone(),
                        parameter_name: parameter_dep_name.clone(),
                    },
                );
            }
        }

        for (test_index, test) in model.tests() {
            let dependencies = test.dependencies();

            for parameter_dep in dependencies.parameter().keys() {
                dependency_graph.add_test_depends_on_parameter(
                    instance_key.clone(),
                    *test_index,
                    oneil_output::ParameterDependency {
                        parameter_name: parameter_dep.clone(),
                    },
                );
            }

            for ((reference_dep_name, parameter_dep_name), _span) in dependencies.external() {
                let Some(external_instance_key) = resolve_external_key(reference_dep_name) else {
                    continue;
                };
                dependency_graph.add_test_depends_on_external(
                    instance_key.clone(),
                    *test_index,
                    oneil_output::ExternalDependency {
                        instance_key: external_instance_key,
                        reference_name: reference_dep_name.clone(),
                        parameter_name: parameter_dep_name.clone(),
                    },
                );
            }
        }
    }

    dependency_graph
}

/// Resolves a reference name from an instanced IR node to the evaluated instance key it targets.
fn resolve_external_instance_key(
    instance_key: &EvalInstanceKey,
    model: &oneil_frontend::InstancedModel,
    all_model_ir: &indexmap::IndexMap<EvalInstanceKey, &oneil_frontend::InstancedModel>,
    reference_name: &ReferenceName,
) -> Option<EvalInstanceKey> {
    if let Some(reference) = model.references().get(reference_name) {
        return Some(EvalInstanceKey::root(reference.path.clone()));
    }

    if let Some(submodel) = model.submodels().get(reference_name) {
        return Some(EvalInstanceKey {
            model_path: submodel.instance.path().clone(),
            instance_path: instance_key
                .instance_path
                .clone()
                .child(reference_name.clone()),
        });
    }

    let alias = model.aliases().get(reference_name)?;
    let mut segments = alias.alias_path.segments().iter();
    let first = segments.next()?;

    let mut current_key = if let Some(submodel) = model.submodels().get(first) {
        EvalInstanceKey {
            model_path: submodel.instance.path().clone(),
            instance_path: instance_key.instance_path.clone().child(first.clone()),
        }
    } else if let Some(reference) = model.references().get(first) {
        EvalInstanceKey::root(reference.path.clone())
    } else {
        return None;
    };

    for segment in segments {
        let current_model = all_model_ir.get(&current_key)?;
        let submodel = current_model.submodels().get(segment)?;
        current_key = EvalInstanceKey {
            model_path: submodel.instance.path().clone(),
            instance_path: current_key.instance_path.child(segment.clone()),
        };
    }

    Some(current_key)
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use oneil_ir as ir;
    use oneil_output::Value;
    use oneil_shared::{
        EvalInstanceKey,
        symbols::{BuiltinValueName, ParameterName, ReferenceName, TestIndex},
    };

    use super::{get_dependency_tree, get_reference_tree};
    use crate::{
        output::{
            DependencyName, ReferenceTreeValue, error::GetTestValueError, error::GetValueError,
        },
        test_assertions::assert_no_tree_errors,
        test_context::{TestAnalysisContext, test_model_path},
        test_fixtures::{
            deps_on_builtin, deps_on_external, deps_on_parameters, evaluated_parameter,
            evaluated_test_failed, evaluated_test_passed, instanced_model, ir_parameter, ir_test,
            reference_import,
        },
    };

    #[test]
    fn dependency_tree_leaf_has_no_children() {
        let path = test_model_path("leaf");
        let key = EvalInstanceKey::root(path.clone());

        let mut parameters = IndexMap::new();
        parameters.insert(
            ParameterName::from("x"),
            ir_parameter("x", ir::Dependencies::new()),
        );

        let mut context = TestAnalysisContext::new();
        context.insert_model_ir(
            key.clone(),
            instanced_model(&path, parameters, IndexMap::new(), IndexMap::new()),
        );
        context.insert_evaluated_parameter(&key, evaluated_parameter("x", 1.0));

        let (tree, errors) = get_dependency_tree(&path, &ParameterName::from("x"), &mut context);

        assert_no_tree_errors(&errors);
        let tree = tree.expect("tree should exist");
        assert_eq!(
            tree.value().dependency_name,
            DependencyName::Parameter(ParameterName::from("x"))
        );
        assert_eq!(tree.value().parameter_value, Value::from(1.0));
        assert!(tree.children().is_empty());
    }

    #[test]
    fn dependency_tree_includes_parameter_and_builtin_children() {
        let path = test_model_path("deps");
        let key = EvalInstanceKey::root(path.clone());

        let mut parameters = IndexMap::new();
        parameters.insert(
            ParameterName::from("x"),
            ir_parameter("x", ir::Dependencies::new()),
        );
        parameters.insert(
            ParameterName::from("y"),
            ir_parameter("y", {
                let mut deps = deps_on_parameters(&["x"]);
                deps.extend(deps_on_builtin("pi"));
                deps
            }),
        );

        let mut context = TestAnalysisContext::new();
        context.insert_model_ir(
            key.clone(),
            instanced_model(&path, parameters, IndexMap::new(), IndexMap::new()),
        );
        context.insert_evaluated_parameter(&key, evaluated_parameter("x", 2.0));
        context.insert_evaluated_parameter(&key, evaluated_parameter("y", 5.0));
        context.insert_builtin(
            BuiltinValueName::from("pi"),
            Value::from(std::f64::consts::PI),
        );

        let (tree, errors) = get_dependency_tree(&path, &ParameterName::from("y"), &mut context);

        assert_no_tree_errors(&errors);
        let tree = tree.expect("tree should exist");
        assert_eq!(
            tree.value().dependency_name,
            DependencyName::Parameter(ParameterName::from("y"))
        );

        let child_names = tree.child_dependency_names();
        assert_eq!(
            child_names,
            vec![
                &DependencyName::Builtin(BuiltinValueName::from("pi")),
                &DependencyName::Parameter(ParameterName::from("x")),
            ]
        );

        let builtin_child = &tree.children()[0];
        assert_eq!(
            builtin_child.value().parameter_value,
            Value::from(std::f64::consts::PI)
        );
        assert!(builtin_child.children().is_empty());
        assert!(builtin_child.value().display_info.is_none());

        let param_child = &tree.children()[1];
        assert_eq!(param_child.value().parameter_value, Value::from(2.0));
        assert!(param_child.children().is_empty());
    }

    #[test]
    fn dependency_tree_nests_transitive_parameter_dependencies() {
        let path = test_model_path("chain");
        let key = EvalInstanceKey::root(path.clone());

        let mut parameters = IndexMap::new();
        parameters.insert(
            ParameterName::from("a"),
            ir_parameter("a", ir::Dependencies::new()),
        );
        parameters.insert(
            ParameterName::from("b"),
            ir_parameter("b", deps_on_parameters(&["a"])),
        );
        parameters.insert(
            ParameterName::from("c"),
            ir_parameter("c", deps_on_parameters(&["b"])),
        );

        let mut context = TestAnalysisContext::new();
        context.insert_model_ir(
            key.clone(),
            instanced_model(&path, parameters, IndexMap::new(), IndexMap::new()),
        );
        context.insert_evaluated_parameter(&key, evaluated_parameter("a", 1.0));
        context.insert_evaluated_parameter(&key, evaluated_parameter("b", 2.0));
        context.insert_evaluated_parameter(&key, evaluated_parameter("c", 3.0));

        let (tree, errors) = get_dependency_tree(&path, &ParameterName::from("c"), &mut context);

        assert_no_tree_errors(&errors);
        let tree = tree.expect("tree should exist");
        assert_eq!(
            tree.child_dependency_names(),
            vec![&DependencyName::Parameter(ParameterName::from("b"))]
        );

        let b = &tree.children()[0];
        assert_eq!(
            b.child_dependency_names(),
            vec![&DependencyName::Parameter(ParameterName::from("a"))]
        );
        assert!(b.children()[0].children().is_empty());
    }

    #[test]
    fn dependency_tree_includes_external_children() {
        let root_path = test_model_path("root");
        let other_path = test_model_path("other");
        let root_key = EvalInstanceKey::root(root_path.clone());
        let other_key = EvalInstanceKey::root(other_path.clone());

        let mut root_parameters = IndexMap::new();
        root_parameters.insert(
            ParameterName::from("y"),
            ir_parameter("y", deps_on_external("other", "x")),
        );

        let mut other_parameters = IndexMap::new();
        other_parameters.insert(
            ParameterName::from("x"),
            ir_parameter("x", ir::Dependencies::new()),
        );

        let mut references = IndexMap::new();
        references.insert(
            ReferenceName::from("other"),
            reference_import("other", &other_path),
        );

        let mut context = TestAnalysisContext::new();
        context.insert_model_ir(
            root_key.clone(),
            instanced_model(&root_path, root_parameters, IndexMap::new(), references),
        );
        context.insert_model_ir(
            other_key.clone(),
            instanced_model(
                &other_path,
                other_parameters,
                IndexMap::new(),
                IndexMap::new(),
            ),
        );
        context.insert_evaluated_parameter(&root_key, evaluated_parameter("y", 10.0));
        context.insert_evaluated_parameter(&other_key, evaluated_parameter("x", 4.0));

        let (tree, errors) =
            get_dependency_tree(&root_path, &ParameterName::from("y"), &mut context);

        assert_no_tree_errors(&errors);
        let tree = tree.expect("tree should exist");
        assert_eq!(
            tree.child_dependency_names(),
            vec![&DependencyName::External(
                ReferenceName::from("other"),
                ParameterName::from("x"),
            )]
        );

        let external = &tree.children()[0];
        assert_eq!(external.value().parameter_value, Value::from(4.0));
        assert!(external.children().is_empty());
    }

    #[test]
    fn dependency_tree_missing_parameter_returns_none() {
        let path = test_model_path("missing");
        let key = EvalInstanceKey::root(path.clone());

        let mut context = TestAnalysisContext::new();
        context.insert_model_ir(
            key,
            instanced_model(&path, IndexMap::new(), IndexMap::new(), IndexMap::new()),
        );

        let (tree, errors) =
            get_dependency_tree(&path, &ParameterName::from("absent"), &mut context);

        assert!(tree.is_none());
        assert_no_tree_errors(&errors);
    }

    #[test]
    fn dependency_tree_parameter_error_is_reported() {
        let path = test_model_path("err");
        let key = EvalInstanceKey::root(path.clone());

        let mut parameters = IndexMap::new();
        parameters.insert(
            ParameterName::from("x"),
            ir_parameter("x", ir::Dependencies::new()),
        );

        let mut context = TestAnalysisContext::new();
        context.insert_model_ir(
            key.clone(),
            instanced_model(&path, parameters, IndexMap::new(), IndexMap::new()),
        );
        context.insert_parameter_error(&key, ParameterName::from("x"), GetValueError::Parameter);

        let (tree, errors) = get_dependency_tree(&path, &ParameterName::from("x"), &mut context);

        assert!(tree.is_none());
        assert_eq!(errors.model_paths().collect::<Vec<_>>(), vec![&path]);
    }

    #[test]
    fn reference_tree_leaf_has_no_children() {
        let path = test_model_path("ref_leaf");
        let key = EvalInstanceKey::root(path.clone());

        let mut parameters = IndexMap::new();
        parameters.insert(
            ParameterName::from("x"),
            ir_parameter("x", ir::Dependencies::new()),
        );

        let mut context = TestAnalysisContext::new();
        context.insert_model_ir(
            key.clone(),
            instanced_model(&path, parameters, IndexMap::new(), IndexMap::new()),
        );
        context.insert_evaluated_parameter(&key, evaluated_parameter("x", 1.0));

        let (tree, errors) = get_reference_tree(&mut context, &path, &ParameterName::from("x"));

        assert_no_tree_errors(&errors);
        let tree = tree.expect("tree should exist");
        match tree.value() {
            ReferenceTreeValue::Parameter {
                parameter_name,
                parameter_value,
                ..
            } => {
                assert_eq!(*parameter_name, ParameterName::from("x"));
                assert_eq!(*parameter_value, Value::from(1.0));
            }
            ReferenceTreeValue::Test { .. } => panic!("expected parameter node"),
        }
        assert!(tree.children().is_empty());
    }

    #[test]
    fn reference_tree_includes_dependent_parameters() {
        let path = test_model_path("ref_chain");
        let key = EvalInstanceKey::root(path.clone());

        let mut parameters = IndexMap::new();
        parameters.insert(
            ParameterName::from("a"),
            ir_parameter("a", ir::Dependencies::new()),
        );
        parameters.insert(
            ParameterName::from("b"),
            ir_parameter("b", deps_on_parameters(&["a"])),
        );
        parameters.insert(
            ParameterName::from("c"),
            ir_parameter("c", deps_on_parameters(&["b"])),
        );

        let mut context = TestAnalysisContext::new();
        context.insert_model_ir(
            key.clone(),
            instanced_model(&path, parameters, IndexMap::new(), IndexMap::new()),
        );
        context.insert_evaluated_parameter(&key, evaluated_parameter("a", 1.0));
        context.insert_evaluated_parameter(&key, evaluated_parameter("b", 2.0));
        context.insert_evaluated_parameter(&key, evaluated_parameter("c", 3.0));

        let (tree, errors) = get_reference_tree(&mut context, &path, &ParameterName::from("a"));

        assert_no_tree_errors(&errors);
        let tree = tree.expect("tree should exist");
        assert_eq!(tree.children().len(), 1);

        let b = &tree.children()[0];
        match b.value() {
            ReferenceTreeValue::Parameter {
                parameter_name,
                parameter_value,
                ..
            } => {
                assert_eq!(*parameter_name, ParameterName::from("b"));
                assert_eq!(*parameter_value, Value::from(2.0));
            }
            ReferenceTreeValue::Test { .. } => panic!("expected parameter node"),
        }

        assert_eq!(b.children().len(), 1);
        match b.children()[0].value() {
            ReferenceTreeValue::Parameter {
                parameter_name,
                parameter_value,
                ..
            } => {
                assert_eq!(*parameter_name, ParameterName::from("c"));
                assert_eq!(*parameter_value, Value::from(3.0));
            }
            ReferenceTreeValue::Test { .. } => panic!("expected parameter node"),
        }
    }

    #[test]
    fn reference_tree_includes_tests_that_depend_on_parameter() {
        let path = test_model_path("ref_test");
        let key = EvalInstanceKey::root(path.clone());
        let test_index = TestIndex::new(0);

        let mut parameters = IndexMap::new();
        parameters.insert(
            ParameterName::from("x"),
            ir_parameter("x", ir::Dependencies::new()),
        );

        let mut tests = IndexMap::new();
        tests.insert(test_index, ir_test(deps_on_parameters(&["x"])));

        let mut context = TestAnalysisContext::new();
        context.insert_model_ir(
            key.clone(),
            instanced_model(&path, parameters, tests, IndexMap::new()),
        );
        context.insert_evaluated_parameter(&key, evaluated_parameter("x", 1.0));
        context.insert_evaluated_test(&key, test_index, evaluated_test_passed());

        let (tree, errors) = get_reference_tree(&mut context, &path, &ParameterName::from("x"));

        assert_no_tree_errors(&errors);
        let tree = tree.expect("tree should exist");
        assert_eq!(tree.children().len(), 1);
        match tree.children()[0].value() {
            ReferenceTreeValue::Test {
                test_index: index,
                test_passed,
                model_path,
                ..
            } => {
                assert_eq!(*index, test_index);
                assert!(*test_passed);
                assert_eq!(*model_path, path);
            }
            ReferenceTreeValue::Parameter { .. } => panic!("expected test node"),
        }
    }

    #[test]
    fn reference_tree_test_lookup_error_is_reported() {
        let path = test_model_path("ref_test_err");
        let key = EvalInstanceKey::root(path.clone());
        let test_index = TestIndex::new(0);

        let mut parameters = IndexMap::new();
        parameters.insert(
            ParameterName::from("x"),
            ir_parameter("x", ir::Dependencies::new()),
        );

        let mut tests = IndexMap::new();
        tests.insert(test_index, ir_test(deps_on_parameters(&["x"])));

        let mut context = TestAnalysisContext::new();
        context.insert_model_ir(
            key.clone(),
            instanced_model(&path, parameters, tests, IndexMap::new()),
        );
        context.insert_evaluated_parameter(&key, evaluated_parameter("x", 1.0));
        context.insert_test_error(&key, test_index, GetTestValueError::Test);

        let (tree, errors) = get_reference_tree(&mut context, &path, &ParameterName::from("x"));

        let tree = tree.expect("parameter root should still exist");
        assert!(tree.children().is_empty());
        assert_eq!(errors.model_paths().collect::<Vec<_>>(), vec![&path]);
    }

    #[test]
    fn reference_tree_includes_external_parameter_referees() {
        let root_path = test_model_path("ref_root");
        let other_path = test_model_path("ref_other");
        let root_key = EvalInstanceKey::root(root_path.clone());
        let other_key = EvalInstanceKey::root(other_path.clone());

        let mut root_parameters = IndexMap::new();
        root_parameters.insert(
            ParameterName::from("y"),
            ir_parameter("y", deps_on_external("other", "x")),
        );

        let mut other_parameters = IndexMap::new();
        other_parameters.insert(
            ParameterName::from("x"),
            ir_parameter("x", ir::Dependencies::new()),
        );

        let mut references = IndexMap::new();
        references.insert(
            ReferenceName::from("other"),
            reference_import("other", &other_path),
        );

        let mut context = TestAnalysisContext::new();
        context.insert_model_ir(
            root_key.clone(),
            instanced_model(&root_path, root_parameters, IndexMap::new(), references),
        );
        context.insert_model_ir(
            other_key.clone(),
            instanced_model(
                &other_path,
                other_parameters,
                IndexMap::new(),
                IndexMap::new(),
            ),
        );
        context.insert_evaluated_parameter(&root_key, evaluated_parameter("y", 10.0));
        context.insert_evaluated_parameter(&other_key, evaluated_parameter("x", 4.0));

        let (tree, errors) =
            get_reference_tree(&mut context, &other_path, &ParameterName::from("x"));

        assert_no_tree_errors(&errors);
        let tree = tree.expect("tree should exist");
        assert_eq!(tree.children().len(), 1);
        match tree.children()[0].value() {
            ReferenceTreeValue::Parameter {
                model_path,
                parameter_name,
                parameter_value,
                ..
            } => {
                assert_eq!(*model_path, root_path);
                assert_eq!(*parameter_name, ParameterName::from("y"));
                assert_eq!(*parameter_value, Value::from(10.0));
            }
            ReferenceTreeValue::Test { .. } => panic!("expected parameter node"),
        }
    }

    #[test]
    fn reference_tree_records_failed_test_status() {
        let path = test_model_path("failed_test");
        let key = EvalInstanceKey::root(path.clone());
        let test_index = TestIndex::new(1);

        let mut parameters = IndexMap::new();
        parameters.insert(
            ParameterName::from("x"),
            ir_parameter("x", ir::Dependencies::new()),
        );

        let mut tests = IndexMap::new();
        tests.insert(test_index, ir_test(deps_on_parameters(&["x"])));

        let mut context = TestAnalysisContext::new();
        context.insert_model_ir(
            key.clone(),
            instanced_model(&path, parameters, tests, IndexMap::new()),
        );
        context.insert_evaluated_parameter(&key, evaluated_parameter("x", 1.0));
        context.insert_evaluated_test(&key, test_index, evaluated_test_failed());

        let (tree, errors) = get_reference_tree(&mut context, &path, &ParameterName::from("x"));

        assert_no_tree_errors(&errors);
        let tree = tree.expect("tree should exist");
        match tree.children()[0].value() {
            ReferenceTreeValue::Test { test_passed, .. } => assert!(!test_passed),
            ReferenceTreeValue::Parameter { .. } => panic!("expected test node"),
        }
    }
}
