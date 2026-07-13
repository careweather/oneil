//! Analysis of parameters that have no dependencies.

use indexmap::IndexMap;
use oneil_output::DependencySet;
use oneil_shared::paths::ModelPath;

use crate::context::ExternalAnalysisContext;
use crate::output::{Independents, error::IndependentsErrors};

/// Determines which parameters in a model (and its referenced models) have no dependencies.
///
/// Gets the evaluated model at `model_path`, then iterates its parameters; any parameter
/// whose dependency set is empty is recorded as an independent with its value. Then recurses
/// into each reference (evaluated submodel) and merges their independents into the result.
///
/// Returns an [`Independents`] (model path → parameter name → value) and an
/// [`IndependentsErrors`] for model paths that had evaluation errors.
#[must_use]
pub fn get_independents<E: ExternalAnalysisContext>(
    model_path: &ModelPath,
    external_context: &E,
) -> (Independents, IndependentsErrors) {
    let Some(load_result) = external_context.get_evaluated_model(model_path) else {
        let mut errors = IndependentsErrors::new();
        errors.insert(model_path.clone());
        return (Independents::empty(), errors);
    };

    let Some(model) = load_result.value() else {
        let mut errors = IndependentsErrors::new();
        errors.insert(model_path.clone());
        return (Independents::empty(), errors);
    };

    let independents: IndexMap<_, _> = model
        .parameters
        .iter()
        .filter(|(_, p)| is_empty_dependencies(&p.dependencies))
        .map(|(name, p)| (name.clone(), p.value.clone()))
        .collect();

    let mut result = Independents::empty();
    result.insert(model_path.clone(), independents);

    let mut errors = IndependentsErrors::new();
    if load_result.error().is_some() {
        errors.insert(model_path.clone());
    }

    for ref_path in model.references.values() {
        let (nested_independents, nested_errors) =
            get_independents(&ref_path.model_path, external_context);

        result.extend(nested_independents);
        errors.extend(nested_errors);
    }

    (result, errors)
}

fn is_empty_dependencies(deps: &DependencySet) -> bool {
    // NOTE: builtin dependencies are not considered here, since they are
    //       considered to be constant
    deps.parameter_dependencies.is_empty() && deps.external_dependencies.is_empty()
}

#[cfg(test)]
mod tests {
    use indexmap::{IndexMap, IndexSet};
    use oneil_output::{
        BuiltinDependency, DependencySet, ExternalDependency, ParameterDependency, Value,
    };
    use oneil_shared::{
        EvalInstanceKey,
        load_result::LoadResult,
        symbols::{BuiltinValueName, ParameterName, ReferenceName},
    };

    use super::get_independents;
    use crate::{
        output::error::ModelEvalHasErrors,
        test_context::{
            TestAnalysisContext, evaluated_model, evaluated_parameter,
            evaluated_parameter_with_deps, test_model_path,
        },
    };

    /// Builds a dependency set with only same-model parameter dependencies.
    fn parameter_deps(names: &[&str]) -> DependencySet {
        DependencySet {
            builtin_dependencies: IndexSet::new(),
            parameter_dependencies: names
                .iter()
                .map(|name| ParameterDependency {
                    parameter_name: ParameterName::from(*name),
                })
                .collect(),
            external_dependencies: IndexSet::new(),
        }
    }

    /// Builds a dependency set with only builtin dependencies.
    fn builtin_deps(names: &[&str]) -> DependencySet {
        DependencySet {
            builtin_dependencies: names
                .iter()
                .map(|name| BuiltinDependency {
                    name: BuiltinValueName::from(*name),
                })
                .collect(),
            parameter_dependencies: IndexSet::new(),
            external_dependencies: IndexSet::new(),
        }
    }

