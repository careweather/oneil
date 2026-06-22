use std::{collections::HashMap, path::PathBuf};

use anstream::{print, println};
use oneil_runtime::output::{
    Value,
    tree::{DependencyName, DependencyTreeValue, ReferenceTreeValue, Tree},
};
use oneil_shared::{paths::ModelPath, span::Span};

use crate::{
    print_utils::{self, PrintUtilsConfig},
    stylesheet,
};

pub struct TreePrintConfig {
    pub recursive: bool,
    pub depth: Option<usize>,
    pub print_utils_config: PrintUtilsConfig,
}

pub struct TreePrintContext<'input> {
    pub current_depth: usize,
    pub is_first: bool,
    pub top_model_path: &'input ModelPath,
}

/// Prints a reference tree showing which parameters reference a given parameter.
pub fn print_reference_tree(
    top_model_path: &ModelPath,
    reference_tree: &Tree<ReferenceTreeValue>,
    tree_print_config: &TreePrintConfig,
    file_cache: &mut HashMap<PathBuf, String>,
) {
    let start_context = TreePrintContext {
        current_depth: 0,
        is_first: true,
        top_model_path,
    };

    print_tree_node(
        reference_tree,
        tree_print_config,
        &start_context,
        &mut Vec::new(),
        file_cache,
    );
}

/// Prints a dependency tree showing which parameters are referenced by a given parameter.
pub fn print_dependency_tree(
    top_model_path: &ModelPath,
    dependency_tree: &Tree<DependencyTreeValue>,
    tree_print_config: &TreePrintConfig,
    file_cache: &mut HashMap<PathBuf, String>,
) {
    let start_context = TreePrintContext {
        current_depth: 0,
        is_first: true,
        top_model_path,
    };

    print_tree_node(
        dependency_tree,
        tree_print_config,
        &start_context,
        &mut Vec::new(),
        file_cache,
    );
}

/// Recursively prints a tree node with proper indentation and tree characters.
fn print_tree_node<T: PrintableTreeValue>(
    tree: &Tree<T>,
    config: &TreePrintConfig,
    context: &TreePrintContext<'_>,
    parent_prefixes: &mut Vec<bool>,
    file_cache: &mut HashMap<PathBuf, String>,
) {
    let value = tree.value();

    // Build the prefix for this node
    let (first_prefix, rest_prefix) = if context.current_depth == 0 {
        ("", "")
    } else if context.is_first {
        ("┌── ", "│   ")
    } else {
        ("├── ", "│   ")
    };
    let indent = build_indent(parent_prefixes);

    print_children(
        tree,
        config,
        context,
        parent_prefixes,
        file_cache,
        rest_prefix,
        &indent,
    );

    // Print the parameter name and value
    let value_name = value.get_styled_value_name();
    let styled_value_name = stylesheet::TREE_VALUE_NAME.style(&value_name);
    print!("{indent}{first_prefix}");
    print!("{styled_value_name} = ");
    print_tree_value(&value.get_value(), config.print_utils_config);
    println!();

    // Print the parameter/test equation
    //
    // The goal is for this to be printed as
    //
    // ```
    // value_name = value
    //            = equation
    // ```
    if let Some(display_info) = value.get_display_info() {
        let equation_indent = " ".repeat(value.get_value_name_len());

        let equation_str = get_equation_str(display_info, file_cache);

        match equation_str {
            Ok(equation_str) => {
                let equation_str = format!(" = {equation_str}");
                let equation_str = stylesheet::TREE_VALUE_EQUATION.style(equation_str);
                println!("{indent}{rest_prefix}{equation_indent}{equation_str}");
            }
            Err(error) => {
                let error_label = stylesheet::ERROR_COLOR.style("error");
                println!("{indent}{rest_prefix}{error_label}: {error}");
            }
        }
    }
}

