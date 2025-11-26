use oneil_ir as ir;

use crate::{error::EvalError, value::Value};

pub struct EvalContext {}

impl EvalContext {
    pub const fn new() -> Self {
        Self {}
    }

    pub fn lookup_builtin_variable(
        &self,
        identifier: &ir::Identifier,
    ) -> Result<Value, Vec<EvalError>> {
        todo!()
    }

    pub fn lookup_parameter(
        &self,
        parameter_name: &ir::ParameterName,
    ) -> Result<Value, Vec<EvalError>> {
        todo!()
    }

    pub fn lookup_model_parameter(
        &self,
        model: &ir::ModelPath,
        parameter_name: &ir::ParameterName,
    ) -> Result<Value, Vec<EvalError>> {
        todo!()
    }

    // TODO: figure out what error this should actually be
    pub fn evaluate_builtin_function(
        &self,
        identifier: &ir::Identifier,
        args: Vec<Value>,
    ) -> Result<Value, Vec<EvalError>> {
        todo!()
    }

    pub fn evaluate_imported_function(
        &self,
        identifier: &ir::Identifier,
        args: Vec<Value>,
    ) -> Result<Value, Vec<EvalError>> {
        todo!()
    }

    pub fn values_are_close(&self, a: &Value, b: &Value, epsilon: f64) -> bool {
        todo!()
    }
}
