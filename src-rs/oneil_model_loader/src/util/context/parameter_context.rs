use std::collections::HashMap;

use oneil_ir::{parameter::Parameter, reference::Identifier};

use crate::{
    error::ParameterResolutionError,
    util::context::lookup::{self, LookupResult},
};

pub struct ParameterContext<'parameter> {
    parameters: &'parameter HashMap<Identifier, Parameter>,
    parameter_errors: &'parameter HashMap<Identifier, Vec<ParameterResolutionError>>,
}

impl<'parameter> ParameterContext<'parameter> {
    pub fn new(
        parameters: &'parameter HashMap<Identifier, Parameter>,
        parameter_errors: &'parameter HashMap<Identifier, Vec<ParameterResolutionError>>,
    ) -> Self {
        Self {
            parameters,
            parameter_errors,
        }
    }

    pub fn lookup_parameter(
        &self,
        parameter_identifier: &Identifier,
    ) -> ParameterContextResult<'parameter> {
        let lookup_result = lookup::lookup_with(
            parameter_identifier,
            |parameter_identifier| self.parameters.get(parameter_identifier),
            |parameter_identifier| self.parameter_errors.contains_key(parameter_identifier),
        );

        ParameterContextResult::from(lookup_result)
    }
}

pub enum ParameterContextResult<'parameter> {
    Found(&'parameter Parameter),
    HasError,
    NotFound,
}

impl<'parameter> From<LookupResult<&'parameter Parameter>> for ParameterContextResult<'parameter> {
    fn from(result: LookupResult<&'parameter Parameter>) -> Self {
        match result {
            LookupResult::Found(parameter) => ParameterContextResult::Found(parameter),
            LookupResult::HasError => ParameterContextResult::HasError,
            LookupResult::NotFound => ParameterContextResult::NotFound,
        }
    }
}
