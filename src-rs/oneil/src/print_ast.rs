//! Abstract Syntax Tree (AST) printing functionality for the Oneil CLI

use oneil_ast as ast;

use anstream::println;

/// Prints the AST in a hierarchical tree format for debugging
pub fn print(ast: &ast::Model, print_debug: bool) {
    if print_debug {
        println!("AST: {ast:?}");
    } else {
        print_model(ast, 0);
    }
}

/// Prints a model node with its declarations and sections
fn print_model(model: &ast::Model, indent: usize) {
    println!("{}Model", "  ".repeat(indent));

    // Print note if present
    if let Some(note) = model.note() {
        println!("{}├── Note: \"{}\"", "  ".repeat(indent), note.value());
    }

    // Print declarations
    if !model.decls().is_empty() {
        println!("{}├── Declarations:", "  ".repeat(indent));
        for (i, decl) in model.decls().iter().enumerate() {
            let is_last = i == model.decls().len() - 1 && model.sections().is_empty();
            let prefix = if is_last { "└──" } else { "├──" };
            print_decl(decl, indent + 2, prefix);
        }
    }

    // Print sections
    if !model.sections().is_empty() {
        for (i, section) in model.sections().iter().enumerate() {
            let is_last = i == model.sections().len() - 1;
            let prefix = if is_last { "└──" } else { "├──" };
            print_section(section, indent, prefix);
        }
    }
}

/// Prints a declaration node
fn print_decl(decl: &ast::DeclNode, indent: usize, prefix: &str) {
    match &**decl {
        ast::Decl::Import(import) => {
            println!(
                "{}{} Import: \"{}\"",
                "  ".repeat(indent),
                prefix,
                import.path().as_str()
            );
        }
        ast::Decl::UseModel(use_model) => {
            let alias = use_model.model_info().get_alias();

            let alias = format!(" as {}", alias.as_str());
            println!(
                "{}{} UseModel: \"{}\"{}",
                "  ".repeat(indent),
                prefix,
                use_model.model_info().top_component().as_str(),
                alias
            );

            // Print subcomponents if any
            if !use_model.model_info().subcomponents().is_empty() {
                let subcomps: Vec<String> = use_model
                    .model_info()
                    .subcomponents()
                    .iter()
                    .map(|id| id.as_str().to_string())
                    .collect();
                println!(
                    "{}    └── Subcomponents: [{}]",
                    "  ".repeat(indent),
                    subcomps.join(", ")
                );
            }
        }
        ast::Decl::Parameter(param) => {
            println!(
                "{}{} Parameter: {}",
                "  ".repeat(indent),
                prefix,
                param.ident().as_str()
            );

            // Print parameter details
            println!(
                "{}    ├── Label: \"{}\"",
                "  ".repeat(indent),
                param.label().as_str()
            );

            // Print parameter value
            println!("{}    ├── Value:", "  ".repeat(indent));
            print_parameter_value(param.value(), indent + 4);

            // Print limits if any
            if let Some(limits) = param.limits() {
                println!("{}    ├── Limits:", "  ".repeat(indent));
                print_limits(limits, indent + 4);
            }

            // Print performance marker if any
            if param.performance_marker().is_some() {
                println!("{}    ├── Performance Marker", "  ".repeat(indent));
            }

            // Print trace level if any
            if let Some(trace_level) = param.trace_level() {
                println!(
                    "{}    ├── Trace Level: {:?}",
                    "  ".repeat(indent),
                    trace_level
                );
            }

            // Print note if any
            if let Some(note) = param.note() {
                println!("{}    └── Note: \"{}\"", "  ".repeat(indent), note.value());
            }
        }
        ast::Decl::Test(test) => {
            println!("{}{} Test:", "  ".repeat(indent), prefix);

            if let Some(trace_level) = test.trace_level() {
                println!(
                    "{}    ├── Trace Level: {:?}",
                    "  ".repeat(indent),
                    trace_level
                );
            }

            println!("{}    └── Expression:", "  ".repeat(indent));
            print_expression(test.expr(), indent + 4);
        }
    }
}

/// Prints a section node
fn print_section(section: &ast::SectionNode, indent: usize, prefix: &str) {
    println!(
        "{}{} Section: \"{}\"",
        "  ".repeat(indent),
        prefix,
        section.header().label().as_str()
    );

    // Print section note if present
    if let Some(note) = section.note() {
        println!("{}    ├── Note: \"{}\"", "  ".repeat(indent), note.value());
    }

    // Print section declarations
    if !section.decls().is_empty() {
        for (i, decl) in section.decls().iter().enumerate() {
            let is_last = i == section.decls().len() - 1;
            let sub_prefix = if is_last { "└──" } else { "├──" };
            print_decl(decl, indent + 2, sub_prefix);
        }
    }
}

