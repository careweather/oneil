//! Symbol lookup utilities for finding definitions in Oneil models

use oneil_ir::{self as ir, ModelCollection, ModelPath, ParameterName};
use oneil_shared::span::Span;
use tower_lsp_server::lsp_types::{Location, Position, Range, Uri};
use tower_lsp_server::UriExt;

/// Represents a symbol found at a cursor position
#[derive(Debug, Clone)]
pub enum SymbolAtPosition {
    /// A parameter definition (cursor is on the parameter name in its declaration)
    ParameterDefinition {
        name: ParameterName,
        span: Span,
    },
    /// A parameter reference (cursor is on a parameter used in an expression)
    ParameterReference {
        name: ParameterName,
        span: Span,
    },
    /// An external variable reference (e.g., `x.model_name`)
    ExternalReference {
        model_name: String,
        model_span: Span,
        parameter_name: ParameterName,
        parameter_span: Span,
    },
    /// A submodel or reference import name
    ModelImport { name: String, span: Span, path: ModelPath },
}

/// Finds the symbol at a given byte offset in a model
pub fn find_symbol_at_offset(
    model: &ir::Model,
    _model_path: &ModelPath,
    offset: usize,
) -> Option<SymbolAtPosition> {
    // Check if cursor is on a parameter definition
    for (param_name, param) in model.get_parameters() {
        if span_contains_offset(param.name_span(), offset) {
            return Some(SymbolAtPosition::ParameterDefinition {
                name: param_name.clone(),
                span: param.name_span(),
            });
        }
    }

    // Check if cursor is on a submodel import name
    for (submodel_name, submodel_import) in model.get_submodels() {
        if span_contains_offset(*submodel_import.name_span(), offset) {
            return Some(SymbolAtPosition::ModelImport {
                name: submodel_name.to_string(),
                span: *submodel_import.name_span(),
                path: submodel_import.path().clone(),
            });
        }
    }

    // Check if cursor is on a reference import name
    for (reference_name, reference_import) in model.get_references() {
        if span_contains_offset(*reference_import.name_span(), offset) {
            return Some(SymbolAtPosition::ModelImport {
                name: reference_name.to_string(),
                span: *reference_import.name_span(),
                path: reference_import.path().clone(),
            });
        }
    }

    // Check if cursor is on a variable reference in parameter expressions
    for (_param_name, param) in model.get_parameters() {
        if let Some(symbol) = find_symbol_in_parameter_value(param.value(), offset) {
            return Some(symbol);
        }
    }

    None
}

/// Finds a symbol in a parameter value expression
fn find_symbol_in_parameter_value(
    value: &ir::ParameterValue,
    offset: usize,
) -> Option<SymbolAtPosition> {
    match value {
        ir::ParameterValue::Simple(expr, _) => find_symbol_in_expr(expr, offset),
        ir::ParameterValue::Piecewise(exprs, _) => {
            for piecewise_expr in exprs {
                if let Some(symbol) = find_symbol_in_expr(piecewise_expr.expr(), offset) {
                    return Some(symbol);
                }
                if let Some(symbol) = find_symbol_in_expr(piecewise_expr.if_expr(), offset) {
                    return Some(symbol);
                }
            }
            None
        }
    }
}

