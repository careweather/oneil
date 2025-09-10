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

use crate::{
    BuiltinRef,
    error::{LoadError, ResolutionErrors},
    util::{
        FileLoader, Stack,
        builder::ModelCollectionBuilder,
        context::{ModelContext, ParameterContext, ReferenceContext},
    },
};

mod importer;
mod resolver;

/// Loads a model and all its dependencies, building a complete model collection.
///
/// This function is the main entry point for model loading. It performs the following steps:
///
/// 1. **Circular dependency detection**: Checks if loading this model would create a circular dependency
/// 2. **Model visitation tracking**: Prevents loading the same model multiple times
/// 3. **AST parsing**: Parses the model file into an AST using the provided file loader
/// 4. **Declaration splitting**: Separates imports, use models, parameters, and tests from the AST
/// 5. **Import validation**: Validates Python imports and collects any validation errors
/// 6. **Use model loading**: Recursively loads all referenced use models
/// 7. **Resolution**: Resolves submodels, parameters, and tests using the loaded model information
/// 8. **Model construction**: Builds the final model and adds it to the collection
///
/// # Arguments
///
/// * `model_path` - The path to the model to load
/// * `builder` - The model collection builder that accumulates models and errors
/// * `load_stack` - A stack tracking the current loading path for circular dependency detection
/// * `file_loader` - The file loader implementation for parsing and validation
///
/// # Returns
///
/// Returns the updated model collection builder containing all loaded models and any errors.
///
/// # Error Handling
///
/// The function handles various error conditions:
///
/// - **Circular dependencies**: Returns early with a circular dependency error
/// - **Parse errors**: Adds parse errors to the builder and returns early without attempting recovery
/// - **Resolution errors**: Collects all resolution errors and adds them to the builder, but continues processing
///
/// # Circular Dependency Detection
///
/// The function detects circular dependencies by maintaining a loading stack. If a model
/// appears in the stack while it's being loaded, a circular dependency is detected.
///
/// # Model Visitation
///
/// To prevent loading the same model multiple times, the function tracks visited models
/// in the builder. If a model has already been visited, it returns early without
/// re-processing the model.
pub fn load_model<F>(
    model_path: ir::ModelPath,
    mut builder: ModelCollectionBuilder<F::ParseError, F::PythonError>,
    builtin_ref: &impl BuiltinRef,
    load_stack: &mut Stack<ir::ModelPath>,
    file_loader: &F,
) -> ModelCollectionBuilder<F::ParseError, F::PythonError>
where
    F: FileLoader,
{
    // check for circular dependencies
    //
    // this happens before we check if the model has been visited because if
    // there is a circular dependency, it will have already been visited
    if let Some(circular_dependency) = load_stack.find_circular_dependency(&model_path) {
        builder.add_circular_dependency_error(model_path, circular_dependency);
        return builder;
    }

    // check if model is already been visited, then mark as visited if not
    if builder.model_has_been_visited(&model_path) {
        return builder;
    }
    builder.mark_model_as_visited(&model_path);

    // parse model ast
    let model_ast = file_loader.parse_ast(&model_path);

    // TODO: this might be able to recover and produce a partial model?
    let model_ast = match model_ast {
        Ok(model_ast) => model_ast,
        Err(error) => {
            builder.add_model_error(model_path, LoadError::ParseError(error));
            return builder;
        }
    };

    // split model ast into imports, use models, parameters, and tests
    let (imports, use_models, parameters, tests) = split_model_ast(&model_ast);

    // validate imports
    let (python_imports, import_resolution_errors, builder) =
        importer::validate_imports(&model_path, builder, imports, file_loader);

    // load use models and resolve them
    let mut builder = load_use_models(
        &model_path,
        builtin_ref,
        load_stack,
        file_loader,
        &use_models,
        builder,
    );

    let models = builder.get_models();
    let models_with_errors = builder.get_models_with_errors();

    let model_context = ModelContext::new(models, models_with_errors);

    // resolve submodels
    let (submodels, references, submodel_resolution_errors, reference_resolution_errors) =
        resolver::resolve_model_imports(use_models, &model_path, &model_context);

    let models_with_errors = builder.get_models_with_errors();

    let reference_context = ReferenceContext::new(
        models,
        models_with_errors,
        &references,
        &reference_resolution_errors,
    );

    // resolve parameters
    let (parameters, parameter_resolution_errors) =
        resolver::resolve_parameters(parameters, builtin_ref, &reference_context);

    let parameter_context = ParameterContext::new(&parameters, &parameter_resolution_errors);

    // resolve tests
    let (tests, test_resolution_errors) =
        resolver::resolve_tests(tests, builtin_ref, &reference_context, &parameter_context);

    let resolution_errors = ResolutionErrors::new(
        import_resolution_errors,
        submodel_resolution_errors,
        reference_resolution_errors,
        parameter_resolution_errors,
        test_resolution_errors,
    );

    if !resolution_errors.is_empty() {
        let resolution_errors = LoadError::resolution_errors(resolution_errors);
        builder.add_model_error(model_path.clone(), resolution_errors);
    }

    // build model
    let model = ir::Model::new(python_imports, submodels, references, parameters, tests);

    // add model to builder
    builder.add_model(model_path, model);

    builder
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
    model_ast: &ast::ModelNode,
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
        match decl.node_value() {
            ast::Decl::Import(import) => imports.push(import),
            ast::Decl::UseModel(use_model) => use_models.push(use_model),
            ast::Decl::Parameter(parameter) => parameters.push(parameter),
            ast::Decl::Test(test) => tests.push(test),
        }
    }

    for section in model_ast.sections() {
        for decl in section.decls() {
            match decl.node_value() {
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
fn load_use_models<F>(
    model_path: &ir::ModelPath,
    builtin_ref: &impl BuiltinRef,
    load_stack: &mut Stack<ir::ModelPath>,
    file_loader: &F,
    use_models: &[&ast::UseModelNode],
    builder: ModelCollectionBuilder<F::ParseError, F::PythonError>,
) -> ModelCollectionBuilder<F::ParseError, F::PythonError>
where
    F: FileLoader,
{
    load_stack.push(model_path.clone());

    let builder = use_models.iter().fold(builder, |builder, use_model| {
        // get the use model path
        let use_model_relative_path = use_model.get_model_relative_path();
        let use_model_path = model_path.get_sibling_path(&use_model_relative_path);
        let use_model_path = ir::ModelPath::new(use_model_path);

        // load the use model (and its submodels)
        load_model(
            use_model_path,
            builder,
            builtin_ref,
            load_stack,
            file_loader,
        )
    });

    load_stack.pop();

    builder
}

#[cfg(test)]
mod tests {
    use oneil_ast::{self as ast};
    use oneil_ir as ir;

    use super::*;
    use crate::{
        error::{CircularDependencyError, ModelImportResolutionError},
        test::{TestBuiltinRef, TestFileParser, construct::test_ast},
    };
    use std::collections::HashSet;

    #[test]
    fn test_split_model_ast_empty() {
        let model = test_ast::empty_model_node();
        let (imports, use_models, parameters, tests) = split_model_ast(&model);

        assert!(imports.is_empty());
        assert!(use_models.is_empty());
        assert!(parameters.is_empty());
        assert!(tests.is_empty());
    }

    #[test]
    fn test_split_model_ast_with_all_declarations() {
        let model = test_ast::ModelNodeBuilder::new()
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
    fn test_split_model_ast_use_model_only() {
        let model = test_ast::ModelNodeBuilder::new()
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
    fn test_load_model_success() {
        // create initial context
        let model_path = ir::ModelPath::new("test");
        let initial_models = HashSet::from([model_path.clone()]);
        let builder = ModelCollectionBuilder::new(initial_models);
        let builtin_ref = TestBuiltinRef::new();
        let mut load_stack = Stack::new();

        let file_loader = TestFileParser::new([("test.on", test_ast::empty_model_node())]);

        // load the model
        let result = load_model(
            model_path.clone(),
            builder,
            &builtin_ref,
            &mut load_stack,
            &file_loader,
        );

        // check the errors
        let errors = result.get_model_errors();
        assert!(errors.is_empty());

        // check the models
        let models = result.get_models();
        assert_eq!(models.len(), 1);
        assert!(models.contains_key(&model_path));
    }

    #[test]
    fn test_load_model_parse_error() {
        // create initial context
        let model_path = ir::ModelPath::new("nonexistent");
        let initial_models = HashSet::from([model_path.clone()]);
        let builder = ModelCollectionBuilder::new(initial_models);
        let builtin_ref = TestBuiltinRef::new();
        let mut load_stack = Stack::new();

        let file_loader = TestFileParser::empty();

        // load the model
        let result = load_model(
            model_path.clone(),
            builder,
            &builtin_ref,
            &mut load_stack,
            &file_loader,
        );

        // check the errors
        let errors = result.get_model_errors();
        assert_eq!(errors.len(), 1);

        let error = errors.get(&model_path);
        assert_eq!(error, Some(&LoadError::ParseError(())));

        // check the models
        let models = result.get_models();
        assert!(models.is_empty());
    }

    #[test]
    fn test_load_model_circular_dependency() {
        // create initial context
        let model_path = ir::ModelPath::new("main");
        let initial_models = HashSet::from([model_path.clone()]);
        let builder = ModelCollectionBuilder::new(initial_models);
        let builtin_ref = TestBuiltinRef::new();
        let mut load_stack = Stack::new();

        // create a circular dependency: main.on -> sub.on -> main.on
        let main_test_model = test_ast::ModelNodeBuilder::new()
            .with_submodel("main")
            .build();
        let sub_test_model = test_ast::ModelNodeBuilder::new()
            .with_submodel("sub")
            .build();
        let file_loader =
            TestFileParser::new([("main.on", sub_test_model), ("sub.on", main_test_model)]);

        // load the model
        let result = load_model(
            model_path,
            builder,
            &builtin_ref,
            &mut load_stack,
            &file_loader,
        );

        // check the errors
        let errors = result.get_model_errors();
        assert_eq!(errors.len(), 2);

        let main_errors = errors
            .get(&ir::ModelPath::new("main"))
            .expect("main errors should be present");

        let LoadError::ResolutionErrors(resolution_errors) = main_errors else {
            panic!("main errors should be a resolution error");
        };

        assert!(resolution_errors.get_import_errors().is_empty());
        assert!(
            resolution_errors
                .get_reference_resolution_errors()
                .is_empty()
        );
        assert!(
            resolution_errors
                .get_parameter_resolution_errors()
                .is_empty()
        );
        assert!(resolution_errors.get_test_resolution_errors().is_empty());

        let sub_error = resolution_errors
            .get_submodel_resolution_errors()
            .get(&ir::SubmodelName::new("sub".to_string()))
            .expect("sub errors should be present");

        let ModelImportResolutionError::ModelHasError { model_path, .. } = sub_error else {
            panic!("sub errors should be a model has error error");
        };

        assert_eq!(model_path, &ir::ModelPath::new("sub"));

        // check the circular dependency errors
        let circular_dependency_errors = result.get_circular_dependency_errors();
        assert_eq!(circular_dependency_errors.len(), 1);

        let circular_dependency_error = circular_dependency_errors.get(&ir::ModelPath::new("main"));
        assert!(circular_dependency_error.is_some());

        let circular_dependency_error =
            circular_dependency_error.expect("circular dependency error should be present");
        assert_eq!(circular_dependency_error.len(), 1);
        assert_eq!(
            circular_dependency_error[0],
            CircularDependencyError::new(vec![
                ir::ModelPath::new("main"),
                ir::ModelPath::new("sub"),
                ir::ModelPath::new("main"),
            ])
        );

        // check the models
        let models = result.get_models();
        assert_eq!(models.len(), 2);
        assert!(models.contains_key(&ir::ModelPath::new("main")));
        assert!(models.contains_key(&ir::ModelPath::new("sub")));
    }

    #[test]
    fn test_load_model_already_visited() {
        // create initial context
        let model_path = ir::ModelPath::new("test");
        let initial_models = HashSet::from([model_path.clone()]);
        let mut builder = ModelCollectionBuilder::new(initial_models);
        let builtin_ref = TestBuiltinRef::new();
        let mut load_stack = Stack::new();

        // mark the model as already visited
        builder.mark_model_as_visited(&model_path);

        // load the model
        let file_loader = TestFileParser::new([("test.on", test_ast::empty_model_node())]);

        let result = load_model(
            model_path,
            builder,
            &builtin_ref,
            &mut load_stack,
            &file_loader,
        );

        // check the errors
        let errors = result.get_model_errors();
        assert!(errors.is_empty());

        // check the models
        let models = result.get_models();
        assert!(models.is_empty());
    }

    #[test]
    fn test_load_use_models_empty() {
        // create initial context
        let model_path = ir::ModelPath::new("parent");
        let mut load_stack = Stack::new();
        let builtin_ref = TestBuiltinRef::new();
        let file_loader = TestFileParser::empty();
        let use_models = vec![];
        let builder = ModelCollectionBuilder::new(HashSet::new());

        // load the use models
        let result = load_use_models(
            &model_path,
            &builtin_ref,
            &mut load_stack,
            &file_loader,
            &use_models,
            builder,
        );

        // check the errors
        let errors = result.get_model_errors();
        assert!(errors.is_empty());

        // check the models
        let models = result.get_models();
        assert!(models.is_empty());
    }

    #[test]
    fn test_load_use_models_with_existing_models() {
        // create initial context
        let model_path = ir::ModelPath::new("parent");
        let builtin_ref = TestBuiltinRef::new();
        let mut load_stack = Stack::new();
        let builder = ModelCollectionBuilder::new(HashSet::new());
        let file_loader = TestFileParser::new([
            ("child1.on", test_ast::empty_model_node()),
            ("child2.on", test_ast::empty_model_node()),
        ]);

        let use_models = [
            test_ast::ImportModelNodeBuilder::new()
                .with_top_component("child1")
                .with_kind(ast::ModelKind::Submodel)
                .build(),
            test_ast::ImportModelNodeBuilder::new()
                .with_top_component("child2")
                .with_kind(ast::ModelKind::Submodel)
                .build(),
        ];

        let use_models_ref = use_models.iter().collect::<Vec<_>>();

        // load the use models
        let result = load_use_models(
            &model_path,
            &builtin_ref,
            &mut load_stack,
            &file_loader,
            &use_models_ref,
            builder,
        );

        // check the errors
        let errors = result.get_model_errors();
        assert!(errors.is_empty());

        // check the models
        let models = result.get_models();
        assert_eq!(models.len(), 2);
        assert!(models.contains_key(&ir::ModelPath::new("child1")));
        assert!(models.contains_key(&ir::ModelPath::new("child2")));
    }

    #[test]
    fn test_load_use_models_with_parse_errors() {
        // create initial context
        let model_path = ir::ModelPath::new("parent");
        let builtin_ref = TestBuiltinRef::new();
        let mut load_stack = Stack::new();
        let file_loader = TestFileParser::empty(); // No models available

        let use_models = [test_ast::ImportModelNodeBuilder::new()
            .with_top_component("nonexistent")
            .with_kind(ast::ModelKind::Submodel)
            .build()];
        let use_models_ref = use_models.iter().collect::<Vec<_>>();

        let builder = ModelCollectionBuilder::new(HashSet::new());

        // load the use models
        let result = load_use_models(
            &model_path,
            &builtin_ref,
            &mut load_stack,
            &file_loader,
            &use_models_ref,
            builder,
        );

        // check the errors
        let errors = result.get_model_errors();
        assert_eq!(errors.len(), 1);

        let error = errors.get(&ir::ModelPath::new("nonexistent"));
        assert_eq!(error, Some(&LoadError::ParseError(())));

        // check the models
        let models = result.get_models();
        assert!(models.is_empty());
    }

    #[test]
    fn test_load_model_complex_dependency_chain() {
        // create initial context
        let model_path = ir::ModelPath::new("root");
        let initial_models = HashSet::from([model_path.clone()]);
        let builder = ModelCollectionBuilder::new(initial_models);
        let builtin_ref = TestBuiltinRef::new();
        let mut load_stack = Stack::new();

        // create a dependency chain: root.on -> level1.on -> level2.on
        let root_model = test_ast::ModelNodeBuilder::new()
            .with_submodel("level1")
            .build();
        let level1_model = test_ast::ModelNodeBuilder::new()
            .with_submodel("level2")
            .build();
        let level2_model = test_ast::empty_model_node();

        let file_loader = TestFileParser::new([
            ("root.on", root_model),
            ("level1.on", level1_model),
            ("level2.on", level2_model),
        ]);

        // load the model
        let result = load_model(
            model_path,
            builder,
            &builtin_ref,
            &mut load_stack,
            &file_loader,
        );

        // check the errors
        let errors = result.get_model_errors();
        assert!(errors.is_empty());

        // check the models
        let models = result.get_models();
        assert_eq!(models.len(), 3);
        assert!(models.contains_key(&ir::ModelPath::new("root")));
        assert!(models.contains_key(&ir::ModelPath::new("level1")));
        assert!(models.contains_key(&ir::ModelPath::new("level2")));
    }

    #[test]
    fn test_load_model_with_sections() {
        // create initial context
        let model_path = ir::ModelPath::new("test");
        let initial_models = HashSet::from([model_path.clone()]);
        let builder = ModelCollectionBuilder::new(initial_models);
        let builtin_ref = TestBuiltinRef::new();
        let mut load_stack = Stack::new();

        // create a model with sections
        let submodel_node = test_ast::empty_model_node();

        // Create a use model declaration
        let use_model_decl = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("submodel")
            .with_kind(ast::ModelKind::Submodel)
            .build_as_decl_node();

        // Create the model
        let model_node = test_ast::ModelNodeBuilder::new()
            .with_section("section1", vec![use_model_decl])
            .build();

        let file_loader =
            TestFileParser::new([("test.on", model_node), ("submodel.on", submodel_node)]);

        // load the model
        let result = load_model(
            model_path,
            builder,
            &builtin_ref,
            &mut load_stack,
            &file_loader,
        );

        // check the errors
        let errors = result.get_model_errors();
        assert!(errors.is_empty());

        // check the models
        let models = result.get_models();
        assert_eq!(models.len(), 2);
        assert!(models.contains_key(&ir::ModelPath::new("test")));
        assert!(models.contains_key(&ir::ModelPath::new("submodel")));
    }
}
