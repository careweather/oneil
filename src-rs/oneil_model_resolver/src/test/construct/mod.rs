//! Helper functions for creating test data
//!
//! Creating test data can be a tedious and repetitive process, especially where `Span`s are
//! involved. This module provides helper functions to create test data that can be used in tests.

use std::collections::{HashMap, HashSet};

use oneil_ir as ir;
use oneil_shared::span::{SourceLocation, Span};

use crate::{
    error::{ModelImportResolutionError, ParameterResolutionError},
    util::{
        builder::ModelCollectionBuilder,
        context::{ModelContext, ParameterContext, ReferenceContext},
    },
};

pub mod test_ast;
pub mod test_ir;

pub fn empty_model_collection_builder() -> ModelCollectionBuilder<(), ()> {
    ModelCollectionBuilder::new(HashSet::new())
}

pub struct ModelContextBuilder {
    models: HashMap<ir::ModelPath, ir::Model>,
    model_errors: HashSet<ir::ModelPath>,
}

impl ModelContextBuilder {
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
            model_errors: HashSet::new(),
        }
    }

    pub fn with_model_context(
        mut self,
        model_context: impl IntoIterator<Item = (ir::ModelPath, ir::Model)>,
    ) -> Self {
        self.models.extend(model_context);
        self
    }

    pub fn with_model_error_context(
        mut self,
        model_error: impl IntoIterator<Item = ir::ModelPath>,
    ) -> Self {
        self.model_errors.extend(model_error);
        self
    }

    pub fn build(&self) -> ModelContext<'_> {
        ModelContext::new(&self.models, self.model_errors.iter().collect())
    }
}

pub struct ReferenceContextBuilder {
    models: HashMap<ir::ModelPath, ir::Model>,
    model_errors: HashSet<ir::ModelPath>,
    references: HashMap<ir::ReferenceName, ir::ReferenceImport>,
    reference_errors: HashMap<ir::ReferenceName, ModelImportResolutionError>,
}

impl ReferenceContextBuilder {
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
            model_errors: HashSet::new(),
            references: HashMap::new(),
            reference_errors: HashMap::new(),
        }
    }

    pub fn with_reference_context(
        mut self,
        reference_context: impl IntoIterator<Item = (ir::ReferenceName, ir::ModelPath, ir::Model)>,
    ) -> Self {
        let (references, models): (HashMap<_, _>, HashMap<_, _>) = reference_context
            .into_iter()
            .map(|(reference_name_with_span, model_path, model)| {
                let reference_name = reference_name_with_span.clone();
                let reference_import =
                    ir::ReferenceImport::new(reference_name_with_span, model_path.clone());

                let reference_entry = (reference_name, reference_import);
                let model_entry = (model_path, model);

                (reference_entry, model_entry)
            })
            .unzip();

        self.references.extend(references);
        self.models.extend(models);
        self
    }

    pub fn with_reference_errors(
        mut self,
        reference_errors: impl IntoIterator<Item = ir::ReferenceName>,
    ) -> Self {
        let arbitrary_location = SourceLocation {
            offset: 0,
            line: 0,
            column: 0,
        };
        let arbitrary_span = Span::new(arbitrary_location, arbitrary_location);
        let arbitrary_reference = ir::ReferenceName::new("arbitrary_reference".to_string());
        let arbitrary_error = ModelImportResolutionError::duplicate_reference(
            arbitrary_reference,
            arbitrary_span,
            arbitrary_span,
        );

        let reference_errors = reference_errors
            .into_iter()
            .map(|reference_name| (reference_name, arbitrary_error.clone()));

        self.reference_errors.extend(reference_errors);
        self
    }

    pub fn with_model_error(
        mut self,
        model_errors: impl IntoIterator<Item = ir::ModelPath>,
    ) -> Self {
        self.model_errors.extend(model_errors);
        self
    }

    pub fn build(&self) -> ReferenceContext<'_, '_> {
        ReferenceContext::new(
            &self.models,
            self.model_errors.iter().collect(),
            &self.references,
            &self.reference_errors,
        )
    }
}

pub struct ParameterContextBuilder {
    parameters: HashMap<ir::Identifier, ir::Parameter>,
    parameter_errors: HashMap<ir::Identifier, Vec<ParameterResolutionError>>,
}

impl ParameterContextBuilder {
    pub fn new() -> Self {
        Self {
            parameters: HashMap::new(),
            parameter_errors: HashMap::new(),
        }
    }

    pub fn with_parameter_context(
        mut self,
        parameter_context: impl IntoIterator<Item = ir::Parameter>,
    ) -> Self {
        let parameter_context = parameter_context
            .into_iter()
            .map(|parameter| (parameter.identifier().clone(), parameter));

        self.parameters.extend(parameter_context);
        self
    }

    pub fn with_parameter_error(
        mut self,
        parameter_errors: impl IntoIterator<Item = ir::Identifier>,
    ) -> Self {
        let parameter_errors = parameter_errors
            .into_iter()
            // the presence of the identifier in the map indicates an error
            .map(|identifier| (identifier, vec![]));

        self.parameter_errors.extend(parameter_errors);

        self
    }

    pub fn build(&self) -> ParameterContext<'_> {
        ParameterContext::new(&self.parameters, &self.parameter_errors)
    }
}