/// Recursively finds a symbol in an expression
fn find_symbol_in_expr(expr: &ir::Expr, offset: usize) -> Option<SymbolAtPosition> {
    match expr {
        ir::Expr::Variable { span, variable } => {
            if !span_contains_offset(*span, offset) {
                return None;
            }

            match variable {
                ir::Variable::Parameter {
                    parameter_name,
                    parameter_span,
                } => {
                    if span_contains_offset(*parameter_span, offset) {
                        Some(SymbolAtPosition::ParameterReference {
                            name: parameter_name.clone(),
                            span: *parameter_span,
                        })
                    } else {
                        None
                    }
                }
                ir::Variable::External {
                    model,
                    model_span,
                    parameter_name,
                    parameter_span,
                } => {
                    // Check if cursor is on the model name or parameter name
                    if span_contains_offset(*model_span, offset) {
                        // Cursor is on the model name part
                        Some(SymbolAtPosition::ModelImport {
                            name: model.as_ref().to_string_lossy().to_string(),
                            span: *model_span,
                            path: model.clone(),
                        })
                    } else if span_contains_offset(*parameter_span, offset) {
                        // Cursor is on the parameter name part
                        Some(SymbolAtPosition::ExternalReference {
                            model_name: model.as_ref().to_string_lossy().to_string(),
                            model_span: *model_span,
                            parameter_name: parameter_name.clone(),
                            parameter_span: *parameter_span,
                        })
                    } else {
                        None
                    }
                }
                ir::Variable::Builtin { .. } => None, // Builtins don't have definitions
            }
        }
        ir::Expr::ComparisonOp {
            left,
            right,
            rest_chained,
            ..
        } => {
            if let Some(symbol) = find_symbol_in_expr(left, offset) {
                return Some(symbol);
            }
            if let Some(symbol) = find_symbol_in_expr(right, offset) {
                return Some(symbol);
            }
            for (_, chained_expr) in rest_chained {
                if let Some(symbol) = find_symbol_in_expr(chained_expr, offset) {
                    return Some(symbol);
                }
            }
            None
        }
        ir::Expr::BinaryOp { left, right, .. } => {
            if let Some(symbol) = find_symbol_in_expr(left, offset) {
                return Some(symbol);
            }
            find_symbol_in_expr(right, offset)
        }
        ir::Expr::UnaryOp { expr, .. } => find_symbol_in_expr(expr, offset),
        ir::Expr::FunctionCall { args, .. } => {
            // TODO: Handle function name span
            for arg in args {
                if let Some(symbol) = find_symbol_in_expr(arg, offset) {
                    return Some(symbol);
                }
            }
            None
        }
        ir::Expr::Literal { .. } => None,
    }
}

/// Resolves a symbol to its definition location
pub fn resolve_definition(
    symbol: &SymbolAtPosition,
    model_collection: &ModelCollection,
    current_model_path: &ModelPath,
) -> Option<Location> {
    match symbol {
        SymbolAtPosition::ParameterDefinition { span, .. } => {
            // Already at the definition
            Some(span_to_location(current_model_path, *span))
        }
        SymbolAtPosition::ParameterReference { name, .. } => {
            // Find the parameter in the current model
            let model = model_collection.get_models().get(current_model_path)?;
            let param = model.get_parameter(name)?;
            Some(span_to_location(current_model_path, param.name_span()))
        }
        SymbolAtPosition::ExternalReference {
            parameter_name,
            model_name,
            ..
        } => {
            // Find the parameter in the external model
            // First, resolve the model name to a ModelPath through imports
            let current_model = model_collection.get_models().get(current_model_path)?;

            // Check submodels
            if let Some(submodel) = current_model.get_submodels().get(&ir::SubmodelName::new(model_name.clone())) {
                let external_model = model_collection.get_models().get(submodel.path())?;
                let param = external_model.get_parameter(parameter_name)?;
                return Some(span_to_location(submodel.path(), param.name_span()));
            }

            // Check references
            if let Some(reference) = current_model.get_references().get(&ir::ReferenceName::new(model_name.clone())) {
                let external_model = model_collection.get_models().get(reference.path())?;
                let param = external_model.get_parameter(parameter_name)?;
                return Some(span_to_location(reference.path(), param.name_span()));
            }

            None
        }
        SymbolAtPosition::ModelImport { path, .. } => {
            // Navigate to the imported model file
            let uri = Uri::from_file_path(path.as_ref())?;
            Some(Location {
                uri,
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 0,
                    },
                },
            })
        }
    }
}

/// Checks if a span contains a given byte offset
fn span_contains_offset(span: Span, offset: usize) -> bool {
    span.start().offset <= offset && offset < span.end().offset
}

/// Converts a Span to an LSP Location
fn span_to_location(model_path: &ModelPath, span: Span) -> Location {
    let uri = Uri::from_file_path(model_path.as_ref())
        .unwrap_or_else(|| panic!("Failed to convert model path to URI: {:?}", model_path.as_ref()));
    Location {
        uri,
        range: span_to_range(span),
    }
}

/// Converts a Span to an LSP Range
fn span_to_range(span: Span) -> Range {
    Range {
        start: Position {
            line: (span.start().line - 1) as u32, // Span uses 1-indexed lines, LSP uses 0-indexed
            character: (span.start().column - 1) as u32, // Span uses 1-indexed columns, LSP uses 0-indexed
        },
        end: Position {
            line: (span.end().line - 1) as u32,
            character: (span.end().column - 1) as u32,
        },
    }
}

