//! Symbol lookup utilities for finding definitions in Oneil models

use oneil_runtime::output::ir;
use oneil_shared::{
    paths::{ModelPath, PythonPath},
    span::Span,
    symbols::{
        BuiltinFunctionName, BuiltinValueName, ParameterName, PyFunctionName, ReferenceName,
        SubmodelName,
    },
};

#[derive(Debug, Clone)]
pub enum ModelImportName {
    Submodel(SubmodelName),
    Reference(ReferenceName),
}

/// Represents a symbol found at a cursor position, including the source range to highlight.
#[derive(Debug, Clone)]
pub enum SymbolAtPosition {
    /// A parameter definition (cursor is on the parameter name in its declaration)
    ParameterDefinition { name: ParameterName, span: Span },
    /// A parameter reference (cursor is on a parameter used in an expression)
    ParameterReference { name: ParameterName, span: Span },
    /// An external parameter reference (e.g., `parameter.reference_model`)
    ///
    /// This occurs when the cursor is on the parameter name part.
    /// The model path is resolved lazily from the live instance graph
    /// (via `reference_name`) rather than stored at resolve time.
    ExternalParameterReference {
        reference_name: ReferenceName,
        parameter_name: ParameterName,
        span: Span,
    },
    /// A reference to a builtin value
    BuiltinValueReference { name: BuiltinValueName, span: Span },
    /// A submodel or reference import name
    ModelImportDefinition {
        name: ModelImportName,
        path: ModelPath,
        span: Span,
    },
    /// A submodel or reference import alias name
    ModelImportAlias { alias: ReferenceName, span: Span },
    /// A reference to a model import (e.g., `x.model_name`)
    ///
    /// This occurs when the cursor is on the model name part (e.g., `model_name`)
    ModelImportReference {
        reference_name: ReferenceName,
        span: Span,
    },
    /// A python import (e.g., `import math`)
    PythonImport { path: PythonPath, span: Span },
    /// A python function reference
    PythonFunctionReference {
        python_path: PythonPath,
        name: PyFunctionName,
        span: Span,
    },
    /// A builtin function reference
    BuiltinFunctionReference {
        name: BuiltinFunctionName,
        span: Span,
    },
}

impl SymbolAtPosition {
    /// Span of the matched symbol in the document containing the cursor.
    #[must_use]
    pub fn span(&self) -> Span {
        match self {
            Self::ParameterDefinition { span, .. }
            | Self::ParameterReference { span, .. }
            | Self::ExternalParameterReference { span, .. }
            | Self::BuiltinValueReference { span, .. }
            | Self::ModelImportDefinition { span, .. }
            | Self::ModelImportAlias { span, .. }
            | Self::ModelImportReference { span, .. }
            | Self::PythonImport { span, .. }
            | Self::PythonFunctionReference { span, .. }
            | Self::BuiltinFunctionReference { span, .. } => span.clone(),
        }
    }
}