fn print_children<T: PrintableTreeValue>(
    tree: &Tree<T>,
    config: &TreePrintConfig,
    context: &TreePrintContext<'_>,
    parent_prefixes: &mut Vec<bool>,
    file_cache: &mut HashMap<PathBuf, String>,
    rest_prefix: &str,
    indent: &str,
) {
    let value = tree.value();
    let children = tree.children();

    // Check if we've reached the maximum depth
    let has_reached_max_depth = config
        .depth
        .is_some_and(|max_depth| context.current_depth >= max_depth);

    // Print the children first if they should be printed
    let has_children = !children.is_empty();
    let skip_printing_children = has_reached_max_depth
        || (!config.recursive && value.is_outside_top_model(context.top_model_path));

    if has_children && skip_printing_children {
        print_truncated_node(indent, rest_prefix, context.is_first);
    } else if has_children {
        parent_prefixes.push(context.is_first);

        for (i, child) in children.iter().enumerate() {
            let is_first_child = i == 0;
            let child_context = TreePrintContext {
                current_depth: context.current_depth + 1,
                is_first: is_first_child,
                top_model_path: context.top_model_path,
            };

            print_tree_node(child, config, &child_context, parent_prefixes, file_cache);
        }

        parent_prefixes.pop();
    }
}

/// Builds the indentation string based on parent prefixes.
fn build_indent(parent_prefixes: &[bool]) -> String {
    parent_prefixes
        .iter()
        .enumerate()
        .map(|(i, is_last)| {
            if i == 0 {
                ""
            } else if *is_last {
                "    "
            } else {
                "│   "
            }
        })
        .collect()
}

fn print_tree_value(value: &TreeValue<'_>, config: PrintUtilsConfig) {
    match value {
        TreeValue::Parameter { value } => print_utils::print_value(value, config),
        TreeValue::Test { passed } => {
            let styled_passed = if *passed {
                stylesheet::TESTS_PASS_COLOR.style("PASS")
            } else {
                stylesheet::TESTS_FAIL_COLOR.style("FAIL")
            };
            print!("{styled_passed}");
        }
    }
}

/// Gets the equation string from the source file using the display info.
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be read
/// - The span offsets are out of bounds for the file contents
fn get_equation_str(
    display_info: &(ModelPath, Span),
    file_cache: &mut HashMap<PathBuf, String>,
) -> Result<String, String> {
    let (model_path, span) = display_info;
    let file_path = model_path.as_path().to_path_buf();

    // Get file contents from cache or read from disk
    if !file_cache.contains_key(&file_path) {
        let file_contents = std::fs::read_to_string(model_path.as_path())
            .map_err(|e| format!("couldn't read `{}` - {}", model_path.as_path().display(), e))?;
        file_cache.insert(file_path.clone(), file_contents);
    }

    let file_contents = file_cache
        .get(&file_path)
        .expect("file should be in cache after insertion");

    // Extract the equation string using the span offsets
    let start_offset = span.start().offset;
    let end_offset = span.end().offset;

    file_contents
        .get(start_offset..end_offset)
        .map(str::to_string)
        .ok_or_else(|| {
            format!(
                "couldn't get equation for parameter at line {}, column {}",
                span.start().line,
                span.start().column
            )
        })
}

fn print_truncated_node(indent: &str, rest_prefix: &str, is_first: bool) {
    if is_first {
        println!("{indent}    ┌──╶╶╶");
    } else {
        println!("{indent}{rest_prefix}┌──╶╶╶");
    }
}

trait PrintableTreeValue {
    /// Gets the name of the value, styled for display.
    fn get_styled_value_name(&self) -> String;
    /// Gets the length of the value name.
    ///
    /// This is necessary because the styled value name may include
    /// ANSI escape codes, which would affect the length of the string.
    fn get_value_name_len(&self) -> usize;
    /// Gets the value of the parameter or .
    fn get_value(&self) -> TreeValue<'_>;
    /// Gets the display information for the value, if available.
    ///
    /// This is used to get the equation string from the source file.
    fn get_display_info(&self) -> Option<&(ModelPath, Span)>;
    /// Checks if the value is outside the top model.
    ///
    /// This is used to determine whether to recursively print the
    /// children of the value.
    fn is_outside_top_model(&self, top_model_path: &ModelPath) -> bool;
}

