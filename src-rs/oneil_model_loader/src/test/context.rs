use std::collections::{HashMap, HashSet};

use oneil_ir::{
    model::Model,
    model_import::{ReferenceImport, ReferenceName},
    parameter::Parameter,
    reference::{Identifier, ModelPath},
};

use crate::{
    error::ParameterResolutionError,
    util::context::{LookupResult, ModelContext, ModelImportsContext, ParameterContext},
};

#[derive(Debug, Clone)]
pub struct TestContext {
    model_context: HashMap<ModelPath, Model>,
    model_errors: HashSet<ModelPath>,
    reference_context: HashMap<ReferenceName, ReferenceImport>,
    reference_errors: HashSet<ReferenceName>,
    parameter_context: HashMap<Identifier, Parameter>,
    parameter_errors: HashMap<Identifier, Vec<ParameterResolutionError>>,
}

impl TestContext {
    pub fn new() -> Self {
        Self {
            model_context: HashMap::new(),
            model_errors: HashSet::new(),
            reference_context: HashMap::new(),
            reference_errors: HashSet::new(),
            parameter_context: HashMap::new(),
            parameter_errors: HashMap::new(),
        }
    }

    pub fn with_model_context(
        mut self,
        model_context: impl IntoIterator<Item = (ModelPath, Model)>,
    ) -> Self {
        self.model_context.extend(model_context);
        self
    }

    pub fn with_model_errors(mut self, model_errors: impl IntoIterator<Item = ModelPath>) -> Self {
        self.model_errors.extend(model_errors);
        self
    }

    pub fn with_reference_context(
        mut self,
        reference_context: impl IntoIterator<Item = (ReferenceName, ReferenceImport)>,
    ) -> Self {
        self.reference_context.extend(reference_context);
        self
    }

    pub fn with_reference_errors(
        mut self,
        reference_errors: impl IntoIterator<Item = ReferenceName>,
    ) -> Self {
        self.reference_errors.extend(reference_errors);
        self
    }

    pub fn with_parameter_context(
        mut self,
        parameter_context: impl IntoIterator<Item = (Identifier, Parameter)>,
    ) -> Self {
        self.parameter_context.extend(parameter_context);
        self
    }

    pub fn with_parameter_errors(
        mut self,
        parameter_errors: impl IntoIterator<Item = Identifier>,
    ) -> Self {
        self.parameter_errors.extend(
            parameter_errors
                .into_iter()
                // initialize the errors to an empty vector since just the
                // presence of the key is enough to indicate an error
                .map(|identifier| (identifier, Vec::new())),
        );
        self
    }

    pub fn parameters(&self) -> &HashMap<Identifier, Parameter> {
        &self.parameter_context
    }

    pub fn parameter_errors(&self) -> &HashMap<Identifier, Vec<ParameterResolutionError>> {
        &self.parameter_errors
    }
}

impl ModelContext for TestContext {
    #[allow(clippy::redundant_closure, reason = "explicit map is clearer")]
    fn lookup_model(&self, model_path: &ModelPath) -> LookupResult<&Model> {
        let has_error = self.model_errors.contains(model_path);

        if has_error {
            return LookupResult::HasError;
        }

        self.model_context
            .get(model_path)
            .map_or(LookupResult::NotFound, |model| LookupResult::Found(model))
    }
}

impl ModelImportsContext for TestContext {
    #[allow(clippy::redundant_closure, reason = "explicit map function is clearer")]
    fn lookup_reference(&self, reference_name: &ReferenceName) -> LookupResult<&ReferenceImport> {
        let has_error = self.reference_errors.contains(reference_name);

        if has_error {
            return LookupResult::HasError;
        }

        self.reference_context
            .get(reference_name)
            .map_or(LookupResult::NotFound, |reference| {
                LookupResult::Found(reference)
            })
    }
}

impl ParameterContext for TestContext {
    #[allow(clippy::redundant_closure, reason = "explicit map function is clearer")]
    fn lookup_parameter(&self, parameter_path: &Identifier) -> LookupResult<&Parameter> {
        let has_error = self.parameter_errors.contains_key(parameter_path);

        if has_error {
            return LookupResult::HasError;
        }

        self.parameter_context
            .get(parameter_path)
            .map_or(LookupResult::NotFound, |parameter| {
                LookupResult::Found(parameter)
            })
    }

    fn add_parameter(&mut self, parameter_name: Identifier, parameter: Parameter) {
        self.parameter_context.insert(parameter_name, parameter);
    }

    fn add_parameter_error(&mut self, parameter_name: Identifier, error: ParameterResolutionError) {
        self.parameter_errors
            .entry(parameter_name)
            .or_default()
            .push(error);
    }
}