/// Finds the symbol at a given byte offset in a model
pub fn find_symbol_at_offset(
    model: oneil_runtime::output::reference::ModelTemplateReference<'_>,
    offset: usize,
) -> Option<SymbolAtPosition> {
    // Check if cursor is on a parameter definition or in the parameter expressions
    for param in model.parameters().values() {
        // Check if cursor is on the parameter name
        if span_contains_offset(param.name_span(), offset) {
            return Some(SymbolAtPosition::ParameterDefinition {
                name: param.name().clone(),
                span: param.name_span().clone(),
            });
        }

        // Check if cursor is on the parameter value
        if let Some(symbol) = find_symbol_in_parameter_value(param.value(), offset) {
            return Some(symbol);
        }

        // Check if the cursor is on the parameter limits
        if let Some(value) = find_symbol_in_limits(param.limits(), offset) {
            return Some(value);
        }
    }

    // Check if cursor is on a submodel import name. The submodel map is keyed
    // by alias (= reference name); the underlying source-level model name
    // surfaced to the LSP comes from the `SubmodelImport.name()` field.
    for submodel_import in model.submodel_models().values() {
        if let Some(span) = submodel_import.alias_span()
            && span_contains_offset(span, offset)
        {
            let alias = submodel_import
                .alias()
                .expect("submodel import must have an alias if it has an alias span");

            return Some(SymbolAtPosition::ModelImportAlias {
                alias: alias.clone(),
                span: span.clone(),
            });
        }

        if span_contains_offset(submodel_import.name_span(), offset) {
            let submodel_path = submodel_import.path().clone();

            return Some(SymbolAtPosition::ModelImportDefinition {
                name: ModelImportName::Submodel(submodel_import.name().clone()),
                path: submodel_path,
                span: submodel_import.name_span().clone(),
            });
        }
    }

    // Check if cursor is on an extracted alias from a `[…]` block.
    for alias_import in model.alias_imports().values() {
        if let Some(span) = &alias_import.alias_span
            && span_contains_offset(span, offset)
        {
            let alias = alias_import
                .alias
                .as_ref()
                .expect("alias import must have an alias if it has an alias span");

            return Some(SymbolAtPosition::ModelImportAlias {
                alias: alias.clone(),
                span: span.clone(),
            });
        }

        if span_contains_offset(&alias_import.name_span, offset) {
            return Some(SymbolAtPosition::ModelImportDefinition {
                name: ModelImportName::Submodel(alias_import.source.clone()),
                path: model.path().clone(),
                span: alias_import.name_span.clone(),
            });
        }
    }

    // Check if cursor is on a reference import name
    for reference_import in model.reference_models().values() {
        if let Some(span) = reference_import.alias_span()
            && span_contains_offset(span, offset)
        {
            let alias = reference_import
                .alias()
                .expect("reference import must have an alias if it has an alias span");

            return Some(SymbolAtPosition::ModelImportAlias {
                alias: alias.clone(),
                span: span.clone(),
            });
        }

        if span_contains_offset(reference_import.name_span(), offset) {
            let reference_name = reference_import.name().clone();

            return Some(SymbolAtPosition::ModelImportDefinition {
                name: ModelImportName::Reference(reference_name),
                path: reference_import.path().clone(),
                span: reference_import.name_span().clone(),
            });
        }
    }

    // Check if cursor is on a python import
    for (python_path, python_import) in model.python_imports() {
        if span_contains_offset(python_import.import_path_span(), offset) {
            return Some(SymbolAtPosition::PythonImport {
                path: python_path.clone(),
                span: python_import.import_path_span().clone(),
            });
        }
    }

    // Check if cursor is on a test expression
    for test in model.tests().values() {
        if span_contains_offset(test.span(), offset) {
            return find_symbol_in_expr(test.expr(), offset);
        }
    }

    None
}

