//! Intermediate Representation (IR) printing functionality for the Oneil CLI

#![expect(
    clippy::use_debug,
    reason = "this is a debug function for developers, not for end users"
)]

use std::{
    collections::HashMap,
    io::{self, Write},
};

use oneil_ir as ir;

/// Prints the IR in a hierarchical tree format for debugging
pub fn print(ir: &ir::ModelCollection, writer: &mut impl Write) -> io::Result<()> {
    writeln!(writer, "ModelCollection")?;

    // Print Python imports
    writeln!(writer, "├── Python Imports:")?;
    let python_imports = ir.get_python_imports();
    if python_imports.is_empty() {
        writeln!(writer, "│   └── [none]")?;
    } else {
        for (i, import) in python_imports.iter().enumerate() {
            let is_last = i == python_imports.len() - 1;
            let prefix = if is_last { "└──" } else { "├──" };
            writeln!(
                writer,
                "│   {}Import: \"{}\"",
                prefix,
                import.as_ref().display()
            )?;
        }
    }

    // Print initial models
    writeln!(writer, "├── Initial Models:")?;
    let initial_models = ir.get_initial_models();
    if initial_models.is_empty() {
        writeln!(writer, "│   └── [none]")?;
    } else {
        for (i, model_path) in initial_models.iter().enumerate() {
            let is_last = i == initial_models.len() - 1;
            let prefix = if is_last { "└──" } else { "├──" };
            writeln!(
                writer,
                "│   {}Model: \"{}\"",
                prefix,
                model_path.as_ref().display()
            )?;
        }
    }

    // Print all models
    writeln!(writer, "└── Models:")?;
    let models = ir.get_models();
    if models.is_empty() {
        writeln!(writer, "    └── [none]")?;
    } else {
        for (i, (model_path, model)) in models.iter().enumerate() {
            let is_last = i == models.len() - 1;
            let prefix = if is_last { "└──" } else { "├──" };
            print_model(model_path, model, writer, 2, prefix)?;
        }
    }

    Ok(())
}

/// Prints a single model with its components
fn print_model(
    model_path: &ir::ModelPath,
    model: &ir::Model,
    writer: &mut impl Write,
    indent: usize,
    prefix: &str,
) -> io::Result<()> {
    writeln!(
        writer,
        "{}    {}Model: \"{}\"",
        "  ".repeat(indent),
        prefix,
        model_path.as_ref().display()
    )?;

    let indent = indent + 2;
    let mut sections = Vec::new();

    // Collect submodels
    let submodels = model.get_submodels();
    if !submodels.is_empty() {
        sections.push(("Submodels", submodels.len()));
    }

    // Collect parameters
    let parameters = model.get_parameters();
    if !parameters.is_empty() {
        sections.push(("Parameters", parameters.len()));
    }

    // Collect references
    let references = model.get_references();
    if !references.is_empty() {
        sections.push(("References", references.len()));
    }

    // Collect tests
    let tests = model.get_tests();
    if !tests.is_empty() {
        sections.push(("Tests", tests.len()));
    }

    // Print sections
    for (i, (section_name, count)) in sections.iter().enumerate() {
        let is_last = i == sections.len() - 1;
        let section_prefix = if is_last { "└──" } else { "├──" };
        writeln!(
            writer,
            "{}    {} {} ({}):",
            "  ".repeat(indent),
            section_prefix,
            section_name,
            count
        )?;

        match *section_name {
            "Submodels" => print_submodels(submodels, writer, indent + 2)?,
            "Parameters" => print_parameters(parameters, writer, indent + 2)?,
            "References" => print_references(references, writer, indent + 2)?,
            "Tests" => print_tests(tests, writer, indent + 2)?,
            _ => {}
        }
    }

    Ok(())
}

/// Prints submodels
fn print_submodels(
    submodels: &ir::SubmodelMap,
    writer: &mut impl Write,
    indent: usize,
) -> io::Result<()> {
    for (i, (identifier, submodel)) in submodels.iter().enumerate() {
        let is_last = i == submodels.len() - 1;
        let prefix = if is_last { "└──" } else { "├──" };
        writeln!(
            writer,
            "{}    {}Submodel: \"{}\" -> \"{}\"",
            "  ".repeat(indent),
            prefix,
            identifier.as_str(),
            submodel.path().as_ref().display()
        )?;
    }
    Ok(())
}

/// Prints submodels
fn print_references(
    references: &ir::ReferenceMap,
    writer: &mut impl Write,
    indent: usize,
) -> io::Result<()> {
    for (i, (identifier, reference)) in references.iter().enumerate() {
        let is_last = i == references.len() - 1;
        let prefix = if is_last { "└──" } else { "├──" };
        writeln!(
            writer,
            "{}    {}Reference: \"{}\" -> \"{}\"",
            "  ".repeat(indent),
            prefix,
            identifier.as_str(),
            reference.path().as_ref().display()
        )?;
    }
    Ok(())
}

