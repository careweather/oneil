use oneil_ast::{
    Decl, Expr, Model,
    declaration::DeclNode,
    expression::{ExprNode, Literal, LiteralNode, Variable, VariableNode},
    model::SectionNode,
};
use std::fmt::Write;

/// Prints the AST in a hierarchical tree format for debugging
pub fn print(ast: &Model, print_debug: bool) {
    if print_debug {
        println!("AST: {:?}", ast);
        return;
    }

    let mut output = String::new();
    print_model(ast, &mut output, 0);
    println!("{}", output);
}

/// Prints a model node with its declarations and sections
fn print_model(model: &Model, output: &mut String, indent: usize) {
    writeln!(output, "{}Model", "  ".repeat(indent)).unwrap();

    // Print note if present
    if let Some(note) = model.note() {
        writeln!(
            output,
            "{}├── Note: \"{}\"",
            "  ".repeat(indent),
            note.value()
        )
        .unwrap();
    }

    // Print declarations
    if !model.decls().is_empty() {
        writeln!(output, "{}├── Declarations:", "  ".repeat(indent)).unwrap();
        for (i, decl) in model.decls().iter().enumerate() {
            let is_last = i == model.decls().len() - 1 && model.sections().is_empty();
            let prefix = if is_last { "└──" } else { "├──" };
            print_decl(decl, output, indent + 2, prefix);
        }
    }

    // Print sections
    if !model.sections().is_empty() {
        for (i, section) in model.sections().iter().enumerate() {
            let is_last = i == model.sections().len() - 1;
            let prefix = if is_last { "└──" } else { "├──" };
            print_section(section, output, indent, prefix);
        }
    }
}

/// Prints a declaration node
fn print_decl(decl: &DeclNode, output: &mut String, indent: usize, prefix: &str) {
    match decl.node_value() {
        Decl::Import(import) => {
            writeln!(
                output,
                "{}{} Import: \"{}\"",
                "  ".repeat(indent),
                prefix,
                import.path()
            )
            .unwrap();
        }
        Decl::UseModel(use_model) => {
            let alias = use_model
                .alias()
                .map(|a| format!(" as {}", a.node_value().as_str()))
                .unwrap_or_default();
            writeln!(
                output,
                "{}{} UseModel: \"{}\"{}",
                "  ".repeat(indent),
                prefix,
                use_model.model_name().node_value().as_str(),
                alias
            )
            .unwrap();

            // Print subcomponents if any
            if !use_model.subcomponents().is_empty() {
                let subcomps: Vec<String> = use_model
                    .subcomponents()
                    .iter()
                    .map(|id| id.node_value().as_str().to_string())
                    .collect();
                writeln!(
                    output,
                    "{}    └── Subcomponents: [{}]",
                    "  ".repeat(indent),
                    subcomps.join(", ")
                )
                .unwrap();
            }

            // Print inputs if any
            if let Some(inputs) = use_model.inputs() {
                writeln!(output, "{}    └── Inputs:", "  ".repeat(indent)).unwrap();
                for input in inputs.inputs() {
                    writeln!(
                        output,
                        "{}        └── {} = ",
                        "  ".repeat(indent),
                        input.ident().node_value().as_str()
                    )
                    .unwrap();
                    print_expression(input.value(), output, indent + 6);
                }
            }
        }
        Decl::Parameter(param) => {
            writeln!(
                output,
                "{}{} Parameter: {}",
                "  ".repeat(indent),
                prefix,
                param.ident().node_value().as_str()
            )
            .unwrap();

            // Print parameter details
            writeln!(
                output,
                "{}    ├── Label: \"{}\"",
                "  ".repeat(indent),
                param.label().node_value().as_str()
            )
            .unwrap();

            // Print parameter value
            writeln!(output, "{}    ├── Value:", "  ".repeat(indent)).unwrap();
            print_parameter_value(param.value(), output, indent + 4);

            // Print limits if any
            if let Some(limits) = param.limits() {
                writeln!(output, "{}    ├── Limits:", "  ".repeat(indent)).unwrap();
                print_limits(limits, output, indent + 4);
            }

            // Print performance marker if any
            if param.performance_marker().is_some() {
                writeln!(output, "{}    ├── Performance Marker", "  ".repeat(indent)).unwrap();
            }

            // Print trace level if any
            if let Some(trace_level) = param.trace_level() {
                writeln!(
                    output,
                    "{}    ├── Trace Level: {:?}",
                    "  ".repeat(indent),
                    trace_level.node_value()
                )
                .unwrap();
            }

            // Print note if any
            if let Some(note) = param.note() {
                writeln!(
                    output,
                    "{}    └── Note: \"{}\"",
                    "  ".repeat(indent),
                    note.value()
                )
                .unwrap();
            }
        }
        Decl::Test(test) => {
            writeln!(output, "{}{} Test:", "  ".repeat(indent), prefix).unwrap();

            if let Some(trace_level) = test.trace_level() {
                writeln!(
                    output,
                    "{}    ├── Trace Level: {:?}",
                    "  ".repeat(indent),
                    trace_level.node_value()
                )
                .unwrap();
            }

            if let Some(inputs) = test.inputs() {
                let input_names: Vec<String> = inputs
                    .iter()
                    .map(|id| id.node_value().as_str().to_string())
                    .collect();
                writeln!(
                    output,
                    "{}    ├── Inputs: [{}]",
                    "  ".repeat(indent),
                    input_names.join(", ")
                )
                .unwrap();
            }

            writeln!(output, "{}    └── Expression:", "  ".repeat(indent)).unwrap();
            print_expression(test.expr(), output, indent + 4);
        }
    }
}

