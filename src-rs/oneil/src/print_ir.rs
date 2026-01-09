//! Intermediate Representation (IR) printing functionality for the Oneil CLI

use std::collections::HashMap;

use anstream::println;
use oneil_ir as ir;

/// Prints the IR in a hierarchical tree format for debugging
pub fn print(ir: &ir::ModelCollection, print_debug: bool) {
    if print_debug {
        println!("IR: {ir:?}");
        return;
    }

    println!("ModelCollection");

    // Print Python imports
    println!("├── Python Imports:");
    let python_imports = ir.get_python_imports();
    if python_imports.is_empty() {
        println!("│   └── [none]");
    } else {
        for (i, import) in python_imports.iter().enumerate() {
            let is_last = i == python_imports.len() - 1;
            let prefix = if is_last { "└──" } else { "├──" };
            println!(
                "│   {}Import: \"{}\"",
                prefix,
                import.import_path().as_ref().display()
            );
        }
    }

    // Print initial models
    println!("├── Initial Models:");
    let initial_models = ir.get_initial_models();
    if initial_models.is_empty() {
        println!("│   └── [none]");
    } else {
        for (i, model_path) in initial_models.iter().enumerate() {
            let is_last = i == initial_models.len() - 1;
            let prefix = if is_last { "└──" } else { "├──" };
            println!("│   {}Model: \"{}\"", prefix, model_path.as_ref().display());
        }
    }

    // Print all models
    println!("└── Models:");
    let models = ir.get_models();
    if models.is_empty() {
        println!("    └── [none]");
    } else {
        for (i, (model_path, model)) in models.iter().enumerate() {
            let is_last = i == models.len() - 1;
            let prefix = if is_last { "└──" } else { "├──" };
            print_model(model_path, model, 2, prefix);
        }
    }
}

/// Prints a single model with its components
fn print_model(model_path: &ir::ModelPath, model: &ir::Model, indent: usize, prefix: &str) {
    println!(
        "{}    {}Model: \"{}\"",
        "  ".repeat(indent),
        prefix,
        model_path.as_ref().display()
    );

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
        println!(
            "{}    {} {} ({}):",
            "  ".repeat(indent),
            section_prefix,
            section_name,
            count
        );

        match *section_name {
            "Submodels" => print_submodels(submodels, indent + 2),
            "Parameters" => print_parameters(parameters, indent + 2),
            "References" => print_references(references, indent + 2),
            "Tests" => print_tests(tests, indent + 2),
            _ => {}
        }
    }
}

/// Prints submodels
fn print_submodels(submodels: &HashMap<ir::SubmodelName, ir::SubmodelImport>, indent: usize) {
    for (i, (identifier, submodel)) in submodels.iter().enumerate() {
        let is_last = i == submodels.len() - 1;
        let prefix = if is_last { "└──" } else { "├──" };
        println!(
            "{}    {}Submodel: \"{}\" -> \"{}\"",
            "  ".repeat(indent),
            prefix,
            identifier.as_str(),
            submodel.path().as_ref().display()
        );
    }
}

/// Prints submodels
fn print_references(references: &HashMap<ir::ReferenceName, ir::ReferenceImport>, indent: usize) {
    for (i, (identifier, reference)) in references.iter().enumerate() {
        let is_last = i == references.len() - 1;
        let prefix = if is_last { "└──" } else { "├──" };
        println!(
            "{}    {}Reference: \"{}\" -> \"{}\"",
            "  ".repeat(indent),
            prefix,
            identifier.as_str(),
            reference.path().as_ref().display()
        );
    }
}

/// Prints parameters
fn print_parameters(parameters: &HashMap<ir::ParameterName, ir::Parameter>, indent: usize) {
    for (i, (parameter_name, parameter)) in parameters.iter().enumerate() {
        let is_last = i == parameters.len() - 1;
        let prefix = if is_last { "└──" } else { "├──" };
        print_parameter(parameter_name, parameter, indent, prefix);
    }
}