enum TreeValue<'tree> {
    Parameter { value: &'tree Value },
    Test { passed: bool },
}

impl PrintableTreeValue for ReferenceTreeValue {
    fn get_styled_value_name(&self) -> String {
        match self {
            Self::Parameter {
                model_path,
                parameter_name,
                ..
            } => {
                let model_path = model_path.as_path().display().to_string();
                let styled_model_path = stylesheet::MODEL_LABEL.style(model_path);
                let param = parameter_name.as_str();
                let styled_param = stylesheet::PARAMETER_IDENTIFIER.style(param);
                format!("{styled_model_path} {styled_param}")
            }
            Self::Test { model_path, .. } => {
                let model_path = model_path.as_path().display().to_string();
                let styled_model_path = stylesheet::MODEL_LABEL.style(model_path);
                let label = "test";
                let styled_test = stylesheet::PARAMETER_IDENTIFIER.style(label);
                format!("{styled_model_path} {styled_test}")
            }
        }
    }

    fn get_value_name_len(&self) -> usize {
        match self {
            Self::Parameter {
                model_path,
                parameter_name,
                ..
            } => {
                let model_path_len = model_path.as_path().as_os_str().len();
                let param_name_len = parameter_name.as_str().len();
                model_path_len + 1 + param_name_len
            }
            Self::Test { model_path, .. } => {
                let model_path_len = model_path.as_path().as_os_str().len();
                let test_label_len = "test".len();
                model_path_len + 1 + test_label_len
            }
        }
    }

    fn get_value(&self) -> TreeValue<'_> {
        match self {
            Self::Parameter {
                parameter_value, ..
            } => TreeValue::Parameter {
                value: parameter_value,
            },
            Self::Test { test_passed, .. } => TreeValue::Test {
                passed: *test_passed,
            },
        }
    }

    fn get_display_info(&self) -> Option<&(ModelPath, Span)> {
        match self {
            Self::Parameter { display_info, .. } | Self::Test { display_info, .. } => {
                Some(display_info)
            }
        }
    }

    fn is_outside_top_model(&self, top_model_path: &ModelPath) -> bool {
        let model_path = match self {
            Self::Parameter { model_path, .. } | Self::Test { model_path, .. } => model_path,
        };
        *model_path != *top_model_path
    }
}

impl PrintableTreeValue for DependencyTreeValue {
    fn get_styled_value_name(&self) -> String {
        let value_name = match &self.dependency_name {
            DependencyName::External(ref_name, param_name) => {
                format!("{}.{}", param_name.as_str(), ref_name.as_str())
            }
            DependencyName::Parameter(name) => name.as_str().to_string(),
            DependencyName::Builtin(name) => name.as_str().to_string(),
        };
        let styled_value_name = stylesheet::PARAMETER_IDENTIFIER.style(&value_name);
        format!("{styled_value_name}")
    }

    fn get_value_name_len(&self) -> usize {
        match &self.dependency_name {
            DependencyName::External(ref_name, param_name) => {
                param_name.as_str().len() + 1 + ref_name.as_str().len()
            }
            DependencyName::Parameter(name) => name.as_str().len(),
            DependencyName::Builtin(name) => name.as_str().len(),
        }
    }

    fn get_value(&self) -> TreeValue<'_> {
        TreeValue::Parameter {
            value: &self.parameter_value,
        }
    }

    fn get_display_info(&self) -> Option<&(ModelPath, Span)> {
        self.display_info.as_ref()
    }

    fn is_outside_top_model(&self, _top_model_path: &ModelPath) -> bool {
        matches!(self.dependency_name, DependencyName::External(..))
    }
}
