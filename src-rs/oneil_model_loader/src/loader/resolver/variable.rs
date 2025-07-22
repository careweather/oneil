//! Variable resolution for the Oneil model loader
//!
//! This module provides functionality for resolving variables in Oneil expressions.
//! Variable resolution involves determining the scope and type of variables based
//! on their context within a model hierarchy.
//!
//! # Overview
//!
//! Variables in Oneil can be:
//! - **Local variables**: Defined within the current scope (e.g., test inputs)
//! - **Parameter variables**: Defined as parameters in the current model
//! - **Submodel variables**: Accessible through submodel paths (e.g., `parameter.submodel`)
//!
//! # Variable Types
//!
//! ## Simple Identifiers
//! Simple variable names like `x`, `temperature`, etc. are resolved by checking:
//! 1. Local variables (test inputs, function parameters)
//! 2. Model parameters
//! 3. Error conditions (undefined parameters, parameters with errors)
//!
//! ## Accessor Variables
//! Dot-notation variables like `parameter.submodel` are resolved by:
//! 1. Resolving the parent submodel path
//! 2. Recursively resolving the component within that submodel
//! 3. Handling nested accessors (e.g., `parameter.submodel2.submodel1`)
//!
//! # Error Handling
//!
//! The model provides comprehensive error handling for various failure scenarios:
//! - Undefined parameters or submodels
//! - Parameters or submodels with resolution errors
//! - Model loading errors
//!

use std::collections::HashSet;

use oneil_ast as ast;
use oneil_ir::reference::{Identifier, ModelPath};

use crate::{
    error::VariableResolutionError,
    loader::resolver::{ModelInfo, ParameterInfo, SubmodelInfo},
    util::info::InfoResult,
};

