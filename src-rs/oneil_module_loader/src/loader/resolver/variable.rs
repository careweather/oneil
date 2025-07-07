use std::collections::HashSet;

use oneil_ast as ast;
use oneil_module::reference::{Identifier, ModulePath};

use crate::{
    error::VariableResolutionError,
    loader::resolver::{ModuleInfo, ParameterInfo, SubmodelInfo},
    util::info::InfoResult,
};

pub fn resolve_variable(
    variable: &ast::expression::Variable,
    local_variables: &HashSet<Identifier>,
    defined_parameters: &ParameterInfo,
    submodel_info: &SubmodelInfo,
    module_info: &ModuleInfo,
) -> Result<oneil_module::expr::Expr, VariableResolutionError> {
    match variable {
        ast::expression::Variable::Identifier(identifier) => {
            let identifier = Identifier::new(identifier);
            if local_variables.contains(&identifier) {
                Ok(oneil_module::expr::Expr::local_variable(identifier))
            } else {
                match defined_parameters.get(&identifier) {
                    InfoResult::Found(_parameter) => {
                        Ok(oneil_module::expr::Expr::parameter_variable(identifier))
                    }
                    InfoResult::HasError => {
                        return Err(VariableResolutionError::parameter_has_error(identifier));
                    }
                    InfoResult::NotFound => {
                        return Err(VariableResolutionError::undefined_parameter(identifier));
                    }
                }
            }
        }
        ast::expression::Variable::Accessor { parent, component } => {
            let parent_identifier = Identifier::new(parent);
            let submodel_path = match submodel_info.get(&parent_identifier) {
                InfoResult::Found(submodel_path) => submodel_path,
                InfoResult::HasError => {
                    return Err(VariableResolutionError::submodel_has_error(
                        parent_identifier,
                    ));
                }
                InfoResult::NotFound => {
                    return Err(VariableResolutionError::undefined_submodel(
                        parent_identifier,
                    ));
                }
            };

            resolve_variable_recursive(
                submodel_path,
                component,
                defined_parameters,
                submodel_info,
                module_info,
            )
        }
    }
}

fn resolve_variable_recursive(
    submodel_path: &ModulePath,
    variable: &ast::expression::Variable,
    defined_parameters: &ParameterInfo,
    submodel_info: &SubmodelInfo,
    module_info: &ModuleInfo,
) -> Result<oneil_module::expr::Expr, VariableResolutionError> {
    let module = match module_info.get(submodel_path) {
        InfoResult::Found(module) => module,
        InfoResult::HasError => {
            return Err(VariableResolutionError::module_has_error(
                submodel_path.clone(),
            ));
        }
        InfoResult::NotFound => panic!("submodel should have been visited already"),
    };

    match variable {
        // if the variable is an identifier, this means that the variable refers to a parameter
        ast::expression::Variable::Identifier(identifier) => {
            let identifier = Identifier::new(identifier);
            if module.get_parameter(&identifier).is_some() {
                Ok(oneil_module::expr::Expr::parameter_variable(identifier))
            } else {
                return Err(VariableResolutionError::undefined_parameter_in_submodel(
                    submodel_path.clone(),
                    identifier,
                ));
            }
        }

        // if the variable is an accessor, this means that the variable refers to a submodel
        ast::expression::Variable::Accessor { parent, component } => {
            let parent_identifier = Identifier::new(parent);
            let submodel_path = match module.get_submodel(&parent_identifier) {
                Some(submodel_path) => submodel_path,
                None => {
                    let source = VariableResolutionError::undefined_submodel_in_submodel(
                        submodel_path.clone(),
                        parent_identifier,
                    );
                    return Err(source);
                }
            };

            resolve_variable_recursive(
                submodel_path,
                component,
                defined_parameters,
                submodel_info,
                module_info,
            )
        }
    }
}
