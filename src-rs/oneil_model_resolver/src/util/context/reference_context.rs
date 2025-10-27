use std::collections::{HashMap, HashSet};

use oneil_ir as ir;

use crate::{
    error::ModelImportResolutionError,
    util::context::lookup::{self, LookupResult},
};

#[derive(Debug)]
pub struct ReferenceContext<'model, 'reference> {
    models: &'model HashMap<ir::ModelPath, ir::Model>,
    model_errors: HashSet<&'model ir::ModelPath>,
    references: &'reference HashMap<ir::ReferenceName, ir::ReferenceImport>,
    reference_errors: &'reference HashMap<ir::ReferenceName, ModelImportResolutionError>,
}

impl<'model, 'reference> ReferenceContext<'model, 'reference> {
    #[must_use]
    pub const fn new(
        models: &'model HashMap<ir::ModelPath, ir::Model>,
        model_errors: HashSet<&'model ir::ModelPath>,
        references: &'reference HashMap<ir::ReferenceName, ir::ReferenceImport>,
        reference_errors: &'reference HashMap<ir::ReferenceName, ModelImportResolutionError>,
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
        reference_name: &ir::ReferenceName,
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

#[derive(Debug)]
pub enum ReferenceContextResult<'model, 'reference> {
    Found(&'model ir::Model, &'reference ir::ModelPath),
    ReferenceHasResolutionError,
    ReferenceNotFound,
    ModelHasResolutionError(&'reference ir::ModelPath),
    ModelNotFound(&'reference ir::ModelPath),
}
