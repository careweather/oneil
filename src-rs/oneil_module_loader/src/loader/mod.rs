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
        builder.add_module_error(
            module_path,
            LoadError::module_circular_dependency(circular_dependency),
        );
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
