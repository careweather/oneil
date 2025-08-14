//! Abstract Syntax Tree (AST) printing functionality for the Oneil CLI
//!
//! This module provides functionality for printing Oneil AST structures in a
//! hierarchical tree format suitable for debugging and development. It handles
//! all AST node types including models, declarations, expressions, parameters,
//! and units.

use oneil_ast::{
    Decl, Expr, Model,
    declaration::DeclNode,
    expression::{ExprNode, Literal, LiteralNode, Variable, VariableNode},
    model::SectionNode,
};
use std::io::{self, Write};

/// Prints the AST in a hierarchical tree format for debugging
pub fn print(ast: &Model, writer: &mut impl Write) -> io::Result<()> {
    print_model(ast, writer, 0)
}

/// Prints a model node with its declarations and sections
fn print_model(model: &Model, writer: &mut impl Write, indent: usize) -> io::Result<()> {
    writeln!(writer, "{}Model", "  ".repeat(indent))?;

    // Print note if present
    if let Some(note) = model.note() {
        writeln!(
            writer,
            "{}├── Note: \"{}\"",
            "  ".repeat(indent),
            note.value()
        )?;
    }

    // Print declarations
    if !model.decls().is_empty() {
        writeln!(writer, "{}├── Declarations:", "  ".repeat(indent))?;
        for (i, decl) in model.decls().iter().enumerate() {
            let is_last = i == model.decls().len() - 1 && model.sections().is_empty();
            let prefix = if is_last { "└──" } else { "├──" };
            print_decl(decl, writer, indent + 2, prefix)?;
        }
    }

    // Print sections
    if !model.sections().is_empty() {
        for (i, section) in model.sections().iter().enumerate() {
            let is_last = i == model.sections().len() - 1;
            let prefix = if is_last { "└──" } else { "├──" };
            print_section(section, writer, indent, prefix)?;
        }
    }

    Ok(())
}

/// Prints a declaration node
fn print_decl(
    decl: &DeclNode,
    writer: &mut impl Write,
    indent: usize,
    prefix: &str,
) -> io::Result<()> {
    match decl.node_value() {
        Decl::Import(import) => {
            writeln!(
                writer,
                "{}{} Import: \"{}\"",
                "  ".repeat(indent),
                prefix,
                import.path().node_value()
            )?;
        }
        Decl::UseModel(use_model) => {
            let alias = use_model
                .alias()
                .map(|a| format!(" as {}", a.node_value().as_str()))
                .unwrap_or_default();
            writeln!(
                writer,
                "{}{} UseModel: \"{}\"{}",
                "  ".repeat(indent),
                prefix,
                use_model.model_name().node_value().as_str(),
                alias
            )?;

            // Print subcomponents if any
            if !use_model.subcomponents().is_empty() {
                let subcomps: Vec<String> = use_model
                    .subcomponents()
                    .iter()
                    .map(|id| id.node_value().as_str().to_string())
                    .collect();
                writeln!(
                    writer,
                    "{}    └── Subcomponents: [{}]",
                    "  ".repeat(indent),
                    subcomps.join(", ")
                )?;
            }
        }
        Decl::Parameter(param) => {
            writeln!(
                writer,
                "{}{} Parameter: {}",
                "  ".repeat(indent),
                prefix,
                param.ident().node_value().as_str()
            )?;

            // Print parameter details
            writeln!(
                writer,
                "{}    ├── Label: \"{}\"",
                "  ".repeat(indent),
                param.label().node_value().as_str()
            )?;

            // Print parameter value
            writeln!(writer, "{}    ├── Value:", "  ".repeat(indent))?;
            print_parameter_value(param.value(), writer, indent + 4)?;

            // Print limits if any
            if let Some(limits) = param.limits() {
                writeln!(writer, "{}    ├── Limits:", "  ".repeat(indent))?;
                print_limits(limits, writer, indent + 4)?;
            }

            // Print performance marker if any
            if param.performance_marker().is_some() {
                writeln!(writer, "{}    ├── Performance Marker", "  ".repeat(indent))?;
            }

            // Print trace level if any
            if let Some(trace_level) = param.trace_level() {
                writeln!(
                    writer,
                    "{}    ├── Trace Level: {:?}",
                    "  ".repeat(indent),
                    trace_level.node_value()
                )?;
            }

            // Print note if any
            if let Some(note) = param.note() {
                writeln!(
                    writer,
                    "{}    └── Note: \"{}\"",
                    "  ".repeat(indent),
                    note.value()
                )?;
            }
        }
        Decl::Test(test) => {
            writeln!(writer, "{}{} Test:", "  ".repeat(indent), prefix)?;

            if let Some(trace_level) = test.trace_level() {
                writeln!(
                    writer,
                    "{}    ├── Trace Level: {:?}",
                    "  ".repeat(indent),
                    trace_level.node_value()
                )?;
            }

            if let Some(inputs) = test.inputs() {
                let input_names: Vec<String> = inputs
                    .iter()
                    .map(|id| id.node_value().as_str().to_string())
                    .collect();
                writeln!(
                    writer,
                    "{}    ├── Inputs: [{}]",
                    "  ".repeat(indent),
                    input_names.join(", ")
                )?;
            }

            writeln!(writer, "{}    └── Expression:", "  ".repeat(indent))?;
            print_expression(test.expr(), writer, indent + 4)?;
        }
    }
    Ok(())
}

