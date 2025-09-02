use std::collections::{HashMap, HashSet};

use oneil_ir::{
    model::Model,
    parameter::Parameter,
    reference::{Identifier, ModelPath},
    span::Span,
};

use crate::util::context::{LookupResult, ModelContext, ModelImportsContext, ParameterContext};

#[derive(Debug, Clone)]
pub struct TestContext {
    model_context: HashMap<ModelPath, Model>,
    model_errors: HashSet<ModelPath>,
    submodel_context: HashMap<Identifier, (ModelPath, Span)>,
    submodel_errors: HashSet<Identifier>,
    parameter_context: HashMap<Identifier, Parameter>,
    parameter_errors: HashSet<Identifier>,
}

impl TestContext {
    pub fn new() -> Self {
        Self {
            model_context: HashMap::new(),
            model_errors: HashSet::new(),
            submodel_context: HashMap::new(),
            submodel_errors: HashSet::new(),
            parameter_context: HashMap::new(),
            parameter_errors: HashSet::new(),
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

    pub fn with_submodel_context(
        mut self,
        submodel_context: impl IntoIterator<Item = (Identifier, (ModelPath, Span))>,
    ) -> Self {
        self.submodel_context.extend(submodel_context);
        self
    }

    pub fn with_submodel_errors(
        mut self,
        submodel_errors: impl IntoIterator<Item = Identifier>,
    ) -> Self {
        self.submodel_errors.extend(submodel_errors);
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
        self.parameter_errors.extend(parameter_errors);
        self
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
    fn lookup_submodel(&self, submodel_name: &Identifier) -> LookupResult<&(ModelPath, Span)> {
        let has_error = self.submodel_errors.contains(submodel_name);

        if has_error {
            return LookupResult::HasError;
        }

        self.submodel_context
            .get(submodel_name)
            .map_or(LookupResult::NotFound, |model| LookupResult::Found(model))
    }
}

impl ParameterContext for TestContext {
    #[allow(clippy::redundant_closure, reason = "explicit map function is clearer")]
    fn lookup_parameter(&self, parameter_path: &Identifier) -> LookupResult<&Parameter> {
        let has_error = self.parameter_errors.contains(parameter_path);

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

    fn add_parameter_error(&mut self, parameter_name: Identifier) {
        self.parameter_errors.insert(parameter_name);
    }
}