/// Prints a single parameter
fn print_parameter(
    parameter_name: &ir::ParameterName,
    parameter: &ir::Parameter,
    indent: usize,
    prefix: &str,
) {
    println!(
        "{}    {}Parameter: \"{}\"",
        "  ".repeat(indent),
        prefix,
        parameter_name.as_str()
    );

    let indent = indent + 2;

    // Print dependencies
    let dependencies = parameter.dependencies();
    if !dependencies.is_empty() {
        println!("{}    ├── Dependencies:", "  ".repeat(indent));
        for (i, dep) in dependencies.iter().enumerate() {
            let is_last = i == dependencies.len() - 1;
            let dep_prefix = if is_last { "└──" } else { "├──" };
            println!(
                "{}        {}Dependency: \"{}\"",
                "  ".repeat(indent),
                dep_prefix,
                dep.as_str()
            );
        }
    }

    // Print value
    println!("{}    ├── Value:", "  ".repeat(indent));
    print_parameter_value(parameter.value(), indent + 2);

    // Print limits
    println!("{}    ├── Limits:", "  ".repeat(indent));
    print_limits(parameter.limits(), indent + 2);

    // Print metadata
    if parameter.is_performance() {
        println!("{}    ├── Performance: true", "  ".repeat(indent));
    }
    println!(
        "{}    └── Trace Level: {:?}",
        "  ".repeat(indent),
        parameter.trace_level()
    );
}

/// Prints a parameter value
fn print_parameter_value(value: &ir::ParameterValue, indent: usize) {
    match value {
        ir::ParameterValue::Simple(expr, unit) => {
            println!("{}    ├── Type: Simple", "  ".repeat(indent));
            println!("{}    ├── Expression:", "  ".repeat(indent));
            print_expression(expr, indent + 2);
            if let Some(unit) = unit {
                println!("{}    └── Unit:", "  ".repeat(indent));
                print_unit(unit, indent + 2);
            }
        }
        ir::ParameterValue::Piecewise(exprs, unit) => {
            println!("{}    ├── Type: Piecewise", "  ".repeat(indent));
            for (i, piecewise_expr) in exprs.iter().enumerate() {
                let is_last = i == exprs.len() - 1 && unit.is_none();
                let prefix = if is_last { "└──" } else { "├──" };
                println!("{}    {}Piece {}:", "  ".repeat(indent), prefix, i + 1);
                println!("{}        ├── Expression:", "  ".repeat(indent));
                print_expression(piecewise_expr.expr(), indent + 4);
                println!("{}        └── Condition:", "  ".repeat(indent));
                print_expression(piecewise_expr.if_expr(), indent + 4);
            }
            if let Some(unit) = unit {
                println!("{}    └── Unit:", "  ".repeat(indent));
                print_unit(unit, indent + 4);
            }
        }
    }
}

/// Prints limits
fn print_limits(limits: &ir::Limits, indent: usize) {
    match limits {
        ir::Limits::Default => {
            println!("{}    └── Type: Default", "  ".repeat(indent));
        }
        ir::Limits::Continuous {
            min,
            max,
            limit_expr_span: _,
        } => {
            println!("{}    ├── Type: Continuous", "  ".repeat(indent));
            println!("{}    ├── Min:", "  ".repeat(indent));
            print_expression(min, indent + 2);
            println!("{}    └── Max:", "  ".repeat(indent));
            print_expression(max, indent + 2);
        }
        ir::Limits::Discrete {
            values,
            limit_expr_span: _,
        } => {
            println!("{}    ├── Type: Discrete", "  ".repeat(indent));
            for (i, value) in values.iter().enumerate() {
                let is_last = i == values.len() - 1;
                let prefix = if is_last { "└──" } else { "├──" };
                println!("{}    {}Value {}:", "  ".repeat(indent), prefix, i + 1);
                print_expression(value, indent + 2);
            }
        }
    }
}