/// Prints parameters
fn print_parameters(
    parameters: &ir::ParameterCollection,
    writer: &mut impl Write,
    indent: usize,
) -> io::Result<()> {
    for (i, (identifier, parameter)) in parameters.iter().enumerate() {
        let is_last = i == parameters.len() - 1;
        let prefix = if is_last { "└──" } else { "├──" };
        print_parameter(identifier, parameter, writer, indent, prefix)?;
    }
    Ok(())
}

/// Prints a single parameter
fn print_parameter(
    identifier: &ir::Identifier,
    parameter: &ir::Parameter,
    writer: &mut impl Write,
    indent: usize,
    prefix: &str,
) -> io::Result<()> {
    writeln!(
        writer,
        "{}    {}Parameter: \"{}\"",
        "  ".repeat(indent),
        prefix,
        identifier.as_str()
    )?;

    let indent = indent + 2;

    // Print dependencies
    let dependencies = parameter.dependencies();
    if !dependencies.is_empty() {
        writeln!(writer, "{}    ├── Dependencies:", "  ".repeat(indent))?;
        for (i, dep) in dependencies.iter().enumerate() {
            let is_last = i == dependencies.len() - 1;
            let dep_prefix = if is_last { "└──" } else { "├──" };
            writeln!(
                writer,
                "{}        {}Dependency: \"{}\"",
                "  ".repeat(indent),
                dep_prefix,
                dep.as_str()
            )?;
        }
    }

    // Print value
    writeln!(writer, "{}    ├── Value:", "  ".repeat(indent))?;
    print_parameter_value(parameter.value(), writer, indent + 2)?;

    // Print limits
    writeln!(writer, "{}    ├── Limits:", "  ".repeat(indent))?;
    print_limits(parameter.limits(), writer, indent + 2)?;

    // Print metadata
    if parameter.is_performance() {
        writeln!(writer, "{}    ├── Performance: true", "  ".repeat(indent))?;
    }
    writeln!(
        writer,
        "{}    └── Trace Level: {:?}",
        "  ".repeat(indent),
        parameter.trace_level()
    )?;

    Ok(())
}

/// Prints a parameter value
fn print_parameter_value(
    value: &ir::ParameterValue,
    writer: &mut impl Write,
    indent: usize,
) -> io::Result<()> {
    match value {
        ir::ParameterValue::Simple(expr, unit) => {
            writeln!(writer, "{}    ├── Type: Simple", "  ".repeat(indent))?;
            writeln!(writer, "{}    ├── Expression:", "  ".repeat(indent))?;
            print_expression(expr, writer, indent + 2)?;
            if let Some(unit) = unit {
                writeln!(writer, "{}    └── Unit:", "  ".repeat(indent))?;
                print_unit(unit, writer, indent + 2)?;
            }
        }
        ir::ParameterValue::Piecewise(exprs, unit) => {
            writeln!(writer, "{}    ├── Type: Piecewise", "  ".repeat(indent))?;
            for (i, piecewise_expr) in exprs.iter().enumerate() {
                let is_last = i == exprs.len() - 1 && unit.is_none();
                let prefix = if is_last { "└──" } else { "├──" };
                writeln!(
                    writer,
                    "{}    {}Piece {}:",
                    "  ".repeat(indent),
                    prefix,
                    i + 1
                )?;
                writeln!(writer, "{}        ├── Expression:", "  ".repeat(indent))?;
                print_expression(piecewise_expr.expr(), writer, indent + 4)?;
                writeln!(writer, "{}        └── Condition:", "  ".repeat(indent))?;
                print_expression(piecewise_expr.if_expr(), writer, indent + 4)?;
            }
            if let Some(unit) = unit {
                writeln!(writer, "{}    └── Unit:", "  ".repeat(indent))?;
                print_unit(unit, writer, indent + 4)?;
            }
        }
    }
    Ok(())
}

/// Prints limits
fn print_limits(limits: &ir::Limits, writer: &mut impl Write, indent: usize) -> io::Result<()> {
    match limits {
        ir::Limits::Default => {
            writeln!(writer, "{}    └── Type: Default", "  ".repeat(indent))?;
        }
        ir::Limits::Continuous { min, max } => {
            writeln!(writer, "{}    ├── Type: Continuous", "  ".repeat(indent))?;
            writeln!(writer, "{}    ├── Min:", "  ".repeat(indent))?;
            print_expression(min, writer, indent + 2)?;
            writeln!(writer, "{}    └── Max:", "  ".repeat(indent))?;
            print_expression(max, writer, indent + 2)?;
        }
        ir::Limits::Discrete { values } => {
            writeln!(writer, "{}    ├── Type: Discrete", "  ".repeat(indent))?;
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
                print_expression(value, writer, indent + 2)?;
            }
        }
    }
    Ok(())
}

