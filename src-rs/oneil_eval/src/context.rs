use std::collections::HashMap;

use oneil_ir as ir;

use crate::{
    error::EvalError,
    value::{SizedUnit, Value},
};

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

    pub fn values_are_close(&self, a: &Value, b: &Value) -> bool {
        todo!()
    }

    pub fn lookup_unit(&self, name: &str) -> Option<SizedUnit> {
        todo!()
    }

    pub fn available_prefixes(&self) -> HashMap<String, f64> {
        todo!()
    }

    pub fn load_python_import(&mut self, python_path: &ir::PythonPath) {
        todo!()
    }

    pub fn activate_model(&mut self, model_path: &ir::ModelPath) {
        todo!()
    }

    pub fn activate_python_imports(
        &mut self,
        python_imports: &HashMap<ir::PythonPath, ir::PythonImport>,
    ) {
        todo!()
    }

    pub fn add_parameter_result(
        &mut self,
        parameter_name: ir::ParameterName,
        value: Result<Value, Vec<EvalError>>,
    ) {
        todo!()
    }

    pub fn add_submodel(&mut self, submodel_name: &str, submodel_import: &ir::ModelPath) {
        todo!()
    }

    pub fn activate_references(
        &mut self,
        references: &HashMap<ir::ReferenceName, ir::ReferenceImport>,
    ) {
        todo!()
    }
}

impl Default for EvalContext {
    fn default() -> Self {
        Self {}
    }
}
