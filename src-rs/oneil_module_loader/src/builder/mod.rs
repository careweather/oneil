use std::collections::HashSet;

use oneil_ast as ast;
use oneil_module::{
    Dependency, Identifier, Module, ModulePath, ModuleReference, PythonPath, Reference,
    SectionDecl, SectionLabel, Symbol, TestInputs,
};

mod util;
use util::ModuleBuilder;

pub fn build_model_module(model: ast::Model, module_path: &ModulePath) -> Module {
    let module_builder = ModuleBuilder::new(module_path.clone());

    // Build the top-level section
    let module_builder = build_section(
        SectionLabel::new_top_level(),
        model.note,
        model.decls,
        module_path,
        module_builder,
    );

    // Build the subsections
    let module_builder =
        model
            .sections
            .into_iter()
            .fold(module_builder, |module_builder, section| {
                let section_label = SectionLabel::new_subsection(section.label);
                let module_builder = build_section(
                    section_label,
                    section.note,
                    section.decls,
                    module_path,
                    module_builder,
                );

                module_builder
            });

    // Build the module
    let module = module_builder.into_module();

    module
}

fn build_section(
    section_label: SectionLabel,
    note: Option<ast::Note>,
    decls: Vec<ast::Decl>,
    module_path: &ModulePath,
    mut builder: ModuleBuilder,
) -> ModuleBuilder {
    // Add the section note if it exists
    if let Some(note) = note {
        builder.add_section_note(section_label.clone(), note);
    }

    decls.into_iter().fold(builder, |builder, decl| {
        let section_label = section_label.clone();

        match decl {
            ast::Decl::Import { path } => {
                process_import_decl(section_label, module_path, path, builder)
            }

            ast::Decl::UseModel {
                model_name,
                subcomponents,
                inputs,
                as_name,
            } => process_use_model_decl(
                section_label,
                module_path,
                model_name,
                subcomponents,
                inputs,
                as_name,
                builder,
            ),

            ast::Decl::Parameter(parameter) => {
                process_parameter_decl(section_label, parameter, builder)
            }

            ast::Decl::Test(test) => process_test_decl(builder, section_label, test),
        }
    })
}

fn process_import_decl(
    section_label: SectionLabel,
    module_path: &ModulePath,
    path: String,
    mut builder: ModuleBuilder,
) -> ModuleBuilder {
    // Build the import path from the current module path and the given path
    let import_path = module_path.join(&path);
    let import_path = PythonPath::new(import_path);

    // Add the dependency and external import to the builder
    builder.add_dependency(Dependency::Python(import_path.clone()));
    builder.add_external_import(import_path.clone());
    builder.add_section_decl(section_label, SectionDecl::ExternalImport(import_path));
    builder
}

fn process_use_model_decl(
    section_label: SectionLabel,
    module_path: &ModulePath,
    model_name: String,
    subcomponents: Vec<String>,
    inputs: Option<Vec<oneil_ast::declaration::ModelInput>>,
    as_name: Option<String>,
    mut builder: ModuleBuilder,
) -> ModuleBuilder {
    // Build the use path from the current module path and the given model name
    let use_path = module_path.join(&model_name);
    let use_path = ModulePath::new(use_path);

    // Build the test inputs
    let test_inputs = inputs
        .map(|inputs| {
            inputs
                .into_iter()
                .fold(TestInputs::new(), |mut test_inputs, input| {
                    let ident = Identifier::new(input.name);
                    let expr = input.value;
                    test_inputs.add_input(ident, expr);
                    test_inputs
                })
        })
        .unwrap_or(TestInputs::new());

    // Compute the symbol name
    let symbol_name = as_name
        .as_ref()
        .unwrap_or(subcomponents.last().unwrap_or(&model_name));
    let symbol_name = Identifier::new(symbol_name.clone());

    // Convert the subcomponent names to identifiers
    let subcomponents = subcomponents
        .into_iter()
        .map(|s| Identifier::new(s))
        .collect::<Vec<_>>();

    // Build the symbol
    let symbol = Symbol::Import(ModuleReference::new(use_path.clone(), subcomponents));

    // Add the symbol, dependency, and dependency test to the builder
    builder.add_symbol(symbol_name.clone(), symbol);
    builder.add_dependency(Dependency::Module(use_path.clone()));
    builder.add_dependency_test(use_path, test_inputs);
    builder.add_section_decl(section_label, SectionDecl::InternalImport(symbol_name));
    builder
}