/// Prints a section node
fn print_section(
    section: &SectionNode,
    writer: &mut impl Write,
    indent: usize,
    prefix: &str,
) -> io::Result<()> {
    writeln!(
        writer,
        "{}{} Section: \"{}\"",
        "  ".repeat(indent),
        prefix,
        section.header().label().node_value().as_str()
    )?;

    // Print section note if present
    if let Some(note) = section.note() {
        writeln!(
            writer,
            "{}    ├── Note: \"{}\"",
            "  ".repeat(indent),
            note.value()
        )?;
    }

    // Print section declarations
    if !section.decls().is_empty() {
        for (i, decl) in section.decls().iter().enumerate() {
            let is_last = i == section.decls().len() - 1;
            let sub_prefix = if is_last { "└──" } else { "├──" };
            print_decl(decl, writer, indent + 2, sub_prefix)?;
        }
    }

    Ok(())
}

/// Prints an expression node
fn print_expression(expr: &ExprNode, writer: &mut impl Write, indent: usize) -> io::Result<()> {
    match expr.node_value() {
        Expr::BinaryOp { op, left, right } => {
            writeln!(
                writer,
                "{}BinaryOp: {:?}",
                "  ".repeat(indent),
                op.node_value()
            )?;
            print_expression(left, writer, indent + 2)?;
            print_expression(right, writer, indent + 2)?;
        }
        Expr::UnaryOp { op, expr } => {
            writeln!(
                writer,
                "{}UnaryOp: {:?}",
                "  ".repeat(indent),
                op.node_value()
            )?;
            print_expression(expr, writer, indent + 2)?;
        }
        Expr::FunctionCall { name, args } => {
            writeln!(
                writer,
                "{}FunctionCall: \"{}\"",
                "  ".repeat(indent),
                name.node_value().as_str()
            )?;
            for (i, arg) in args.iter().enumerate() {
                let is_last = i == args.len() - 1;
                let prefix = if is_last { "└──" } else { "├──" };
                writeln!(
                    writer,
                    "{}    {}Arg {}:",
                    "  ".repeat(indent),
                    prefix,
                    i + 1
                )?;
                print_expression(arg, writer, indent + 4)?;
            }
        }
        Expr::Parenthesized { expr } => {
            writeln!(writer, "{}Parenthesized:", "  ".repeat(indent))?;
            print_expression(expr, writer, indent + 2)?;
        }
        Expr::Variable(var) => {
            print_variable(var, writer, indent)?;
        }
        Expr::Literal(lit) => {
            print_literal(lit, writer, indent)?;
        }
    }
    Ok(())
}

/// Prints a variable node
fn print_variable(var: &VariableNode, writer: &mut impl Write, indent: usize) -> io::Result<()> {
    match var.node_value() {
        Variable::Identifier(id) => {
            writeln!(
                writer,
                "{}Variable: \"{}\"",
                "  ".repeat(indent),
                id.node_value().as_str()
            )?;
        }
        Variable::Accessor { parent, component } => {
            writeln!(
                writer,
                "{}Accessor: \"{}\"",
                "  ".repeat(indent),
                parent.node_value().as_str()
            )?;
            print_variable(component, writer, indent + 2)?;
        }
    }
    Ok(())
}

/// Prints a literal node
fn print_literal(lit: &LiteralNode, writer: &mut impl Write, indent: usize) -> io::Result<()> {
    match lit.node_value() {
        Literal::Number(n) => {
            writeln!(writer, "{}Literal: {}", "  ".repeat(indent), n)?;
        }
        Literal::String(s) => {
            writeln!(writer, "{}Literal: \"{}\"", "  ".repeat(indent), s)?;
        }
        Literal::Boolean(b) => {
            writeln!(writer, "{}Literal: {}", "  ".repeat(indent), b)?;
        }
    }
    Ok(())
}