    /// Builds a dependency set with a single external dependency.
    fn external_deps(
        instance_key: EvalInstanceKey,
        reference: &str,
        parameter: &str,
    ) -> DependencySet {
        DependencySet {
            builtin_dependencies: IndexSet::new(),
            parameter_dependencies: IndexSet::new(),
            external_dependencies: IndexSet::from([ExternalDependency {
                instance_key,
                reference_name: ReferenceName::from(reference),
                parameter_name: ParameterName::from(parameter),
            }]),
        }
    }

    #[test]
    fn missing_model_reports_error() {
        let path = test_model_path("missing");
        let context = TestAnalysisContext::new();

        let (independents, errors) = get_independents(&path, &context);

        assert!(independents.is_empty());
        assert_eq!(errors.paths().collect::<Vec<_>>(), vec![&path]);
    }

    #[test]
    fn failure_load_reports_error() {
        let path = test_model_path("failed");
        let mut context = TestAnalysisContext::new();
        context.insert_evaluated_model(path.clone(), LoadResult::failure());

        let (independents, errors) = get_independents(&path, &context);

        assert!(independents.is_empty());
        assert_eq!(errors.paths().collect::<Vec<_>>(), vec![&path]);
    }

    #[test]
    fn leaf_parameters_are_independent() {
        let path = test_model_path("leaves");
        let mut parameters = IndexMap::new();
        parameters.insert(ParameterName::from("x"), evaluated_parameter("x", 1.0));
        parameters.insert(ParameterName::from("y"), evaluated_parameter("y", 2.0));

        let mut context = TestAnalysisContext::new();
        context.insert_evaluated_model(
            path.clone(),
            LoadResult::success(evaluated_model(&path, parameters, IndexMap::new())),
        );

        let (independents, errors) = get_independents(&path, &context);

        assert!(errors.is_empty());
        let params = independents.get(&path).expect("model entry");
        assert_eq!(params.len(), 2);
        assert_eq!(
            params.get(&ParameterName::from("x")),
            Some(&Value::from(1.0))
        );
        assert_eq!(
            params.get(&ParameterName::from("y")),
            Some(&Value::from(2.0))
        );
    }

    #[test]
    fn excludes_parameters_with_parameter_dependencies() {
        let path = test_model_path("param_deps");
        let mut parameters = IndexMap::new();
        parameters.insert(ParameterName::from("x"), evaluated_parameter("x", 1.0));
        parameters.insert(
            ParameterName::from("y"),
            evaluated_parameter_with_deps("y", 3.0, parameter_deps(&["x"])),
        );

        let mut context = TestAnalysisContext::new();
        context.insert_evaluated_model(
            path.clone(),
            LoadResult::success(evaluated_model(&path, parameters, IndexMap::new())),
        );

        let (independents, errors) = get_independents(&path, &context);

        assert!(errors.is_empty());
        let params = independents.get(&path).expect("model entry");
        assert_eq!(params.len(), 1);
        assert_eq!(
            params.get(&ParameterName::from("x")),
            Some(&Value::from(1.0))
        );
        assert!(!params.contains_key(&ParameterName::from("y")));
    }

    #[test]
    fn excludes_parameters_with_external_dependencies() {
        let path = test_model_path("ext_deps");
        let other = EvalInstanceKey::root(test_model_path("other"));
        let mut parameters = IndexMap::new();
        parameters.insert(ParameterName::from("x"), evaluated_parameter("x", 1.0));
        parameters.insert(
            ParameterName::from("y"),
            evaluated_parameter_with_deps("y", 3.0, external_deps(other, "other", "z")),
        );

        let mut context = TestAnalysisContext::new();
        context.insert_evaluated_model(
            path.clone(),
            LoadResult::success(evaluated_model(&path, parameters, IndexMap::new())),
        );

        let (independents, errors) = get_independents(&path, &context);

        assert!(errors.is_empty());
        let params = independents.get(&path).expect("model entry");
        assert_eq!(
            params.keys().collect::<Vec<_>>(),
            vec![&ParameterName::from("x")]
        );
    }

