use std::{
    collections::HashMap,
    fmt::Write,
    path::{Path, PathBuf},
};

use oneil_output::util::{DEFAULT_SIG_FIGS, format_value_for_display};
use oneil_runtime::{
    CacheReadPolicy, CacheWritePolicy, Runtime,
    output::{
        self, Independents, OneilDiagnostic, Value,
        tree::{DependencyName, DependencyTreeValue, ReferenceTreeValue, Tree},
    },
};
use oneil_shared::{paths::ModelPath, span::Span, symbols::ParameterName};

/// Runs the full evaluation pipeline on an Oneil model or design file and
/// returns a formatted string containing any errors and the evaluation output.
///
/// The output format is deterministic and suitable for snapshot testing:
/// errors are listed first (if any), then a separator, then the model
/// output (tests and parameters).
///
/// If the path is a `.one` design file that declares `design <target>`, the
/// target model is evaluated with the design applied. Otherwise the model
/// at the given path is evaluated directly.
///
/// Paths in the output are normalized by stripping `path_prefix` when present,
/// so that snapshots are portable (e.g. use `CARGO_MANIFEST_DIR` as the prefix).
///
/// # Errors
///
/// This function does not return a `Result`; parse, resolution, and
/// evaluation errors are included in the returned string.
#[expect(clippy::unwrap_used, reason = "writing to a String is infallible")]
#[must_use]
pub fn run_model_and_format(path: &Path, path_prefix: Option<&Path>) -> String {
    let path = ModelPath::from_path_with_ext(path);

    let mut runtime = Runtime::new(CacheReadPolicy::Never, CacheWritePolicy::Never);
    let (model_opt, errors) = runtime.eval_model(&path);

    let mut out = String::new();

    let errors_str = format_errors(errors.to_vec(), path_prefix);
    if !errors_str.is_empty() {
        writeln!(out, "{errors_str}").unwrap();
    }

    if let Some(model_ref) = model_opt {
        let model_str = format_model(model_ref, path_prefix);
        if !out.is_empty() {
            writeln!(out, "---\n").unwrap();
        }
        write!(out, "{model_str}").unwrap();
    }

    if out.is_empty() {
        write!(out, "(no output)").unwrap();
    }

    out
}

/// Returns a path string normalized for snapshots: if it starts with `prefix`, strip it.
fn normalize_path(path: &Path, prefix: Option<&Path>) -> String {
    let path_str = path.display().to_string();

    let prefix = match prefix {
        Some(p) => p.display().to_string(),
        None => return path_str,
    };

    if path_str.starts_with(&prefix) {
        path_str[prefix.len()..]
            .trim_start_matches(std::path::MAIN_SEPARATOR)
            .to_string()
    } else {
        path_str
    }
}