/// Prints an expression node
fn print_expression(expr: &ast::ExprNode, indent: usize) {
    match &**expr {
        ast::Expr::BinaryOp { op, left, right } => {
            println!("{}BinaryOp: {:?}", "  ".repeat(indent), op);
            print_expression(left, indent + 2);
            print_expression(right, indent + 2);
        }
        ast::Expr::UnaryOp { op, expr } => {
            println!("{}UnaryOp: {:?}", "  ".repeat(indent), op);
            print_expression(expr, indent + 2);
        }
        ast::Expr::FunctionCall { name, args } => {
            println!("{}FunctionCall: \"{}\"", "  ".repeat(indent), name.as_str());
            for (i, arg) in args.iter().enumerate() {
                let is_last = i == args.len() - 1;
                let prefix = if is_last { "└──" } else { "├──" };
                println!("{}    {}Arg {}:", "  ".repeat(indent), prefix, i + 1);
                print_expression(arg, indent + 4);
            }
        }
        ast::Expr::Parenthesized { expr } => {
            println!("{}Parenthesized:", "  ".repeat(indent));
            print_expression(expr, indent + 2);
        }
        ast::Expr::ComparisonOp {
            op,
            left,
            right,
            rest_chained,
        } => {
            println!("{}ComparisonOp: {:?}", "  ".repeat(indent), op);
            print_expression(left, indent + 2);
            print_expression(right, indent + 2);
            for (i, (op, expr)) in rest_chained.iter().enumerate() {
                let is_last = i == rest_chained.len() - 1;
                let prefix = if is_last { "└──" } else { "├──" };
                println!("{}    {}Chained: {:?}", "  ".repeat(indent), prefix, op);
                print_expression(expr, indent + 4);
            }
        }
        ast::Expr::Variable(var) => {
            print_variable(var, indent);
        }
        ast::Expr::Literal(lit) => {
            print_literal(lit, indent);
        }
    }
}

/// Prints a variable node
fn print_variable(var: &ast::VariableNode, indent: usize) {
    match &**var {
        ast::Variable::Identifier(id) => {
            println!("{}Variable: \"{}\"", "  ".repeat(indent), id.as_str());
        }
        ast::Variable::ModelParameter {
            reference_model,
            parameter,
        } => {
            println!(
                "{}ReferenceModelParameter: \"{}.{}\"",
                "  ".repeat(indent),
                reference_model.as_str(),
                parameter.as_str()
            );
        }
    }
}

/// Prints a literal node
fn print_literal(lit: &ast::LiteralNode, indent: usize) {
    match &**lit {
        ast::Literal::Number(n) => {
            println!("{}Literal: {}", "  ".repeat(indent), n);
        }
        ast::Literal::String(s) => {
            println!("{}Literal: \"{}\"", "  ".repeat(indent), s);
        }
        ast::Literal::Boolean(b) => {
            println!("{}Literal: {}", "  ".repeat(indent), b);
        }
    }
}

/// Prints a parameter value node
fn print_parameter_value(value: &ast::ParameterValueNode, indent: usize) {
    match &**value {
        ast::ParameterValue::Simple(expr, unit) => {
            println!("{}Simple:", "  ".repeat(indent));
            print_expression(expr, indent + 2);
            if let Some(unit_expr) = unit {
                println!("{}Unit:", "  ".repeat(indent));
                print_unit_expression(unit_expr, indent + 2);
            }
        }
        ast::ParameterValue::Piecewise(parts, unit) => {
            println!("{}Piecewise:", "  ".repeat(indent));
            for (i, part) in parts.iter().enumerate() {
                let is_last = i == parts.len() - 1;
                let prefix = if is_last { "└──" } else { "├──" };
                println!("{}    {}Part {}:", "  ".repeat(indent), prefix, i + 1);
                println!("{}        ├── Expression:", "  ".repeat(indent));
                print_expression(part.expr(), indent + 6);
                println!("{}        └── Condition:", "  ".repeat(indent));
                print_expression(part.if_expr(), indent + 6);
            }
            if let Some(unit_expr) = unit {
                println!("{}    └── Unit:", "  ".repeat(indent));
                print_unit_expression(unit_expr, indent + 4);
            }
        }
    }
}

/// Prints a limits node
fn print_limits(limits: &ast::LimitsNode, indent: usize) {
    match &**limits {
        ast::Limits::Continuous { min, max } => {
            println!("{}Continuous:", "  ".repeat(indent));
            println!("{}    ├── Min:", "  ".repeat(indent));
            print_expression(min, indent + 4);
            println!("{}    └── Max:", "  ".repeat(indent));
            print_expression(max, indent + 4);
        }
        ast::Limits::Discrete { values } => {
            println!("{}Discrete:", "  ".repeat(indent));
            for (i, value) in values.iter().enumerate() {
                let is_last = i == values.len() - 1;
                let prefix = if is_last { "└──" } else { "├──" };
                println!("{}    {}Value {}:", "  ".repeat(indent), prefix, i + 1);
                print_expression(value, indent + 4);
            }
        }
    }
}

/// Prints a unit expression node
fn print_unit_expression(unit_expr: &ast::UnitExprNode, indent: usize) {
    match &**unit_expr {
        ast::UnitExpr::BinaryOp { op, left, right } => {
            println!("{}BinaryOp: {:?}", "  ".repeat(indent), op);
            print_unit_expression(left, indent + 2);
            print_unit_expression(right, indent + 2);
        }
        ast::UnitExpr::Parenthesized { expr } => {
            println!("{}Parenthesized:", "  ".repeat(indent));
            print_unit_expression(expr, indent + 2);
        }
        ast::UnitExpr::Unit {
            identifier,
            exponent,
        } => {
            println!("{}Unit: \"{}\"", "  ".repeat(indent), identifier.as_str());
            if let Some(exp) = exponent {
                println!("{}    └── Exponent: {}", "  ".repeat(indent), exp.value());
            }
        }
        ast::UnitExpr::UnitOne => {
            println!("{}Unit: 1", "  ".repeat(indent));
        }
    }
}