/// Resolves a variable expression to its corresponding model expression.
///
/// This function handles the resolution of variables in Oneil expressions,
/// determining whether they refer to local variables, parameters, or submodel
/// components. The resolution process follows a hierarchical lookup pattern
/// that respects the model structure and error states.
///
/// # Arguments
///
/// * `variable` - The AST variable to resolve
/// * `local_variables` - Set of local variable identifiers (e.g., test inputs)
/// * `defined_parameters` - Information about available parameters and their error states
/// * `submodel_info` - Information about available submodels and their error states
/// * `modelinfo` - Information about loaded models and their error states
///
/// # Returns
///
/// Returns a `Result` containing either:
/// * `Ok(Expr)` - The resolved expression representing the variable
/// * `Err(VariableResolutionError)` - An error describing why resolution failed
///
/// # Error Cases
///
/// The function can return various error types:
///
/// * `VariableResolutionError::UndefinedParameter` - When a parameter is referenced but not defined
/// * `VariableResolutionError::ParameterHasError` - When a parameter exists but has resolution errors
/// * `VariableResolutionError::UndefinedSubmodel` - When a submodel is referenced but not defined
/// * `VariableResolutionError::SubmodelHasError` - When a submodel exists but has resolution errors
/// * `VariableResolutionError::ModelHasError` - When a model has loading errors
///
/// # Algorithm
///
/// 1. **Simple Identifier Resolution**:
///    - Check if the identifier is in `local_variables`
///    - If not, check if it's a defined parameter
///    - Handle parameter error states appropriately
///
/// 2. **Accessor Resolution**:
///    - Resolve the parent submodel path
///    - Recursively resolve the component within that submodel
///    - Handle nested accessors through recursive calls
pub fn resolve_variable(
    variable: &ast::expression::VariableNode,
    local_variables: &HashSet<Identifier>,
    defined_parameters: &ParameterInfo,
    submodel_info: &SubmodelInfo,
    modelinfo: &ModelInfo,
) -> Result<oneil_ir::expr::Expr, VariableResolutionError> {
    match variable.node_value() {
        ast::expression::Variable::Identifier(identifier) => {
            let identifier = Identifier::new(identifier.as_str());
            if local_variables.contains(&identifier) {
                Ok(oneil_ir::expr::Expr::local_variable(identifier))
            } else {
                match defined_parameters.get(&identifier) {
                    InfoResult::Found(_parameter) => {
                        Ok(oneil_ir::expr::Expr::parameter_variable(identifier))
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
            let parent_identifier = Identifier::new(parent.as_str());
            let submodel_path = match submodel_info.get(&parent_identifier) {
                InfoResult::Found(submodel_path) => submodel_path,
                InfoResult::HasError => {
                    return Err(VariableResolutionError::submodel_resolution_failed(
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
                modelinfo,
            )
        }
    }
}

/// Recursively resolves a variable within a specific submodel context.
///
/// This internal function handles the recursive resolution of variables within
/// submodel contexts. It's called by `resolve_variable` when dealing with
/// accessor variables (e.g., `submodel.parameter`).
///
/// # Safety
///
/// This function assumes that the submodel path has been validated and the model
/// exists in the modelinfo. If this assumption is violated, the function will panic.
/// This is by design as it indicates a bug in the model loading process.
fn resolve_variable_recursive(
    submodel_path: &ModelPath,
    variable: &ast::expression::Variable,
    defined_parameters: &ParameterInfo,
    submodel_info: &SubmodelInfo,
    modelinfo: &ModelInfo,
) -> Result<oneil_ir::expr::Expr, VariableResolutionError> {
    let model = match modelinfo.get(submodel_path) {
        InfoResult::Found(model) => model,
        InfoResult::HasError => {
            return Err(VariableResolutionError::model_has_error(
                submodel_path.clone(),
            ));
        }
        InfoResult::NotFound => panic!("submodel should have been visited already"),
    };

    match variable {
        // if the variable is an identifier, this means that the variable refers to a parameter
        ast::expression::Variable::Identifier(identifier) => {
            let identifier = Identifier::new(identifier.as_str());
            if model.get_parameter(&identifier).is_some() {
                Ok(oneil_ir::expr::Expr::parameter_variable(identifier))
            } else {
                return Err(VariableResolutionError::undefined_parameter_in_submodel(
                    submodel_path.clone(),
                    identifier,
                ));
            }
        }

        // if the variable is an accessor, this means that the variable refers to a submodel
        ast::expression::Variable::Accessor { parent, component } => {
            let parent_identifier = Identifier::new(parent.as_str());
            let submodel_path = match model.get_submodel(&parent_identifier) {
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
                modelinfo,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oneil_ast as ast;
    use oneil_ir::{
        model::Model,
        parameter::Parameter,
        reference::{Identifier, ModelPath},
    };
    use std::collections::{HashMap, HashSet};

    /// Helper function to create a test span
    fn test_span(start: usize, end: usize) -> ast::Span {
        ast::Span::new(start, end, end)
    }

    /// Helper function to create an identifier node
    fn create_identifier_node(name: &str, start: usize) -> ast::naming::IdentifierNode {
        let identifier = ast::naming::Identifier::new(name.to_string());
        ast::node::Node::new(test_span(start, start + name.len()), identifier)
    }

    /// Helper function to create a variable node
    fn create_variable_node(
        variable: ast::expression::Variable,
        start: usize,
        end: usize,
    ) -> ast::expression::VariableNode {
        ast::node::Node::new(test_span(start, end), variable)
    }

    /// Helper function to create a simple identifier variable
    fn create_identifier_variable(name: &str) -> ast::expression::VariableNode {
        let identifier_node = create_identifier_node(name, 0);
        let variable = ast::expression::Variable::Identifier(identifier_node);
        create_variable_node(variable, 0, name.len())
    }

    /// Helper function to create an accessor variable
    fn create_accessor_variable(
        parent: &str,
        component: ast::expression::VariableNode,
    ) -> ast::expression::VariableNode {
        let parent_node = create_identifier_node(parent, 0);
        let component_end = component.node_span().end();
        let variable = ast::expression::Variable::Accessor {
            parent: parent_node,
            component: Box::new(component),
        };
        create_variable_node(variable, 0, parent.len() + 1 + component_end)
    }

    /// Helper function to create a basic model info for testing
    fn create_test_modelinfo() -> ModelInfo<'static> {
        ModelInfo::new(HashMap::new(), HashSet::new())
    }

    /// Helper function to create a basic parameter info for testing
    fn create_test_parameter_info() -> ParameterInfo<'static> {
        ParameterInfo::new(HashMap::new(), HashSet::new())
    }

    /// Helper function to create a basic submodel info for testing
    fn create_test_submodel_info() -> SubmodelInfo<'static> {
        SubmodelInfo::new(HashMap::new(), HashSet::new())
    }

    /// Helper function to create a model with a parameter
    fn create_modelwith_parameter(param_name: &str) -> Model {
        let mut param_map = HashMap::new();
        let param = Parameter::new(
            HashSet::new(),
            Identifier::new(param_name),
            oneil_ir::parameter::ParameterValue::simple(
                oneil_ir::expr::Expr::literal(oneil_ir::expr::Literal::number(42.0)),
                None,
            ),
            oneil_ir::parameter::Limits::default(),
            false,
            oneil_ir::debug_info::TraceLevel::None,
        );
        param_map.insert(Identifier::new(param_name), param);
        let param_collection = oneil_ir::parameter::ParameterCollection::new(param_map);

        Model::new(
            HashSet::new(),
            HashMap::new(),
            param_collection,
            HashMap::new(),
            Vec::new(),
        )
    }

    /// Helper function to create a model with a submodel
    fn create_modelwith_submodel(submodel_name: &str, submodel_path: ModelPath) -> Model {
        let mut submodel_map = HashMap::new();
        submodel_map.insert(Identifier::new(submodel_name), submodel_path);

        Model::new(
            HashSet::new(),
            submodel_map,
            oneil_ir::parameter::ParameterCollection::new(HashMap::new()),
            HashMap::new(),
            Vec::new(),
        )
    }

    #[test]
    fn test_resolve_local_variable() {
        let variable = create_identifier_variable("local_var");
        let mut local_vars = HashSet::new();
        local_vars.insert(Identifier::new("local_var"));

        let result = resolve_variable(
            &variable,
            &local_vars,
            &create_test_parameter_info(),
            &create_test_submodel_info(),
            &create_test_modelinfo(),
        );

        assert!(result.is_ok());
        match result.unwrap() {
            oneil_ir::expr::Expr::Variable(oneil_ir::expr::Variable::Local(ident)) => {
                assert_eq!(ident, Identifier::new("local_var"));
            }
            error => panic!("Expected local variable expression, got {:?}", error),
        }
    }

    #[test]
    fn test_resolve_parameter_variable() {
        let variable = create_identifier_variable("temperature");
        let local_vars = HashSet::new();

        let mut param_map = HashMap::new();
        let temp_param = Parameter::new(
            HashSet::new(),
            Identifier::new("temperature"),
            oneil_ir::parameter::ParameterValue::simple(
                oneil_ir::expr::Expr::literal(oneil_ir::expr::Literal::number(42.0)),
                None,
            ),
            oneil_ir::parameter::Limits::default(),
            false,
            oneil_ir::debug_info::TraceLevel::None,
        );
        let temp_param_id = Identifier::new("temperature");
        param_map.insert(&temp_param_id, &temp_param);
        let param_info = ParameterInfo::new(param_map, HashSet::new());

        let result = resolve_variable(
            &variable,
            &local_vars,
            &param_info,
            &create_test_submodel_info(),
            &create_test_modelinfo(),
        );

        assert!(result.is_ok());
        match result.unwrap() {
            oneil_ir::expr::Expr::Variable(oneil_ir::expr::Variable::Parameter(ident)) => {
                assert_eq!(ident, Identifier::new("temperature"));
            }
            error => panic!("Expected parameter variable expression, got {:?}", error),
        }
    }

    #[test]
    fn test_resolve_undefined_parameter() {
        let variable = create_identifier_variable("undefined_param");
        let local_vars = HashSet::new();

        let result = resolve_variable(
            &variable,
            &local_vars,
            &create_test_parameter_info(),
            &create_test_submodel_info(),
            &create_test_modelinfo(),
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            VariableResolutionError::UndefinedParameter(_, ident) => {
                assert_eq!(ident, Identifier::new("undefined_param"));
            }
            error => panic!("Expected undefined parameter error, got {:?}", error),
        }
    }

    #[test]
    fn test_resolve_parameter_with_error() {
        let variable = create_identifier_variable("error_param");
        let local_vars = HashSet::new();

        let mut param_with_errors = HashSet::new();
        let error_param_id = Identifier::new("error_param");
        param_with_errors.insert(&error_param_id);
        let param_info = ParameterInfo::new(HashMap::new(), param_with_errors);

        let result = resolve_variable(
            &variable,
            &local_vars,
            &param_info,
            &create_test_submodel_info(),
            &create_test_modelinfo(),
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            VariableResolutionError::ParameterHasError(ident) => {
                assert_eq!(ident, Identifier::new("error_param"));
            }
            error => panic!("Expected parameter has error, got {:?}", error),
        }
    }

    #[test]
    fn test_resolve_undefined_submodel() {
        let inner_var = create_identifier_variable("parameter");
        let variable = create_accessor_variable("undefined_submodel", inner_var);

        let local_vars = HashSet::new();

        let result = resolve_variable(
            &variable,
            &local_vars,
            &create_test_parameter_info(),
            &create_test_submodel_info(),
            &create_test_modelinfo(),
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            VariableResolutionError::UndefinedSubmodel(submodel_path, ident) => {
                assert_eq!(submodel_path, None);
                assert_eq!(ident, Identifier::new("undefined_submodel"));
            }
            error => panic!("Expected undefined submodel error, got {:?}", error),
        }
    }

    #[test]
    fn test_resolve_submodel_with_error() {
        let inner_var = create_identifier_variable("parameter");
        let variable = create_accessor_variable("error_submodel", inner_var);

        let local_vars = HashSet::new();

        let mut submodel_with_errors = HashSet::new();
        let error_submodel_id = Identifier::new("error_submodel");
        submodel_with_errors.insert(&error_submodel_id);
        let submodel_info = SubmodelInfo::new(HashMap::new(), submodel_with_errors);

        let result = resolve_variable(
            &variable,
            &local_vars,
            &create_test_parameter_info(),
            &submodel_info,
            &create_test_modelinfo(),
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            VariableResolutionError::SubmodelResolutionFailed(ident) => {
                assert_eq!(ident, Identifier::new("error_submodel"));
            }
            error => panic!("Expected submodel has error, got {:?}", error),
        }
    }

    #[test]
    fn test_resolve_nested_accessor() {
        // Create a nested variable: submodel.parameter
        let inner_var = create_identifier_variable("parameter");
        let variable = create_accessor_variable("submodel", inner_var);

        let local_vars = HashSet::new();

        // Create submodel info with the submodel
        let mut submodel_map = HashMap::new();
        let submodel_path = ModelPath::new("test_submodel");
        let submodel_id = Identifier::new("submodel");
        submodel_map.insert(&submodel_id, &submodel_path);
        let submodel_info = SubmodelInfo::new(submodel_map, HashSet::new());

        // Create model info with the submodel model
        let mut modelmap = HashMap::new();
        let submodel_model = create_modelwith_parameter("parameter");
        modelmap.insert(&submodel_path, &submodel_model);
        let modelinfo = ModelInfo::new(modelmap, HashSet::new());

        let result = resolve_variable(
            &variable,
            &local_vars,
            &create_test_parameter_info(),
            &submodel_info,
            &modelinfo,
        );

        assert!(result.is_ok());
        match result.unwrap() {
            oneil_ir::expr::Expr::Variable(oneil_ir::expr::Variable::Parameter(ident)) => {
                assert_eq!(ident, Identifier::new("parameter"));
            }
            error => panic!("Expected parameter variable expression, got {:?}", error),
        }
    }

    #[test]
    fn test_resolve_deeply_nested_accessor() {
        // Create a deeply nested variable: parameter.submodel2.submodel1
        let parameter_var = create_identifier_variable("parameter");
        let submodel2_var = create_accessor_variable("submodel2", parameter_var);
        let variable = create_accessor_variable("submodel1", submodel2_var);

        let local_vars = HashSet::new();

        // Create submodel info
        let mut submodel_map = HashMap::new();
        let submodel1_path = ModelPath::new("test_submodel1");
        let submodel1_id = Identifier::new("submodel1");
        submodel_map.insert(&submodel1_id, &submodel1_path);
        let submodel_info = SubmodelInfo::new(submodel_map, HashSet::new());

        // Create model info with nested models
        let mut modelmap = HashMap::new();
        let submodel2_path = ModelPath::new("test_submodel2");
        let submodel2_model = create_modelwith_parameter("parameter");
        let submodel1_model = create_modelwith_submodel("submodel2", submodel2_path.clone());
        modelmap.insert(&submodel1_path, &submodel1_model);
        modelmap.insert(&submodel2_path, &submodel2_model);
        let modelinfo = ModelInfo::new(modelmap, HashSet::new());

        let result = resolve_variable(
            &variable,
            &local_vars,
            &create_test_parameter_info(),
            &submodel_info,
            &modelinfo,
        );

        assert!(result.is_ok());
        match result.unwrap() {
            oneil_ir::expr::Expr::Variable(oneil_ir::expr::Variable::Parameter(ident)) => {
                assert_eq!(ident, Identifier::new("parameter"));
            }
            error => panic!("Expected parameter variable expression, got {:?}", error),
        }
    }

    #[test]
    fn test_resolve_undefined_parameter_in_submodel() {
        let inner_var = create_identifier_variable("undefined_param");
        let variable = create_accessor_variable("submodel", inner_var);

        let local_vars = HashSet::new();

        // Create submodel info
        let mut submodel_map = HashMap::new();
        let submodel_path = ModelPath::new("test_submodel");
        let submodel_id = Identifier::new("submodel");
        submodel_map.insert(&submodel_id, &submodel_path);
        let submodel_info = SubmodelInfo::new(submodel_map, HashSet::new());

        // Create model info with empty submodel model
        let mut modelmap = HashMap::new();
        let submodel_model = Model::new(
            HashSet::new(),
            HashMap::new(),
            oneil_ir::parameter::ParameterCollection::new(HashMap::new()),
            HashMap::new(),
            Vec::new(),
        );
        modelmap.insert(&submodel_path, &submodel_model);
        let modelinfo = ModelInfo::new(modelmap, HashSet::new());

        let result = resolve_variable(
            &variable,
            &local_vars,
            &create_test_parameter_info(),
            &submodel_info,
            &modelinfo,
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            VariableResolutionError::UndefinedParameter(Some(path), ident) => {
                assert_eq!(path, submodel_path);
                assert_eq!(ident, Identifier::new("undefined_param"));
            }
            error => panic!(
                "Expected undefined parameter in submodel error, got {:?}",
                error
            ),
        }
    }

    #[test]
    fn test_resolve_undefined_submodel_in_submodel() {
        let inner_var = create_identifier_variable("parameter");
        let variable = create_accessor_variable("undefined_submodel", inner_var);
        let variable = create_accessor_variable("submodel", variable);

        let local_vars = HashSet::new();

        // Create submodel info
        let mut submodel_map = HashMap::new();
        let submodel_path = ModelPath::new("test_submodel");
        let submodel_id = Identifier::new("submodel");
        submodel_map.insert(&submodel_id, &submodel_path);
        let submodel_info = SubmodelInfo::new(submodel_map, HashSet::new());

        // Create model info with empty submodel model
        let mut modelmap = HashMap::new();
        let submodel_model = Model::new(
            HashSet::new(),
            HashMap::new(),
            oneil_ir::parameter::ParameterCollection::new(HashMap::new()),
            HashMap::new(),
            Vec::new(),
        );
        modelmap.insert(&submodel_path, &submodel_model);
        let modelinfo = ModelInfo::new(modelmap, HashSet::new());

        let result = resolve_variable(
            &variable,
            &local_vars,
            &create_test_parameter_info(),
            &submodel_info,
            &modelinfo,
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            VariableResolutionError::UndefinedSubmodel(Some(path), ident) => {
                assert_eq!(path, submodel_path);
                assert_eq!(ident, Identifier::new("undefined_submodel"));
            }
            error => panic!(
                "Expected undefined submodel in submodel error, got {:?}",
                error
            ),
        }
    }

    #[test]
    fn test_resolve_modelwith_error() {
        let inner_var = create_identifier_variable("parameter");
        let variable = create_accessor_variable("submodel", inner_var);

        let local_vars = HashSet::new();

        // Create submodel info
        let mut submodel_map = HashMap::new();
        let submodel_path = ModelPath::new("test_submodel");
        let submodel_id = Identifier::new("submodel");
        submodel_map.insert(&submodel_id, &submodel_path);
        let submodel_info = SubmodelInfo::new(submodel_map, HashSet::new());

        // Create model info with error
        let mut modelwith_errors = HashSet::new();
        modelwith_errors.insert(&submodel_path);
        let modelinfo = ModelInfo::new(HashMap::new(), modelwith_errors);

        let result = resolve_variable(
            &variable,
            &local_vars,
            &create_test_parameter_info(),
            &submodel_info,
            &modelinfo,
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            VariableResolutionError::ModelHasError(path) => {
                assert_eq!(path, submodel_path);
            }
            error => panic!("Expected model has error, got {:?}", error),
        }
    }

    #[test]
    fn test_local_variable_takes_precedence_over_parameter() {
        let variable = create_identifier_variable("conflict");
        let mut local_vars = HashSet::new();
        local_vars.insert(Identifier::new("conflict"));

        let mut param_map = HashMap::new();
        let param = Parameter::new(
            HashSet::new(),
            Identifier::new("conflict"),
            oneil_ir::parameter::ParameterValue::simple(
                oneil_ir::expr::Expr::literal(oneil_ir::expr::Literal::number(42.0)),
                None,
            ),
            oneil_ir::parameter::Limits::default(),
            false,
            oneil_ir::debug_info::TraceLevel::None,
        );
        let conflict_id = Identifier::new("conflict");
        param_map.insert(&conflict_id, &param);
        let param_info = ParameterInfo::new(param_map, HashSet::new());

        let result = resolve_variable(
            &variable,
            &local_vars,
            &param_info,
            &create_test_submodel_info(),
            &create_test_modelinfo(),
        );

        assert!(result.is_ok());
        match result.unwrap() {
            oneil_ir::expr::Expr::Variable(oneil_ir::expr::Variable::Local(ident)) => {
                assert_eq!(ident, Identifier::new("conflict"));
            }
            error => panic!(
                "Expected local variable expression (should take precedence), got {:?}",
                error
            ),
        }
    }

    #[test]
    fn test_empty_local_variables() {
        let variable = create_identifier_variable("parameter");
        let local_vars = HashSet::new();

        let mut param_map = HashMap::new();
        let param = Parameter::new(
            HashSet::new(),
            Identifier::new("parameter"),
            oneil_ir::parameter::ParameterValue::simple(
                oneil_ir::expr::Expr::literal(oneil_ir::expr::Literal::number(42.0)),
                None,
            ),
            oneil_ir::parameter::Limits::default(),
            false,
            oneil_ir::debug_info::TraceLevel::None,
        );
        let parameter_id = Identifier::new("parameter");
        param_map.insert(&parameter_id, &param);
        let param_info = ParameterInfo::new(param_map, HashSet::new());

        let result = resolve_variable(
            &variable,
            &local_vars,
            &param_info,
            &create_test_submodel_info(),
            &create_test_modelinfo(),
        );

        assert!(result.is_ok());
        match result.unwrap() {
            oneil_ir::expr::Expr::Variable(oneil_ir::expr::Variable::Parameter(ident)) => {
                assert_eq!(ident, Identifier::new("parameter"));
            }
            error => panic!("Expected parameter variable expression, got {:?}", error),
        }
    }
}
