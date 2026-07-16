//! Display formatting for analysis results.

use std::{
    collections::HashMap,
    fmt::Write,
    path::{Path, PathBuf},
};

use oneil_output::{
    Value,
    util::{DEFAULT_SIG_FIGS, format_number_for_display, format_value_for_display},
};
use oneil_shared::{paths::ModelPath, span::Span};
use owo_colors::Style;

use crate::output::{DependencyName, DependencyTreeValue, ReferenceTreeValue, Tree};

const MODEL_PATH_STYLE: Style = Style::new().blue();
const VALUE_NAME_STYLE: Style = Style::new().green();
const EQUATION_STYLE: Style = Style::new().dimmed();
const PASS_STYLE: Style = Style::new().green().bold();
const FAIL_STYLE: Style = Style::new().red().bold();
const ERROR_STYLE: Style = Style::new().red();
const UNIT_STYLE: Style = Style::new().blue();

/// Configuration for formatting an analysis tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeDisplayConfig<'path> {
    /// Whether to descend into referenced models.
    pub recursive: bool,
    /// Maximum tree depth to display.
    pub depth: Option<usize>,
    /// Number of significant figures used for numeric values.
    pub sig_figs: usize,
    /// Whether to include ANSI color styling.
    pub color: bool,
    /// Prefix to remove from displayed model paths.
    pub path_prefix: Option<&'path Path>,
}

impl Default for TreeDisplayConfig<'_> {
    /// Creates a plain-text configuration using the default precision.
    fn default() -> Self {
        Self {
            recursive: false,
            depth: None,
            sig_figs: DEFAULT_SIG_FIGS,
            color: false,
            path_prefix: None,
        }
    }
}

/// Formats a reference tree showing which values reference a parameter.
#[must_use]
pub fn format_reference_tree(
    top_model_path: &ModelPath,
    reference_tree: &Tree<ReferenceTreeValue>,
    config: TreeDisplayConfig<'_>,
) -> String {
    format_tree(top_model_path, reference_tree, config)
}

/// Formats a dependency tree showing which values a parameter references.
#[must_use]
pub fn format_dependency_tree(
    top_model_path: &ModelPath,
    dependency_tree: &Tree<DependencyTreeValue>,
    config: TreeDisplayConfig<'_>,
) -> String {
    format_tree(top_model_path, dependency_tree, config)
}

/// Formats a tree using the shared traversal and display implementation.
fn format_tree<T: DisplayTreeValue>(
    top_model_path: &ModelPath,
    tree: &Tree<T>,
    config: TreeDisplayConfig<'_>,
) -> String {
    let context = TreeDisplayContext {
        current_depth: 0,
        is_first: true,
        top_model_path,
    };
    let mut output = String::new();
    let mut file_cache = HashMap::new();
    format_tree_node(
        &mut output,
        tree,
        config,
        &context,
        &mut Vec::new(),
        &mut file_cache,
    );
    output
}

/// State carried while recursively formatting a tree.
struct TreeDisplayContext<'path> {
    current_depth: usize,
    is_first: bool,
    top_model_path: &'path ModelPath,
}

/// Recursively formats a tree node with indentation and tree characters.
fn format_tree_node<T: DisplayTreeValue>(
    output: &mut String,
    tree: &Tree<T>,
    config: TreeDisplayConfig<'_>,
    context: &TreeDisplayContext<'_>,
    parent_prefixes: &mut Vec<bool>,
    file_cache: &mut HashMap<PathBuf, String>,
) {
    let value = tree.value();
    let (first_prefix, rest_prefix) = tree_prefixes(context.current_depth, context.is_first);
    let indent = build_indent(parent_prefixes);

    format_children(output, tree, config, context, parent_prefixes, file_cache);

    let value_name = value.value_name(config.path_prefix);
    let styled_value_name = value.display_name(config.path_prefix, config.color);
    let displayed_value = value.display_value(config.sig_figs, config.color);
    writeln!(
        output,
        "{indent}{first_prefix}{styled_value_name} = {displayed_value}"
    )
    .expect("writing to a String is infallible");

    if let Some(display_info) = value.display_info() {
        let equation_indent = " ".repeat(value_name.chars().count());
        match equation_str(display_info, file_cache) {
            Ok(equation) => {
                let equation = style(&format!(" = {equation}"), EQUATION_STYLE, config.color);
                writeln!(output, "{indent}{rest_prefix}{equation_indent}{equation}")
                    .expect("writing to a String is infallible");
            }
            Err(error) => {
                let error_label = style("error", ERROR_STYLE, config.color);
                writeln!(output, "{indent}{rest_prefix}{error_label}: {error}")
                    .expect("writing to a String is infallible");
            }
        }
    }
}

