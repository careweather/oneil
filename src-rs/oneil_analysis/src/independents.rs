//! Analysis of parameters that have no dependencies.

use std::path::Path;

use indexmap::IndexMap;
use oneil_output::DependencySet;

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
    model_path: &Path,
    external_context: &E,
) -> (Independents, IndependentsErrors) {
    let path_buf = model_path.to_path_buf();

    let Some(load_result) = external_context.get_evaluated_model(model_path) else {
        let mut errors = IndependentsErrors::new();
        errors.insert(path_buf);
        return (Independents::empty(), errors);
    };

    let Some(model) = load_result.value() else {
        let mut errors = IndependentsErrors::new();
        errors.insert(path_buf);
        return (Independents::empty(), errors);
    };

    let independents: IndexMap<String, _> = model
        .parameters
        .iter()
        .filter(|(_, p)| is_empty_dependencies(&p.dependencies))
        .map(|(name, p)| (name.clone(), p.value.clone()))
        .collect();

    let mut result = Independents::empty();
    result.insert(path_buf.clone(), independents);

    let mut errors = IndependentsErrors::new();
    if load_result.error().is_some() {
        errors.insert(path_buf);
    }

    for ref_path in model.references.values() {
        let (nested_independents, nested_errors) =
            get_independents(ref_path.as_path(), external_context);

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
