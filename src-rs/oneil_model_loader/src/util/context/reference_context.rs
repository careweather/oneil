use std::collections::{HashMap, HashSet};

use oneil_ir::{
    model::Model,
    model_import::{ReferenceImport, ReferenceName},
    reference::ModelPath,
};

use crate::{
    error::ModelImportResolutionError,
    util::context::lookup::{self, LookupResult},
};

pub struct ReferenceContext<'model, 'reference> {
    models: &'model HashMap<ModelPath, Model>,
    model_errors: &'model HashSet<&'model ModelPath>,
    references: &'reference HashMap<ReferenceName, ReferenceImport>,
    reference_errors: &'reference HashMap<ReferenceName, ModelImportResolutionError>,
}

impl<'model, 'reference> ReferenceContext<'model, 'reference> {
    #[must_use]
    pub const fn new(
        models: &'model HashMap<ModelPath, Model>,
        model_errors: &'model HashSet<&'model ModelPath>,
        references: &'reference HashMap<ReferenceName, ReferenceImport>,
        reference_errors: &'reference HashMap<ReferenceName, ModelImportResolutionError>,
    ) -> Self {
        Self {
            models,
            model_errors,
            references,
            reference_errors,
        }
    }

    #[must_use]
    pub fn lookup_reference(
        &self,
        reference_name: &ReferenceName,
    ) -> ReferenceContextResult<'model, 'reference> {
        let lookup_reference_result = lookup::lookup_with(
            reference_name,
            |reference_name| self.references.get(reference_name),
            |reference_name| self.reference_errors.contains_key(reference_name),
        );

        let reference_path = match lookup_reference_result {
            LookupResult::Found(reference) => reference.path(),
            LookupResult::HasError => {
                return ReferenceContextResult::ReferenceHasResolutionError;
            }
            LookupResult::NotFound => return ReferenceContextResult::ReferenceNotFound,
        };

        let lookup_model_result = lookup::lookup_with(
            reference_path,
            |reference_path| self.models.get(reference_path),
            |reference_path| self.model_errors.contains(reference_path),
        );

        match lookup_model_result {
            LookupResult::Found(model) => ReferenceContextResult::Found(model, reference_path),
            LookupResult::HasError => {
                ReferenceContextResult::ModelHasResolutionError(reference_path)
            }
            LookupResult::NotFound => ReferenceContextResult::ModelNotFound(reference_path),
        }
    }
}

pub enum ReferenceContextResult<'model, 'reference> {
    Found(&'model Model, &'reference ModelPath),
    ReferenceHasResolutionError,
    ReferenceNotFound,
    ModelHasResolutionError(&'reference ModelPath),
    ModelNotFound(&'reference ModelPath),
}
