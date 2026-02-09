//! Debug printing of evaluated model results for the Oneil CLI.
//!
//! Mirrors the process used for `print_ir`: collect errors from the evaluation result,
//! print them, then optionally display the model result tree (with optional recursion).

use anstream::println;
use indexmap::IndexMap;
use oneil_runtime::output::reference::{EvalErrorReference, ModelReference};

use crate::print_error;

/// Configuration for debug-printing an evaluated model result.
pub struct DebugModelResultPrintConfig {
    /// When true, show partial results even if there are errors.
    pub display_partial: bool,
    /// When true, recurse into submodels and references when printing the tree.
    pub recursive: bool,
}

/// Prints the evaluated model result in a hierarchical tree format for debugging.
///
/// Collects all errors from the evaluation result by traversing the hierarchy. If there are
/// errors, they are printed. If there are no errors or `display_partial` is true, the
/// model result tree is displayed. When displaying, only the top-level model is shown
/// unless `recursive` is true.
pub fn print(
    eval_result: Result<ModelReference<'_>, EvalErrorReference<'_>>,
    config: &DebugModelResultPrintConfig,
) {
    let errors = collect_errors(&eval_result);

    if !errors.is_empty() {
        for error in &errors {
            print_error::print(error, false);
        }
    }

    let should_display = errors.is_empty() || config.display_partial;
    if !should_display {
        return;
    }

    let model_ref = match &eval_result {
        Ok(r) => Some(*r),
        Err(e) => e.partial_result(),
    };

    if let Some(model_ref) = model_ref {
        println!("ModelResult");
        println!("└── Models:");
        let prefix = "└──";
        print_model(model_ref, 2, prefix, config.recursive);
    }
}

/// Collects all errors from the evaluation result by traversing the model hierarchy.
///
/// Uses a breadth-first search over references (each submodel is a reference).
fn collect_errors(
    eval_result: &Result<ModelReference<'_>, EvalErrorReference<'_>>,
) -> Vec<oneil_shared::error::OneilError> {
    let mut errors = Vec::new();

    let mut queue: Vec<Result<ModelReference<'_>, EvalErrorReference<'_>>> = match eval_result {
        Ok(r) => vec![Ok(*r)],
        Err(e) => {
            errors.extend(e.model_errors());
            if let Some(partial) = e.partial_result() {
                vec![Ok(partial)]
            } else {
                vec![]
            }
        }
    };

    while let Some(r) = queue.pop() {
        match r {
            Err(e) => {
                errors.extend(e.model_errors());
                if let Some(partial) = e.partial_result() {
                    queue.push(Ok(partial));
                }
            }
            Ok(model_ref) => {
                for nested in model_ref.references().values() {
                    queue.push(*nested);
                }
            }
        }
    }

    errors
}

/// Prints a single evaluated model with its components.
///
/// When `recursive` is true, also prints nested references (and submodels as references).
fn print_model(model_ref: ModelReference<'_>, indent: usize, prefix: &str, recursive: bool) {
    println!(
        "{}    {}Model: \"{}\"",
        "  ".repeat(indent),
        prefix,
        model_ref.path().display()
    );

    let indent = indent + 2;
    let submodels = model_ref.submodels();
    let parameters = model_ref.parameters();
    let references = model_ref.references();
    let tests = model_ref.tests();

    let mut sections: Vec<(&str, usize)> = Vec::new();
    if !submodels.is_empty() {
        sections.push(("Submodels", submodels.len()));
    }
    if !parameters.is_empty() {
        sections.push(("Parameters", parameters.len()));
    }
    if !references.is_empty() {
        sections.push(("References", references.len()));
    }
    if !tests.is_empty() {
        sections.push(("Tests", tests.len()));
    }

    for (i, (section_name, count)) in sections.iter().enumerate() {
        let is_last = i == sections.len() - 1;
        let section_prefix = if is_last { "└──" } else { "├──" };
        println!(
            "{}    {} {} ({}):",
            "  ".repeat(indent),
            section_prefix,
            section_name,
            count
        );

        match *section_name {
            "Submodels" => print_submodels(&submodels, indent + 2),
            "Parameters" => print_parameters(&parameters, indent + 2),
            "References" => print_references(&references, indent + 2),
            "Tests" => print_tests(&tests, indent + 2),
            _ => {}
        }
    }

    if recursive {
        let refs_to_print: Vec<ModelReference<'_>> = references
            .values()
            .filter_map(|r| match r {
                Ok(m) => Some(*m),
                Err(e) => e.partial_result(),
            })
            .collect();

        for (i, nested_ref) in refs_to_print.iter().enumerate() {
            let is_last = i == refs_to_print.len() - 1;
            let nested_prefix = if is_last { "└──" } else { "├──" };
            print_model(*nested_ref, indent + 2, nested_prefix, recursive);
        }
    }
}

fn print_submodels(submodels: &IndexMap<&str, &str>, indent: usize) {
    for (i, (name, reference_name)) in submodels.iter().enumerate() {
        let is_last = i == submodels.len() - 1;
        let prefix = if is_last { "└──" } else { "├──" };
        println!(
            "{}    {}Submodel: \"{}\" -> \"{}\"",
            "  ".repeat(indent),
            prefix,
            name,
            reference_name
        );
    }
}

fn print_parameters(
    parameters: &IndexMap<&str, &oneil_runtime::output::eval::Parameter>,
    indent: usize,
) {
    for (i, (name, param)) in parameters.iter().enumerate() {
        let is_last = i == parameters.len() - 1;
        let prefix = if is_last { "└──" } else { "├──" };
        println!(
            "{}    {}Parameter: \"{}\" = {:?}",
            "  ".repeat(indent),
            prefix,
            name,
            param.value
        );
    }
}

fn print_references(
    references: &IndexMap<&str, Result<ModelReference<'_>, EvalErrorReference<'_>>>,
    indent: usize,
) {
    for (i, (name, result)) in references.iter().enumerate() {
        let is_last = i == references.len() - 1;
        let prefix = if is_last { "└──" } else { "├──" };
        let path_str = match result {
            Ok(r) => r.path().display().to_string(),
            Err(e) => e
                .partial_result()
                .map(|r| r.path().display().to_string())
                .unwrap_or_else(|| "<error>".to_string()),
        };
        println!(
            "{}    {}Reference: \"{}\" -> \"{}\"",
            "  ".repeat(indent),
            prefix,
            name,
            path_str
        );
    }
}

fn print_tests(tests: &[&oneil_runtime::output::eval::Test], indent: usize) {
    for (i, test) in tests.iter().enumerate() {
        let is_last = i == tests.len() - 1;
        let prefix = if is_last { "└──" } else { "├──" };
        let result_str = match &test.result {
            oneil_runtime::output::eval::TestResult::Passed => "passed",
            oneil_runtime::output::eval::TestResult::Failed { .. } => "failed",
        };
        println!("{}    {}Test: {}", "  ".repeat(indent), prefix, result_str);
    }
}
