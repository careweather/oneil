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

use std::collections::HashSet;

use oneil_ast as ast;
use oneil_ir::{
    model::Model,
    parameter::Parameter,
    reference::{Identifier, ModelPath},
};

use crate::{
    error::{LoadError, ResolutionErrors},
    util::{FileLoader, Stack, builder::ModelCollectionBuilder, info::InfoMap},
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
/// # Errors
///
/// The function handles various error conditions:
///
/// - **Circular dependencies**: Returns early with a circular dependency error
/// - **Parse errors**: Adds parse errors to the builder and returns early
/// - **Resolution errors**: Collects all resolution errors and adds them to the builder
///
/// # Circular Dependency Detection
///
/// The function detects circular dependencies by maintaining a loading stack. If a model
/// appears in the stack while it's being loaded, a circular dependency is detected:
///
/// # Model Visitation
///
/// To prevent loading the same model multiple times, the function tracks visited models
/// in the builder. If a model has already been visited, it returns early without
/// re-processing the model.
pub fn load_model<F>(
    model_path: ModelPath,
    mut builder: ModelCollectionBuilder<F::ParseError, F::PythonError>,
    load_stack: &mut Stack<ModelPath>,
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
    let model_ast = file_loader.parse_ast(model_path.clone());

    // TODO: this might be able to recover and produce a partial model?
    let model_ast = match model_ast {
        Ok(model_ast) => model_ast,
        Err(error) => {
            builder.add_model_error(model_path, LoadError::ParseError(error));
            return builder;
        }
    };

    // split model ast into imports, use models, parameters, and tests
    let (imports, use_models, parameters, tests) = split_model_ast(model_ast);

    // validate imports
    let (python_imports, import_resolution_errors, builder) =
        importer::validate_imports(&model_path, builder, imports, file_loader);

    // load use models and resolve them
    let mut builder = load_use_models(
        model_path.clone(),
        load_stack,
        file_loader,
        &use_models,
        builder,
    );

    let model_info: InfoMap<&ModelPath, &Model> = InfoMap::new(
        builder.get_models().into_iter().collect(),
        builder.get_models_with_errors(),
    );

    // resolve submodels
    let (submodels, submodel_tests, submodel_resolution_errors) =
        resolver::resolve_submodels_and_tests(use_models, &model_path, &model_info);

    let submodels_with_errors: HashSet<&Identifier> = submodel_resolution_errors.keys().collect();

    let submodel_info: InfoMap<&Identifier, &ModelPath> =
        InfoMap::new(submodels.iter().collect(), submodels_with_errors);

    // resolve parameters
    let (parameters, parameter_resolution_errors) =
        resolver::resolve_parameters(parameters, &submodel_info, &model_info);

    let parameters_with_errors: HashSet<&Identifier> = parameter_resolution_errors.keys().collect();

    let parameter_info: InfoMap<&Identifier, &Parameter> =
        InfoMap::new(parameters.iter().collect(), parameters_with_errors);

    // resolve model tests
    let (model_tests, model_test_resolution_errors) =
        resolver::resolve_model_tests(tests, &parameter_info, &submodel_info, &model_info);

    // resolve submodel tests
    let (submodel_tests, submodel_test_resolution_errors) = resolver::resolve_submodel_tests(
        submodel_tests,
        &parameter_info,
        &submodel_info,
        &model_info,
    );

    let resolution_errors = ResolutionErrors::new(
        import_resolution_errors,
        submodel_resolution_errors,
        parameter_resolution_errors,
        model_test_resolution_errors,
        submodel_test_resolution_errors,
    );

    if !resolution_errors.is_empty() {
        let resolution_errors = LoadError::resolution_errors(resolution_errors);
        builder.add_model_error(model_path.clone(), resolution_errors);
    }

    // build model
    let model = Model::new(
        python_imports,
        submodels,
        parameters,
        model_tests,
        submodel_tests,
    );

    // add model to builder
    builder.add_model(model_path, model);

    builder
}

/// Splits a model AST into its constituent declaration types.
///
/// This function processes the declarations in a model AST and categorizes them into
/// separate collections for imports, use models, parameters, and tests. This separation
/// is necessary for the different processing steps in model loading.
///
/// # Arguments
///
/// * `model_ast` - The parsed model AST containing all declarations
///
/// # Returns
///
/// Returns a tuple containing:
/// - `Vec<ast::declaration::Import>` - All import declarations
/// - `Vec<ast::declaration::UseModel>` - All use model declarations
/// - `Vec<ast::Parameter>` - All parameter declarations
/// - `Vec<ast::Test>` - All test declarations
fn split_model_ast(
    model_ast: ast::Model,
) -> (
    Vec<ast::declaration::Import>,
    Vec<ast::declaration::UseModel>,
    Vec<ast::Parameter>,
    Vec<ast::Test>,
) {
    let mut imports = vec![];
    let mut use_models = vec![];
    let mut parameters = vec![];
    let mut tests = vec![];

    for decl in model_ast.decls {
        match decl {
            ast::declaration::Decl::Import(import) => imports.push(import),
            ast::declaration::Decl::UseModel(use_model) => use_models.push(use_model),
            ast::declaration::Decl::Parameter(parameter) => parameters.push(parameter),
            ast::declaration::Decl::Test(test) => tests.push(test),
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
    model_path: ModelPath,
    load_stack: &mut Stack<ModelPath>,
    file_loader: &F,
    use_models: &Vec<ast::declaration::UseModel>,
    builder: ModelCollectionBuilder<F::ParseError, F::PythonError>,
) -> ModelCollectionBuilder<F::ParseError, F::PythonError>
where
    F: FileLoader,
{
    load_stack.push(model_path.clone());

    // TODO: check for duplicate use models
    let builder = use_models.into_iter().fold(builder, |builder, use_model| {
        // get the use model path
        let use_model_path = model_path.get_sibling_path(&use_model.model_name);
        let use_model_path = ModelPath::new(use_model_path);

        // load the use model (and its submodels)
        load_model(use_model_path, builder, load_stack, file_loader)
    });

    load_stack.pop();

    builder
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        error::{CircularDependencyError, SubmodelResolutionError},
        test::TestFileParser,
    };
    use oneil_ast::{
        Model,
        declaration::{Decl, UseModel},
    };
    use std::collections::{HashMap, HashSet};
    use std::path::PathBuf;

    /// Creates an empty test model
    fn create_empty_model() -> Model {
        Model {
            note: None,
            decls: vec![],
            sections: vec![],
        }
    }

    /// Creates a simple test model that uses multiple submodels
    fn create_test_model(submodel_names: &[&str]) -> Model {
        let decls = submodel_names
            .iter()
            .map(|name| {
                Decl::UseModel(UseModel {
                    model_name: name.to_string(),
                    subcomponents: vec![],
                    inputs: None,
                    as_name: None,
                })
            })
            .collect();

        Model {
            note: None,
            decls,
            sections: vec![],
        }
    }

    #[test]
    fn test_split_model_ast_empty() {
        let model = create_empty_model();
        let (imports, use_models, parameters, tests) = split_model_ast(model);

        assert!(imports.is_empty());
        assert!(use_models.is_empty());
        assert!(parameters.is_empty());
        assert!(tests.is_empty());
    }

    #[test]
    fn test_split_model_ast_with_all_declarations() {
        let model = create_test_model(&["submodel"]);
        let (imports, use_models, parameters, tests) = split_model_ast(model);

        assert_eq!(imports.len(), 0);
        assert_eq!(use_models.len(), 1);
        assert_eq!(use_models[0].model_name, "submodel");
        assert!(use_models[0].subcomponents.is_empty());
        assert!(parameters.is_empty());
        assert!(tests.is_empty());
    }

    #[test]
    fn test_split_model_ast_use_model_only() {
        let model = create_test_model(&["submodel1", "submodel2"]);
        let (imports, use_models, parameters, tests) = split_model_ast(model);

        assert!(imports.is_empty());
        assert_eq!(use_models.len(), 2);
        assert_eq!(use_models[0].model_name, "submodel1");
        assert_eq!(use_models[1].model_name, "submodel2");

        assert!(parameters.is_empty());
        assert!(tests.is_empty());
    }

    #[test]
    fn test_load_model_success() {
        // create initial context
        let model_path = ModelPath::new("test");
        let initial_models = HashSet::from([model_path.clone()]);
        let builder = ModelCollectionBuilder::new(initial_models);
        let mut load_stack = Stack::new();

        let file_loader =
            TestFileParser::new(vec![(PathBuf::from("test.on"), create_test_model(&[]))]);

        // load the model
        let result = load_model(model_path.clone(), builder, &mut load_stack, &file_loader);

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
        let model_path = ModelPath::new("nonexistent");
        let initial_models = HashSet::from([model_path.clone()]);
        let builder = ModelCollectionBuilder::new(initial_models);
        let mut load_stack = Stack::new();

        let file_loader = TestFileParser::empty();

        // load the model
        let result = load_model(model_path.clone(), builder, &mut load_stack, &file_loader);

        // check the errors
        let errors = result.get_model_errors();
        assert_eq!(errors.len(), 1);

        let error = errors.get(&model_path).unwrap();
        assert_eq!(error, &LoadError::ParseError(()));

        // check the models
        let models = result.get_models();
        assert!(models.is_empty());
    }

    #[test]
    fn test_load_model_circular_dependency() {
        // create initial context
        let model_path = ModelPath::new("main");
        let initial_models = HashSet::from([model_path.clone()]);
        let builder = ModelCollectionBuilder::new(initial_models);
        let mut load_stack = Stack::new();

        // create a circular dependency: main.on -> sub.on -> main.on
        let file_loader = TestFileParser::new(vec![
            (PathBuf::from("main.on"), create_test_model(&["sub"])),
            (PathBuf::from("sub.on"), create_test_model(&["main"])),
        ]);

        // load the model
        let result = load_model(model_path, builder, &mut load_stack, &file_loader);

        // check the errors
        let errors = result.get_model_errors();
        assert_eq!(errors.len(), 2);

        let main_errors = errors.get(&ModelPath::new("main")).unwrap();
        assert_eq!(
            main_errors,
            &LoadError::resolution_errors(ResolutionErrors::new(
                HashMap::new(),
                HashMap::from([(
                    Identifier::new("sub"),
                    SubmodelResolutionError::model_has_error(ModelPath::new("sub"))
                )]),
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
            ))
        );

        let sub_errors = errors.get(&ModelPath::new("sub")).unwrap();
        assert_eq!(
            sub_errors,
            &LoadError::resolution_errors(ResolutionErrors::new(
                HashMap::new(),
                HashMap::from([(
                    Identifier::new("main"),
                    SubmodelResolutionError::model_has_error(ModelPath::new("main"))
                )]),
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
            ))
        );

        // check the circular dependency errors
        let circular_dependency_errors = result.get_circular_dependency_errors();
        assert_eq!(circular_dependency_errors.len(), 1);

        let circular_dependency_error = circular_dependency_errors
            .get(&ModelPath::new("main"))
            .unwrap();
        assert_eq!(circular_dependency_error.len(), 1);
        assert_eq!(
            circular_dependency_error[0],
            CircularDependencyError::new(vec![
                ModelPath::new("main"),
                ModelPath::new("sub"),
                ModelPath::new("main"),
            ])
        );

        // check the models
        let models = result.get_models();
        assert_eq!(models.len(), 2);
        assert!(models.contains_key(&ModelPath::new("main")));
        assert!(models.contains_key(&ModelPath::new("sub")));
    }

    #[test]
    fn test_load_model_already_visited() {
        // create initial context
        let model_path = ModelPath::new("test");
        let initial_models = HashSet::from([model_path.clone()]);
        let mut builder = ModelCollectionBuilder::new(initial_models);
        let mut load_stack = Stack::new();

        // mark the model as already visited
        builder.mark_model_as_visited(&model_path);

        // load the model
        let file_loader =
            TestFileParser::new(vec![(PathBuf::from("test.on"), create_empty_model())]);

        let result = load_model(model_path, builder, &mut load_stack, &file_loader);

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
        let model_path = ModelPath::new("parent");
        let mut load_stack = Stack::new();
        let file_loader = TestFileParser::empty();
        let use_models = vec![];
        let builder = ModelCollectionBuilder::new(HashSet::new());

        // load the use models
        let result = load_use_models(
            model_path,
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
        let model_path = ModelPath::new("parent");
        let mut load_stack = Stack::new();
        let file_loader = TestFileParser::new(vec![
            (PathBuf::from("child1.on"), create_empty_model()),
            (PathBuf::from("child2.on"), create_empty_model()),
        ]);

        let use_models = vec![
            UseModel {
                model_name: "child1".to_string(),
                subcomponents: vec![],
                inputs: None,
                as_name: None,
            },
            UseModel {
                model_name: "child2".to_string(),
                subcomponents: vec![],
                inputs: None,
                as_name: None,
            },
        ];

        let builder = ModelCollectionBuilder::new(HashSet::new());

        // load the use models
        let result = load_use_models(
            model_path,
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
        assert_eq!(models.len(), 2);
        assert!(models.contains_key(&ModelPath::new("child1")));
        assert!(models.contains_key(&ModelPath::new("child2")));
    }

    #[test]
    fn test_load_use_models_with_parse_errors() {
        // create initial context
        let model_path = ModelPath::new("parent");
        let mut load_stack = Stack::new();
        let file_loader = TestFileParser::empty(); // No models available

        let use_models = vec![UseModel {
            model_name: "nonexistent".to_string(),
            subcomponents: vec![],
            inputs: None,
            as_name: None,
        }];

        let builder = ModelCollectionBuilder::new(HashSet::new());

        // load the use models
        let result = load_use_models(
            model_path,
            &mut load_stack,
            &file_loader,
            &use_models,
            builder,
        );

        // check the errors
        let errors = result.get_model_errors();
        assert_eq!(errors.len(), 1);

        let error = errors.get(&ModelPath::new("nonexistent")).unwrap();
        assert_eq!(error, &LoadError::ParseError(()));

        // check the models
        let models = result.get_models();
        assert!(models.is_empty());
    }

    #[test]
    fn test_load_model_complex_dependency_chain() {
        // create initial context
        let model_path = ModelPath::new("root");
        let initial_models = HashSet::from([model_path.clone()]);
        let builder = ModelCollectionBuilder::new(initial_models);
        let mut load_stack = Stack::new();

        // create a dependency chain: root.on -> level1.on -> level2.on
        let root_model = create_test_model(&["level1"]);
        let level1_model = create_test_model(&["level2"]);
        let level2_model = create_empty_model();

        let file_loader = TestFileParser::new(vec![
            (PathBuf::from("root.on"), root_model),
            (PathBuf::from("level1.on"), level1_model),
            (PathBuf::from("level2.on"), level2_model),
        ]);

        // load the model
        let result = load_model(model_path, builder, &mut load_stack, &file_loader);

        // check the errors
        let errors = result.get_model_errors();
        assert!(errors.is_empty());

        // check the models
        let models = result.get_models();
        assert_eq!(models.len(), 3);
        assert!(models.contains_key(&ModelPath::new("root")));
        assert!(models.contains_key(&ModelPath::new("level1")));
        assert!(models.contains_key(&ModelPath::new("level2")));
    }
}
