use oneil_ir::{
    model::Model,
    model_import::{ReferenceImport, ReferenceName, SubmodelImport, SubmodelName},
    parameter::Parameter,
    reference::{Identifier, ModelPath},
    span::Span,
};

use crate::{error::ParameterResolutionError, util::context::LookupResult};

pub trait ModelContext {
    fn lookup_model(&self, model_path: &ModelPath) -> LookupResult<&Model>;
}

pub trait ModelImportsContext {
    fn lookup_submodel(&self, submodel_name: &SubmodelName) -> LookupResult<&SubmodelImport>;
    fn lookup_reference(&self, reference_name: &ReferenceName) -> LookupResult<&ReferenceImport>;
}

pub trait ParameterContext {
    fn lookup_parameter(&self, parameter_name: &Identifier) -> LookupResult<&Parameter>;
    fn add_parameter(&mut self, parameter_name: Identifier, parameter: Parameter);
    fn add_parameter_error(&mut self, parameter_name: Identifier, error: ParameterResolutionError);
}
