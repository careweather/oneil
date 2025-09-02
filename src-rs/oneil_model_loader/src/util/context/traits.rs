use oneil_ir::{
    model::Model,
    parameter::Parameter,
    reference::{Identifier, ModelPath},
    span::Span,
};

use crate::util::context::LookupResult;

pub trait ModelContext {
    fn lookup_model(&self, model_path: &ModelPath) -> LookupResult<&Model>;
}

pub trait ModelImportsContext {
    fn lookup_submodel(&self, submodel_name: &Identifier) -> LookupResult<&(ModelPath, Span)>;
}

pub trait ParameterContext: std::fmt::Debug {
    fn lookup_parameter(&self, parameter_name: &Identifier) -> LookupResult<&Parameter>;
    fn add_parameter(&mut self, parameter_name: Identifier, parameter: Parameter);
    fn add_parameter_error(&mut self, parameter_name: Identifier);
}