    #[test]
    fn includes_parameters_with_only_builtin_dependencies() {
        let path = test_model_path("builtins");
        let mut parameters = IndexMap::new();
        parameters.insert(
            ParameterName::from("circ"),
            evaluated_parameter_with_deps("circ", std::f64::consts::PI, builtin_deps(&["pi"])),
        );

        let mut context = TestAnalysisContext::new();
        context.insert_evaluated_model(
            path.clone(),
            LoadResult::success(evaluated_model(&path, parameters, IndexMap::new())),
        );

        let (independents, errors) = get_independents(&path, &context);

        assert!(errors.is_empty());
        let params = independents.get(&path).expect("model entry");
        assert_eq!(
            params.get(&ParameterName::from("circ")),
            Some(&Value::from(std::f64::consts::PI))
        );
    }

    #[test]
    fn recurses_into_referenced_models() {
        let root_path = test_model_path("root");
        let child_path = test_model_path("child");
        let child_key = EvalInstanceKey::root(child_path.clone());

        let mut root_parameters = IndexMap::new();
        root_parameters.insert(ParameterName::from("a"), evaluated_parameter("a", 1.0));
        root_parameters.insert(
            ParameterName::from("b"),
            evaluated_parameter_with_deps("b", 2.0, parameter_deps(&["a"])),
        );

        let mut child_parameters = IndexMap::new();
        child_parameters.insert(ParameterName::from("c"), evaluated_parameter("c", 3.0));

        let mut references = IndexMap::new();
        references.insert(ReferenceName::from("child"), child_key);

        let mut context = TestAnalysisContext::new();
        context.insert_evaluated_model(
            root_path.clone(),
            LoadResult::success(evaluated_model(&root_path, root_parameters, references)),
        );
        context.insert_evaluated_model(
            child_path.clone(),
            LoadResult::success(evaluated_model(
                &child_path,
                child_parameters,
                IndexMap::new(),
            )),
        );

        let (independents, errors) = get_independents(&root_path, &context);

        assert!(errors.is_empty());
        let root_params = independents.get(&root_path).expect("root entry");
        assert_eq!(
            root_params.get(&ParameterName::from("a")),
            Some(&Value::from(1.0))
        );
        assert!(!root_params.contains_key(&ParameterName::from("b")));

        let child_params = independents.get(&child_path).expect("child entry");
        assert_eq!(
            child_params.get(&ParameterName::from("c")),
            Some(&Value::from(3.0))
        );
    }

    #[test]
    fn partial_result_keeps_independents_and_records_error() {
        let path = test_model_path("partial");
        let mut parameters = IndexMap::new();
        parameters.insert(ParameterName::from("x"), evaluated_parameter("x", 1.0));

        let mut context = TestAnalysisContext::new();
        context.insert_evaluated_model(
            path.clone(),
            LoadResult::partial(
                evaluated_model(&path, parameters, IndexMap::new()),
                ModelEvalHasErrors,
            ),
        );

        let (independents, errors) = get_independents(&path, &context);

        assert_eq!(errors.paths().collect::<Vec<_>>(), vec![&path]);
        let params = independents.get(&path).expect("model entry");
        assert_eq!(
            params.get(&ParameterName::from("x")),
            Some(&Value::from(1.0))
        );
    }

    #[test]
    fn nested_reference_errors_are_accumulated() {
        let root_path = test_model_path("err_root");
        let child_path = test_model_path("err_child");
        let child_key = EvalInstanceKey::root(child_path.clone());

        let mut references = IndexMap::new();
        references.insert(ReferenceName::from("child"), child_key);

        let mut context = TestAnalysisContext::new();
        context.insert_evaluated_model(
            root_path.clone(),
            LoadResult::success(evaluated_model(&root_path, IndexMap::new(), references)),
        );
        // child is missing from the context

        let (independents, errors) = get_independents(&root_path, &context);

        assert!(independents.get(&root_path).expect("root entry").is_empty());
        assert_eq!(errors.paths().collect::<Vec<_>>(), vec![&child_path]);
    }
}
