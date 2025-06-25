use std::collections::HashSet;

use oneil_module::{Dependency, ModuleGraph, ModulePath};

use crate::{
    FileLoader, builder,
    error::{ModuleErrorCollection, ModuleLoaderError, ResolutionError},
    util::Stack,
};

pub fn load_module<F>(
    module_path: ModulePath,
    module_stack: &mut Stack<ModulePath>,
    module_collection: ModuleGraph,
    mut module_errors: ModuleErrorCollection<F::ParseError>,
    file_parser: &F,
) -> (ModuleGraph, ModuleErrorCollection<F::ParseError>)
where
    F: FileLoader,
{
    // Check for cyclical dependencies in modules
    let cyclical_dependency = module_stack.check_for_cyclical_dependency(&module_path);
    if let Some(cyclical_deps) = cyclical_dependency {
        module_errors.add_error(
            &module_path,
            ModuleLoaderError::cyclical_dependency(cyclical_deps),
        );
        return (module_collection, module_errors);
    }

    // Check if the module has already been loaded
    if module_collection.has_loaded_for(&module_path) || module_errors.has_error_for(&module_path) {
        return (module_collection, module_errors);
    }

    // Parse the module
    let file_ast = file_parser.parse_ast(&module_path);

    // If the module fails to parse, add an error and return
    let file_ast = match file_ast {
        Ok(ast) => ast,
        Err(e) => {
            // TODO: we might be able to return a partial module here
            module_errors.add_error(&module_path, ModuleLoaderError::parse_error(e));
            return (module_collection, module_errors);
        }
    };

    // Process the AST into a module
    let module = builder::build_model_module(file_ast, &module_path);

    // Load module dependencies
    let (mut module_collection, module_errors) = load_dependencies(
        &module_path,
        todo!(),
        module_stack,
        module_collection,
        module_errors,
        file_parser,
    );

    // TODO: check for circular dependencies within module parameters

    module_collection.add_module(&module_path, module);

    // Return the module collection and errors
    (module_collection, module_errors)
}

fn load_dependencies<F>(
    module_path: &ModulePath,
    dependencies: &HashSet<Dependency>,
    module_stack: &mut Stack<ModulePath>,
    module_collection: ModuleGraph,
    module_errors: ModuleErrorCollection<F::ParseError>,
    file_parser: &F,
) -> (ModuleGraph, ModuleErrorCollection<F::ParseError>)
where
    F: FileLoader,
{
    // Push the current module onto the stack
    module_stack.push(module_path.clone());

    let (module_collection, module_errors) = dependencies.iter().fold(
        (module_collection, module_errors),
        |(module_collection, mut module_errors), dependency| {
            match dependency {
                Dependency::Python(python_path) => {
                    if !file_parser.file_exists(&python_path) {
                        module_errors.add_error(
                            module_path,
                            ModuleLoaderError::resolution_error(
                                ResolutionError::python_file_not_found(python_path.clone()),
                            ),
                        );
                    }

                    // TODO: should we validate that it's valid python?

                    (module_collection, module_errors)
                }
                Dependency::Module(dependency_path) => {
                    let (mut module_collection, module_errors) = load_module(
                        dependency_path.clone(),
                        module_stack,
                        module_collection,
                        module_errors,
                        file_parser,
                    );

                    // Add the current module as a dependent of the dependency module
                    module_collection.add_dependent_module(module_path, dependency_path.clone());

                    (module_collection, module_errors)
                }
            }
        },
    );

    // Pop the current module off the stack
    module_stack.pop();

    (module_collection, module_errors)
}
