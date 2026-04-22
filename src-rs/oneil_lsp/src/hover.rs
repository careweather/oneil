//! Hover content for [`crate::symbol_lookup::SymbolAtPosition`].

#[cfg(feature = "python")]
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use oneil_runtime::{
    Runtime,
    output::{Value, ir},
};
use oneil_shared::{
    paths::{ModelPath, PythonPath},
    symbols::{BuiltinFunctionName, BuiltinValueName, PyFunctionName},
};
use tower_lsp_server::ls_types::{HoverContents, MarkedString};

use crate::{path::trim_path, symbol_lookup::SymbolAtPosition};

const PLAINTEXT_LANG_CODE: &str = "plaintext";

/// Markdown body for a hover, when available.
///
/// Paths shown in hovers are shortened by stripping a matching prefix from `workspace_roots`
/// (longest match first) when the filesystem path starts with that root.
pub fn hover_markdown(
    symbol: &SymbolAtPosition,
    runtime: &mut Runtime,
    current_model_path: &ModelPath,
    workspace_roots: &[PathBuf],
) -> Option<HoverContents> {
    match symbol {
        SymbolAtPosition::ParameterDefinition { name, .. }
        | SymbolAtPosition::ParameterReference { name, .. } => {
            let (model, _) = runtime.load_and_lower(current_model_path);
            let model = model?;
            let param = model.get_parameter(name)?;
            Some(format_parameter_hover(
                current_model_path,
                param,
                workspace_roots,
            ))
        }
        SymbolAtPosition::ExternalParameterReference {
            reference_name,
            parameter_name,
            ..
        } => {
            // Resolve the reference name to a model path via the current model's imports.
            let (current_model, _) = runtime.load_and_lower(current_model_path);
            let current_model = current_model?;
            let external_model_path = current_model
                .reference_imports()
                .get(reference_name)
                .map(|r| r.path.clone())
                .or_else(|| {
                    current_model
                        .submodel_imports()
                        .get(reference_name)
                        .map(|s| s.instance.path().clone())
                })?;
            let (model, _) = runtime.load_and_lower(&external_model_path);
            let model = model?;
            let param = model.get_parameter(parameter_name)?;
            Some(format_parameter_hover(
                &external_model_path,
                param,
                workspace_roots,
            ))
        }
        SymbolAtPosition::ModelImportDefinition { path, .. } => {
            format_model_hover_from_path(runtime, path, workspace_roots)
        }
        SymbolAtPosition::ModelImportReference { reference_name, .. } => {
            let imported_path = {
                let (model, _) = runtime.load_and_lower(current_model_path);
                let model = model?;
                model
                    .reference_imports()
                    .get(reference_name)
                    .map(|r| r.path.clone())
                    .or_else(|| {
                        model
                            .submodel_imports()
                            .get(reference_name)
                            .map(|s| s.instance.path().clone())
                    })?
            };

            format_model_hover_from_path(runtime, &imported_path, workspace_roots)
        }

        SymbolAtPosition::PythonImport { path, .. } => {
            Some(format_python_import_hover(runtime, path, workspace_roots))
        }

        SymbolAtPosition::PythonFunctionReference {
            python_path, name, ..
        } => Some(format_python_function_hover(
            runtime,
            python_path,
            name,
            workspace_roots,
        )),

        SymbolAtPosition::BuiltinValueReference { name, .. } => runtime
            .lookup_builtin_value_docs(name)
            .map(|(_, value)| format_builtin_value_hover(name, value)),

        SymbolAtPosition::BuiltinFunctionReference { name, .. } => runtime
            .lookup_builtin_function_docs(name)
            .map(|(args, doc)| format_builtin_function_hover(name, args, doc)),
    }
}

fn format_parameter_hover(
    model_path: &ModelPath,
    param: &ir::Parameter,
    workspace_roots: &[PathBuf],
) -> HoverContents {
    let path = path_as_marked_string(model_path.as_path(), workspace_roots);

    let name = param.name().as_str();
    let label = param.label().as_str();

    let name_and_label = format!("{label}: {name}");
    let name_and_label =
        MarkedString::from_language_code(PLAINTEXT_LANG_CODE.to_string(), name_and_label);

    if let Some(note) = param.note() {
        let note = note_as_marked_string(note);
        HoverContents::Array(vec![path, name_and_label, note])
    } else {
        HoverContents::Array(vec![path, name_and_label])
    }
}