/// Prints an expression
fn print_expression(expr: &ir::Expr, indent: usize) {
    match expr {
        ir::Expr::BinaryOp {
            span: _,
            op,
            left,
            right,
        } => {
            println!("{}    ├── BinaryOp: {:?}", "  ".repeat(indent), op);
            print_expression(left, indent + 2);
            print_expression(right, indent + 2);
        }
        ir::Expr::UnaryOp { span: _, op, expr } => {
            println!("{}    ├── UnaryOp: {:?}", "  ".repeat(indent), op);
            print_expression(expr, indent + 2);
        }
        ir::Expr::FunctionCall {
            span: _,
            name_span: _,
            name,
            args,
        } => {
            let name = match name {
                ir::FunctionName::Builtin(name, _) | ir::FunctionName::Imported(name, _) => {
                    name.as_str()
                }
            };

            println!("{}    ├── FunctionCall: \"{}\"", "  ".repeat(indent), name);
            for (i, arg) in args.iter().enumerate() {
                let is_last = i == args.len() - 1;
                let prefix = if is_last { "└──" } else { "├──" };
                println!("{}        {}Arg {}:", "  ".repeat(indent), prefix, i + 1);
                print_expression(arg, indent + 4);
            }
        }
        ir::Expr::Variable { span: _, variable } => {
            print_variable(variable, indent);
        }
        ir::Expr::Literal { span: _, value } => {
            print_literal(value, indent);
        }
        ir::Expr::ComparisonOp {
            span: _,
            op,
            left,
            right,
            rest_chained,
        } => {
            println!("{}    ├── ComparisonOp: {:?}", "  ".repeat(indent), op);
            print_expression(left, indent + 2);
            print_expression(right, indent + 2);
            for (i, (op, expr)) in rest_chained.iter().enumerate() {
                let is_last = i == rest_chained.len() - 1;
                let prefix = if is_last { "└──" } else { "├──" };
                println!("{}        {}Chained: {:?}", "  ".repeat(indent), prefix, op);
                print_expression(expr, indent + 4);
            }
        }
    }
}

/// Prints a variable
fn print_variable(var: &ir::Variable, indent: usize) {
    match var {
        ir::Variable::Builtin { ident, .. } => {
            println!(
                "{}    ├── Builtin Variable: \"{}\"",
                "  ".repeat(indent),
                ident.as_str()
            );
        }
        ir::Variable::Parameter { parameter_name, .. } => {
            println!(
                "{}    ├── Parameter Variable: \"{}\"",
                "  ".repeat(indent),
                parameter_name.as_str()
            );
        }
        ir::Variable::External {
            model,
            parameter_name,
            ..
        } => {
            println!(
                "{}    ├── External Variable: \"{}\" from \"{}\"",
                "  ".repeat(indent),
                parameter_name.as_str(),
                model.as_ref().display()
            );
        }
    }
}

/// Prints a literal
fn print_literal(lit: &ir::Literal, indent: usize) {
    match lit {
        ir::Literal::Number(n) => {
            println!("{}    ├── Literal: {}", "  ".repeat(indent), n);
        }
        ir::Literal::String(s) => {
            println!("{}    ├── Literal: \"{}\"", "  ".repeat(indent), s);
        }
        ir::Literal::Boolean(b) => {
            println!("{}    ├── Literal: {}", "  ".repeat(indent), b);
        }
    }
}

/// Prints a unit
fn print_unit(unit: &ir::CompositeUnit, indent: usize) {
    println!("{}    └── Unit: {:?}", "  ".repeat(indent), unit);
}

/// Prints tests
fn print_tests(tests: &HashMap<ir::TestIndex, ir::Test>, indent: usize) {
    for (i, (test_index, test)) in tests.iter().enumerate() {
        let is_last = i == tests.len() - 1;
        let prefix = if is_last { "└──" } else { "├──" };
        println!(
            "{}    {}Test {:?}:",
            "  ".repeat(indent),
            prefix,
            test_index
        );
        print_test(test, indent + 2);
    }
}

/// Prints a single test
fn print_test(test: &ir::Test, indent: usize) {
    // Print trace level
    println!(
        "{}    ├── Trace Level: {:?}",
        "  ".repeat(indent),
        test.trace_level()
    );

    // Print test expression
    println!("{}    └── Test Expression:", "  ".repeat(indent));
    print_expression(test.test_expr(), indent + 2);
}