/// Prints a section node
fn print_section(section: &SectionNode, output: &mut String, indent: usize, prefix: &str) {
    writeln!(
        output,
        "{}{} Section: \"{}\"",
        "  ".repeat(indent),
        prefix,
        section.header().label().node_value().as_str()
    )
    .unwrap();

    // Print section note if present
    if let Some(note) = section.note() {
        writeln!(
            output,
            "{}    ├── Note: \"{}\"",
            "  ".repeat(indent),
            note.value()
        )
        .unwrap();
    }

    // Print section declarations
    if !section.decls().is_empty() {
        for (i, decl) in section.decls().iter().enumerate() {
            let is_last = i == section.decls().len() - 1;
            let sub_prefix = if is_last { "└──" } else { "├──" };
            print_decl(decl, output, indent + 2, sub_prefix);
        }
    }
}

/// Prints an expression node
fn print_expression(expr: &ExprNode, output: &mut String, indent: usize) {
    match expr.node_value() {
        Expr::BinaryOp { op, left, right } => {
            writeln!(
                output,
                "{}BinaryOp: {:?}",
                "  ".repeat(indent),
                op.node_value()
            )
            .unwrap();
            print_expression(left, output, indent + 2);
            print_expression(right, output, indent + 2);
        }
        Expr::UnaryOp { op, expr } => {
            writeln!(
                output,
                "{}UnaryOp: {:?}",
                "  ".repeat(indent),
                op.node_value()
            )
            .unwrap();
            print_expression(expr, output, indent + 2);
        }
        Expr::FunctionCall { name, args } => {
            writeln!(
                output,
                "{}FunctionCall: \"{}\"",
                "  ".repeat(indent),
                name.node_value().as_str()
            )
            .unwrap();
            for (i, arg) in args.iter().enumerate() {
                let is_last = i == args.len() - 1;
                let prefix = if is_last { "└──" } else { "├──" };
                writeln!(
                    output,
                    "{}    {}Arg {}:",
                    "  ".repeat(indent),
                    prefix,
                    i + 1
                )
                .unwrap();
                print_expression(arg, output, indent + 4);
            }
        }
        Expr::Parenthesized { expr } => {
            writeln!(output, "{}Parenthesized:", "  ".repeat(indent)).unwrap();
            print_expression(expr, output, indent + 2);
        }
        Expr::Variable(var) => {
            print_variable(var, output, indent);
        }
        Expr::Literal(lit) => {
            print_literal(lit, output, indent);
        }
    }
}