/// Prints an expression
fn print_expression(
    expr: &ir::ExprWithSpan,
    writer: &mut impl Write,
    indent: usize,
) -> io::Result<()> {
    match &**expr {
        ir::Expr::BinaryOp { op, left, right } => {
            writeln!(
                writer,
                "{}    ├── BinaryOp: {:?}",
                "  ".repeat(indent),
                &**op
            )?;
            print_expression(left, writer, indent + 2)?;
            print_expression(right, writer, indent + 2)?;
        }
        ir::Expr::UnaryOp { op, expr } => {
            writeln!(
                writer,
                "{}    ├── UnaryOp: {:?}",
                "  ".repeat(indent),
                &**op
            )?;
            print_expression(expr, writer, indent + 2)?;
        }
        ir::Expr::FunctionCall { name, args } => {
            writeln!(
                writer,
                "{}    ├── FunctionCall: {:?}",
                "  ".repeat(indent),
                &**name
            )?;
            for (i, arg) in args.iter().enumerate() {
                let is_last = i == args.len() - 1;
                let prefix = if is_last { "└──" } else { "├──" };
                writeln!(
                    writer,
                    "{}        {}Arg {}:",
                    "  ".repeat(indent),
                    prefix,
                    i + 1
                )?;
                print_expression(arg, writer, indent + 4)?;
            }
        }
        ir::Expr::Variable(var) => {
            print_variable(var, writer, indent)?;
        }
        ir::Expr::Literal { value } => {
            print_literal(value, writer, indent)?;
        }
        ir::Expr::ComparisonOp {
            op,
            left,
            right,
            rest_chained,
        } => {
            writeln!(
                writer,
                "{}    ├── ComparisonOp: {:?}",
                "  ".repeat(indent),
                &**op
            )?;
            print_expression(left, writer, indent + 2)?;
            print_expression(right, writer, indent + 2)?;
            for (i, (op, expr)) in rest_chained.iter().enumerate() {
                let is_last = i == rest_chained.len() - 1;
                let prefix = if is_last { "└──" } else { "├──" };
                writeln!(
                    writer,
                    "{}        {}Chained: {:?}",
                    "  ".repeat(indent),
                    prefix,
                    &**op
                )?;
                print_expression(expr, writer, indent + 4)?;
            }
        }
    }
    Ok(())
}

/// Prints a variable
fn print_variable(var: &ir::Variable, writer: &mut impl Write, indent: usize) -> io::Result<()> {
    match var {
        ir::Variable::Builtin(id) => {
            writeln!(
                writer,
                "{}    ├── Builtin Variable: \"{}\"",
                "  ".repeat(indent),
                id.as_str()
            )?;
        }
        ir::Variable::Parameter(id) => {
            writeln!(
                writer,
                "{}    ├── Parameter Variable: \"{}\"",
                "  ".repeat(indent),
                id.as_str()
            )?;
        }
        ir::Variable::External { model, ident } => {
            writeln!(
                writer,
                "{}    ├── External Variable: \"{}\" from \"{}\"",
                "  ".repeat(indent),
                ident.as_str(),
                model.as_ref().display()
            )?;
        }
    }
    Ok(())
}

/// Prints a literal
fn print_literal(lit: &ir::Literal, writer: &mut impl Write, indent: usize) -> io::Result<()> {
    match lit {
        ir::Literal::Number(n) => {
            writeln!(writer, "{}    ├── Literal: {}", "  ".repeat(indent), n)?;
        }
        ir::Literal::String(s) => {
            writeln!(writer, "{}    ├── Literal: \"{}\"", "  ".repeat(indent), s)?;
        }
        ir::Literal::Boolean(b) => {
            writeln!(writer, "{}    ├── Literal: {}", "  ".repeat(indent), b)?;
        }
    }
    Ok(())
}

/// Prints a unit
fn print_unit(unit: &ir::CompositeUnit, writer: &mut impl Write, indent: usize) -> io::Result<()> {
    writeln!(writer, "{}    └── Unit: {:?}", "  ".repeat(indent), unit)?;
    Ok(())
}

/// Prints tests
fn print_tests(
    tests: &HashMap<ir::TestIndex, ir::Test>,
    writer: &mut impl Write,
    indent: usize,
) -> io::Result<()> {
    for (i, (test_index, test)) in tests.iter().enumerate() {
        let is_last = i == tests.len() - 1;
        let prefix = if is_last { "└──" } else { "├──" };
        writeln!(
            writer,
            "{}    {}Test {:?}:",
            "  ".repeat(indent),
            prefix,
            test_index
        )?;
        print_test(test, writer, indent + 2)?;
    }
    Ok(())
}

/// Prints a single test
fn print_test(test: &ir::Test, writer: &mut impl Write, indent: usize) -> io::Result<()> {
    // Print trace level
    writeln!(
        writer,
        "{}    ├── Trace Level: {:?}",
        "  ".repeat(indent),
        test.trace_level()
    )?;

    // Print test expression
    writeln!(writer, "{}    └── Test Expression:", "  ".repeat(indent))?;
    print_expression(test.test_expr(), writer, indent + 2)?;

    Ok(())
}