/// Formats a collection of Oneil errors into a canonical string for snapshots.
fn format_errors(errors: Vec<&OneilDiagnostic>, path_prefix: Option<&Path>) -> String {
    errors
        .into_iter()
        .filter(|e| !e.is_internal_diagnostic())
        .map(|e| format_error(e, path_prefix))
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Formats a single error as a stable, readable string.
#[expect(clippy::unwrap_used, reason = "writing to a String is infallible")]
fn format_error(error: &OneilDiagnostic, path_prefix: Option<&Path>) -> String {
    let path_str = normalize_path(error.path(), path_prefix);

    let loc = error
        .location()
        .map(|l| format!("{}:{}", l.line(), l.column()));

    let at = loc
        .as_deref()
        .map_or_else(|| path_str.clone(), |loc| format!("{path_str}:{loc}"));

    let message = normalize_message(error.message(), path_prefix);
    let mut out = format!("error: {message}\n  at {at}");

    for ctx in error.context() {
        let (kind, text) = match ctx {
            oneil_shared::error::Context::Note(msg) => {
                ("note", normalize_message(msg, path_prefix))
            }
            oneil_shared::error::Context::Help(msg) => {
                ("help", normalize_message(msg, path_prefix))
            }
        };
        write!(out, "\n  {kind}: {text}").unwrap();
    }

    out
}

/// Strips occurrences of `prefix` from anywhere in `message`, so
/// diagnostic strings that embed absolute paths (e.g. cycle chains
/// of compilation units) render portably across machines.
fn normalize_message(message: &str, prefix: Option<&Path>) -> String {
    let Some(prefix) = prefix else {
        return message.to_string();
    };
    let mut prefix_str = prefix.display().to_string();
    if !prefix_str.ends_with(std::path::MAIN_SEPARATOR) {
        prefix_str.push(std::path::MAIN_SEPARATOR);
    }
    message.replace(&prefix_str, "")
}

/// Formats an evaluated model's output (tests and parameters) for snapshots.
#[expect(clippy::unwrap_used, reason = "writing to a String is infallible")]
fn format_model(
    model_ref: output::reference::ModelReference<'_>,
    path_prefix: Option<&Path>,
) -> String {
    let mut out = String::new();

    let path = normalize_path(model_ref.path().as_path(), path_prefix);
    let tests = model_ref.tests();
    let passed = tests.iter().filter(|(_, test)| test.passed()).count();
    let total = tests.len();

    writeln!(out, "Model: {path}").unwrap();
    writeln!(out, "Tests: {passed}/{total}").unwrap();

    for (index, test) in tests {
        let result_str = if test.passed() { "PASS" } else { "FAIL" };
        writeln!(out, "  test {}: {result_str}", index.into_usize() + 1).unwrap();
    }

    let params = model_ref.parameters();
    if !params.is_empty() {
        out.push_str("Parameters:\n");
        for (name, param) in params {
            let prefix = match param.print_level {
                output::PrintLevel::Performance => "$ ",
                output::PrintLevel::Trace => "* ",
                output::PrintLevel::None => "",
            };
            let value_str = format_value_for_display(&param.value, DEFAULT_SIG_FIGS);
            writeln!(out, "  {prefix}{name} = {value_str}").unwrap();
        }
    }

    out
}

/// Runs dependency-tree analysis for `parameter` on the model at `path` and
/// formats the result for snapshot testing (plain text mirroring CLI topology).
#[must_use]
pub fn run_dependency_tree_and_format(
    path: &Path,
    parameter: &str,
    path_prefix: Option<&Path>,
) -> String {
    let model_path = ModelPath::from_path_with_ext(path);
    let param = ParameterName::from(parameter);
    let mut runtime = Runtime::new(CacheReadPolicy::Never, CacheWritePolicy::Never);
    let (tree, errors) = runtime.get_dependency_tree(&model_path, &param);
    format_tree_analysis_result(
        &format!("dependency tree for `{parameter}`"),
        tree.as_ref().map(format_dependency_tree),
        errors.to_vec(),
        path_prefix,
    )
}

/// Runs reference-tree analysis for `parameter` on the model at `path` and
/// formats the result for snapshot testing.
#[must_use]
pub fn run_reference_tree_and_format(
    path: &Path,
    parameter: &str,
    path_prefix: Option<&Path>,
) -> String {
    let model_path = ModelPath::from_path_with_ext(path);
    let param = ParameterName::from(parameter);
    let mut runtime = Runtime::new(CacheReadPolicy::Never, CacheWritePolicy::Never);
    let (tree, errors) = runtime.get_reference_tree(&model_path, &param);
    format_tree_analysis_result(
        &format!("reference tree for `{parameter}`"),
        tree.as_ref().map(|t| format_reference_tree(t, path_prefix)),
        errors.to_vec(),
        path_prefix,
    )
}

/// Runs independents analysis on the model at `path` and formats the result.
///
/// When `recursive` is true, every referenced model’s independents are
/// included (mirroring `oneil independents --recursive`).
#[must_use]
pub fn run_independents_and_format(
    path: &Path,
    recursive: bool,
    path_prefix: Option<&Path>,
) -> String {
    let model_path = ModelPath::from_path_with_ext(path);
    let mut runtime = Runtime::new(CacheReadPolicy::Never, CacheWritePolicy::Never);
    let (independents, errors) = runtime.get_independents(&model_path);
    let mut out = String::new();

    let errors_str = format_errors(errors.to_vec(), path_prefix);
    if !errors_str.is_empty() {
        writeln!(out, "{errors_str}").expect("String write is infallible");
        writeln!(out, "---").expect("String write is infallible");
    }

    write!(
        out,
        "{}",
        format_independents(&model_path, &independents, recursive, path_prefix)
    )
    .expect("String write is infallible");

    if out.is_empty() {
        write!(out, "(no output)").expect("String write is infallible");
    }
    out
}

fn format_tree_analysis_result(
    header: &str,
    tree_body: Option<String>,
    errors: Vec<&OneilDiagnostic>,
    path_prefix: Option<&Path>,
) -> String {
    let mut out = String::new();
    let errors_str = format_errors(errors, path_prefix);
    if !errors_str.is_empty() {
        writeln!(out, "{errors_str}").expect("String write is infallible");
        writeln!(out, "---").expect("String write is infallible");
    }
    writeln!(out, "{header}").expect("String write is infallible");
    match tree_body {
        Some(body) => write!(out, "{body}").expect("String write is infallible"),
        None => writeln!(out, "(parameter not found)").expect("String write is infallible"),
    }
    out
}

fn format_independents(
    top_model_path: &ModelPath,
    independents: &Independents,
    recursive: bool,
    path_prefix: Option<&Path>,
) -> String {
    let mut out = String::new();
    if recursive {
        for (model_path, params) in independents.iter() {
            let path = normalize_path(model_path.as_path(), path_prefix);
            writeln!(out, "{path}:").expect("String write is infallible");
            format_independent_params(&mut out, params.iter());
        }
    } else if let Some(params) = independents.get(top_model_path) {
        format_independent_params(&mut out, params.iter());
    } else {
        writeln!(out, "(no independents for top model)").expect("String write is infallible");
    }
    out
}

fn format_independent_params<'a>(
    out: &mut String,
    params: impl Iterator<Item = (&'a ParameterName, &'a Value)>,
) {
    for (name, value) in params {
        let value_str = format_value_for_display(value, DEFAULT_SIG_FIGS);
        writeln!(out, "{} = {value_str}", name.as_str()).expect("String write is infallible");
    }
}

