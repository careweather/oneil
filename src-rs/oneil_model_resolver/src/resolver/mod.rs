//! Model loading and resolution functionality for Oneil programs.
//!
//! This module provides the core functionality for loading Oneil models from files,
//! resolving dependencies, and building model collections. It handles:
//!
//! - Circular dependency detection
//! - Model parsing and AST processing
//! - Import validation
//! - Submodel resolution
//! - Parameter resolution
//! - Test resolution
//!

use oneil_ast as ast;
use oneil_ir as ir;

use crate::{ExternalResolutionContext, ResolutionContext, error::CircularDependencyError};

mod resolve_expr;
mod resolve_model_import;
mod resolve_parameter;
mod resolve_python_import;
mod resolve_test;
mod resolve_trace_level;
mod resolve_unit;
mod resolve_variable;

/// Loads a model and all its dependencies, building a complete model collection.
pub fn load_model<E>(model_path: &ir::ModelPath, resolution_context: &mut ResolutionContext<'_, E>)
where
    E: ExternalResolutionContext,
{
    // check for circular dependencies
    //
    // this happens before we check if the model has been visited because if
    // there is a circular dependency, it will have already been visited
    if resolution_context.is_model_active(model_path) {
        // find the circular dependency path (stack from model_path to top, then model_path again to close the cycle)
        let active_models = resolution_context.active_models();
        let mut circular_dependency: Vec<ir::ModelPath> = active_models
            .iter()
            .skip_while(|m| *m != model_path)
            .cloned()
            .collect();
        circular_dependency.push(model_path.clone());

        // add the circular dependency error
        let error = CircularDependencyError::new(circular_dependency);
        resolution_context.add_circular_dependency_error_to_active_model(error);
        return;
    }

    // check if model is already been visited, then mark as visited if not
    if resolution_context.has_visited_model(model_path) {
        return;
    }

    // push the model onto the active models stack
    resolution_context.push_active_model(model_path);

    // parse model ast
    let load_ast_result = resolution_context.load_ast(model_path);

    // TODO: this might be able to recover and produce a partial model?
    let Ok(model_ast) = load_ast_result else {
        resolution_context.mark_ast_not_loaded(model_path);
        resolution_context.pop_active_model(model_path);
        return;
    };

    // split model ast into imports, use models, parameters, and tests
    let (imports, model_imports, parameters, tests) = split_model_ast(&model_ast);

    // resolve python imports
    resolve_python_import::resolve_python_imports(model_path, imports, resolution_context);

    // load the models imported
    load_model_imports(model_path, &model_imports, resolution_context);

    // resolve submodels and references to external models
    resolve_model_import::resolve_model_imports(model_path, model_imports, resolution_context);

    // resolve parameters
    resolve_parameter::resolve_parameters(parameters, resolution_context);

    // resolve tests
    resolve_test::resolve_tests(tests, resolution_context);

    // pop the model from the active models stack
    resolution_context.pop_active_model(model_path);
}

/// Splits a model AST into its constituent declaration types.
///
/// This function processes the declarations in a model AST and categorizes them into
/// separate collections for imports, use models, parameters, and tests. It processes
/// both top-level declarations and declarations within sections.
///
/// # Arguments
///
/// * `model_ast` - The parsed model AST containing all declarations
///
/// # Returns
///
/// A tuple containing:
/// * `Vec<&ImportNode>` - All import declarations from the model
/// * `Vec<&UseModelNode>` - All use model declarations from the model
/// * `Vec<&ParameterNode>` - All parameter declarations from the model
/// * `Vec<&TestNode>` - All test declarations from the model
///
/// # Behavior
///
/// The function processes declarations in the following order:
/// 1. Top-level declarations in the model
/// 2. Declarations within each section of the model
///
/// This separation is necessary for the different processing steps in model loading.
fn split_model_ast(
    model_ast: &ast::Model,
) -> (
    Vec<&ast::ImportNode>,
    Vec<&ast::UseModelNode>,
    Vec<&ast::ParameterNode>,
    Vec<&ast::TestNode>,
) {
    let mut imports = vec![];
    let mut use_models = vec![];
    let mut parameters = vec![];
    let mut tests = vec![];

    for decl in model_ast.decls() {
        match &**decl {
            ast::Decl::Import(import) => imports.push(import),
            ast::Decl::UseModel(use_model) => use_models.push(use_model),
            ast::Decl::Parameter(parameter) => parameters.push(parameter),
            ast::Decl::Test(test) => tests.push(test),
        }
    }

    for section in model_ast.sections() {
        for decl in section.decls() {
            match &**decl {
                ast::Decl::Import(import) => imports.push(import),
                ast::Decl::UseModel(use_model) => use_models.push(use_model),
                ast::Decl::Parameter(parameter) => parameters.push(parameter),
                ast::Decl::Test(test) => tests.push(test),
            }
        }
    }

    (imports, use_models, parameters, tests)
}

/// Recursively loads all use models referenced by a model.
///
/// This function processes all use model declarations in a model and recursively
/// loads each referenced model. It maintains the loading stack for circular
/// dependency detection and accumulates all loaded models in the builder.
///
/// # Arguments
///
/// * `model_path` - The path of the current model (used for resolving relative paths)
/// * `load_stack` - The loading stack for circular dependency detection
/// * `file_loader` - The file loader for parsing referenced models
/// * `use_models` - The use model declarations to process
/// * `builder` - The model collection builder to accumulate results
///
/// # Returns
///
/// Returns the updated model collection builder containing all loaded use models.
///
/// # Circular Dependency Handling
///
/// The function pushes the current model path onto the load stack before processing
/// use models and pops it after processing is complete. This ensures that circular
/// dependencies are properly detected during the recursive loading process.
///
/// # Path Resolution
///
/// Use model paths are resolved relative to the current model path using
/// `model_path.get_sibling_path(&use_model.model_name)`.
fn load_model_imports<E>(
    model_path: &ir::ModelPath,
    model_imports: &[&ast::UseModelNode],
    resolution_context: &mut ResolutionContext<'_, E>,
) where
    E: ExternalResolutionContext,
{
    for model_import in model_imports {
        // get the use model path
        let model_import_relative_path = model_import.get_model_relative_path();
        let model_import_path = model_path.get_sibling_path(&model_import_relative_path);
        let model_import_path = ir::ModelPath::new(model_import_path);

        // load the use model (and its submodels)
        load_model(&model_import_path, resolution_context);
    }
}

#[cfg(test)]
mod tests {
    use oneil_ast as ast;
    use oneil_ir as ir;

    use super::*;
    use crate::{
        error::{CircularDependencyError, ModelImportResolutionError},
        test::{external_context::TestExternalContext, test_ast},
    };

    #[test]
    fn split_model_ast_empty() {
        let model = test_ast::empty_model_node();
        let (imports, use_models, parameters, tests) = split_model_ast(&model);

        assert!(imports.is_empty());
        assert!(use_models.is_empty());
        assert!(parameters.is_empty());
        assert!(tests.is_empty());
    }

    #[test]
    fn split_model_ast_with_all_declarations() {
        let model = test_ast::ModelBuilder::new()
            .with_submodel("submodel")
            .build();
        let (imports, use_models, parameters, tests) = split_model_ast(&model);

        assert_eq!(imports.len(), 0);
        assert_eq!(use_models.len(), 1);
        assert_eq!(
            use_models[0].model_info().top_component().as_str(),
            "submodel"
        );
        assert!(use_models[0].model_info().subcomponents().is_empty());
        assert!(parameters.is_empty());
        assert!(tests.is_empty());
    }

    #[test]
    fn split_model_ast_use_model_only() {
        let model = test_ast::ModelBuilder::new()
            .with_submodel("submodel1")
            .with_submodel("submodel2")
            .build();
        let (imports, use_models, parameters, tests) = split_model_ast(&model);

        assert!(imports.is_empty());
        assert_eq!(use_models.len(), 2);
        assert_eq!(
            use_models[0].model_info().top_component().as_str(),
            "submodel1"
        );
        assert_eq!(
            use_models[1].model_info().top_component().as_str(),
            "submodel2"
        );

        assert!(parameters.is_empty());
        assert!(tests.is_empty());
    }

    #[test]
    fn load_model_success() {
        let model_path = ir::ModelPath::new("test");
        let mut external =
            TestExternalContext::new().with_model_asts([("test.on", test_ast::empty_model())]);
        let mut resolution_context = ResolutionContext::new(&mut external);

        load_model(&model_path, &mut resolution_context);

        let results = resolution_context.into_result();

        // check the models generated
        assert_eq!(results.len(), 1);
        assert!(results.contains_key(&model_path));

        // check the model errors
        assert!(results.values().all(|r| r.model_errors().is_empty()));

        // check the circular dependency errors
        assert!(
            results
                .values()
                .all(|r| r.circular_dependency_errors().is_empty())
        );
    }

    #[test]
    fn load_model_parse_error() {
        // Path "nonexistent" has no AST in context -> load_ast fails
        let model_path = ir::ModelPath::new("nonexistent");
        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContext::new(&mut external);

        load_model(&model_path, &mut resolution_context);

        let results = resolution_context.into_result();

        // check the model errors (parse failure does not record an error in model_errors)
        assert!(results.values().all(|r| r.model_errors().is_empty()));

        // check the circular dependency errors
        assert!(
            results
                .values()
                .all(|r| r.circular_dependency_errors().is_empty())
        );
    }

    #[test]
    fn load_model_circular_dependency() {
        // create a circular dependency: main.on -> sub.on -> main.on
        let main_path = ir::ModelPath::new("main");
        let sub_path = ir::ModelPath::new("sub");
        let main_test_model = test_ast::ModelBuilder::new().with_submodel("sub").build();
        let sub_test_model = test_ast::ModelBuilder::new().with_submodel("main").build();
        let mut external = TestExternalContext::new()
            .with_model_asts([("main.on", main_test_model), ("sub.on", sub_test_model)]);
        let mut resolution_context = ResolutionContext::new(&mut external);

        load_model(&main_path, &mut resolution_context);

        let results = resolution_context.into_result();

        // check the models generated
        assert_eq!(results.len(), 2);
        assert!(results.contains_key(&main_path));
        assert!(results.contains_key(&sub_path));

        // check the model errors (when sub has the circular error, main's resolution of "sub" may get ModelHasError)
        if let Some(main_result) = results.get(&main_path) {
            let main_errors = main_result.model_errors();
            let (_sub_model_name, sub_error) = main_errors
                .get_model_import_resolution_errors()
                .get(&ir::ReferenceName::new("sub".to_string()))
                .expect("sub reference error should exist");

            let ModelImportResolutionError::ModelHasError { model_path, .. } = sub_error else {
                panic!("sub reference error should be ModelHasError, got {sub_error:?}");
            };

            assert_eq!(model_path, &sub_path);
        }

        // check the circular dependency errors (recorded on sub when we detect main is already on the stack)
        let circular_dependency_error = results
            .get(&sub_path)
            .expect("sub should have circular dependency error")
            .circular_dependency_errors();
        assert_eq!(circular_dependency_error.len(), 1);
        assert_eq!(
            circular_dependency_error[0],
            CircularDependencyError::new(vec![main_path.clone(), sub_path.clone(), main_path,])
        );
    }

    #[test]
    fn load_model_already_visited() {
        // Load the same model twice
        let model_path = ir::ModelPath::new("test");
        let mut external =
            TestExternalContext::new().with_model_asts([("test.on", test_ast::empty_model())]);
        let mut resolution_context = ResolutionContext::new(&mut external);

        load_model(&model_path, &mut resolution_context);
        load_model(&model_path, &mut resolution_context);

        let results = resolution_context.into_result();

        // check the models generated
        assert_eq!(results.len(), 1);
        assert!(results.contains_key(&model_path));

        // check the model errors
        assert!(results.values().all(|r| r.model_errors().is_empty()));

        // check the circular dependency errors
        assert!(
            results
                .values()
                .all(|r| r.circular_dependency_errors().is_empty())
        );
    }

    #[test]
    fn load_use_models_empty() {
        // Load a model with no use/ref declarations (only parent in context).
        let model_path = ir::ModelPath::new("parent");
        let mut external =
            TestExternalContext::new().with_model_asts([("parent.on", test_ast::empty_model())]);
        let mut resolution_context = ResolutionContext::new(&mut external);

        load_model(&model_path, &mut resolution_context);

        let results = resolution_context.into_result();

        // check the models generated
        assert_eq!(results.len(), 1);
        assert!(results.contains_key(&model_path));

        // check the model errors
        assert!(results.values().all(|r| r.model_errors().is_empty()));

        // check the circular dependency errors
        assert!(
            results
                .values()
                .all(|r| r.circular_dependency_errors().is_empty())
        );
    }

    #[test]
    fn load_use_models_with_existing_models() {
        // Load parent that uses child1 and child2; all three ASTs in context.
        let parent_path = ir::ModelPath::new("parent");
        let parent_ast = test_ast::ModelBuilder::new()
            .with_submodel("child1")
            .with_submodel("child2")
            .build();
        let mut external = TestExternalContext::new().with_model_asts([
            ("parent.on", parent_ast),
            ("child1.on", test_ast::empty_model()),
            ("child2.on", test_ast::empty_model()),
        ]);
        let mut resolution_context = ResolutionContext::new(&mut external);

        load_model(&parent_path, &mut resolution_context);

        let results = resolution_context.into_result();

        // check the models generated
        assert_eq!(results.len(), 3);
        assert!(results.contains_key(&parent_path));
        assert!(results.contains_key(&ir::ModelPath::new("child1")));
        assert!(results.contains_key(&ir::ModelPath::new("child2")));

        // check the model errors
        assert!(results.values().all(|r| r.model_errors().is_empty()));

        // check the circular dependency errors
        assert!(
            results
                .values()
                .all(|r| r.circular_dependency_errors().is_empty())
        );
    }

    #[test]
    fn load_use_models_with_parse_errors() {
        // Parent uses "nonexistent"; that path has no AST in context.
        let parent_path = ir::ModelPath::new("parent");
        let parent_ast = test_ast::ModelBuilder::new()
            .with_submodel("nonexistent")
            .build();
        let mut external = TestExternalContext::new().with_model_asts([("parent.on", parent_ast)]);
        let mut resolution_context = ResolutionContext::new(&mut external);

        load_model(&parent_path, &mut resolution_context);

        let results = resolution_context.into_result();

        // check the model errors
        assert!(results.contains_key(&parent_path));

        // check the circular dependency errors
        assert!(
            results
                .values()
                .all(|r| r.circular_dependency_errors().is_empty())
        );
    }

    #[test]
    fn load_model_complex_dependency_chain() {
        // Dependency chain: root.on -> level1.on -> level2.on
        let root_path = ir::ModelPath::new("root");
        let root_model = test_ast::ModelBuilder::new()
            .with_submodel("level1")
            .build();
        let level1_model = test_ast::ModelBuilder::new()
            .with_submodel("level2")
            .build();
        let level2_model = test_ast::empty_model();

        let mut external = TestExternalContext::new().with_model_asts([
            ("root.on", root_model),
            ("level1.on", level1_model),
            ("level2.on", level2_model),
        ]);
        let mut resolution_context = ResolutionContext::new(&mut external);

        load_model(&root_path, &mut resolution_context);

        let results = resolution_context.into_result();

        // check the models generated
        assert_eq!(results.len(), 3);
        assert!(results.contains_key(&root_path));
        assert!(results.contains_key(&ir::ModelPath::new("level1")));
        assert!(results.contains_key(&ir::ModelPath::new("level2")));

        // check the model errors
        assert!(results.values().all(|r| r.model_errors().is_empty()));

        // check the circular dependency errors
        assert!(
            results
                .values()
                .all(|r| r.circular_dependency_errors().is_empty())
        );
    }

    #[test]
    fn load_model_with_sections() {
        // Model with a section that declares a use submodel
        let test_path = ir::ModelPath::new("test");
        let submodel_node = test_ast::empty_model();
        let use_model_decl = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("submodel")
            .with_kind(ast::ModelKind::Submodel)
            .build_as_decl_node();
        let model_node = test_ast::ModelBuilder::new()
            .with_section("section1", vec![use_model_decl])
            .build();

        let mut external = TestExternalContext::new()
            .with_model_asts([("test.on", model_node), ("submodel.on", submodel_node)]);
        let mut resolution_context = ResolutionContext::new(&mut external);

        load_model(&test_path, &mut resolution_context);

        let results = resolution_context.into_result();

        // check the models generated
        assert_eq!(results.len(), 2);
        assert!(results.contains_key(&test_path));
        assert!(results.contains_key(&ir::ModelPath::new("submodel")));

        // check the model errors
        assert!(results.values().all(|r| r.model_errors().is_empty()));

        // check the circular dependency errors
        assert!(
            results
                .values()
                .all(|r| r.circular_dependency_errors().is_empty())
        );
    }

    #[test]
    fn load_model_with_reference() {
        // Main model has ref "reference" and parameter y = reference.x
        let test_path = ir::ModelPath::new("test");
        let reference_path = ir::ModelPath::new("reference");
        let reference_node = test_ast::ModelBuilder::new()
            .with_number_parameter("x", 1.0)
            .build();
        let model_node = test_ast::ModelBuilder::new()
            .with_reference("reference")
            .with_reference_variable_parameter("y", "reference", "x")
            .build();

        let mut external = TestExternalContext::new()
            .with_model_asts([("test.on", model_node), ("reference.on", reference_node)]);
        let mut resolution_context = ResolutionContext::new(&mut external);

        load_model(&test_path, &mut resolution_context);

        let results = resolution_context.into_result();

        // check the models generated
        assert_eq!(results.len(), 2);
        let main_model = results
            .get(&test_path)
            .expect("main model should be present")
            .model();
        let y_parameter = main_model
            .get_parameter(&ir::ParameterName::new("y".to_string()))
            .expect("y parameter should be present");

        let ir::ParameterValue::Simple(y_parameter_value, _) = y_parameter.value() else {
            panic!("y parameter value should be a simple value");
        };

        let ir::Expr::Variable {
            span: _,
            variable: variable_expr,
        } = y_parameter_value
        else {
            panic!("y parameter value should be a variable expression");
        };

        let ir::Variable::External {
            model_path: model,
            parameter_name,
            ..
        } = variable_expr
        else {
            panic!("variable expression should be an external variable");
        };

        assert_eq!(model, &reference_path);
        assert_eq!(parameter_name.as_str(), "x");

        // check the model errors
        assert!(results.values().all(|r| r.model_errors().is_empty()));

        // check the circular dependency errors
        assert!(
            results
                .values()
                .all(|r| r.circular_dependency_errors().is_empty())
        );
    }

    #[test]
    fn load_model_with_submodel_with_error() {
        // Main model has ref "reference"; reference model not provided (or has error).
        // Submodel "submodel" exists but has use "nonexistent" (error).
        let test_path = ir::ModelPath::new("test");
        let submodel_node = test_ast::ModelBuilder::new()
            .with_submodel("nonexistent")
            .build();
        let model_node = test_ast::ModelBuilder::new()
            .with_reference("reference")
            .with_reference_variable_parameter("y", "reference", "x")
            .build();

        let mut external = TestExternalContext::new()
            .with_model_asts([("test.on", model_node), ("submodel.on", submodel_node)]);
        let mut resolution_context = ResolutionContext::new(&mut external);

        load_model(&test_path, &mut resolution_context);

        let results = resolution_context.into_result();

        // check the models generated
        assert!(!results.is_empty(), "expected at least one model");

        // check the model errors (test references "reference" with no AST, submodel references "nonexistent")
        assert!(
            results.values().any(|r| !r.model_errors().is_empty()),
            "expected at least one model with resolution errors"
        );

        // check the circular dependency errors
        assert!(
            results
                .values()
                .all(|r| r.circular_dependency_errors().is_empty())
        );
    }
}