/// Formats child nodes before their parent.
fn format_children<T: DisplayTreeValue>(
    output: &mut String,
    tree: &Tree<T>,
    config: TreeDisplayConfig<'_>,
    context: &TreeDisplayContext<'_>,
    parent_prefixes: &mut Vec<bool>,
    file_cache: &mut HashMap<PathBuf, String>,
) {
    let children = tree.children();
    let reached_max_depth = config
        .depth
        .is_some_and(|max_depth| context.current_depth >= max_depth);
    let skip_children = reached_max_depth
        || (!config.recursive && tree.value().is_outside_top_model(context.top_model_path));

    if !children.is_empty() && skip_children {
        let (_, rest_prefix) = tree_prefixes(context.current_depth, context.is_first);
        let indent = build_indent(parent_prefixes);
        format_truncated_node(output, &indent, rest_prefix, context.is_first);
        return;
    }

    if children.is_empty() {
        return;
    }

    parent_prefixes.push(context.is_first);
    children.iter().enumerate().for_each(|(index, child)| {
        let child_context = TreeDisplayContext {
            current_depth: context.current_depth + 1,
            is_first: index == 0,
            top_model_path: context.top_model_path,
        };
        format_tree_node(
            output,
            child,
            config,
            &child_context,
            parent_prefixes,
            file_cache,
        );
    });
    parent_prefixes.pop();
}

/// Returns the branch prefixes for a node.
const fn tree_prefixes(depth: usize, is_first: bool) -> (&'static str, &'static str) {
    if depth == 0 {
        ("", "")
    } else if is_first {
        ("┌── ", "│   ")
    } else {
        ("├── ", "│   ")
    }
}

/// Builds indentation from the branch state of parent nodes.
fn build_indent(parent_prefixes: &[bool]) -> String {
    parent_prefixes
        .iter()
        .enumerate()
        .map(|(index, is_first)| {
            if index == 0 {
                ""
            } else if *is_first {
                "    "
            } else {
                "│   "
            }
        })
        .collect()
}

/// Extracts an equation from its source file.
fn equation_str(
    display_info: &(ModelPath, Span),
    file_cache: &mut HashMap<PathBuf, String>,
) -> Result<String, String> {
    let (model_path, span) = display_info;
    let file_path = model_path.as_path().to_path_buf();

    if !file_cache.contains_key(&file_path) {
        let contents = std::fs::read_to_string(&file_path)
            .map_err(|error| format!("couldn't read `{}` - {error}", file_path.display()))?;
        file_cache.insert(file_path.clone(), contents);
    }

    file_cache
        .get(&file_path)
        .expect("file should be cached after insertion")
        .get(span.start().offset..span.end().offset)
        .map(str::to_string)
        .ok_or_else(|| {
            format!(
                "couldn't get equation for parameter at line {}, column {}",
                span.start().line,
                span.start().column
            )
        })
}

/// Formats a marker indicating that child nodes were omitted.
fn format_truncated_node(output: &mut String, indent: &str, rest_prefix: &str, is_first: bool) {
    let prefix = if is_first { "    " } else { rest_prefix };
    writeln!(output, "{indent}{prefix}┌──╶╶╶").expect("writing to a String is infallible");
}

/// Applies a display style when color output is enabled.
fn style(value: &str, style: Style, color: bool) -> String {
    if color {
        style.style(value).to_string()
    } else {
        value.to_string()
    }
}

/// Behavior required to display a value in an analysis tree.
trait DisplayTreeValue {
    /// Returns the node name.
    fn value_name(&self, path_prefix: Option<&Path>) -> String;
    /// Returns the node name with optional display styling.
    fn display_name(&self, path_prefix: Option<&Path>, color: bool) -> String {
        style(&self.value_name(path_prefix), VALUE_NAME_STYLE, color)
    }
    /// Returns the node value.
    fn display_value(&self, sig_figs: usize, color: bool) -> String;
    /// Returns source information for the node's equation.
    fn display_info(&self) -> Option<&(ModelPath, Span)>;
    /// Returns whether child traversal crosses the top model boundary.
    fn is_outside_top_model(&self, top_model_path: &ModelPath) -> bool;
}

