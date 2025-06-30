use std::collections::HashSet;

use oneil_ast as ast;
use oneil_module::{
    Dependency, Identifier, Module, ModulePath, ModuleReference, PythonPath, Reference,
    SectionDecl, SectionLabel, Symbol, TestDependency, TestInputs, test::Test,
};

mod util;
use util::{ModuleBuilder, TestInputsBuilder};

pub fn build_module(model: ast::Model, module_path: &ModulePath) -> Module {
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
    let import_path = module_path.get_sibling_path(&path);
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
    let use_path = module_path.get_sibling_path(&model_name);
    let use_path = ModulePath::new(use_path);

    // Build the test inputs
    let test_inputs = inputs
        .map(|inputs| {
            inputs
                .into_iter()
                .fold(TestInputsBuilder::new(), |mut test_inputs, input| {
                    let ident = Identifier::new(input.name);
                    let expr = input.value;
                    test_inputs.add_input(ident, expr);
                    test_inputs
                })
                .into_test_inputs()
        })
        .unwrap_or(TestInputs::empty());

    // Compute the symbol name
    let symbol_name = as_name
        .as_ref()
        .unwrap_or(subcomponents.last().unwrap_or(&model_name));
    let symbol_name = Identifier::new(symbol_name.clone());

    // Convert the subcomponent names to a reference
    let reference = subcomponents.into_iter().rfold(None, |acc, s| {
        let ident = Identifier::new(s);
        match acc {
            None => Some(Reference::Identifier(ident)),
            Some(reference) => Some(Reference::Accessor {
                parent: ident,
                component: Box::new(reference),
            }),
        }
    });

    // Build the symbol
    let symbol = Symbol::Import(ModuleReference::new(use_path.clone(), reference));

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
    // Get the parameter dependencies
    let exprs = get_parameter_exprs(&parameter);
    let dependencies = exprs.iter().fold(HashSet::new(), |dependencies, expr| {
        let dependencies = extract_expr_dependencies(expr, dependencies);
        dependencies
    });

    // Build the symbol
    let ident = Identifier::new(parameter.ident.clone());
    let symbol = Symbol::Parameter(parameter);

    // Add the symbol to the builder
    builder.add_symbol(ident.clone(), symbol);
    builder.add_parameter_dependencies(ident.clone(), dependencies);
    builder.add_section_decl(section_label, SectionDecl::Parameter(ident));
    builder
}

fn process_test_decl(
    mut builder: ModuleBuilder,
    section_label: SectionLabel,
    test: oneil_ast::Test,
) -> ModuleBuilder {
    // Get the test dependencies
    let expr = get_test_expr(&test);
    let dependencies = extract_expr_dependencies(expr, HashSet::new());
    let dependencies = sort_test_dependencies(&test, dependencies);

    // Get the test inputs
    let inputs = test
        .inputs
        .iter()
        .map(|input| Identifier::new(input.clone()))
        .collect();

    // Add the test to the builder
    let test_index = builder.add_model_test(Test::new(inputs, test));
    builder.add_test_dependencies(test_index.clone(), dependencies);
    builder.add_section_decl(section_label, SectionDecl::Test(test_index));
    builder
}

fn get_parameter_exprs(parameter: &ast::Parameter) -> Vec<&ast::Expr> {
    let mut exprs = vec![];

    // Extract the limit expressions
    match &parameter.limits {
        Some(ast::parameter::Limits::Continuous { min, max }) => {
            exprs.push(min);
            exprs.push(max);
        }
        Some(ast::parameter::Limits::Discrete { values }) => {
            values.iter().for_each(|value| {
                exprs.push(value);
            });
        }
        None => (),
    };

    // Extract the parameter value expression
    match &parameter.value {
        ast::parameter::ParameterValue::Simple(expr, _unit_expr) => {
            exprs.push(expr);
        }
        ast::parameter::ParameterValue::Piecewise(piecewise_expr, _unit_expr) => {
            // Extract the piecewise parts
            piecewise_expr.parts.iter().for_each(|part| {
                exprs.push(&part.expr);
                exprs.push(&part.if_expr);
            });
        }
    }

    exprs
}

