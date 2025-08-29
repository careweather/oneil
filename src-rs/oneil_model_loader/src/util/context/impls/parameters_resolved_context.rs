use std::collections::{HashMap, HashSet};

use oneil_ir::{parameter::Parameter, reference::Identifier};

use crate::util::context::{
    LookupResult, ModelContext, ModelImportsContext, ModelImportsResolvedContext, ParameterContext,
    lookup,
};

pub struct ParametersResolvingContext<'builder, 'model_imports> {
    model_imports_resolved_context: ModelImportsResolvedContext<'builder, 'model_imports>,
    parameters: HashMap<Identifier, Parameter>,
    parameters_with_errors: HashSet<Identifier>,
}

impl<'builder, 'model_imports> ParametersResolvingContext<'builder, 'model_imports> {
    pub fn new(
        model_imports_resolved_context: ModelImportsResolvedContext<'builder, 'model_imports>,
    ) -> Self {
        Self {
            model_imports_resolved_context,
            parameters: HashMap::new(),
            parameters_with_errors: HashSet::new(),
        }
    }
}

impl ModelContext for ParametersResolvingContext<'_, '_> {
    fn lookup_model(
        &self,
        model_path: &oneil_ir::reference::ModelPath,
    ) -> LookupResult<&oneil_ir::model::Model> {
        self.model_imports_resolved_context.lookup_model(model_path)
    }
}

impl ModelImportsContext for ParametersResolvingContext<'_, '_> {
    fn lookup_submodel(
        &self,
        submodel_name: &oneil_ir::reference::Identifier,
    ) -> LookupResult<&(oneil_ir::reference::ModelPath, oneil_ir::span::Span)> {
        self.model_imports_resolved_context
            .lookup_submodel(submodel_name)
    }
}

impl ParameterContext for ParametersResolvingContext<'_, '_> {
    fn lookup_parameter(
        &self,
        parameter_name: &oneil_ir::reference::Identifier,
    ) -> LookupResult<&oneil_ir::parameter::Parameter> {
        lookup::lookup_with(
            parameter_name,
            |parameter_name| self.parameters.get(parameter_name),
            |parameter_name| self.parameters_with_errors.contains(parameter_name),
        )
    }

    fn add_parameter(&mut self, parameter_name: Identifier, parameter: Parameter) {
        self.parameters.insert(parameter_name, parameter);
    }

    fn add_parameter_error(&mut self, parameter_name: Identifier) {
        self.parameters_with_errors.insert(parameter_name);
    }
}