impl DisplayTreeValue for ReferenceTreeValue {
    /// Returns a path-qualified parameter or test name.
    fn value_name(&self, path_prefix: Option<&Path>) -> String {
        let (model_path, label) = match self {
            Self::Parameter {
                model_path,
                parameter_name,
                ..
            } => (model_path, parameter_name.as_str()),
            Self::Test { model_path, .. } => (model_path, "test"),
        };
        let path = normalize_path(model_path.as_path(), path_prefix);
        format!("{path} {label}")
    }

    /// Returns a path-qualified name with separate path and label styling.
    fn display_name(&self, path_prefix: Option<&Path>, color: bool) -> String {
        let (model_path, label) = match self {
            Self::Parameter {
                model_path,
                parameter_name,
                ..
            } => (model_path, parameter_name.as_str()),
            Self::Test { model_path, .. } => (model_path, "test"),
        };
        let path = normalize_path(model_path.as_path(), path_prefix);
        format!(
            "{} {}",
            style(&path, MODEL_PATH_STYLE, color),
            style(label, VALUE_NAME_STYLE, color)
        )
    }

    /// Returns the parameter value or test status.
    fn display_value(&self, sig_figs: usize, color: bool) -> String {
        match self {
            Self::Parameter {
                parameter_value, ..
            } => display_value(parameter_value, sig_figs, color),
            Self::Test { test_passed, .. } => {
                let (status, status_style) = if *test_passed {
                    ("PASS", PASS_STYLE)
                } else {
                    ("FAIL", FAIL_STYLE)
                };
                style(status, status_style, color)
            }
        }
    }

    /// Returns source information for the node.
    fn display_info(&self) -> Option<&(ModelPath, Span)> {
        match self {
            Self::Parameter { display_info, .. } | Self::Test { display_info, .. } => {
                Some(display_info)
            }
        }
    }

    /// Returns whether the node belongs to another model.
    fn is_outside_top_model(&self, top_model_path: &ModelPath) -> bool {
        let model_path = match self {
            Self::Parameter { model_path, .. } | Self::Test { model_path, .. } => model_path,
        };
        model_path != top_model_path
    }
}

impl DisplayTreeValue for DependencyTreeValue {
    /// Returns the dependency name.
    fn value_name(&self, _path_prefix: Option<&Path>) -> String {
        match &self.dependency_name {
            DependencyName::External(reference, parameter) => {
                format!("{}.{}", parameter.as_str(), reference.as_str())
            }
            DependencyName::Parameter(parameter) => parameter.as_str().to_string(),
            DependencyName::Builtin(builtin) => builtin.as_str().to_string(),
        }
    }

    /// Returns the dependency value.
    fn display_value(&self, sig_figs: usize, color: bool) -> String {
        display_value(&self.parameter_value, sig_figs, color)
    }

    /// Returns source information for the dependency.
    fn display_info(&self) -> Option<&(ModelPath, Span)> {
        self.display_info.as_ref()
    }

    /// Returns whether the dependency is external.
    fn is_outside_top_model(&self, _top_model_path: &ModelPath) -> bool {
        matches!(self.dependency_name, DependencyName::External(..))
    }
}

/// Formats a value with optional unit styling.
fn display_value(value: &Value, sig_figs: usize, color: bool) -> String {
    if !color {
        return format_value_for_display(value, sig_figs);
    }

    match value {
        Value::String(string) => format!("'{string}'"),
        Value::Boolean(boolean) => boolean.to_string(),
        Value::Number(number) => format_number_for_display(number, sig_figs),
        Value::MeasuredNumber(measured) => {
            let (number, unit) = measured.clone().into_number_and_unit();
            let number = format_number_for_display(&number, sig_figs);
            if unit.is_effectively_unitless() {
                number
            } else {
                format!(
                    "{number} :{}",
                    style(&unit.display_unit.to_string(), UNIT_STYLE, true)
                )
            }
        }
    }
}

/// Normalizes a model path for portable display.
fn normalize_path(path: &Path, prefix: Option<&Path>) -> String {
    prefix
        .and_then(|prefix| path.strip_prefix(prefix).ok())
        .unwrap_or(path)
        .display()
        .to_string()
}
