//! Debug printing of evaluated model results for the Oneil CLI.
//!
//! Mirrors the process used for `print_ir`: collect errors from the evaluation result,
//! print them, then optionally display the model result tree (with optional recursion).

use anstream::println;
use indexmap::IndexMap;
use oneil_runtime::output::reference::ModelReference;

/// Which sections of the model result to include when printing.
#[derive(Clone, Debug)]
pub enum ModelResultSections {
    /// Show all sections.
    All,

    /// Show only the specified sections.
    Specified {
        submodels: bool,
        references: bool,
        parameters: bool,
        tests: bool,
    },
}

impl Default for ModelResultSections {
    fn default() -> Self {
        Self::All
    }
}

impl ModelResultSections {
    /// Returns whether to show the submodels section.
    #[must_use]
    pub const fn show_submodels(&self) -> bool {
        match self {
            Self::All => true,
            Self::Specified { submodels, .. } => *submodels,
        }
    }

    /// Returns whether to show the references section.
    #[must_use]
    pub const fn show_references(&self) -> bool {
        match self {
            Self::All => true,
            Self::Specified { references, .. } => *references,
        }
    }

    /// Returns whether to show the parameters section.
    #[must_use]
    pub const fn show_parameters(&self) -> bool {
        match self {
            Self::All => true,
            Self::Specified { parameters, .. } => *parameters,
        }
    }

    /// Returns whether to show the tests section.
    #[must_use]
    pub const fn show_tests(&self) -> bool {
        match self {
            Self::All => true,
            Self::Specified { tests, .. } => *tests,
        }
    }
}

/// Configuration for debug-printing an evaluated model result.
pub struct DebugModelResultPrintConfig {
    /// When true, recurse into submodels and references when printing the tree.
    pub recursive: bool,

    /// Which sections to include in the output.
    pub sections: ModelResultSections,
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
    print_model(eval_result, 1, prefix, config);
}

/// Prints a single evaluated model with its components.
///
/// When `recursive` is true, also prints nested references (and submodels as references).
fn print_model(
    model_ref: ModelReference<'_>,
    indent: usize,
    prefix: &str,
    config: &DebugModelResultPrintConfig,
) {
    println!(
        "{}    {} Model: \"{}\"",
        "    ".repeat(indent),
        prefix,
        model_ref.path().display()
    );

    let sections = &config.sections;
    let submodels = model_ref.submodels();
    let parameters = model_ref.parameters();
    let references = model_ref.references();
    let tests = model_ref.tests();

    let mut section_list: Vec<(&str, usize, SectionTag)> = Vec::new();

    if sections.show_submodels() && !submodels.is_empty() {
        section_list.push(("Submodels", submodels.len(), SectionTag::Submodels));
    }

    if sections.show_parameters() && !parameters.is_empty() {
        section_list.push(("Parameters", parameters.len(), SectionTag::Parameters));
    }

    if sections.show_references() && !references.is_empty() {
        section_list.push(("References", references.len(), SectionTag::References));
    }

    if sections.show_tests() && !tests.is_empty() {
        section_list.push(("Tests", tests.len(), SectionTag::Tests));
    }

    for (i, (section_name, count, tag)) in section_list.iter().enumerate() {
        let indent = indent + 1;
        let is_last = i == section_list.len() - 1;
        let section_prefix = if is_last { "└──" } else { "├──" };

        println!(
            "{}    {} {} ({}):",
            "    ".repeat(indent),
            section_prefix,
            section_name,
            count
        );

        match tag {
            SectionTag::Submodels => print_submodels(&submodels, indent + 1),
            SectionTag::Parameters => print_parameters(&parameters, indent + 1),
            SectionTag::References => print_references(&references, indent + 1),
            SectionTag::Tests => print_tests(&tests, indent + 1),
        }
    }

    if config.recursive {
        let refs_to_print: Vec<ModelReference<'_>> = references.values().copied().collect();

        for (i, nested_ref) in refs_to_print.iter().enumerate() {
            let is_last = i == refs_to_print.len() - 1;
            let nested_prefix = if is_last { "└──" } else { "├──" };
            print_model(*nested_ref, indent + 1, nested_prefix, config);
        }
    }
}

enum SectionTag {
    Submodels,
    Parameters,
    References,
    Tests,
}

fn print_submodels(submodels: &IndexMap<&str, &str>, indent: usize) {
    for (i, (name, reference_name)) in submodels.iter().enumerate() {
        let is_last = i == submodels.len() - 1;
        let prefix = if is_last { "└──" } else { "├──" };
        println!(
            "{}    {}Submodel: \"{}\" -> \"{}\"",
            "    ".repeat(indent),
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
            "    ".repeat(indent),
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
            "    ".repeat(indent),
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
        println!(
            "{}    {}Test: {}",
            "    ".repeat(indent),
            prefix,
            result_str
        );
    }
}
