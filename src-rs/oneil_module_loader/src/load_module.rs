use oneil_ast as ast;
use oneil_module::{
    ExternalImportMap, Ident, ImportIndex, Module, ModuleCollection, ModulePath, ModuleReference,
    PythonPath, SectionItem, Symbol, SymbolMap, TestIndex, Tests,
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
    file_parser: F,
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

    let result = process_model(file_ast, &module_path);

    match result {
        Ok(module) => {
            module_collection.add_module(&module_path, module);
        }
        Err(e) => {
            module_errors.add_error(&module_path, ModuleLoaderError::resolution_error(e));
        }
    };

    (module_collection, module_errors)
}

fn process_model(ast: ast::Model, module_path: &ModulePath) -> Result<Module, ResolutionError> {
    // Gather symbol, test, external_imports, and section data
    let (top_symbols, top_tests, top_external_imports, top_section_data) =
        process_section(ast.decls, module_path);

    todo!("load the modules");
    todo!("verify that symbols are valid");
    todo!("return the constructed module")
}

fn process_section(
    decls: Vec<ast::Decl>,
    module_path: &ModulePath,
) -> (SymbolMap, Tests, ExternalImportMap, Vec<SectionItem>) {
    let mut symbols = SymbolMap::new();
    let mut tests = Tests::new();
    let mut external_imports = ExternalImportMap::new();
    let mut section_items = vec![];

    for decl in decls {
        match decl {
            ast::Decl::Import { path } => {
                let import_path = module_path.join(&path);
                external_imports.add_import(PythonPath::new(import_path));
                // TODO: make this the right index
                section_items.push(SectionItem::ExternalImport(ImportIndex::new(0)));
            }
            ast::Decl::From {
                path,
                use_model,
                inputs,
                as_name,
            } => {
                let (path, rest) = path
                    .split_first()
                    .expect("TODO: in AST, store path and 'child accessors' seperately");
                let use_path = ModulePath::new(module_path.join(&path));

                // TODO: figure out what to do with inputs - maybe turn them into tests?

                let ident = Ident::new(as_name.clone());

                // TODO: try not to clone the idents
                let mut subcomponents = rest
                    .into_iter()
                    .map(|s| Ident::new(s.clone()))
                    .collect::<Vec<_>>();
                subcomponents.push(Ident::new(use_model));

                let symbol = Symbol::Import(ModuleReference::new(use_path, subcomponents));
                symbols.add_symbol(ident.clone(), symbol);

                // TODO: I think this needs more information to be useful
                section_items.push(SectionItem::InternalImport(ident));
            }
            ast::Decl::Use {
                path,
                inputs,
                as_name,
            } => {
                let (path, rest) = path
                    .split_first()
                    .expect("TODO: in AST, store path and 'child accessors' seperately");
                let use_path = ModulePath::new(module_path.join(&path));

                let ident = Ident::new(as_name.clone());

                let subcomponents = rest
                    .into_iter()
                    .map(|s| Ident::new(s.clone()))
                    .collect::<Vec<_>>();

                let symbol = Symbol::Import(ModuleReference::new(use_path, subcomponents));
                symbols.add_symbol(ident.clone(), symbol);

                // TODO: I think this needs more information to be useful
                section_items.push(SectionItem::InternalImport(ident));
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

    (symbols, tests, external_imports, section_items)
}

// TODO: write tests for the module loader