fn format_dependency_tree(tree: &Tree<DependencyTreeValue>) -> String {
    let mut out = String::new();
    let mut file_cache = HashMap::new();
    format_dependency_node(&mut out, tree, &[], true, 0, &mut file_cache);
    out
}

fn format_reference_tree(tree: &Tree<ReferenceTreeValue>, path_prefix: Option<&Path>) -> String {
    let mut out = String::new();
    let mut file_cache = HashMap::new();
    format_reference_node(&mut out, tree, &[], true, 0, path_prefix, &mut file_cache);
    out
}

/// Formats a dependency-tree node in CLI topology order (children first).
fn format_dependency_node(
    out: &mut String,
    tree: &Tree<DependencyTreeValue>,
    parent_prefixes: &[bool],
    is_first: bool,
    depth: usize,
    file_cache: &mut HashMap<PathBuf, String>,
) {
    let mut child_prefixes = parent_prefixes.to_vec();
    if depth > 0 {
        child_prefixes.push(is_first);
    }
    for (i, child) in tree.children().iter().enumerate() {
        format_dependency_node(out, child, &child_prefixes, i == 0, depth + 1, file_cache);
    }

    let (first_prefix, rest_prefix) = tree_prefixes(depth, is_first);
    let indent = build_indent(parent_prefixes);
    let name = dependency_name_label(&tree.value().dependency_name);
    let value_str = format_value_for_display(&tree.value().parameter_value, DEFAULT_SIG_FIGS);
    writeln!(out, "{indent}{first_prefix}{name} = {value_str}")
        .expect("String write is infallible");

    if let Some(display_info) = tree.value().display_info.as_ref()
        && let Some(equation) = equation_str(display_info, file_cache)
    {
        let pad = " ".repeat(name.len());
        writeln!(out, "{indent}{rest_prefix}{pad} = {equation}")
            .expect("String write is infallible");
    }
}

fn format_reference_node(
    out: &mut String,
    tree: &Tree<ReferenceTreeValue>,
    parent_prefixes: &[bool],
    is_first: bool,
    depth: usize,
    path_prefix: Option<&Path>,
    file_cache: &mut HashMap<PathBuf, String>,
) {
    let mut child_prefixes = parent_prefixes.to_vec();
    if depth > 0 {
        child_prefixes.push(is_first);
    }
    for (i, child) in tree.children().iter().enumerate() {
        format_reference_node(
            out,
            child,
            &child_prefixes,
            i == 0,
            depth + 1,
            path_prefix,
            file_cache,
        );
    }

    let (first_prefix, rest_prefix) = tree_prefixes(depth, is_first);
    let indent = build_indent(parent_prefixes);
    let (name, value_str, display_info) = match tree.value() {
        ReferenceTreeValue::Parameter {
            model_path,
            parameter_name,
            parameter_value,
            display_info,
        } => {
            let path = normalize_path(model_path.as_path(), path_prefix);
            (
                format!("{path} {}", parameter_name.as_str()),
                format_value_for_display(parameter_value, DEFAULT_SIG_FIGS),
                display_info,
            )
        }
        ReferenceTreeValue::Test {
            model_path,
            test_passed,
            display_info,
            ..
        } => {
            let path = normalize_path(model_path.as_path(), path_prefix);
            let status = if *test_passed { "PASS" } else { "FAIL" };
            (format!("{path} test"), status.to_string(), display_info)
        }
    };
    writeln!(out, "{indent}{first_prefix}{name} = {value_str}")
        .expect("String write is infallible");

    if let Some(equation) = equation_str(display_info, file_cache) {
        let pad = " ".repeat(name.len());
        writeln!(out, "{indent}{rest_prefix}{pad} = {equation}")
            .expect("String write is infallible");
    }
}

fn tree_prefixes(depth: usize, is_first: bool) -> (&'static str, &'static str) {
    if depth == 0 {
        ("", "")
    } else if is_first {
        ("┌── ", "│   ")
    } else {
        ("├── ", "│   ")
    }
}

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

fn dependency_name_label(name: &DependencyName) -> String {
    match name {
        DependencyName::External(reference, parameter) => {
            format!("{}.{}", parameter.as_str(), reference.as_str())
        }
        DependencyName::Parameter(parameter) => parameter.as_str().to_string(),
        DependencyName::Builtin(builtin) => builtin.as_str().to_string(),
    }
}

fn equation_str(
    display_info: &(ModelPath, Span),
    file_cache: &mut HashMap<PathBuf, String>,
) -> Option<String> {
    let (model_path, span) = display_info;
    let file_path = model_path.as_path().to_path_buf();
    if !file_cache.contains_key(&file_path) {
        let Ok(contents) = std::fs::read_to_string(&file_path) else {
            return None;
        };
        file_cache.insert(file_path.clone(), contents);
    }
    let contents = file_cache.get(&file_path)?;
    contents
        .get(span.start().offset..span.end().offset)
        .map(str::to_string)
}