fn find_symbol_in_limits(limits: &ir::Limits, offset: usize) -> Option<SymbolAtPosition> {
    match limits {
        ir::Limits::Default => {}
        ir::Limits::Continuous {
            min,
            max,
            limit_expr_span: _,
        } => {
            if let Some(symbol) = find_symbol_in_expr(min, offset) {
                return Some(symbol);
            }
            if let Some(symbol) = find_symbol_in_expr(max, offset) {
                return Some(symbol);
            }
        }
        ir::Limits::Discrete {
            values,
            limit_expr_span: _,
        } => {
            for value in values {
                if let Some(symbol) = find_symbol_in_expr(value, offset) {
                    return Some(symbol);
                }
            }
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

/// Recursively finds a symbol in an expression.
fn find_symbol_in_expr(expr: &ir::Expr, offset: usize) -> Option<SymbolAtPosition> {
    match expr {
        ir::Expr::Variable { span, variable } => find_symbol_in_variable(variable, span, offset),
        ir::Expr::ComparisonOp {
            left,
            right,
            rest_chained,
            ..
        } => find_symbol_in_comparison_op(left, right, rest_chained, offset),
        ir::Expr::BinaryOp { left, right, .. } | ir::Expr::Fallback { left, right, .. } => {
            find_symbol_in_binary_op(left, right, offset)
        }
        ir::Expr::UnaryOp { expr, .. } | ir::Expr::UnitCast { expr, .. } => {
            find_symbol_in_expr(expr, offset)
        }
        ir::Expr::FunctionCall {
            name_span,
            name,
            args,
            ..
        } => find_symbol_in_function_call(name, name_span, args, offset),
        ir::Expr::Literal { .. } => None,
    }
}

fn find_symbol_in_variable(
    variable: &ir::Variable,
    outer_span: &Span,
    offset: usize,
) -> Option<SymbolAtPosition> {
    if !span_contains_offset(outer_span, offset) {
        return None;
    }

    match variable {
        ir::Variable::Parameter {
            parameter_name,
            parameter_span,
        } => span_contains_offset(parameter_span, offset).then(|| {
            SymbolAtPosition::ParameterReference {
                name: parameter_name.clone(),
                span: parameter_span.clone(),
            }
        }),
        ir::Variable::External {
            reference_name,
            reference_span,
            parameter_name,
            parameter_span,
        } => find_symbol_in_external_variable(
            reference_name,
            reference_span.clone(),
            parameter_name,
            parameter_span.clone(),
            offset,
        ),
        ir::Variable::Builtin { ident, ident_span } => {
            Some(SymbolAtPosition::BuiltinValueReference {
                name: ident.clone(),
                span: ident_span.clone(),
            })
        }
    }
}

fn find_symbol_in_external_variable(
    reference_name: &ReferenceName,
    reference_span: Span,
    parameter_name: &ParameterName,
    parameter_span: Span,
    offset: usize,
) -> Option<SymbolAtPosition> {
    if span_contains_offset(&reference_span, offset) {
        Some(SymbolAtPosition::ModelImportReference {
            reference_name: reference_name.clone(),
            span: reference_span,
        })
    } else if span_contains_offset(&parameter_span, offset) {
        Some(SymbolAtPosition::ExternalParameterReference {
            reference_name: reference_name.clone(),
            parameter_name: parameter_name.clone(),
            span: parameter_span,
        })
    } else {
        None
    }
}

fn find_symbol_in_comparison_op(
    left: &ir::Expr,
    right: &ir::Expr,
    rest_chained: &[(ir::ComparisonOp, ir::Expr)],
    offset: usize,
) -> Option<SymbolAtPosition> {
    find_symbol_in_expr(left, offset)
        .or_else(|| find_symbol_in_expr(right, offset))
        .or_else(|| {
            rest_chained
                .iter()
                .find_map(|(_, expr)| find_symbol_in_expr(expr, offset))
        })
}

fn find_symbol_in_binary_op(
    left: &ir::Expr,
    right: &ir::Expr,
    offset: usize,
) -> Option<SymbolAtPosition> {
    find_symbol_in_expr(left, offset).or_else(|| find_symbol_in_expr(right, offset))
}

fn find_symbol_in_function_call(
    name: &ir::FunctionName,
    name_span: &Span,
    args: &[ir::Expr],
    offset: usize,
) -> Option<SymbolAtPosition> {
    if span_contains_offset(name_span, offset) {
        return match name {
            ir::FunctionName::Builtin(builtin_name, builtin_name_span) => {
                Some(SymbolAtPosition::BuiltinFunctionReference {
                    name: builtin_name.clone(),
                    span: builtin_name_span.clone(),
                })
            }
            ir::FunctionName::Imported {
                python_path,
                name,
                name_span: imported_name_span,
            } => Some(SymbolAtPosition::PythonFunctionReference {
                python_path: python_path.clone(),
                name: name.clone(),
                span: imported_name_span.clone(),
            }),
        };
    }

    args.iter().find_map(|arg| find_symbol_in_expr(arg, offset))
}

/// Checks if a span contains a given byte offset
const fn span_contains_offset(span: &Span, offset: usize) -> bool {
    span.start().offset <= offset && offset < span.end().offset
}
