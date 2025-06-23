use std::collections::HashMap;

use oneil_ast as ast;
use oneil_module::{
    Dependency, DocumentationMap, ExternalImportMap, Ident, ImportIndex, Module, ModuleCollection,
    ModulePath, ModuleReference, PythonPath, SectionData, SectionItem, SectionLabel, Symbol,
    SymbolMap, TestIndex, Tests,
};

use crate::{
    error::{ModuleErrorCollection, ModuleLoaderError, ResolutionError},
    module_stack::ModuleStack,
    traits::FileParser,
};

pub fn load_module<F>(
    module_path: ModulePath,
    module_stack: ModuleStack,
    mut module_collection: ModuleCollection,
    mut module_errors: ModuleErrorCollection<F::ParseError>,
    file_parser: &F,
) -> (ModuleCollection, ModuleErrorCollection<F::ParseError>)
where
    F: FileParser,
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

    let file_ast = file_parser.parse_ast(&module_path);

    let file_ast = match file_ast {
        Ok(ast) => ast,
        Err(e) => {
            module_errors.add_error(&module_path, ModuleLoaderError::parse_error(e));
            return (module_collection, module_errors);
        }
    };

    let (module, module_processing_errors) = process_model(file_ast, &module_path);

    let (module_collection, module_errors) = load_dependencies(
        &module_path,
        module.get_dependencies(),
        module_stack,
        module_collection,
        module_errors,
        file_parser,
    );

    todo!("verify that symbols are valid");
    todo!("return the constructed module");

    if module_processing_errors.is_empty() {
        module_collection.add_module(&module_path, module);
    } else {
        for error in module_processing_errors {
            module_errors.add_error(&module_path, ModuleLoaderError::resolution_error(error));
        }
    }

    (module_collection, module_errors)
}

fn process_model(model: ast::Model, module_path: &ModulePath) -> (Module, Vec<ResolutionError>) {
    let module_errors = vec![];

    // TODO: if this fails, return a partial module with the errors
    let symbols = SymbolMap::new();
    let tests = Tests::new();
    let external_imports = ExternalImportMap::new();
    let section_items = vec![];
    let dependencies = vec![];

    // Gather symbol, test, external_imports, and section data
    let (top_symbols, top_tests, top_external_imports, top_section_data, top_dependencies) =
        process_section(
            model.decls,
            module_path,
            symbols,
            tests,
            external_imports,
            section_items,
            dependencies,
        );

    let top_section_data = SectionData::new(model.note, top_section_data);

    let (symbols, tests, external_imports, section_data, dependencies) =
        model.sections.into_iter().fold(
            (
                top_symbols,
                top_tests,
                top_external_imports,
                HashMap::new(),
                top_dependencies,
            ),
            |(
                acc_symbols,
                acc_tests,
                acc_external_imports,
                mut acc_section_items,
                dependencies,
            ),
             section| {
                let section_label = SectionLabel::new(section.label);
                let (symbols, tests, external_imports, section_items, dependencies) =
                    process_section(
                        section.decls,
                        module_path,
                        acc_symbols,
                        acc_tests,
                        acc_external_imports,
                        vec![],
                        dependencies,
                    );
                let section_items = SectionData::new(section.note, section_items);
                acc_section_items.insert(section_label, section_items);
                (
                    symbols,
                    tests,
                    external_imports,
                    acc_section_items,
                    dependencies,
                )
            },
        );

    let documentation_map = DocumentationMap::new(top_section_data, section_data);

    let module = Module::new(
        module_path.clone(),
        symbols,
        tests,
        external_imports,
        documentation_map,
        dependencies,
    );

    (module, module_errors)
}

fn process_section(
    decls: Vec<ast::Decl>,
    module_path: &ModulePath,
    mut symbols: SymbolMap,
    mut tests: Tests,
    mut external_imports: ExternalImportMap,
    mut section_items: Vec<SectionItem>,
    mut dependencies: Vec<Dependency>,
) -> (
    SymbolMap,
    Tests,
    ExternalImportMap,
    Vec<SectionItem>,
    Vec<Dependency>,
) {
    for decl in decls {
        match decl {
            ast::Decl::Import { path } => {
                let import_path = module_path.join(&path);
                let import_path = PythonPath::new(import_path);
                dependencies.push(Dependency::Python(import_path.clone()));
                external_imports.add_import(import_path);
                // TODO: make this the right index
                section_items.push(SectionItem::ExternalImport(ImportIndex::new(0)));
            }
            ast::Decl::UseModel {
                model_name,
                subcomponents,
                inputs,
                as_name,
            } => {
                let use_path = ModulePath::new(module_path.join(&model_name));

                // TODO: figure out what to do with inputs - maybe turn them into tests?

                let symbol_name = as_name
                    .as_ref()
                    .unwrap_or(subcomponents.last().unwrap_or(&model_name));
                let symbol_name = Identifier::new(symbol_name.clone());

                let subcomponents = subcomponents
                    .into_iter()
                    .map(|s| Identifier::new(s))
                    .collect::<Vec<_>>();

                let symbol = Symbol::Import(ModuleReference::new(use_path.clone(), subcomponents));
                symbols.add_symbol(symbol_name.clone(), symbol);

                // TODO: I think this needs more information to be useful
                section_items.push(SectionItem::InternalImport(symbol_name));

                dependencies.push(Dependency::Module(use_path));
            }
            ast::Decl::Parameter(parameter) => {
                // TODO: figure out if these clones are necessary
                let ident = Ident::new(parameter.name.clone());
                let symbol = Symbol::Parameter(parameter);
                symbols.add_symbol(ident.clone(), symbol);
                section_items.push(SectionItem::Parameter(ident));
            }
            ast::Decl::Test(test) => {
                tests.add_test(test);
                // TODO: Figure out what the right index is for this
                section_items.push(SectionItem::Test(TestIndex::new(0)));
            }
        }
    }

    (
        symbols,
        tests,
        external_imports,
        section_items,
        dependencies,
    )
}

fn load_dependencies<F>(
    module_path: &ModulePath,
    dependencies: &[Dependency],
    mut module_stack: ModuleStack,
    mut module_collection: ModuleCollection,
    mut module_errors: ModuleErrorCollection<F::ParseError>,
    file_parser: &F,
) -> (ModuleCollection, ModuleErrorCollection<F::ParseError>)
where
    F: FileParser,
{
    module_stack.push(module_path.clone());

    for dependency in dependencies {
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
            }
            Dependency::Module(module_path) => {
                let (dependency_collection, dependency_errors) = load_module(
                    module_path.clone(),
                    module_stack.clone(),
                    module_collection,
                    module_errors,
                    file_parser,
                );
                module_collection = dependency_collection;
                module_errors = dependency_errors;
            }
        }
    }

    (module_collection, module_errors)
}
// TODO: write tests for the module loader