/// Prints a parameter value node
fn print_parameter_value(
    value: &oneil_ast::parameter::ParameterValueNode,
    writer: &mut impl Write,
    indent: usize,
) -> io::Result<()> {
    match value.node_value() {
        oneil_ast::parameter::ParameterValue::Simple(expr, unit) => {
            writeln!(writer, "{}Simple:", "  ".repeat(indent))?;
            print_expression(expr, writer, indent + 2)?;
            if let Some(unit_expr) = unit {
                writeln!(writer, "{}Unit:", "  ".repeat(indent))?;
                print_unit_expression(unit_expr, writer, indent + 2)?;
            }
        }
        oneil_ast::parameter::ParameterValue::Piecewise(parts, unit) => {
            writeln!(writer, "{}Piecewise:", "  ".repeat(indent))?;
            for (i, part) in parts.iter().enumerate() {
                let is_last = i == parts.len() - 1;
                let prefix = if is_last { "└──" } else { "├──" };
                writeln!(
                    writer,
                    "{}    {}Part {}:",
                    "  ".repeat(indent),
                    prefix,
                    i + 1
                )?;
                writeln!(writer, "{}        ├── Expression:", "  ".repeat(indent))?;
                print_expression(part.expr(), writer, indent + 6)?;
                writeln!(writer, "{}        └── Condition:", "  ".repeat(indent))?;
                print_expression(part.if_expr(), writer, indent + 6)?;
            }
            if let Some(unit_expr) = unit {
                writeln!(writer, "{}    └── Unit:", "  ".repeat(indent))?;
                print_unit_expression(unit_expr, writer, indent + 4)?;
            }
        }
    }
    Ok(())
}

/// Prints a limits node
fn print_limits(
    limits: &oneil_ast::parameter::LimitsNode,
    writer: &mut impl Write,
    indent: usize,
) -> io::Result<()> {
    match limits.node_value() {
        oneil_ast::parameter::Limits::Continuous { min, max } => {
            writeln!(writer, "{}Continuous:", "  ".repeat(indent))?;
            writeln!(writer, "{}    ├── Min:", "  ".repeat(indent))?;
            print_expression(min, writer, indent + 4)?;
            writeln!(writer, "{}    └── Max:", "  ".repeat(indent))?;
            print_expression(max, writer, indent + 4)?;
        }
        oneil_ast::parameter::Limits::Discrete { values } => {
            writeln!(writer, "{}Discrete:", "  ".repeat(indent))?;
            for (i, value) in values.iter().enumerate() {
                let is_last = i == values.len() - 1;
                let prefix = if is_last { "└──" } else { "├──" };
                writeln!(
                    writer,
                    "{}    {}Value {}:",
                    "  ".repeat(indent),
                    prefix,
                    i + 1
                )?;
                print_expression(value, writer, indent + 4)?;
            }
        }
    }
    Ok(())
}

/// Prints a unit expression node
fn print_unit_expression(
    unit_expr: &oneil_ast::unit::UnitExprNode,
    writer: &mut impl Write,
    indent: usize,
) -> io::Result<()> {
    match unit_expr.node_value() {
        oneil_ast::unit::UnitExpr::BinaryOp { op, left, right } => {
            writeln!(
                writer,
                "{}BinaryOp: {:?}",
                "  ".repeat(indent),
                op.node_value()
            )?;
            print_unit_expression(left, writer, indent + 2)?;
            print_unit_expression(right, writer, indent + 2)?;
        }
        oneil_ast::unit::UnitExpr::Parenthesized { expr } => {
            writeln!(writer, "{}Parenthesized:", "  ".repeat(indent))?;
            print_unit_expression(expr, writer, indent + 2)?;
        }
        oneil_ast::unit::UnitExpr::Unit {
            identifier,
            exponent,
        } => {
            writeln!(
                writer,
                "{}Unit: \"{}\"",
                "  ".repeat(indent),
                identifier.node_value().as_str()
            )?;
            if let Some(exp) = exponent {
                writeln!(
                    writer,
                    "{}    └── Exponent: {}",
                    "  ".repeat(indent),
                    exp.node_value().value()
                )?;
            }
        }
        oneil_ast::unit::UnitExpr::UnitOne => {
            writeln!(writer, "{}Unit: 1", "  ".repeat(indent))?;
        }
    }
    Ok(())
}
