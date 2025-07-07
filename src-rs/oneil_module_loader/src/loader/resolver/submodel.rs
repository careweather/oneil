use std::collections::HashMap;

use oneil_ast as ast;
use oneil_module::reference::{Identifier, ModulePath};

use crate::{error::SubmodelResolutionError, loader::resolver::ModuleInfo, util::info::InfoResult};

pub fn resolve_submodels_and_tests(
    use_models: Vec<ast::declaration::UseModel>,
    module_path: &ModulePath,
    module_info: &ModuleInfo,
) -> (
    HashMap<Identifier, ModulePath>,
    Vec<(Identifier, Vec<ast::declaration::ModelInput>)>,
    HashMap<Identifier, SubmodelResolutionError>,
) {
    use_models.into_iter().fold(
        (HashMap::new(), Vec::new(), HashMap::new()),
        |(mut submodels, mut submodel_tests, mut resolution_errors), use_model| {
            // get the use model path
            let use_model_path = module_path.get_sibling_path(&use_model.model_name);
            let use_model_path = ModulePath::new(use_model_path);

            // get the submodel name
            let submodel_name = use_model.as_name.as_ref().unwrap_or(
                use_model
                    .subcomponents
                    .last()
                    .unwrap_or(&use_model.model_name),
            );
            let submodel_name = Identifier::new(submodel_name);

            // resolve the use model path
            let resolved_use_model_path = resolve_module_path(
                None, // indicates that the "parent module" is the module that is being resolved
                use_model_path.clone(),
                &use_model.subcomponents,
                module_info,
            );

            // insert the use model path into the submodels map if it was resolved successfully
            // otherwise, add the error to the builder
            match resolved_use_model_path {
                Ok(resolved_use_model_path) => {
                    submodels.insert(submodel_name.clone(), resolved_use_model_path.clone());

                    // store the inputs for the submodel tests
                    // (the inputs are stored in their AST form for now and converted to
                    // the model input type once all the submodels have been resolved)
                    let inputs = use_model.inputs.unwrap_or_default();
                    submodel_tests.push((submodel_name, inputs));
                }
                Err(error) => {
                    resolution_errors.insert(submodel_name, error);
                }
            }

            (submodels, submodel_tests, resolution_errors)
        },
    )
}

fn resolve_module_path(
    parent_module_path: Option<ModulePath>,
    module_path: ModulePath,
    subcomponents: &[String],
    module_info: &ModuleInfo,
) -> Result<ModulePath, SubmodelResolutionError> {
    // if the module that we are trying to resolve has had an error, this
    // operation should fail
    let module = match module_info.get(&module_path) {
        InfoResult::Found(module) => module,
        InfoResult::HasError => return Err(SubmodelResolutionError::module_has_error(module_path)),
        InfoResult::NotFound => panic!("module should have been visited already"),
    };

    // if there are no more subcomponents, we have resolved the module path
    if subcomponents.is_empty() {
        return Ok(module_path);
    }

    let submodel_name = Identifier::new(subcomponents[0].clone());
    let submodel_path = module
        .get_submodel(&submodel_name)
        .ok_or(match parent_module_path {
            Some(parent_module_path) => SubmodelResolutionError::undefined_submodel_in_submodel(
                parent_module_path,
                submodel_name,
            ),
            None => SubmodelResolutionError::undefined_submodel(submodel_name),
        })?
        .clone();

    resolve_module_path(
        Some(module_path),
        submodel_path,
        &subcomponents[1..],
        module_info,
    )
}
