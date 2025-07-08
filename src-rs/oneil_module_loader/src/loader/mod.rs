//! Module loading and resolution functionality for Oneil programs.
//!
//! This module provides the core functionality for loading Oneil modules from files,
//! resolving dependencies, and building module collections. It handles:
//!
//! - Circular dependency detection
//! - Module parsing and AST processing
//! - Import validation
//! - Submodel resolution
//! - Parameter resolution
//! - Test resolution
//!

use std::collections::HashSet;

use oneil_ast as ast;
use oneil_module::{
    module::Module,
    parameter::Parameter,
    reference::{Identifier, ModulePath},
};

use crate::{
    error::{LoadError, ResolutionErrors},
    util::{FileLoader, Stack, builder::ModuleCollectionBuilder, info::InfoMap},
};

mod importer;
mod resolver;

/// Loads a module and all its dependencies, building a complete module collection.
///
/// This function is the main entry point for module loading. It performs the following steps:
///
/// 1. **Circular dependency detection**: Checks if loading this module would create a circular dependency
/// 2. **Module visitation tracking**: Prevents loading the same module multiple times
/// 3. **AST parsing**: Parses the module file into an AST using the provided file loader
/// 4. **Declaration splitting**: Separates imports, use models, parameters, and tests from the AST
/// 5. **Import validation**: Validates Python imports and collects any validation errors
/// 6. **Use model loading**: Recursively loads all referenced use models
/// 7. **Resolution**: Resolves submodels, parameters, and tests using the loaded module information
/// 8. **Module construction**: Builds the final module and adds it to the collection
///
/// # Arguments
///
/// * `module_path` - The path to the module to load
/// * `builder` - The module collection builder that accumulates modules and errors
/// * `load_stack` - A stack tracking the current loading path for circular dependency detection
/// * `file_loader` - The file loader implementation for parsing and validation
///
/// # Returns
///
/// Returns the updated module collection builder containing all loaded modules and any errors.
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
/// The function detects circular dependencies by maintaining a loading stack. If a module
/// appears in the stack while it's being loaded, a circular dependency is detected:
///
/// # Module Visitation
///
/// To prevent loading the same module multiple times, the function tracks visited modules
/// in the builder. If a module has already been visited, it returns early without
/// re-processing the module.
pub fn load_module<F>(
    module_path: ModulePath,
    mut builder: ModuleCollectionBuilder<F::ParseError, F::PythonError>,
    load_stack: &mut Stack<ModulePath>,
    file_loader: &F,
) -> ModuleCollectionBuilder<F::ParseError, F::PythonError>
where
    F: FileLoader,
{
    // check for circular dependencies
    //
    // this happens before we check if the module has been visited because if
    // there is a circular dependency, it will have already been visited
    if let Some(circular_dependency) = load_stack.find_circular_dependency(&module_path) {
        builder.add_circular_dependency_error(module_path, circular_dependency);
        return builder;
    }

    // check if module is already been visited, then mark as visited if not
    if builder.module_has_been_visited(&module_path) {
        return builder;
    }
    builder.mark_module_as_visited(&module_path);

    // parse model ast
    let model_ast = file_loader.parse_ast(module_path.clone());

    // TODO: this might be able to recover and produce a partial module?
    let model_ast = match model_ast {
        Ok(model_ast) => model_ast,
        Err(error) => {
            builder.add_module_error(module_path, LoadError::ParseError(error));
            return builder;
        }
    };

    // split model ast into imports, use models, parameters, and tests
    let (imports, use_models, parameters, tests) = split_model_ast(model_ast);

    // validate imports
    let (python_imports, import_resolution_errors, builder) =
        importer::validate_imports(&module_path, builder, imports, file_loader);

    // load use models and resolve them
    let mut builder = load_use_models(
        module_path.clone(),
        load_stack,
        file_loader,
        &use_models,
        builder,
    );

    let module_info: InfoMap<&ModulePath, &Module> = InfoMap::new(
        builder.get_modules().into_iter().collect(),
        builder.get_modules_with_errors(),
    );

    // resolve submodels
    let (submodels, submodel_tests, submodel_resolution_errors) =
        resolver::resolve_submodels_and_tests(use_models, &module_path, &module_info);

    let submodels_with_errors: HashSet<&Identifier> = submodel_resolution_errors.keys().collect();

    let submodel_info: InfoMap<&Identifier, &ModulePath> =
        InfoMap::new(submodels.iter().collect(), submodels_with_errors);

    // resolve parameters
    let (parameters, parameter_resolution_errors) =
        resolver::resolve_parameters(parameters, &submodel_info, &module_info);

    let parameters_with_errors: HashSet<&Identifier> = parameter_resolution_errors.keys().collect();

    let parameter_info: InfoMap<&Identifier, &Parameter> =
        InfoMap::new(parameters.iter().collect(), parameters_with_errors);

    // resolve model tests
    let (model_tests, model_test_resolution_errors) =
        resolver::resolve_model_tests(tests, &parameter_info, &submodel_info, &module_info);

    // resolve submodel tests
    let (submodel_tests, submodel_test_resolution_errors) = resolver::resolve_submodel_tests(
        submodel_tests,
        &parameter_info,
        &submodel_info,
        &module_info,
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
        builder.add_module_error(module_path.clone(), resolution_errors);
    }

    // build module
    let module = Module::new(
        python_imports,
        submodels,
        parameters,
        model_tests,
        submodel_tests,
    );

    // add module to builder
    builder.add_module(module_path, module);

    builder
}

