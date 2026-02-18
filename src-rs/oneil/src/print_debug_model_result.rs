//! Debug printing of evaluated model results for the Oneil CLI.
//!
//! Mirrors the process used for `print_ir`: collect errors from the evaluation result,
//! print them, then optionally display the model result tree (with optional recursion).

use anstream::println;
use indexmap::IndexMap;
use oneil_runtime::output::reference::ModelReference;

/// Configuration for debug-printing an evaluated model result.
pub struct DebugModelResultPrintConfig {
    /// When true, recurse into submodels and references when printing the tree.
    pub recursive: bool,
}

/// Prints the evaluated model result in a hierarchical tree format for debugging.
///
/// Collects all errors from the evaluation result by traversing the hierarchy. If there are
/// errors, they are printed. If there are no errors or `display_partial` is true, the
/// model result tree is displayed. When displaying, only the top-level model is shown
/// unless `recursive` is true.
pub fn print(eval_result: ModelReference<'_>, config: &DebugModelResultPrintConfig) {
    println!("ModelResult");
    println!("└── Models:");
    let prefix = "└──";
    print_model(eval_result, 2, prefix, config.recursive);
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
        let refs_to_print: Vec<ModelReference<'_>> = references.values().copied().collect();

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

fn print_parameters(parameters: &IndexMap<&str, &oneil_runtime::output::Parameter>, indent: usize) {
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

fn print_references(references: &IndexMap<&str, ModelReference<'_>>, indent: usize) {
    for (i, (name, result)) in references.iter().enumerate() {
        let is_last = i == references.len() - 1;
        let prefix = if is_last { "└──" } else { "├──" };
        let path_str = result.path().display().to_string();

        println!(
            "{}    {}Reference: \"{}\" -> \"{}\"",
            "  ".repeat(indent),
            prefix,
            name,
            path_str
        );
    }
}

fn print_tests(tests: &[&oneil_runtime::output::Test], indent: usize) {
    for (i, test) in tests.iter().enumerate() {
        let is_last = i == tests.len() - 1;
        let prefix = if is_last { "└──" } else { "├──" };
        let result_str = match &test.result {
            oneil_runtime::output::TestResult::Passed => "passed",
            oneil_runtime::output::TestResult::Failed { .. } => "failed",
        };
        println!("{}    {}Test: {}", "  ".repeat(indent), prefix, result_str);
    }
}
