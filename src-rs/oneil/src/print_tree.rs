use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anstream::{print, println};
use oneil_eval::{
    output::{
        dependency::{DependencyTreeValue, RequiresTreeValue},
        tree::Tree,
    },
    value::Value,
};
use oneil_shared::span::Span;

use crate::{print_utils, stylesheet};

pub struct TreePrintConfig {
    pub recursive: bool,
    pub depth: Option<usize>,
}

/// Prints a requires tree showing which parameters require a given parameter.
pub fn print_requires_tree(
    top_model_path: &Path,
    requires_tree: &Tree<RequiresTreeValue>,
    tree_print_config: &TreePrintConfig,
    file_cache: &mut HashMap<PathBuf, String>,
) {
    print_tree_node(
        requires_tree,
        tree_print_config,
        0,
        true,
        top_model_path,
        &mut Vec::new(),
        file_cache,
    );
}

/// Prints a dependency tree showing which parameters are required by a given parameter.
pub fn print_dependency_tree(
    top_model_path: &Path,
    dependency_tree: &Tree<DependencyTreeValue>,
    tree_print_config: &TreePrintConfig,
    file_cache: &mut HashMap<PathBuf, String>,
) {
    print_tree_node(
        dependency_tree,
        tree_print_config,
        0,
        true,
        top_model_path,
        &mut Vec::new(),
        file_cache,
    );
}

/// Recursively prints a tree node with proper indentation and tree characters.
fn print_tree_node<T: PrintableTreeValue>(
    tree: &Tree<T>,
    config: &TreePrintConfig,
    current_depth: usize,
    is_last: bool,
    top_model_path: &Path,
    parent_prefixes: &mut Vec<bool>,
    file_cache: &mut HashMap<PathBuf, String>,
) {
    let value = tree.value();
    let children = tree.children();

    // Build the prefix for this node
    let (first_prefix, rest_prefix) = if current_depth == 0 {
        ("", "")
    } else if is_last {
        ("└── ", "    ")
    } else {
        ("├── ", "│   ")
    };
    let indent = build_indent(parent_prefixes);

    // Print the parameter name and value
    let value_name = value.get_styled_value_name();
    let styled_value_name = stylesheet::TREE_VALUE_NAME.style(&value_name);
    print!("{indent}{first_prefix}");
    print!("{styled_value_name} = ");
    print_utils::print_value(value.get_value());
    println!();

    // Print the parameter equation
    //
    // The goal is for this to be printed as
    //
    // ```
    // value_name = value
    //            = equation
    // ```
    let (maybe_bar, equation_indent) = if children.is_empty() {
        // replace the bar with a space if there are no children
        ("", " ".repeat(value.get_value_name_len()))
    } else {
        // otherwise, include the bar
        ("│", " ".repeat(value.get_value_name_len() - 1))
    }; // -1 to account for the bar

    if let Some(display_info) = value.get_display_info() {
        let equation_str = get_equation_str(display_info, file_cache);

        match equation_str {
            Ok(equation_str) => {
                let equation_str = format!(" = {equation_str}");
                let equation_str = stylesheet::TREE_VALUE_EQUATION.style(equation_str);
                println!("{indent}{rest_prefix}{maybe_bar}{equation_indent}{equation_str}");
            }
            Err(error) => {
                let error_label = stylesheet::ERROR_COLOR.style("error");
                println!("{indent}{rest_prefix}{error_label}: {error}");
            }
        }
    }

    // Check if we've exceeded the depth limit
    if let Some(max_depth) = config.depth
        && current_depth >= max_depth
    {
        return;
    }

    // Check if the parameter is outside the top model
    if !config.recursive && value.is_outside_top_model(top_model_path) {
        return;
    }

    // Print children
    if !children.is_empty() {
        parent_prefixes.push(is_last);

        for (i, child) in children.iter().enumerate() {
            let is_last_child = i == children.len() - 1;
            print_tree_node(
                child,
                config,
                current_depth + 1,
                is_last_child,
                top_model_path,
                parent_prefixes,
                file_cache,
            );
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

/// Gets the equation string from the source file using the display info.
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be read
/// - The span offsets are out of bounds for the file contents
fn get_equation_str(
    display_info: &(PathBuf, Span),
    file_cache: &mut HashMap<PathBuf, String>,
) -> Result<String, String> {
    let (file_path, span) = display_info;

    // Get file contents from cache or read from disk
    if !file_cache.contains_key(file_path) {
        let file_contents = std::fs::read_to_string(file_path)
            .map_err(|e| format!("couldn't read `{}` - {}", file_path.display(), e))?;
        file_cache.insert(file_path.clone(), file_contents);
    }

    let file_contents = file_cache
        .get(file_path)
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

trait PrintableTreeValue {
    /// Gets the name of the value, styled for display.
    fn get_styled_value_name(&self) -> String;
    /// Gets the length of the value name.
    ///
    /// This is necessary because the styled value name may include
    /// ANSI escape codes, which would affect the length of the string.
    fn get_value_name_len(&self) -> usize;
    /// Gets the value.
    fn get_value(&self) -> &Value;
    /// Gets the display information for the value, if available.
    ///
    /// This is used to get the equation string from the source file.
    fn get_display_info(&self) -> Option<&(PathBuf, Span)>;
    /// Checks if the value is outside the top model.
    ///
    /// This is used to determine whether to recursively print the
    /// children of the value.
    fn is_outside_top_model(&self, top_model_path: &Path) -> bool;
}

impl PrintableTreeValue for RequiresTreeValue {
    fn get_styled_value_name(&self) -> String {
        let model_path = self.model_path.display();
        let styled_model_path = stylesheet::MODEL_LABEL.style(model_path);

        let param = &self.parameter_name;
        let styled_param = stylesheet::PARAMETER_IDENTIFIER.style(param);

        format!("{styled_model_path} {styled_param}")
    }

    fn get_value_name_len(&self) -> usize {
        let model_path_len = self.model_path.as_os_str().len();
        let param_name_len = self.parameter_name.len();
        model_path_len + 1 + param_name_len // +1 for the space
    }

    fn get_value(&self) -> &Value {
        &self.parameter_value
    }

    fn get_display_info(&self) -> Option<&(PathBuf, Span)> {
        Some(&self.display_info)
    }

    fn is_outside_top_model(&self, top_model_path: &Path) -> bool {
        self.model_path != top_model_path
    }
}

impl PrintableTreeValue for DependencyTreeValue {
    fn get_styled_value_name(&self) -> String {
        let value_name = self.reference_name.as_ref().map_or_else(
            || self.parameter_name.clone(),
            |reference_name| format!("{}.{reference_name}", &self.parameter_name),
        );
        let styled_value_name = stylesheet::PARAMETER_IDENTIFIER.style(&value_name);
        format!("{styled_value_name}")
    }

    fn get_value_name_len(&self) -> usize {
        self.reference_name.as_ref().map_or_else(
            || self.parameter_name.len(),
            |reference_name| self.parameter_name.len() + 1 + reference_name.len(), // +1 for the dot
        )
    }

    fn get_value(&self) -> &Value {
        &self.parameter_value
    }

    fn get_display_info(&self) -> Option<&(PathBuf, Span)> {
        self.display_info.as_ref()
    }

    fn is_outside_top_model(&self, _top_model_path: &Path) -> bool {
        self.reference_name.is_some()
    }
}
