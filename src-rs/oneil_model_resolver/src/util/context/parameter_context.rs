use indexmap::IndexMap;

use oneil_ir as ir;

use crate::{
    error::ParameterResolutionError,
    util::context::lookup::{self, LookupResult},
};

#[derive(Debug)]
pub struct ParameterContext<'parameter> {
    parameters: &'parameter IndexMap<ir::ParameterName, ir::Parameter>,
    parameter_errors: &'parameter IndexMap<ir::ParameterName, Vec<ParameterResolutionError>>,
}

impl<'parameter> ParameterContext<'parameter> {
    pub const fn new(
        parameters: &'parameter IndexMap<ir::ParameterName, ir::Parameter>,
        parameter_errors: &'parameter IndexMap<ir::ParameterName, Vec<ParameterResolutionError>>,
    ) -> Self {
        Self {
            parameters,
            parameter_errors,
        }
    }

    #[must_use]
    pub fn lookup_parameter(
        &self,
        parameter_name: &ir::ParameterName,
    ) -> ParameterContextResult<'parameter> {
        let lookup_result = lookup::lookup_with(
            parameter_name,
            |parameter_name| self.parameters.get(parameter_name),
            |parameter_name| self.parameter_errors.contains_key(parameter_name),
        );

        ParameterContextResult::from(lookup_result)
    }
}

#[derive(Debug)]
pub enum ParameterContextResult<'parameter> {
    Found(&'parameter ir::Parameter),
    HasError,
    NotFound,
}

impl<'parameter> From<LookupResult<&'parameter ir::Parameter>>
    for ParameterContextResult<'parameter>
{
    fn from(result: LookupResult<&'parameter ir::Parameter>) -> Self {
        match result {
            LookupResult::Found(parameter) => ParameterContextResult::Found(parameter),
            LookupResult::HasError => ParameterContextResult::HasError,
            LookupResult::NotFound => ParameterContextResult::NotFound,
        }
    }
}
