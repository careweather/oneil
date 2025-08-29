use std::collections::HashMap;

use oneil_ir::{
    reference::{Identifier, ModelPath},
    span::Span,
};

use crate::{
    error::SubmodelResolutionError,
    util::context::{
        LookupResult, ModelContext, ModelImportsContext, ModelsLoadedContext,
        impls::ParametersResolvingContext, lookup,
    },
};

pub struct ModelImportsResolvedContext<'builder, 'model_imports> {
    models_loaded_context: ModelsLoadedContext<'builder>,
    submodels: &'model_imports HashMap<Identifier, (ModelPath, Span)>,
    submodel_resolution_errors: &'model_imports HashMap<Identifier, SubmodelResolutionError>,
}

impl<'builder, 'model_imports> ModelImportsResolvedContext<'builder, 'model_imports> {
    pub fn new(
        models_loaded_context: ModelsLoadedContext<'builder>,
        submodels: &'model_imports HashMap<Identifier, (ModelPath, Span)>,
        submodel_resolution_errors: &'model_imports HashMap<Identifier, SubmodelResolutionError>,
    ) -> Self {
        Self {
            models_loaded_context,
            submodels,
            submodel_resolution_errors,
        }
    }

    pub fn begin_parameter_resolution(
        self,
    ) -> ParametersResolvingContext<'builder, 'model_imports> {
        ParametersResolvingContext::new(self)
    }
}

impl ModelContext for ModelImportsResolvedContext<'_, '_> {
    fn lookup_model(
        &self,
        model_path: &oneil_ir::reference::ModelPath,
    ) -> LookupResult<&oneil_ir::model::Model> {
        self.models_loaded_context.lookup_model(model_path)
    }
}

impl ModelImportsContext for ModelImportsResolvedContext<'_, '_> {
    fn lookup_submodel(
        &self,
        submodel_name: &oneil_ir::reference::Identifier,
    ) -> LookupResult<&(oneil_ir::reference::ModelPath, oneil_ir::span::Span)> {
        lookup::lookup_with(
            submodel_name,
            |submodel_name| self.submodels.get(submodel_name),
            |submodel_name| self.submodel_resolution_errors.contains_key(submodel_name),
        )
    }
}
