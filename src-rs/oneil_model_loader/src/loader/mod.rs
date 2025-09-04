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
use oneil_ir::{model::Model, reference::ModelPath};

use crate::{
    BuiltinRef,
    error::{LoadError, ResolutionErrors},
    util::{FileLoader, Stack, builder::ModelCollectionBuilder, context::ModelsLoadedContext},
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
    model_path: ModelPath,
    mut builder: ModelCollectionBuilder<F::ParseError, F::PythonError>,
    builtin_ref: &impl BuiltinRef,
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

    let context = ModelsLoadedContext::from_builder(&builder);

    // resolve submodels
    let (submodels, references, submodel_resolution_errors, reference_resolution_errors) =
        resolver::resolve_model_imports(use_models, &model_path, &context);

    // TODO: add references to the context as well
    let context = context.with_model_imports_resolved(&references, &reference_resolution_errors);
    let context = context.begin_parameter_resolution();

    // resolve parameters
    let context = resolver::resolve_parameters(parameters, builtin_ref, context);

    // resolve tests
    let (tests, test_resolution_errors) = resolver::resolve_tests(tests, builtin_ref, &context);

    // get the parameters and parameter resolution errors
    //
    // this needs to be done after test resolution because we need the parameter
    // context for resolving tests
    let (parameters, parameter_resolution_errors) = context.into_parameters_and_errors();

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
    let model = Model::new(python_imports, submodels, references, parameters, tests);

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
    model_ast: &ast::model::ModelNode,
) -> (
    Vec<&ast::declaration::ImportNode>,
    Vec<&ast::declaration::UseModelNode>,
    Vec<&ast::parameter::ParameterNode>,
    Vec<&ast::test::TestNode>,
) {
    let mut imports = vec![];
    let mut use_models = vec![];
    let mut parameters = vec![];
    let mut tests = vec![];

    for decl in model_ast.decls() {
        match decl.node_value() {
            ast::declaration::Decl::Import(import) => imports.push(import),
            ast::declaration::Decl::UseModel(use_model) => use_models.push(use_model),
            ast::declaration::Decl::Parameter(parameter) => parameters.push(parameter),
            ast::declaration::Decl::Test(test) => tests.push(test),
        }
    }

    for section in model_ast.sections() {
        for decl in section.decls() {
            match decl.node_value() {
                ast::declaration::Decl::Import(import) => imports.push(import),
                ast::declaration::Decl::UseModel(use_model) => use_models.push(use_model),
                ast::declaration::Decl::Parameter(parameter) => parameters.push(parameter),
                ast::declaration::Decl::Test(test) => tests.push(test),
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
    model_path: &ModelPath,
    builtin_ref: &impl BuiltinRef,
    load_stack: &mut Stack<ModelPath>,
    file_loader: &F,
    use_models: &[&ast::declaration::UseModelNode],
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
        let use_model_path = ModelPath::new(use_model_path);

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
    use oneil_ast::declaration::ModelKind;
    use oneil_ir::model_import::SubmodelName;

    use super::*;
    use crate::{
        error::{CircularDependencyError, ModelImportResolutionError},
        test::TestFileParser,
    };
    use std::collections::{HashMap, HashSet};
    use std::path::PathBuf;

    // re-export ast types for testing convenience
    mod ast {
        pub use oneil_ast::{
            Span,
            declaration::{Decl, ModelInfo, ModelKind, UseModel},
            model::{Model, ModelNode, Section, SectionHeader},
            naming::{Identifier, Label},
            node::Node,
        };
    }

    mod helper {
        use super::ast;

        use crate::test::TestBuiltinRef;

        pub fn unimportant_span() -> ast::Span {
            ast::Span::new(0, 0, 0)
        }

        pub fn span_from_str(s: &str) -> ast::Span {
            ast::Span::new(0, s.len(), 0)
        }

        /// Creates an empty test model
        pub fn create_empty_model_node() -> ast::ModelNode {
            let span = unimportant_span();

            let model = ast::Model::new(None, vec![], vec![]);

            ast::Node::new(&span, model)
        }

        /// Creates a test builtin ref
        pub fn create_test_builtin_ref() -> TestBuiltinRef {
            TestBuiltinRef::new()
        }

        /// Creates a simple test model that uses multiple submodels
        pub fn create_test_model(submodel_names: &[&str]) -> ast::ModelNode {
            let decls = submodel_names
                .iter()
                .map(|name| {
                    let use_model_name = ast::Identifier::new((*name).to_string());
                    let use_model_name_node = ast::Node::new(&span_from_str(name), use_model_name);
                    let use_model_info = ast::ModelInfo::new(use_model_name_node, vec![], None);
                    let use_model_info_node = ast::Node::new(&unimportant_span(), use_model_info);
                    let use_model = ast::UseModel::new(
                        vec![],
                        use_model_info_node,
                        None,
                        ast::ModelKind::Submodel,
                    );
                    let use_model_node = ast::Node::new(&unimportant_span(), use_model);

                    ast::Node::new(&unimportant_span(), ast::Decl::use_model(use_model_node))
                })
                .collect();

            let model = ast::Model::new(None, decls, vec![]);
            ast::Node::new(&unimportant_span(), model)
        }
    }

    #[test]
    fn test_split_model_ast_empty() {
        let model = helper::create_empty_model_node();
        let (imports, use_models, parameters, tests) = split_model_ast(&model);

        assert!(imports.is_empty());
        assert!(use_models.is_empty());
        assert!(parameters.is_empty());
        assert!(tests.is_empty());
    }

    #[test]
    fn test_split_model_ast_with_all_declarations() {
        let model = helper::create_test_model(&["submodel"]);
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
        let model = helper::create_test_model(&["submodel1", "submodel2"]);
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
        let model_path = ModelPath::new("test");
        let initial_models = HashSet::from([model_path.clone()]);
        let builder = ModelCollectionBuilder::new(initial_models);
        let builtin_ref = helper::create_test_builtin_ref();
        let mut load_stack = Stack::new();

        let file_loader = TestFileParser::new(vec![(
            PathBuf::from("test.on"),
            helper::create_test_model(&[]),
        )]);

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
        let model_path = ModelPath::new("nonexistent");
        let initial_models = HashSet::from([model_path.clone()]);
        let builder = ModelCollectionBuilder::new(initial_models);
        let builtin_ref = helper::create_test_builtin_ref();
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
        let model_path = ModelPath::new("main");
        let initial_models = HashSet::from([model_path.clone()]);
        let builder = ModelCollectionBuilder::new(initial_models);
        let builtin_ref = helper::create_test_builtin_ref();
        let mut load_stack = Stack::new();

        // create a circular dependency: main.on -> sub.on -> main.on
        let main_test_model = helper::create_test_model(&["main"]);
        let sub_test_model = helper::create_test_model(&["sub"]);
        let file_loader = TestFileParser::new(vec![
            (PathBuf::from("main.on"), sub_test_model),
            (PathBuf::from("sub.on"), main_test_model),
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
        assert_eq!(errors.len(), 2);

        let main_errors = errors.get(&ModelPath::new("main"));
        assert_eq!(
            main_errors,
            Some(&LoadError::resolution_errors(ResolutionErrors::new(
                HashMap::new(),
                HashMap::from([(
                    SubmodelName::new("sub".to_string()),
                    ModelImportResolutionError::model_has_error(
                        ModelPath::new("sub"),
                        oneil_ir::span::Span::new(0, 3)
                    )
                )]),
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
            )))
        );

        let sub_errors = errors.get(&ModelPath::new("sub"));
        assert_eq!(
            sub_errors,
            Some(&LoadError::resolution_errors(ResolutionErrors::new(
                HashMap::new(),
                HashMap::from([(
                    SubmodelName::new("main".to_string()),
                    ModelImportResolutionError::model_has_error(
                        ModelPath::new("main"),
                        oneil_ir::span::Span::new(0, 4)
                    )
                )]),
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
            )))
        );

        // check the circular dependency errors
        let circular_dependency_errors = result.get_circular_dependency_errors();
        assert_eq!(circular_dependency_errors.len(), 1);

        let circular_dependency_error = circular_dependency_errors.get(&ModelPath::new("main"));
        assert!(circular_dependency_error.is_some());

        let circular_dependency_error =
            circular_dependency_error.expect("circular dependency error should be present");
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
        let builtin_ref = helper::create_test_builtin_ref();
        let mut load_stack = Stack::new();

        // mark the model as already visited
        builder.mark_model_as_visited(&model_path);

        // load the model
        let file_loader = TestFileParser::new(vec![(
            PathBuf::from("test.on"),
            helper::create_empty_model_node(),
        )]);

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
        let model_path = ModelPath::new("parent");
        let mut load_stack = Stack::new();
        let builtin_ref = helper::create_test_builtin_ref();
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
        let model_path = ModelPath::new("parent");
        let mut load_stack = Stack::new();
        let builder = ModelCollectionBuilder::new(HashSet::new());
        let file_loader = TestFileParser::new(vec![
            (
                PathBuf::from("child1.on"),
                helper::create_empty_model_node(),
            ),
            (
                PathBuf::from("child2.on"),
                helper::create_empty_model_node(),
            ),
        ]);

        let child1_identifier = ast::Identifier::new("child1".to_string());
        let child1_identifier_node =
            ast::Node::new(&helper::span_from_str("child1"), child1_identifier);
        let child2_identifier = ast::Identifier::new("child2".to_string());
        let child2_identifier_node =
            ast::Node::new(&helper::span_from_str("child2"), child2_identifier);

        let use_models = vec![
            {
                let model_info = ast::ModelInfo::new(child1_identifier_node, vec![], None);
                let model_info_node = ast::Node::new(&helper::unimportant_span(), model_info);
                ast::UseModel::new(vec![], model_info_node, None, ModelKind::Submodel)
            },
            {
                let model_info = ast::ModelInfo::new(child2_identifier_node, vec![], None);
                let model_info_node = ast::Node::new(&helper::unimportant_span(), model_info);
                ast::UseModel::new(vec![], model_info_node, None, ModelKind::Submodel)
            },
        ];
        let use_models = use_models
            .into_iter()
            .map(|use_model| ast::Node::new(&helper::unimportant_span(), use_model))
            .collect::<Vec<_>>();
        let use_models_ref = use_models.iter().collect::<Vec<_>>();

        let builtin_ref = helper::create_test_builtin_ref();

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
        assert!(models.contains_key(&ModelPath::new("child1")));
        assert!(models.contains_key(&ModelPath::new("child2")));
    }

    #[test]
    fn test_load_use_models_with_parse_errors() {
        // create initial context
        let model_path = ModelPath::new("parent");
        let builtin_ref = helper::create_test_builtin_ref();
        let mut load_stack = Stack::new();
        let file_loader = TestFileParser::empty(); // No models available

        let use_model_name = ast::Identifier::new("nonexistent".to_string());
        let use_model_name_node =
            ast::Node::new(&helper::span_from_str("nonexistent"), use_model_name);

        let use_models = vec![{
            let model_info = ast::ModelInfo::new(use_model_name_node, vec![], None);
            let model_info_node = ast::Node::new(&helper::unimportant_span(), model_info);
            ast::UseModel::new(vec![], model_info_node, None, ModelKind::Submodel)
        }];
        let use_models = use_models
            .into_iter()
            .map(|use_model| ast::Node::new(&helper::unimportant_span(), use_model))
            .collect::<Vec<_>>();
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

        let error = errors.get(&ModelPath::new("nonexistent"));
        assert_eq!(error, Some(&LoadError::ParseError(())));

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
        let builtin_ref = helper::create_test_builtin_ref();
        let mut load_stack = Stack::new();

        // create a dependency chain: root.on -> level1.on -> level2.on
        let root_model = helper::create_test_model(&["level1"]);
        let level1_model = helper::create_test_model(&["level2"]);
        let level2_model = helper::create_empty_model_node();

        let file_loader = TestFileParser::new(vec![
            (PathBuf::from("root.on"), root_model),
            (PathBuf::from("level1.on"), level1_model),
            (PathBuf::from("level2.on"), level2_model),
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
        assert!(models.contains_key(&ModelPath::new("root")));
        assert!(models.contains_key(&ModelPath::new("level1")));
        assert!(models.contains_key(&ModelPath::new("level2")));
    }

    #[test]
    fn test_load_model_with_sections() {
        // create initial context
        let model_path = ModelPath::new("test");
        let initial_models = HashSet::from([model_path.clone()]);
        let builder = ModelCollectionBuilder::new(initial_models);
        let builtin_ref = helper::create_test_builtin_ref();
        let mut load_stack = Stack::new();

        // create a model with sections
        let submodel_node = helper::create_empty_model_node();

        // Create a section header
        let section_label = ast::Label::new("section1".to_string());
        let section_label_node = ast::Node::new(&helper::span_from_str("section1"), section_label);
        let section_header = ast::SectionHeader::new(section_label_node);
        let section_header_node = ast::Node::new(&helper::unimportant_span(), section_header);

        // Create a use model declaration
        let use_model_name = ast::Identifier::new("submodel".to_string());
        let use_model_name_node =
            ast::Node::new(&helper::span_from_str("submodel"), use_model_name);
        let model_info = ast::ModelInfo::new(use_model_name_node, vec![], None);
        let model_info_node = ast::Node::new(&helper::unimportant_span(), model_info);
        let use_model = ast::UseModel::new(vec![], model_info_node, None, ModelKind::Submodel);
        let use_model_node = ast::Node::new(&helper::unimportant_span(), use_model);
        let use_model_decl = ast::Node::new(
            &helper::unimportant_span(),
            ast::Decl::use_model(use_model_node),
        );

        // Create a section
        let section = ast::Section::new(section_header_node, None, vec![use_model_decl]);
        let section_node = ast::Node::new(&helper::unimportant_span(), section);

        // Create the model
        let model = ast::Model::new(None, vec![], vec![section_node]);
        let model_node = ast::Node::new(&helper::unimportant_span(), model);

        let file_loader = TestFileParser::new(vec![
            (PathBuf::from("test.on"), model_node),
            (PathBuf::from("submodel.on"), submodel_node),
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
        assert_eq!(models.len(), 2);
        assert!(models.contains_key(&ModelPath::new("test")));
        assert!(models.contains_key(&ModelPath::new("submodel")));
    }
}
