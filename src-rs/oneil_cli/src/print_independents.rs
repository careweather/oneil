//! Print independent parameters from evaluated models.

use std::path::Path;

use anstream::{print, println};
use indexmap::IndexMap;
use oneil_runtime::output::{Independents, Value};

use crate::{print_utils, stylesheet};

pub struct IndependentPrintConfig {
    pub print_values: bool,
    pub recursive: bool,
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
pub fn print(top_model_path: &Path, independents: &Independents, config: &IndependentPrintConfig) {
    if config.recursive {
        for (model_path, independent_params) in independents.iter() {
            let model_path_display = model_path.display();
            let styled_model_name = stylesheet::MODEL_PATH_HEADER.style(model_path_display);
            println!("{styled_model_name}:");

            print_model_independents(independent_params, config);
        }
    } else {
        let independent_params = independents
            .get(top_model_path)
            .expect("top model path should be found");

        print_model_independents(independent_params, config);
    }
}

/// Recursively prints independent parameters for a model and its submodels.
fn print_model_independents(
    independent_params: &IndexMap<String, Value>,
    config: &IndependentPrintConfig,
) {
    for (name, value) in independent_params {
        print_parameter(name, value, config.print_values);
    }
}

/// Prints a single parameter.
fn print_parameter(name: &str, value: &Value, print_values: bool) {
    let styled_ident = stylesheet::PARAMETER_IDENTIFIER.style(name);

    if print_values {
        print!("{styled_ident} = ");
        print_utils::print_value(value);
    } else {
        print!("{styled_ident}");
    }

    println!();
}