/// Prints a variable node
fn print_variable(var: &VariableNode, output: &mut String, indent: usize) {
    match var.node_value() {
        Variable::Identifier(id) => {
            writeln!(
                output,
                "{}Variable: \"{}\"",
                "  ".repeat(indent),
                id.node_value().as_str()
            )
            .unwrap();
        }
        Variable::Accessor { parent, component } => {
            writeln!(
                output,
                "{}Accessor: \"{}\"",
                "  ".repeat(indent),
                parent.node_value().as_str()
            )
            .unwrap();
            print_variable(component, output, indent + 2);
        }
    }
}

/// Prints a literal node
fn print_literal(lit: &LiteralNode, output: &mut String, indent: usize) {
    match lit.node_value() {
        Literal::Number(n) => {
            writeln!(output, "{}Literal: {}", "  ".repeat(indent), n).unwrap();
        }
        Literal::String(s) => {
            writeln!(output, "{}Literal: \"{}\"", "  ".repeat(indent), s).unwrap();
        }
        Literal::Boolean(b) => {
            writeln!(output, "{}Literal: {}", "  ".repeat(indent), b).unwrap();
        }
    }
}

/// Prints a parameter value node
fn print_parameter_value(
    value: &oneil_ast::parameter::ParameterValueNode,
    output: &mut String,
    indent: usize,
) {
    match value.node_value() {
        oneil_ast::parameter::ParameterValue::Simple(expr, unit) => {
            writeln!(output, "{}Simple:", "  ".repeat(indent)).unwrap();
            print_expression(expr, output, indent + 2);
            if let Some(unit_expr) = unit {
                writeln!(output, "{}Unit:", "  ".repeat(indent)).unwrap();
                print_unit_expression(unit_expr, output, indent + 2);
            }
        }
        oneil_ast::parameter::ParameterValue::Piecewise(parts, unit) => {
            writeln!(output, "{}Piecewise:", "  ".repeat(indent)).unwrap();
            for (i, part) in parts.iter().enumerate() {
                let is_last = i == parts.len() - 1;
                let prefix = if is_last { "└──" } else { "├──" };
                writeln!(
                    output,
                    "{}    {}Part {}:",
                    "  ".repeat(indent),
                    prefix,
                    i + 1
                )
                .unwrap();
                writeln!(output, "{}        ├── Expression:", "  ".repeat(indent)).unwrap();
                print_expression(part.expr(), output, indent + 6);
                writeln!(output, "{}        └── Condition:", "  ".repeat(indent)).unwrap();
                print_expression(part.if_expr(), output, indent + 6);
            }
            if let Some(unit_expr) = unit {
                writeln!(output, "{}    └── Unit:", "  ".repeat(indent)).unwrap();
                print_unit_expression(unit_expr, output, indent + 4);
            }
        }
    }
}

/// Prints a limits node
fn print_limits(limits: &oneil_ast::parameter::LimitsNode, output: &mut String, indent: usize) {
    match limits.node_value() {
        oneil_ast::parameter::Limits::Continuous { min, max } => {
            writeln!(output, "{}Continuous:", "  ".repeat(indent)).unwrap();
            writeln!(output, "{}    ├── Min:", "  ".repeat(indent)).unwrap();
            print_expression(min, output, indent + 4);
            writeln!(output, "{}    └── Max:", "  ".repeat(indent)).unwrap();
            print_expression(max, output, indent + 4);
        }
        oneil_ast::parameter::Limits::Discrete { values } => {
            writeln!(output, "{}Discrete:", "  ".repeat(indent)).unwrap();
            for (i, value) in values.iter().enumerate() {
                let is_last = i == values.len() - 1;
                let prefix = if is_last { "└──" } else { "├──" };
                writeln!(
                    output,
                    "{}    {}Value {}:",
                    "  ".repeat(indent),
                    prefix,
                    i + 1
                )
                .unwrap();
                print_expression(value, output, indent + 4);
            }
        }
    }
}

/// Prints a unit expression node
fn print_unit_expression(
    unit_expr: &oneil_ast::unit::UnitExprNode,
    output: &mut String,
    indent: usize,
) {
    // This is a placeholder - implement based on UnitExpr structure
    writeln!(
        output,
        "{}UnitExpr: {:?}",
        "  ".repeat(indent),
        unit_expr.node_value()
    )
    .unwrap();
}