/// Splits a model AST into its constituent declaration types.
///
/// This function processes the declarations in a model AST and categorizes them into
/// separate collections for imports, use models, parameters, and tests. This separation
/// is necessary for the different processing steps in module loading.
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

/// Recursively loads all use models referenced by a module.
///
/// This function processes all use model declarations in a module and recursively
/// loads each referenced model. It maintains the loading stack for circular
/// dependency detection and accumulates all loaded modules in the builder.
///
/// # Arguments
///
/// * `module_path` - The path of the current module (used for resolving relative paths)
/// * `load_stack` - The loading stack for circular dependency detection
/// * `file_loader` - The file loader for parsing referenced models
/// * `use_models` - The use model declarations to process
/// * `builder` - The module collection builder to accumulate results
///
/// # Returns
///
/// Returns the updated module collection builder containing all loaded use models.
///
/// # Circular Dependency Handling
///
/// The function pushes the current module path onto the load stack before processing
/// use models and pops it after processing is complete. This ensures that circular
/// dependencies are properly detected during the recursive loading process.
///
/// # Path Resolution
///
/// Use model paths are resolved relative to the current module path using
/// `module_path.get_sibling_path(&use_model.model_name)`.
fn load_use_models<F>(
    module_path: ModulePath,
    load_stack: &mut Stack<ModulePath>,
    file_loader: &F,
    use_models: &Vec<ast::declaration::UseModel>,
    builder: ModuleCollectionBuilder<F::ParseError, F::PythonError>,
) -> ModuleCollectionBuilder<F::ParseError, F::PythonError>
where
    F: FileLoader,
{
    load_stack.push(module_path.clone());

    // TODO: check for duplicate use models
    let builder = use_models.into_iter().fold(builder, |builder, use_model| {
        // get the use model path
        let use_model_path = module_path.get_sibling_path(&use_model.model_name);
        let use_model_path = ModulePath::new(use_model_path);

        // load the use model (and its submodels)
        load_module(use_model_path, builder, load_stack, file_loader)
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
    fn test_load_module_success() {
        // create initial context
        let module_path = ModulePath::new("test");
        let initial_modules = HashSet::from([module_path.clone()]);
        let builder = ModuleCollectionBuilder::new(initial_modules);
        let mut load_stack = Stack::new();

        let file_loader =
            TestFileParser::new(vec![(PathBuf::from("test.on"), create_test_model(&[]))]);

        // load the module
        let result = load_module(module_path.clone(), builder, &mut load_stack, &file_loader);

        // check the errors
        let errors = result.get_module_errors();
        assert!(errors.is_empty());

        // check the modules
        let modules = result.get_modules();
        assert_eq!(modules.len(), 1);
        assert!(modules.contains_key(&module_path));
    }

    #[test]
    fn test_load_module_parse_error() {
        // create initial context
        let module_path = ModulePath::new("nonexistent");
        let initial_modules = HashSet::from([module_path.clone()]);
        let builder = ModuleCollectionBuilder::new(initial_modules);
        let mut load_stack = Stack::new();

        let file_loader = TestFileParser::empty();

        // load the module
        let result = load_module(module_path.clone(), builder, &mut load_stack, &file_loader);

        // check the errors
        let errors = result.get_module_errors();
        assert_eq!(errors.len(), 1);

        let error = errors.get(&module_path).unwrap();
        assert_eq!(error, &LoadError::ParseError(()));

        // check the modules
        let modules = result.get_modules();
        assert!(modules.is_empty());
    }

    #[test]
    fn test_load_module_circular_dependency() {
        // create initial context
        let module_path = ModulePath::new("main");
        let initial_modules = HashSet::from([module_path.clone()]);
        let builder = ModuleCollectionBuilder::new(initial_modules);
        let mut load_stack = Stack::new();

        // create a circular dependency: main.on -> sub.on -> main.on
        let file_loader = TestFileParser::new(vec![
            (PathBuf::from("main.on"), create_test_model(&["sub"])),
            (PathBuf::from("sub.on"), create_test_model(&["main"])),
        ]);

        // load the module
        let result = load_module(module_path, builder, &mut load_stack, &file_loader);

        // check the errors
        let errors = result.get_module_errors();
        assert_eq!(errors.len(), 2);

        let main_errors = errors.get(&ModulePath::new("main")).unwrap();
        assert_eq!(
            main_errors,
            &LoadError::resolution_errors(ResolutionErrors::new(
                HashMap::new(),
                HashMap::from([(
                    Identifier::new("sub"),
                    SubmodelResolutionError::module_has_error(ModulePath::new("sub"))
                )]),
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
            ))
        );

        let sub_errors = errors.get(&ModulePath::new("sub")).unwrap();
        assert_eq!(
            sub_errors,
            &LoadError::resolution_errors(ResolutionErrors::new(
                HashMap::new(),
                HashMap::from([(
                    Identifier::new("main"),
                    SubmodelResolutionError::module_has_error(ModulePath::new("main"))
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
            .get(&ModulePath::new("main"))
            .unwrap();
        assert_eq!(circular_dependency_error.len(), 1);
        assert_eq!(
            circular_dependency_error[0],
            CircularDependencyError::new(vec![
                ModulePath::new("main"),
                ModulePath::new("sub"),
                ModulePath::new("main"),
            ])
        );
    }

    // #[test]
    // fn test_load_module_already_visited() {
    //     let module_path = ModulePath::new("test.on");
    //     let initial_modules = HashSet::from([module_path.clone()]);
    //     let mut builder = ModuleCollectionBuilder::new(initial_modules);
    //     let mut load_stack = Stack::new();

    //     // Mark the module as already visited
    //     builder.mark_module_as_visited(&module_path);

    //     let file_loader =
    //         TestFileParser::new(vec![(PathBuf::from("test.on"), create_test_model())]);

    //     let result = load_module(module_path, builder, &mut load_stack, &file_loader);

    //     // Should not have loaded the module again
    //     let modules = result.get_modules();
    //     assert!(modules.is_empty());
    // }

    // #[test]
    // fn test_load_use_models_empty() {
    //     let module_path = ModulePath::new("parent.on");
    //     let mut load_stack = Stack::new();
    //     let file_loader = TestFileParser::empty();
    //     let use_models = vec![];
    //     let builder = ModuleCollectionBuilder::new(HashSet::new());

    //     let result = load_use_models(
    //         module_path,
    //         &mut load_stack,
    //         &file_loader,
    //         &use_models,
    //         builder,
    //     );

    //     // Should have no modules loaded
    //     let modules = result.get_modules();
    //     assert!(modules.is_empty());
    // }

    // #[test]
    // fn test_load_use_models_with_existing_models() {
    //     let module_path = ModulePath::new("parent.on");
    //     let mut load_stack = Stack::new();
    //     let file_loader = TestFileParser::new(vec![
    //         (PathBuf::from("child1.on"), create_empty_model()),
    //         (
    //             PathBuf::from("child2.on"),
    //             create_use_model_only_model_no_imports(),
    //         ),
    //     ]);

    //     let use_models = vec![
    //         UseModel {
    //             model_name: "child1.on".to_string(),
    //             subcomponents: vec![],
    //             inputs: None,
    //             as_name: None,
    //         },
    //         UseModel {
    //             model_name: "child2.on".to_string(),
    //             subcomponents: vec![],
    //             inputs: None,
    //             as_name: None,
    //         },
    //     ];

    //     let builder = ModuleCollectionBuilder::new(HashSet::new());

    //     let result = load_use_models(
    //         module_path,
    //         &mut load_stack,
    //         &file_loader,
    //         &use_models,
    //         builder,
    //     );

    //     // Should have loaded both child modules
    //     let modules = result.get_modules();
    //     assert_eq!(modules.len(), 2);
    //     assert!(modules.contains_key(&ModulePath::new("child1.on")));
    //     assert!(modules.contains_key(&ModulePath::new("child2.on")));
    // }

    // #[test]
    // fn test_load_use_models_with_parse_errors() {
    //     let module_path = ModulePath::new("parent.on");
    //     let mut load_stack = Stack::new();
    //     let file_loader = TestFileParser::empty(); // No models available

    //     let use_models = vec![UseModel {
    //         model_name: "nonexistent.on".to_string(),
    //         subcomponents: vec![],
    //         inputs: None,
    //         as_name: None,
    //     }];

    //     let builder = ModuleCollectionBuilder::new(HashSet::new());

    //     let result = load_use_models(
    //         module_path,
    //         &mut load_stack,
    //         &file_loader,
    //         &use_models,
    //         builder,
    //     );

    //     // Should have parse errors for the nonexistent module
    //     let modules_with_errors = result.get_modules_with_errors();
    //     assert_eq!(modules_with_errors.len(), 1);
    //     assert!(modules_with_errors.contains(&&ModulePath::new("nonexistent.on")));
    // }

    // #[test]
    // fn test_load_module_complex_dependency_chain() {
    //     let module_path = ModulePath::new("root.on");
    //     let initial_modules = HashSet::from([module_path.clone()]);
    //     let builder = ModuleCollectionBuilder::new(initial_modules);
    //     let mut load_stack = Stack::new();

    //     // Create a dependency chain: root.on -> level1.on -> level2.on
    //     let root_model = Model {
    //         note: None,
    //         decls: vec![Decl::UseModel(UseModel {
    //             model_name: "level1.on".to_string(),
    //             subcomponents: vec![],
    //             inputs: None,
    //             as_name: None,
    //         })],
    //         sections: vec![],
    //     };

    //     let level1_model = Model {
    //         note: None,
    //         decls: vec![Decl::UseModel(UseModel {
    //             model_name: "level2.on".to_string(),
    //             subcomponents: vec![],
    //             inputs: None,
    //             as_name: None,
    //         })],
    //         sections: vec![],
    //     };

    //     let level2_model = create_empty_model();

    //     let file_loader = TestFileParser::new(vec![
    //         (PathBuf::from("root.on"), root_model),
    //         (PathBuf::from("level1.on"), level1_model),
    //         (PathBuf::from("level2.on"), level2_model),
    //     ]);

    //     let result = load_module(module_path, builder, &mut load_stack, &file_loader);

    //     // Should have loaded all three modules
    //     let modules = result.get_modules();
    //     assert_eq!(modules.len(), 3);
    //     assert!(modules.contains_key(&ModulePath::new("root.on")));
    //     assert!(modules.contains_key(&ModulePath::new("level1.on")));
    //     assert!(modules.contains_key(&ModulePath::new("level2.on")));
    // }
}
