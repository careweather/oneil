use std::collections::HashMap;

use oneil_ir::model_import::{ReferenceImport, ReferenceName, SubmodelImport, SubmodelName};

use crate::{
    error::ModelImportResolutionError,
    util::context::{
        LookupResult, ModelContext, ModelImportsContext, ModelsLoadedContext,
        impls::ParametersResolvingContext, lookup,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub struct ModelImportsResolvedContext<'builder, 'model_imports> {
    models_loaded_context: ModelsLoadedContext<'builder>,
    submodels: &'model_imports HashMap<SubmodelName, SubmodelImport>,
    submodel_resolution_errors: &'model_imports HashMap<SubmodelName, ModelImportResolutionError>,
    references: &'model_imports HashMap<ReferenceName, ReferenceImport>,
    reference_resolution_errors: &'model_imports HashMap<ReferenceName, ModelImportResolutionError>,
}

impl<'builder, 'model_imports> ModelImportsResolvedContext<'builder, 'model_imports> {
    pub fn new(
        models_loaded_context: ModelsLoadedContext<'builder>,
        submodels: &'model_imports HashMap<SubmodelName, SubmodelImport>,
        submodel_resolution_errors: &'model_imports HashMap<
            SubmodelName,
            ModelImportResolutionError,
        >,
        references: &'model_imports HashMap<ReferenceName, ReferenceImport>,
        reference_resolution_errors: &'model_imports HashMap<
            ReferenceName,
            ModelImportResolutionError,
        >,
    ) -> Self {
        Self {
            models_loaded_context,
            submodels,
            submodel_resolution_errors,
            references,
            reference_resolution_errors,
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
    fn lookup_submodel(&self, submodel_name: &SubmodelName) -> LookupResult<&SubmodelImport> {
        lookup::lookup_with(
            submodel_name,
            |submodel_name| self.submodels.get(submodel_name),
            |submodel_name| self.submodel_resolution_errors.contains_key(submodel_name),
        )
    }

    fn lookup_reference(&self, reference_name: &ReferenceName) -> LookupResult<&ReferenceImport> {
        lookup::lookup_with(
            reference_name,
            |reference_name| self.references.get(reference_name),
            |reference_name| {
                self.reference_resolution_errors
                    .contains_key(reference_name)
            },
        )
    }
}
