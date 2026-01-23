//! Print independent parameters from evaluated models.

use anstream::{print, println};
use oneil_eval::output::eval_result::{self, EvalResult};

use crate::{print_utils, stylesheet};

pub struct IndependentPrintConfig {
    pub print_values: bool,
    pub recursive: bool,
}

/// Prints all independent parameters from the model result.
///
/// Independent parameters are those that don't depend on other parameters
/// (they may still depend on builtin values). This function prints them
/// in a hierarchical format, optionally including submodels recursively.
///
/// # Arguments
///
/// * `model_result` - The evaluation result containing all models
/// * `dependency_graph` - The dependency graph to determine which parameters are independent
/// * `print_values` - Whether to print the parameter values
/// * `recursive` - Whether to print independent parameters in submodels as well
pub fn print(model_result: &EvalResult, independent_print_config: &IndependentPrintConfig) {
    let top_model = model_result.get_top_model();
    print_model_independents(top_model, independent_print_config);
}

/// Recursively prints independent parameters for a model and its submodels.
fn print_model_independents(
    model_ref: eval_result::ModelReference<'_>,
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
        //
        // This includes submodels because submodels are also
        // references, so we can simply use the references map
        for reference in model_ref.references().values() {
            println!();
            print_model_independents(*reference, independent_print_config);
        }
    }
}

/// Gets all independent parameters from a model.
///
/// A parameter is independent if it doesn't have any parameter dependencies
/// or external dependencies (it may still have builtin dependencies).
fn get_independent_parameters(
    model_ref: eval_result::ModelReference<'_>,
) -> Vec<&eval_result::Parameter> {
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
fn print_parameter(parameter: &eval_result::Parameter, print_values: bool) {
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
