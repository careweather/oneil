use oneil_module::{
    Dependency, Module, ModuleCollection, ModulePath, ModuleReference, Reference, Symbol,
    TestDependency,
};

pub fn check(module_collection: &ModuleCollection) -> Result<(), ()> {
    validate_dependencies(module_collection)?;
    ensure_no_circular_dependencies(module_collection)?;

    Ok(())
}

fn validate_dependencies(module_collection: &ModuleCollection) -> Result<(), ()> {
    let modules = module_collection.modules();

    for (module_path, module) in modules {
        validate_module_dependencies(module, module_path, module_collection)?;
        validate_parameter_dependencies(module, module_path, module_collection)?;
        validate_test_dependencies(module, module_path, module_collection)?;
    }

    Ok(())
}

fn validate_module_dependencies(
    module: &Module,
    module_path: &ModulePath,
    module_collection: &ModuleCollection,
) -> Result<(), ()> {
    let modules = module_collection.modules();

    for dependency in module.dependencies() {
        match dependency {
            Dependency::Module(dep_path) => {
                if !modules.contains_key(dep_path) {
                    return Err(todo!("Implement the right error"));
                }
            }
            Dependency::Python(_) => {
                // Python dependencies are external and don't need validation here
                // They would be validated during the loading phase
            }
        }
    }

    Ok(())
}

fn validate_parameter_dependencies(
    module: &Module,
    module_path: &ModulePath,
    module_collection: &ModuleCollection,
) -> Result<(), ()> {
    for (_param_ident, param_deps) in module.parameter_dependencies() {
        for dep in param_deps {
            if !is_valid_reference(dep, module, module_collection) {
                return Err(todo!("Implement the right error"));
            }
        }
    }

    Ok(())
}

//     // check that module test dependencies are valid
//     for (test_index, test_deps) in module.test_dependencies() {
//         // Get the corresponding test from the module's model tests
//         let test_index_usize = test_index.index();
//         let model_tests = module.tests().model_tests();

//         if test_index_usize >= model_tests.len() {
//             return Err(());
//         }

//         let test = &model_tests[test_index_usize];

//         for dep in test_deps {
//             match dep {
//                 TestDependency::TestInput(ref_input) => {
//                     // Check if the test input exists in the test's inputs list
//                     if !is_valid_test_input(ref_input, test) {
//                         return Err(());
//                     }
//                 }
//                 TestDependency::Other(ref_reference) => {
//                     if !is_valid_reference(ref_reference, module, module_collection) {
//                         return Err(());
//                     }
//                 }
//             }
//         }
//     }
// }

fn ensure_no_circular_dependencies(module_collection: &ModuleCollection) -> Result<(), ()> {
    todo!()
}

/// Check if a reference is valid within the context of a module and module collection
fn is_valid_reference(
    reference: &Reference,
    module: &Module,
    module_collection: &ModuleCollection,
) -> Result<(), ()> {
    match reference {
        Reference::Identifier(ident) => {
            // Check if the identifier exists in the current module's symbols
            // and ensure that it's a parameter
            match module.symbols().get(ident) {
                Some(Symbol::Parameter(_)) => Ok(()),
                _ => Err(todo!("Implement the right error")),
            }
        }
        Reference::Accessor { parent, component } => {
            // Check if the parent exists in the current module's symbols
            let parent_symbol = module.symbols().get(parent);
            match parent_symbol {
                Some(Symbol::Import(import_ref)) => {
                    let import_module = resolve_import(import_ref, module_collection)?;

                    // Recursively check the component reference
                    is_valid_reference(component, import_module, module_collection)
                }
                Some(Symbol::Parameter(_)) => Err(todo!("Implement the right error")),
                None => Err(todo!("Implement the right error")),
            }
        }
    }
}

fn resolve_import<'a>(
    import_ref: &ModuleReference,
    module_collection: &'a ModuleCollection,
) -> Result<&'a Module, ()> {
    let modules = module_collection.modules();
    let module_path = import_ref.module_path();

    let import_module = modules
        .get(&module_path)
        .ok_or(todo!("Implement the right error"))?;

    match import_ref.reference() {
        Some(Reference::Identifier(ident)) => {
            let import_module = modules
                .get(&module_path)
                .ok_or(todo!("Implement the right error"));

            import_module
        }
        None => Ok(import_module),
    }
}

/// Check if a reference is a valid test input for a given test
fn is_valid_test_input(reference: &Reference, test: &oneil_ast::Test) -> bool {
    match reference {
        Reference::Identifier(ident) => {
            // Check if the identifier exists in the test's inputs list
            test.inputs.contains(&ident.as_str().to_string())
        }
        Reference::Accessor { parent, .. } => {
            // For accessor references, check if the parent exists in the test's inputs list
            test.inputs.contains(&parent.as_str().to_string())
        }
    }
}
