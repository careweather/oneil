//! Print independent parameters from evaluated models.

use anstream::{print, println};
use oneil_runtime::output::{
    eval,
    reference::{EvalErrorReference, ModelReference},
};
use oneil_shared::error::OneilError;

use crate::{print_error, print_utils, stylesheet};

#[expect(
    clippy::struct_excessive_bools,
    reason = "this is a configuration struct for printing independent parameters"
)]
pub struct IndependentPrintConfig {
    pub print_values: bool,
    pub recursive: bool,
    pub display_partial_results: bool,
    pub show_internal_errors: bool,
}

/// Prints all independent parameters from the model result.
///
/// First collects and displays any evaluation errors from the models. Then prints
/// independent parameters if there were no errors or if `display_partial_results`
/// is true. Independent parameters are those that don't depend on other parameters
/// (they may still depend on builtin values).
///
/// # Arguments
///
/// * `model_result` - The evaluation result containing all models
/// * `independent_print_config` - Configuration for printing
pub fn print(
    model_result: Result<ModelReference<'_>, EvalErrorReference<'_>>,
    independent_print_config: &IndependentPrintConfig,
) {
    let errors = collect_all_errors(&model_result);

    for error in &errors {
        print_error::print(error, false, independent_print_config.show_internal_errors);
    }

    let show_independents = errors.is_empty() || independent_print_config.display_partial_results;

    if show_independents {
        let top_model = match &model_result {
            Ok(model_ref) => Some(*model_ref),
            Err(err_ref) => err_ref.partial_result(),
        };

        if let Some(model_ref) = top_model {
            print_model_independents(model_ref, independent_print_config);
        }
    }
}

/// Collects all evaluation errors from the model result and its nested references.
///
/// Uses a depth-first traversal; errors from failed reference evaluations are included.
fn collect_all_errors(
    model_result: &Result<ModelReference<'_>, EvalErrorReference<'_>>,
) -> Vec<OneilError> {
    let mut errors = Vec::new();
    let mut stack: Vec<Result<ModelReference<'_>, EvalErrorReference<'_>>> = match model_result {
        Ok(r) => vec![Ok(*r)],
        Err(e) => {
            errors.extend(e.model_errors());
            e.partial_result()
                .map_or(vec![], |partial| vec![Ok(partial)])
        }
    };

    while let Some(r) = stack.pop() {
        match r {
            Err(e) => {
                errors.extend(e.model_errors());
                if let Some(partial) = e.partial_result() {
                    stack.push(Ok(partial));
                }
            }
            Ok(model_ref) => {
                for nested in model_ref.references().values() {
                    stack.push(*nested);
                }
            }
        }
    }

    errors
}

/// Recursively prints independent parameters for a model and its submodels.
fn print_model_independents(
    model_ref: ModelReference<'_>,
    independent_print_config: &IndependentPrintConfig,
) {
    if independent_print_config.recursive {
        let model_path = model_ref.path().display();
        let styled_model_name = stylesheet::MODEL_PATH_HEADER.style(model_path);
        println!("{styled_model_name}:");
    }

    let independent_params = get_independent_parameters(model_ref);

    for param in independent_params {
        print_parameter(param, independent_print_config.print_values);
    }

    if independent_print_config.recursive {
        // Print references
        for reference in model_ref.references().values() {
            let nested_ref = match reference {
                Ok(r) => Some(*r),
                Err(e) => e.partial_result(),
            };

            if let Some(nested_ref) = nested_ref {
                println!();
                print_model_independents(nested_ref, independent_print_config);
            }
        }
    }
}

/// Gets all independent parameters from a model.
///
/// A parameter is independent if it doesn't have any parameter dependencies
/// or external dependencies (it may still have builtin dependencies).
fn get_independent_parameters(model_ref: ModelReference<'_>) -> Vec<&eval::Parameter> {
    let parameters = model_ref.parameters();

    parameters
        .values()
        .filter(|param| {
            // Independent if no parameter or external dependencies
            param.dependencies.parameter_dependencies.is_empty()
                && param.dependencies.external_dependencies.is_empty()
        })
        .copied()
        .collect()
}

/// Prints a single parameter.
fn print_parameter(parameter: &eval::Parameter, print_values: bool) {
    let styled_ident = stylesheet::PARAMETER_IDENTIFIER.style(&parameter.ident);

    if print_values {
        print!("{styled_ident} = ");
        print_utils::print_value(&parameter.value);
    } else {
        print!("{styled_ident}");
    }

    let styled_label = stylesheet::PARAMETER_LABEL.style(format!("# {}", parameter.label));
    println!(" {styled_label}");
}