fn format_model_hover_from_path(
    runtime: &mut Runtime,
    path: &ModelPath,
    workspace_roots: &[PathBuf],
) -> Option<HoverContents> {
    let (model, _) = runtime.load_and_lower(path);
    let model = model?;
    Some(format_model_hover(&model, workspace_roots))
}

fn format_model_hover(
    model: &oneil_runtime::output::reference::ModelTemplateReference<'_>,
    workspace_roots: &[PathBuf],
) -> HoverContents {
    let path = path_as_marked_string(model.path().as_path(), workspace_roots);

    if let Some(note) = model.note() {
        let note = note_as_marked_string(note);
        HoverContents::Array(vec![path, note])
    } else {
        HoverContents::Scalar(path)
    }
}

/// Markdown for a Python file import: filesystem path and callable names discovered in the module.
fn format_python_import_hover(
    runtime: &mut Runtime,
    path: &PythonPath,
    workspace_roots: &[PathBuf],
) -> HoverContents {
    #[cfg(not(feature = "python"))]
    let _ = runtime;

    let path_string = path_as_marked_string(path.as_path(), workspace_roots);

    #[cfg(feature = "python")]
    {
        let doc_string = runtime
            .lookup_python_import_docs(path)
            .map(|docs| MarkedString::from_markdown(docs.to_string()));

        let functions = runtime
            .load_python_import(path)
            .ok()
            .map(|module| module.get_function_names().collect::<Vec<_>>())
            .filter(|functions| !functions.is_empty())
            .map(|functions| {
                let mut function_list = "**Functions:**\n".to_string();
                for function in functions {
                    writeln!(function_list, "- `{}`", function.as_str())
                        .expect("writing to string should never fail");
                }

                MarkedString::from_markdown(function_list)
            });

        if doc_string.is_some() || functions.is_some() {
            return HoverContents::Array(
                std::iter::once(path_string)
                    .chain(doc_string)
                    .chain(functions)
                    .collect(),
            );
        }
    }

    HoverContents::Scalar(path_string)
}

/// Plaintext hover for a Python call: module file path and function name.
fn format_python_function_hover(
    runtime: &Runtime,
    python_path: &PythonPath,
    function_name: &PyFunctionName,
    workspace_roots: &[PathBuf],
) -> HoverContents {
    let function_docs = runtime
        .lookup_python_function(python_path, function_name)
        .and_then(|function| function.get_docs())
        .map(|docs| MarkedString::from_markdown(docs.to_string()));

    let path = path_as_marked_string(python_path.as_path(), workspace_roots);
    let name = MarkedString::from_language_code(
        PLAINTEXT_LANG_CODE.to_string(),
        function_name.as_str().to_string(),
    );

    if let Some(function_docs) = function_docs {
        HoverContents::Array(vec![path, name, function_docs])
    } else {
        HoverContents::Array(vec![path, name])
    }
}

const BUILTIN_PSEUDO_PATH: &str = "(builtin)";

fn format_builtin_value_hover(name: &BuiltinValueName, value: &Value) -> HoverContents {
    let name = name.as_str();
    let name_and_value = format!("{name} = {value}");
    let name_and_value =
        MarkedString::from_language_code(PLAINTEXT_LANG_CODE.to_string(), name_and_value);

    HoverContents::Scalar(name_and_value)
}

/// Signature as plaintext plus documentation as markdown.
fn format_builtin_function_hover(
    name: &BuiltinFunctionName,
    args: &[&str],
    documentation: &str,
) -> HoverContents {
    let sig = format!("{}({})", name.as_str(), args.join(", "));
    let sig = MarkedString::from_language_code(PLAINTEXT_LANG_CODE.to_string(), sig);
    let doc = text_to_markdown_string(documentation);

    HoverContents::Array(vec![sig, doc])
}

fn path_as_marked_string(path: &Path, workspace_roots: &[PathBuf]) -> MarkedString {
    let path_string = trim_path(path, workspace_roots)
        .unwrap_or(path)
        .display()
        .to_string();
    MarkedString::from_language_code(PLAINTEXT_LANG_CODE.to_string(), path_string)
}

fn note_as_marked_string(note: &ir::Note) -> MarkedString {
    text_to_markdown_string(note.content())
}

fn text_to_markdown_string(prose: &str) -> MarkedString {
    let prose = prose
        .lines()
        .map(str::trim)
        .collect::<Vec<_>>()
        .join("\n\n");

    MarkedString::from_markdown(prose)
}