fn process_parameter_decl(
    section_label: SectionLabel,
    parameter: oneil_ast::Parameter,
    mut builder: ModuleBuilder,
) -> ModuleBuilder {
    // Build the symbol name
    let ident = Identifier::new(parameter.name.clone());

    // Extract the dependencies
    let dependencies = extract_parameter_dependencies(&parameter);
    let symbol = Symbol::Parameter {
        dependencies,
        parameter,
    };

    // Add the symbol to the builder
    builder.add_symbol(ident.clone(), symbol);
    builder.add_section_decl(section_label, SectionDecl::Parameter(ident));
    builder
}

fn process_test_decl(
    mut builder: ModuleBuilder,
    section_label: SectionLabel,
    test: oneil_ast::Test,
) -> ModuleBuilder {
    // Add the test to the builder
    let test_index = builder.add_model_test(test);
    builder.add_section_decl(section_label, SectionDecl::Test(test_index));
    builder
}

fn extract_parameter_dependencies(parameter: &ast::Parameter) -> HashSet<Reference> {
    let dependencies = HashSet::new();

    // Extract the dependencies from the limits
    let dependencies = match &parameter.limits {
        Some(ast::parameter::Limits::Continuous { min, max }) => {
            let dependencies = extract_expr_dependencies(min, dependencies);
            let dependencies = extract_expr_dependencies(max, dependencies);
            dependencies
        }
        Some(ast::parameter::Limits::Discrete { values }) => {
            values.iter().fold(dependencies, |dependencies, value| {
                extract_expr_dependencies(value, dependencies)
            })
        }
        None => dependencies,
    };

    // Extract the dependencies from the parameter value
    match &parameter.value {
        ast::parameter::ParameterValue::Simple(expr, _unit_expr) => {
            extract_expr_dependencies(expr, dependencies)
        }
        ast::parameter::ParameterValue::Piecewise(piecewise_expr, _unit_expr) => {
            // Extract the dependencies from each part of the piecewise expression
            let dependencies =
                piecewise_expr
                    .parts
                    .iter()
                    .fold(dependencies, |dependencies, part| {
                        let dependencies = extract_expr_dependencies(&part.expr, dependencies);
                        let dependencies = extract_expr_dependencies(&part.if_expr, dependencies);
                        dependencies
                    });

            dependencies
        }
    }
}

fn extract_expr_dependencies(
    expr: &ast::Expr,
    dependencies: HashSet<Reference>,
) -> HashSet<Reference> {
    match expr {
        oneil_ast::Expr::BinaryOp { op: _, left, right } => {
            // Extract the dependencies from the left and right expressions
            let dependencies = extract_expr_dependencies(left, dependencies);
            let dependencies = extract_expr_dependencies(right, dependencies);

            // Return the dependencies
            dependencies
        }
        oneil_ast::Expr::UnaryOp { op: _, expr } => {
            // Extract the dependencies from the unary expression
            let dependencies = extract_expr_dependencies(expr, dependencies);
            dependencies
        }
        oneil_ast::Expr::FunctionCall { name: _, args } => {
            // Extract the dependencies from each argument
            args.iter().fold(dependencies, |dependencies, arg| {
                extract_expr_dependencies(arg, dependencies)
            })
        }
        oneil_ast::Expr::Literal(_literal) => dependencies,
        oneil_ast::Expr::Variable(accessors) => {
            // Convert the accessors to a reference
            let reference = get_reference_for_variable(accessors);

            // Add the reference to the dependencies
            let mut dependencies = dependencies;
            dependencies.insert(reference);
            dependencies
        }
    }
}

fn get_reference_for_variable(variable: &ast::expression::Variable) -> Reference {
    match variable {
        ast::expression::Variable::Identifier(ident) => {
            Reference::Identifier(Identifier::new(ident.clone()))
        }
        ast::expression::Variable::Accessor { parent, component } => Reference::Accessor {
            parent: Identifier::new(parent.clone()),
            component: Box::new(get_reference_for_variable(component)),
        },
    }
}
