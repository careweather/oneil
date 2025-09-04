use std::collections::{HashMap, HashSet};

use oneil_ir::{
    model::Model,
    model_import::{ReferenceImport, ReferenceName, SubmodelImport, SubmodelName},
    reference::ModelPath,
};

use crate::{
    error::ModelImportResolutionError,
    util::{
        builder::ModelCollectionBuilder,
        context::{LookupResult, ModelContext, ModelImportsResolvedContext, lookup},
    },
};

#[derive(Debug, Clone, PartialEq)]
pub struct ModelsLoadedContext<'builder> {
    models: &'builder HashMap<ModelPath, Model>,
    submodels: HashSet<&'builder ModelPath>,
}

impl<'builder> ModelsLoadedContext<'builder> {
    pub fn from_builder<Ps, Py>(builder: &'builder ModelCollectionBuilder<Ps, Py>) -> Self {
        Self {
            models: builder.get_models(),
            submodels: builder.get_models_with_errors(),
        }
    }

    pub fn with_model_imports_resolved<'model_imports>(
        self,
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
    ) -> ModelImportsResolvedContext<'builder, 'model_imports> {
        ModelImportsResolvedContext::new(
            self,
            submodels,
            submodel_resolution_errors,
            references,
            reference_resolution_errors,
        )
    }
}

impl ModelContext for ModelsLoadedContext<'_> {
    fn lookup_model(
        &self,
        model_path: &oneil_ir::reference::ModelPath,
    ) -> LookupResult<&oneil_ir::model::Model> {
        lookup::lookup_with(
            model_path,
            |model_path| self.models.get(model_path),
            |model_path| self.submodels.contains(model_path),
        )
    }
}