fn get_test_expr(test: &ast::Test) -> &ast::Expr {
    &test.expr
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

fn sort_test_dependencies(
    test: &ast::Test,
    dependencies: HashSet<Reference>,
) -> HashSet<TestDependency> {
    dependencies
        .into_iter()
        .map(|dependency| match dependency {
            // TODO: `&ident.as_str().to_string()` feels like an unnecessary clone
            Reference::Identifier(ref ident)
            | Reference::Accessor {
                parent: ref ident, ..
            } if test.inputs.contains(&ident.as_str().to_string()) => {
                TestDependency::TestInput(dependency)
            }
            _ => TestDependency::Other(dependency),
        })
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;
    use oneil_ast::{
        declaration::ModelInput,
        expression::{BinaryOp, Literal, Variable},
        parameter::{Limits, ParameterValue},
    };
    use oneil_module::{DocumentationMap, ExternalImportList, SymbolMap, TestIndex};
    use std::{collections::HashMap, path::PathBuf};

    #[test]
    fn test_build_model_module_empty() {
        let module_path = ModulePath::new(PathBuf::from("test_module"));
        let model = ast::Model {
            note: None,
            decls: vec![],
            sections: vec![],
        };

        let module = build_module(model, &module_path);

        // Module path checks
        assert_eq!(module.path(), &module_path);

        // Symbol checks
        assert!(module.symbols().is_empty());

        // Test checks
        assert!(module.tests().model_tests().is_empty());
        assert!(module.tests().dependency_tests().is_empty());

        // External import checks
        assert!(module.external_imports().is_empty());

        // Documentation map checks
        assert_eq!(
            module.documentation_map(),
            &DocumentationMap::new(HashMap::new(), HashMap::new())
        );

        // Dependency checks
        assert!(module.dependencies().is_empty());
    }

    #[test]
    fn test_build_model_module_with_note() {
        let module_path = ModulePath::new(PathBuf::from("test_module"));
        let model = ast::Model {
            note: Some(ast::Note("Test model".to_string())),
            decls: vec![],
            sections: vec![],
        };

        let module = build_module(model, &module_path);

        // Module path checks
        assert_eq!(module.path(), &module_path);

        // Symbol checks
        assert!(module.symbols().is_empty());

        // Test checks
        assert!(module.tests().model_tests().is_empty());
        assert!(module.tests().dependency_tests().is_empty());

        // External import checks
        assert!(module.external_imports().is_empty());

        // Documentation map checks
        assert_eq!(
            module.documentation_map(),
            &DocumentationMap::new(
                HashMap::from([(
                    SectionLabel::new_top_level(),
                    ast::Note("Test model".to_string()),
                )]),
                HashMap::new(),
            )
        );

        // Dependency checks
        assert!(module.dependencies().is_empty());
    }

    #[test]
    fn test_build_model_module_with_imports() {
        let module_path = ModulePath::new(PathBuf::from("test_module"));
        let model = ast::Model {
            note: None,
            decls: vec![
                ast::Decl::Import {
                    path: "math_functions".to_string(),
                },
                ast::Decl::Import {
                    path: "physics_functions".to_string(),
                },
            ],
            sections: vec![],
        };

        let module = build_module(model, &module_path);

        // Module path checks
        assert_eq!(module.path(), &module_path);

        // Symbol checks
        assert_eq!(module.symbols(), &SymbolMap::empty());

        // Test checks
        assert!(module.tests().model_tests().is_empty());
        assert!(module.tests().dependency_tests().is_empty());

        // External import checks
        assert_eq!(
            module.external_imports(),
            &ExternalImportList::new(vec![
                PythonPath::new("math_functions".into()),
                PythonPath::new("physics_functions".into()),
            ])
        );

        // Documentation map checks
        assert_eq!(
            module
                .documentation_map()
                .section_decls(&SectionLabel::new_top_level()),
            Some(&vec![
                SectionDecl::ExternalImport(PythonPath::new("math_functions".into())),
                SectionDecl::ExternalImport(PythonPath::new("physics_functions".into())),
            ])
        );

        // Dependency checks
        assert_eq!(module.dependencies().len(), 2);

        assert!(
            module
                .dependencies()
                .contains(&Dependency::Python(PythonPath::new(
                    "math_functions".into()
                )))
        );

        assert!(
            module
                .dependencies()
                .contains(&Dependency::Python(PythonPath::new(
                    "physics_functions".into()
                )))
        );
    }

    #[test]
    fn test_build_model_module_with_use_models() {
        let module_path = ModulePath::new(PathBuf::from("test_module"));
        let model = ast::Model {
            note: None,
            decls: vec![
                ast::Decl::UseModel {
                    model_name: "submodel1".to_string(),
                    subcomponents: vec!["comp1".to_string()],
                    inputs: None,
                    as_name: None,
                },
                ast::Decl::UseModel {
                    model_name: "submodel2".to_string(),
                    subcomponents: vec!["comp1".to_string(), "comp2".to_string()],
                    inputs: Some(vec![ModelInput {
                        name: "input1".to_string(),
                        value: ast::Expr::Literal(Literal::Number(10.0)),
                    }]),
                    as_name: Some("alias".to_string()),
                },
                ast::Decl::UseModel {
                    model_name: "submodel3".to_string(),
                    subcomponents: vec![],
                    inputs: None,
                    as_name: None,
                },
            ],
            sections: vec![],
        };

        let module = build_module(model, &module_path);

        // Module path checks
        assert_eq!(module.path(), &module_path);

        // Symbol checks
        assert_eq!(module.symbols().len(), 3);

        assert_eq!(
            module.symbols().get(&Identifier::new("comp1".to_string())),
            Some(&Symbol::Import(ModuleReference::new(
                ModulePath::new(PathBuf::from("submodel1")),
                Some(Reference::identifier(Identifier::new("comp1".to_string()))),
            )))
        );

        assert_eq!(
            module.symbols().get(&Identifier::new("alias".to_string())),
            Some(&Symbol::Import(ModuleReference::new(
                ModulePath::new(PathBuf::from("submodel2")),
                Some(Reference::accessor(
                    Identifier::new("comp1".to_string()),
                    Reference::identifier(Identifier::new("comp2".to_string())),
                )),
            )))
        );

        assert_eq!(
            module
                .symbols()
                .get(&Identifier::new("submodel3".to_string())),
            Some(&Symbol::Import(ModuleReference::new(
                ModulePath::new(PathBuf::from("submodel3")),
                None,
            )))
        );

        // Test checks
        assert!(module.tests().model_tests().is_empty());
        assert_eq!(module.tests().dependency_tests().len(), 3);

        assert_eq!(
            module
                .tests()
                .dependency_tests()
                .get(&ModulePath::new(PathBuf::from("submodel1"))),
            Some(&TestInputs::empty())
        );

        assert_eq!(
            module
                .tests()
                .dependency_tests()
                .get(&ModulePath::new(PathBuf::from("submodel2"))),
            Some(&TestInputs::new(HashMap::from([(
                Identifier::new("input1".to_string()),
                ast::Expr::Literal(Literal::Number(10.0)),
            )])))
        );

        assert_eq!(
            module
                .tests()
                .dependency_tests()
                .get(&ModulePath::new(PathBuf::from("submodel3"))),
            Some(&TestInputs::empty())
        );

        // External import checks
        assert_eq!(module.external_imports(), &ExternalImportList::empty());

        // Documentation map checks
        let top_section_decls = vec![
            SectionDecl::InternalImport(Identifier::new("comp1".to_string())),
            SectionDecl::InternalImport(Identifier::new("alias".to_string())),
            SectionDecl::InternalImport(Identifier::new("submodel3".to_string())),
        ];

        assert_eq!(
            module.documentation_map(),
            &DocumentationMap::new(
                HashMap::new(),
                HashMap::from([(SectionLabel::new_top_level(), top_section_decls,)])
            )
        );

        // Dependency checks
        assert_eq!(module.dependencies().len(), 3);

        assert!(
            module
                .dependencies()
                .contains(&Dependency::Module(ModulePath::new(PathBuf::from(
                    "submodel1"
                ))))
        );

        assert!(
            module
                .dependencies()
                .contains(&Dependency::Module(ModulePath::new(PathBuf::from(
                    "submodel2"
                ))))
        );

        assert!(
            module
                .dependencies()
                .contains(&Dependency::Module(ModulePath::new(PathBuf::from(
                    "submodel3"
                ))))
        );
    }

    #[test]
    fn test_build_model_module_with_parameters() {
        let module_path = ModulePath::new(PathBuf::from("test_module"));

        let param1 = ast::Parameter {
            name: "Parameter 1".to_string(),
            ident: "param1".to_string(),
            value: ParameterValue::Simple(ast::Expr::Literal(Literal::Number(42.0)), None),
            limits: None,
            is_performance: false,
            trace_level: ast::parameter::TraceLevel::None,
            note: None,
        };

        let param2 = ast::Parameter {
            name: "Parameter 2".to_string(),
            ident: "param2".to_string(),
            value: ParameterValue::Simple(
                ast::Expr::Variable(Variable::Identifier("param1".to_string())),
                None,
            ),
            limits: Some(Limits::Continuous {
                min: ast::Expr::Literal(Literal::Number(0.0)),
                max: ast::Expr::Literal(Literal::Number(100.0)),
            }),
            is_performance: false,
            trace_level: ast::parameter::TraceLevel::None,
            note: None,
        };

        let model = ast::Model {
            note: None,
            decls: vec![
                ast::Decl::Parameter(param1.clone()),
                ast::Decl::Parameter(param2.clone()),
            ],
            sections: vec![],
        };

        let module = build_module(model, &module_path);

        // Module path checks
        assert_eq!(module.path(), &module_path);

        // Symbol checks
        assert_eq!(module.symbols().len(), 2);

        assert_eq!(
            module.symbols().get(&Identifier::new("param1".to_string())),
            Some(&Symbol::Parameter(param1))
        );

        assert_eq!(
            module.symbols().get(&Identifier::new("param2".to_string())),
            Some(&Symbol::Parameter(param2))
        );

        // Test checks
        assert!(module.tests().model_tests().is_empty());
        assert!(module.tests().dependency_tests().is_empty());

        // External import checks
        assert!(module.external_imports().is_empty());

        // Documentation map checks
        let top_section_decls = vec![
            SectionDecl::Parameter(Identifier::new("param1".to_string())),
            SectionDecl::Parameter(Identifier::new("param2".to_string())),
        ];

        assert_eq!(
            module.documentation_map(),
            &DocumentationMap::new(
                HashMap::new(),
                HashMap::from([(SectionLabel::new_top_level(), top_section_decls,)])
            )
        );

        // Dependency checks
        assert!(module.dependencies().is_empty());
    }

    #[test]
    fn test_build_model_module_with_tests() {
        let module_path = ModulePath::new(PathBuf::from("test_module"));

        let test1 = ast::Test {
            trace_level: ast::parameter::TraceLevel::None,
            inputs: vec![],
            expr: ast::Expr::Literal(Literal::Boolean(true)),
        };

        let test2 = ast::Test {
            trace_level: ast::parameter::TraceLevel::None,
            inputs: vec!["x".to_string()],
            expr: ast::Expr::BinaryOp {
                op: BinaryOp::GreaterThan,
                left: Box::new(ast::Expr::Literal(Literal::Number(1.0))),
                right: Box::new(ast::Expr::Literal(Literal::Number(2.0))),
            },
        };

        let model = ast::Model {
            note: None,
            decls: vec![
                ast::Decl::Test(test1.clone()),
                ast::Decl::Test(test2.clone()),
            ],
            sections: vec![],
        };

        let module = build_module(model, &module_path);

        // Module path checks
        assert_eq!(module.path(), &module_path);

        // Symbol checks
        assert!(module.symbols().is_empty());

        // Test checks
        let test1 = Test::new(vec![], test1);
        let test2 = Test::new(vec![Identifier::new("x".to_string())], test2);
        assert_eq!(module.tests().model_tests().len(), 2);
        assert!(module.tests().model_tests().contains(&test1));
        assert!(module.tests().model_tests().contains(&test2));
        assert!(module.tests().dependency_tests().is_empty());

        // External import checks
        assert!(module.external_imports().is_empty());

        // Documentation map checks
        let test1_index = module
            .tests()
            .model_tests()
            .iter()
            .position(|t| t == &test1)
            .unwrap();
        let test2_index = module
            .tests()
            .model_tests()
            .iter()
            .position(|t| t == &test2)
            .unwrap();

        assert_eq!(
            module
                .documentation_map()
                .section_decls(&SectionLabel::new_top_level()),
            Some(&vec![
                SectionDecl::Test(TestIndex::new(test1_index)),
                SectionDecl::Test(TestIndex::new(test2_index)),
            ])
        );

        // Dependency checks
        assert!(module.dependencies().is_empty());
    }

    #[test]
    fn test_build_model_module_with_sections() {
        let module_path = ModulePath::new(PathBuf::from("test_module"));

        let param1 = ast::Parameter {
            name: "Parameter 1".to_string(),
            ident: "param1".to_string(),
            value: ParameterValue::Simple(ast::Expr::Literal(Literal::Number(42.0)), None),
            note: None,
            limits: None,
            is_performance: false,
            trace_level: ast::parameter::TraceLevel::None,
        };

        let param2 = ast::Parameter {
            name: "Parameter 2".to_string(),
            ident: "param2".to_string(),
            value: ParameterValue::Simple(ast::Expr::Literal(Literal::Number(42.0)), None),
            note: None,
            limits: None,
            is_performance: false,
            trace_level: ast::parameter::TraceLevel::None,
        };

        let test1 = ast::Test {
            trace_level: ast::parameter::TraceLevel::None,
            inputs: vec![],
            expr: ast::Expr::Literal(Literal::Boolean(true)),
        };

        let test2 = ast::Test {
            trace_level: ast::parameter::TraceLevel::None,
            inputs: vec!["x".to_string()],
            expr: ast::Expr::BinaryOp {
                op: BinaryOp::GreaterThan,
                left: Box::new(ast::Expr::Literal(Literal::Number(1.0))),
                right: Box::new(ast::Expr::Literal(Literal::Number(2.0))),
            },
        };

        let model = ast::Model {
            note: Some(ast::Note("Main model".to_string())),
            decls: vec![
                ast::Decl::Import {
                    path: "math_functions".to_string(),
                },
                ast::Decl::UseModel {
                    model_name: "submodel1".to_string(),
                    subcomponents: vec![],
                    inputs: None,
                    as_name: None,
                },
                ast::Decl::Parameter(param1.clone()),
                ast::Decl::Test(test1.clone()),
            ],
            sections: vec![ast::model::Section {
                label: "section1".to_string(),
                note: Some(ast::Note("Section 1".to_string())),
                decls: vec![
                    ast::Decl::Import {
                        path: "physics_functions".to_string(),
                    },
                    ast::Decl::UseModel {
                        model_name: "submodel2".to_string(),
                        subcomponents: vec![],
                        inputs: None,
                        as_name: None,
                    },
                    ast::Decl::Parameter(param2.clone()),
                    ast::Decl::Test(test2.clone()),
                ],
            }],
        };

        let module = build_module(model, &module_path);

        // Module path checks
        assert_eq!(module.path(), &module_path);

        // Symbol checks
        assert_eq!(module.symbols().len(), 4);
        assert_eq!(
            module.symbols().get(&Identifier::new("param1".to_string())),
            Some(&Symbol::Parameter(param1))
        );

        assert_eq!(
            module.symbols().get(&Identifier::new("param2".to_string())),
            Some(&Symbol::Parameter(param2))
        );

        assert_eq!(
            module
                .symbols()
                .get(&Identifier::new("submodel1".to_string())),
            Some(&Symbol::Import(ModuleReference::new(
                ModulePath::new(PathBuf::from("submodel1")),
                None,
            )))
        );

        assert_eq!(
            module
                .symbols()
                .get(&Identifier::new("submodel2".to_string())),
            Some(&Symbol::Import(ModuleReference::new(
                ModulePath::new(PathBuf::from("submodel2")),
                None,
            )))
        );

        // Test checks
        let test1 = Test::new(vec![], test1);
        let test2 = Test::new(vec![Identifier::new("x".to_string())], test2);
        assert_eq!(module.tests().model_tests().len(), 2);
        assert!(module.tests().model_tests().contains(&test1));
        assert!(module.tests().model_tests().contains(&test2));
        assert_eq!(module.tests().dependency_tests().len(), 2);
        assert!(
            module
                .tests()
                .dependency_tests()
                .get(&ModulePath::new(PathBuf::from("submodel1")))
                .is_some()
        );
        assert!(
            module
                .tests()
                .dependency_tests()
                .get(&ModulePath::new(PathBuf::from("submodel2")))
                .is_some()
        );

        // External import checks
        assert_eq!(module.external_imports().len(), 2);
        assert!(
            module
                .external_imports()
                .contains(&PythonPath::new(PathBuf::from("math_functions")))
        );
        assert!(
            module
                .external_imports()
                .contains(&PythonPath::new(PathBuf::from("physics_functions")))
        );

        // Documentation map checks
        let doc_map = module.documentation_map();

        let test1_index = module
            .tests()
            .model_tests()
            .iter()
            .position(|t| t == &test1)
            .unwrap();

        let test2_index = module
            .tests()
            .model_tests()
            .iter()
            .position(|t| t == &test2)
            .unwrap();

        assert_eq!(
            doc_map.section_notes(&SectionLabel::new_top_level()),
            Some(&ast::Note("Main model".to_string()))
        );

        assert_eq!(
            doc_map.section_decls(&SectionLabel::new_top_level()),
            Some(&vec![
                SectionDecl::ExternalImport(PythonPath::new(PathBuf::from("math_functions"))),
                SectionDecl::InternalImport(Identifier::new("submodel1".to_string())),
                SectionDecl::Parameter(Identifier::new("param1".to_string())),
                SectionDecl::Test(TestIndex::new(test1_index)),
            ])
        );

        assert_eq!(
            doc_map.section_notes(&SectionLabel::new_subsection("section1".to_string())),
            Some(&ast::Note("Section 1".to_string()))
        );

        assert_eq!(
            doc_map.section_decls(&SectionLabel::new_subsection("section1".to_string())),
            Some(&vec![
                SectionDecl::ExternalImport(PythonPath::new(PathBuf::from("physics_functions"))),
                SectionDecl::InternalImport(Identifier::new("submodel2".to_string())),
                SectionDecl::Parameter(Identifier::new("param2".to_string())),
                SectionDecl::Test(TestIndex::new(test2_index)),
            ])
        );

        // Dependency checks
        assert_eq!(module.dependencies().len(), 4);

        assert!(
            module
                .dependencies()
                .contains(&Dependency::Module(ModulePath::new(PathBuf::from(
                    "submodel1"
                ))))
        );

        assert!(
            module
                .dependencies()
                .contains(&Dependency::Module(ModulePath::new(PathBuf::from(
                    "submodel2"
                ))))
        );

        assert!(
            module
                .dependencies()
                .contains(&Dependency::Python(PythonPath::new(
                    "math_functions".into()
                )))
        );

        assert!(
            module
                .dependencies()
                .contains(&Dependency::Python(PythonPath::new(
                    "physics_functions".into()
                )))
        );
    }

    #[test]
    fn test_extract_expr_dependencies_literal() {
        let expr = ast::Expr::Literal(Literal::Number(42.0));
        let dependencies = HashSet::new();
        let result = extract_expr_dependencies(&expr, dependencies);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_extract_expr_dependencies_variable() {
        let expr = ast::Expr::Variable(Variable::Identifier("test_var".to_string()));
        let dependencies = HashSet::new();
        let result = extract_expr_dependencies(&expr, dependencies);
        assert_eq!(result.len(), 1);
        assert!(result.contains(&Reference::Identifier(Identifier::new(
            "test_var".to_string()
        ))));
    }

    #[test]
    fn test_get_reference_for_variable_identifier() {
        let variable = Variable::Identifier("test_var".to_string());
        let reference = get_reference_for_variable(&variable);
        assert_eq!(
            reference,
            Reference::Identifier(Identifier::new("test_var".to_string()))
        );
    }

    #[test]
    fn test_get_reference_for_variable_accessor() {
        let variable = Variable::Accessor {
            parent: "parent".to_string(),
            component: Box::new(Variable::Identifier("child".to_string())),
        };
        let reference = get_reference_for_variable(&variable);
        assert_eq!(
            reference,
            Reference::Accessor {
                parent: Identifier::new("parent".to_string()),
                component: Box::new(Reference::Identifier(Identifier::new("child".to_string()))),
            }
        );
    }

    #[test]
    fn test_get_reference_for_variable_nested_accessor() {
        let variable = Variable::Accessor {
            parent: "parent".to_string(),
            component: Box::new(Variable::Accessor {
                parent: "child".to_string(),
                component: Box::new(Variable::Identifier("grandchild".to_string())),
            }),
        };
        let reference = get_reference_for_variable(&variable);
        assert_eq!(
            reference,
            Reference::Accessor {
                parent: Identifier::new("parent".to_string()),
                component: Box::new(Reference::Accessor {
                    parent: Identifier::new("child".to_string()),
                    component: Box::new(Reference::Identifier(Identifier::new(
                        "grandchild".to_string()
                    ))),
                }),
            }
        );
    }

    #[test]
    fn test_get_test_expr() {
        let test_expr = ast::Expr::Literal(Literal::Number(42.0));
        let test = ast::Test {
            trace_level: ast::parameter::TraceLevel::None,
            inputs: vec![],
            expr: test_expr.clone(),
        };

        let result = get_test_expr(&test);
        assert_eq!(result, &test_expr);
    }

    #[test]
    fn test_get_test_expr_with_complex_expression() {
        let test_expr = ast::Expr::BinaryOp {
            op: BinaryOp::Add,
            left: Box::new(ast::Expr::Literal(Literal::Number(1.0))),
            right: Box::new(ast::Expr::Literal(Literal::Number(2.0))),
        };
        let test = ast::Test {
            trace_level: ast::parameter::TraceLevel::None,
            inputs: vec!["x".to_string()],
            expr: test_expr.clone(),
        };

        let result = get_test_expr(&test);
        assert_eq!(result, &test_expr);
    }

    #[test]
    fn test_sort_test_dependencies_empty() {
        let test = ast::Test {
            trace_level: ast::parameter::TraceLevel::None,
            inputs: vec![],
            expr: ast::Expr::Literal(Literal::Boolean(true)),
        };
        let dependencies = HashSet::new();

        let result = sort_test_dependencies(&test, dependencies);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_sort_test_dependencies_no_test_inputs() {
        let test = ast::Test {
            trace_level: ast::parameter::TraceLevel::None,
            inputs: vec![],
            expr: ast::Expr::Literal(Literal::Boolean(true)),
        };
        let mut dependencies = HashSet::new();
        dependencies.insert(Reference::Identifier(Identifier::new("param1".to_string())));
        dependencies.insert(Reference::Identifier(Identifier::new("param2".to_string())));

        let result = sort_test_dependencies(&test, dependencies);
        assert_eq!(result.len(), 2);

        // All dependencies should be TestDependency::Other since they're not in test inputs
        for dependency in result {
            match dependency {
                TestDependency::Other(_) => (),
                TestDependency::TestInput(_) => panic!("Expected Other, got TestInput"),
            }
        }
    }

    #[test]
    fn test_sort_test_dependencies_with_test_inputs() {
        let test = ast::Test {
            trace_level: ast::parameter::TraceLevel::None,
            inputs: vec!["input1".to_string(), "input2".to_string()],
            expr: ast::Expr::Literal(Literal::Boolean(true)),
        };
        let mut dependencies = HashSet::new();
        dependencies.insert(Reference::Identifier(Identifier::new("input1".to_string())));
        dependencies.insert(Reference::Identifier(Identifier::new("param1".to_string())));
        dependencies.insert(Reference::Identifier(Identifier::new("input2".to_string())));

        let result = sort_test_dependencies(&test, dependencies);
        assert_eq!(result.len(), 3);

        let mut test_input_count = 0;
        let mut other_count = 0;

        for dependency in result {
            match dependency {
                TestDependency::TestInput(ref reference) => {
                    test_input_count += 1;
                    match reference {
                        Reference::Identifier(ident) => {
                            assert!(test.inputs.contains(&ident.as_str().to_string()));
                        }
                        _ => panic!("Unexpected reference type in TestInput"),
                    }
                }
                TestDependency::Other(ref reference) => {
                    other_count += 1;
                    match reference {
                        Reference::Identifier(ident) => {
                            assert!(!test.inputs.contains(&ident.as_str().to_string()));
                        }
                        _ => panic!("Unexpected reference type in Other"),
                    }
                }
            }
        }

        assert_eq!(test_input_count, 2);
        assert_eq!(other_count, 1);
    }

    #[test]
    fn test_sort_test_dependencies_with_accessor_references() {
        let test = ast::Test {
            trace_level: ast::parameter::TraceLevel::None,
            inputs: vec!["parent".to_string()],
            expr: ast::Expr::Literal(Literal::Boolean(true)),
        };
        let mut dependencies = HashSet::new();
        dependencies.insert(Reference::Accessor {
            parent: Identifier::new("parent".to_string()),
            component: Box::new(Reference::Identifier(Identifier::new("child".to_string()))),
        });
        dependencies.insert(Reference::Identifier(Identifier::new(
            "other_param".to_string(),
        )));

        let result = sort_test_dependencies(&test, dependencies);
        assert_eq!(result.len(), 2);

        let mut test_input_count = 0;
        let mut other_count = 0;

        for dependency in result {
            match dependency {
                TestDependency::TestInput(ref reference) => {
                    test_input_count += 1;
                    match reference {
                        Reference::Accessor { parent, .. } => {
                            assert!(test.inputs.contains(&parent.as_str().to_string()));
                        }
                        _ => panic!("Unexpected reference type in TestInput"),
                    }
                }
                TestDependency::Other(ref reference) => {
                    other_count += 1;
                    match reference {
                        Reference::Identifier(ident) => {
                            assert!(!test.inputs.contains(&ident.as_str().to_string()));
                        }
                        _ => panic!("Unexpected reference type in Other"),
                    }
                }
            }
        }

        assert_eq!(test_input_count, 1);
        assert_eq!(other_count, 1);
    }
}
