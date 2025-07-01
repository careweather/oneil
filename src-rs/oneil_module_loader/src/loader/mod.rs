use oneil_ast as ast;
use oneil_module::{module::Module, reference::ModulePath};

use crate::util::{FileLoader, Stack, builder::ModuleCollectionBuilder};

mod importer;
mod resolver;

pub fn load_module<F>(
    module_path: ModulePath,
    mut builder: ModuleCollectionBuilder,
    load_stack: &mut Stack<ModulePath>,
    file_loader: &F,
) -> ModuleCollectionBuilder
where
    F: FileLoader,
{
    // check for circular dependencies
    if load_stack.contains(&module_path) {
        builder.add_error(module_path, todo!("circular dependency error"));
        return builder;
    }

    // check if module is already been visited, then mark as visited if not
    if builder.module_has_been_visited(&module_path) {
        return builder;
    }
    builder.mark_module_as_visited(&module_path);

    // parse model ast
    let model_ast = file_loader
        .parse_ast(module_path)
        .unwrap_or(todo!("failed to parse model ast"));

    // split model ast into imports, use models, parameters, and tests
    let (imports, use_models, parameters, tests) = split_model_ast(model_ast);

    // validate imports
    let (python_imports, builder) =
        importer::validate_imports(&module_path, builder, imports, file_loader);

    // load use models and resolve them
    let builder = load_use_models(module_path, load_stack, file_loader, use_models, builder);

    // resolve submodels
    let (submodels, submodel_tests, builder) =
        resolver::resolve_submodels_and_tests(use_models, &module_path, builder);

    // resolve parameters
    let parameters = resolver::resolve_parameters(parameters, builder);

    // resolve submodel tests and build tests
    let model_tests = resolver::resolve_model_tests(tests, builder);
    let submodel_tests = resolver::resolve_submodel_tests(submodel_tests, builder);

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

fn load_use_models<F>(
    module_path: ModulePath,
    load_stack: &mut Stack<ModulePath>,
    file_loader: &F,
    use_models: Vec<ast::declaration::UseModel>,
    builder: ModuleCollectionBuilder,
) -> ModuleCollectionBuilder
where
    F: FileLoader,
{
    // TODO: check for duplicate use models
    use_models.into_iter().fold(builder, |builder, use_model| {
        // get the use model path
        let use_model_path = module_path.get_sibling_path(&use_model.model_name);
        let use_model_path = ModulePath::new(use_model_path);

        // load the use model (and its submodels)
        load_module(use_model_path.clone(), builder, load_stack, file_loader)
    })
}
