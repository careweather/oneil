use std::collections::HashMap;

use oneil_module::{
    parameter::ParameterCollection,
    reference::{Identifier, ModulePath},
    test::{ModelTest, SubmodelTest},
};

use crate::util::builder::ModuleCollectionBuilder;

pub fn resolve_submodels_and_tests(
    use_models: Vec<oneil_ast::declaration::UseModel>,
    module_path: &ModulePath,
    builder: ModuleCollectionBuilder,
) -> (
    HashMap<Identifier, ModulePath>,
    Vec<(ModulePath, Vec<oneil_ast::declaration::ModelInput>)>,
    ModuleCollectionBuilder,
) {
    use_models.into_iter().fold(
        (HashMap::new(), Vec::new(), builder),
        |(mut submodels, mut submodel_tests, mut builder), use_model| {
            // get the use model path
            let use_model_path = module_path.get_sibling_path(&use_model.model_name);
            let use_model_path = ModulePath::new(use_model_path);

            // resolve the use model path
            let resolved_use_model_path =
                resolve_module_path(use_model_path.clone(), &use_model.subcomponents, &builder);

            // insert the use model path into the submodels map if it was resolved successfully
            // otherwise, add the error to the builder
            match resolved_use_model_path {
                Ok(resolved_use_model_path) => {
                    let submodel_name = use_model.as_name.as_ref().unwrap_or(
                        use_model
                            .subcomponents
                            .last()
                            .unwrap_or(&use_model.model_name),
                    );

                    submodels.insert(
                        Identifier::new(submodel_name),
                        resolved_use_model_path.clone(),
                    );

                    // store the inputs for the submodel tests
                    // (the inputs are stored in their AST form for now and converted to
                    // the model input type once all the submodels have been resolved)
                    let inputs = use_model.inputs.unwrap_or_default();
                    submodel_tests.push((resolved_use_model_path, inputs));
                }
                Err(error) => {
                    builder.add_error(use_model_path, error);
                    todo!("make this more accurate")
                }
            }

            (submodels, submodel_tests, builder)
        },
    )
}

pub fn resolve_parameters(
    parameters: Vec<oneil_ast::Parameter>,
    builder: ModuleCollectionBuilder,
) -> ParameterCollection {
    todo!()
}

fn resolve_module_path(
    module_path: ModulePath,
    subcomponents: &[String],
    builder: &ModuleCollectionBuilder,
) -> Result<ModulePath, ()> {
    assert!(
        builder.module_has_been_visited(&module_path),
        "module path {:?} has not been visited",
        module_path
    );

    if subcomponents.is_empty() {
        return Ok(module_path);
    }

    let module = builder.get_module(&module_path).ok_or(todo!(
        "I think the module had errors? Not sure how to handle this yet {:?}",
        module_path
    ))?;

    let submodel_name = Identifier::new(subcomponents[0]);
    let submodel_path = module
        .get_submodel(&submodel_name)
        .ok_or(todo!("resolution error"))?
        .clone();

    resolve_module_path(submodel_path, &subcomponents[1..], builder)
}

pub(crate) fn resolve_model_tests(
    tests: Vec<oneil_ast::Test>,
    builder: ModuleCollectionBuilder,
) -> Vec<ModelTest> {
    todo!()
}

pub(crate) fn resolve_submodel_tests(
    submodel_tests: Vec<(ModulePath, Vec<oneil_ast::declaration::ModelInput>)>,
    builder: ModuleCollectionBuilder,
) -> Vec<SubmodelTest> {
    todo!()
}
