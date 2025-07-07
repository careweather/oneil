//! Variable resolution for the Oneil module loader
//!
//! This module provides functionality for resolving variables in Oneil expressions.
//! Variable resolution involves determining the scope and type of variables based
//! on their context within a module hierarchy.
//!
//! # Overview
//!
//! Variables in Oneil can be:
//! - **Local variables**: Defined within the current scope (e.g., test inputs)
//! - **Parameter variables**: Defined as parameters in the current module
//! - **Submodel variables**: Accessible through submodel paths (e.g., `parameter.submodel`)
//!
//! # Variable Types
//!
//! ## Simple Identifiers
//! Simple variable names like `x`, `temperature`, etc. are resolved by checking:
//! 1. Local variables (test inputs, function parameters)
//! 2. Module parameters
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
//! The module provides comprehensive error handling for various failure scenarios:
//! - Undefined parameters or submodels
//! - Parameters or submodels with resolution errors
//! - Module loading errors
//!

use std::collections::HashSet;

use oneil_ast as ast;
use oneil_module::reference::{Identifier, ModulePath};

use crate::{
    error::VariableResolutionError,
    loader::resolver::{ModuleInfo, ParameterInfo, SubmodelInfo},
    util::info::InfoResult,
};

/// Resolves a variable expression to its corresponding module expression.
///
/// This function handles the resolution of variables in Oneil expressions,
/// determining whether they refer to local variables, parameters, or submodel
/// components. The resolution process follows a hierarchical lookup pattern
/// that respects the module structure and error states.
///
/// # Arguments
///
/// * `variable` - The AST variable to resolve
/// * `local_variables` - Set of local variable identifiers (e.g., test inputs)
/// * `defined_parameters` - Information about available parameters and their error states
/// * `submodel_info` - Information about available submodels and their error states
/// * `module_info` - Information about loaded modules and their error states
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
/// * `VariableResolutionError::ModuleHasError` - When a module has loading errors
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

/// Recursively resolves a variable within a specific submodel context.
///
/// This internal function handles the recursive resolution of variables within
/// submodel contexts. It's called by `resolve_variable` when dealing with
/// accessor variables (e.g., `submodel.parameter`).
///
/// # Safety
///
/// This function assumes that the submodel path has been validated and the module
/// exists in the module_info. If this assumption is violated, the function will panic.
/// This is by design as it indicates a bug in the module loading process.
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

#[cfg(test)]
mod tests {
    use super::*;
    use oneil_ast as ast;
    use oneil_module::{
        module::Module,
        parameter::Parameter,
        reference::{Identifier, ModulePath},
    };
    use std::collections::{HashMap, HashSet};

    /// Helper function to create a basic module info for testing
    fn create_test_module_info() -> ModuleInfo<'static> {
        ModuleInfo::new(HashMap::new(), HashSet::new())
    }

    /// Helper function to create a basic parameter info for testing
    fn create_test_parameter_info() -> ParameterInfo<'static> {
        ParameterInfo::new(HashMap::new(), HashSet::new())
    }

    /// Helper function to create a basic submodel info for testing
    fn create_test_submodel_info() -> SubmodelInfo<'static> {
        SubmodelInfo::new(HashMap::new(), HashSet::new())
    }

    /// Helper function to create a module with a parameter
    fn create_module_with_parameter(param_name: &str) -> Module {
        let mut param_map = HashMap::new();
        let param = Parameter::new(
            HashSet::new(),
            Identifier::new(param_name),
            oneil_module::parameter::ParameterValue::simple(
                oneil_module::expr::Expr::literal(oneil_module::expr::Literal::number(42.0)),
                None,
            ),
            oneil_module::parameter::Limits::default(),
            false,
            oneil_module::debug_info::TraceLevel::None,
        );
        param_map.insert(Identifier::new(param_name), param);
        let param_collection = oneil_module::parameter::ParameterCollection::new(param_map);

        Module::new(
            HashSet::new(),
            HashMap::new(),
            param_collection,
            HashMap::new(),
            Vec::new(),
        )
    }

    /// Helper function to create a module with a submodel
    fn create_module_with_submodel(submodel_name: &str, submodel_path: ModulePath) -> Module {
        let mut submodel_map = HashMap::new();
        submodel_map.insert(Identifier::new(submodel_name), submodel_path);

        Module::new(
            HashSet::new(),
            submodel_map,
            oneil_module::parameter::ParameterCollection::new(HashMap::new()),
            HashMap::new(),
            Vec::new(),
        )
    }

    #[test]
    fn test_resolve_local_variable() {
        let variable = ast::expression::Variable::Identifier("local_var".to_string());
        let mut local_vars = HashSet::new();
        local_vars.insert(Identifier::new("local_var"));

        let result = resolve_variable(
            &variable,
            &local_vars,
            &create_test_parameter_info(),
            &create_test_submodel_info(),
            &create_test_module_info(),
        );

        assert!(result.is_ok());
        match result.unwrap() {
            oneil_module::expr::Expr::Variable(oneil_module::expr::Variable::Local(ident)) => {
                assert_eq!(ident, Identifier::new("local_var"));
            }
            error => panic!("Expected local variable expression, got {:?}", error),
        }
    }

    #[test]
    fn test_resolve_parameter_variable() {
        let variable = ast::expression::Variable::Identifier("temperature".to_string());
        let local_vars = HashSet::new();

        let mut param_map = HashMap::new();
        let temp_param = Parameter::new(
            HashSet::new(),
            Identifier::new("temperature"),
            oneil_module::parameter::ParameterValue::simple(
                oneil_module::expr::Expr::literal(oneil_module::expr::Literal::number(42.0)),
                None,
            ),
            oneil_module::parameter::Limits::default(),
            false,
            oneil_module::debug_info::TraceLevel::None,
        );
        let temp_param_id = Identifier::new("temperature");
        param_map.insert(&temp_param_id, &temp_param);
        let param_info = ParameterInfo::new(param_map, HashSet::new());

        let result = resolve_variable(
            &variable,
            &local_vars,
            &param_info,
            &create_test_submodel_info(),
            &create_test_module_info(),
        );

        assert!(result.is_ok());
        match result.unwrap() {
            oneil_module::expr::Expr::Variable(oneil_module::expr::Variable::Parameter(ident)) => {
                assert_eq!(ident, Identifier::new("temperature"));
            }
            error => panic!("Expected parameter variable expression, got {:?}", error),
        }
    }

    #[test]
    fn test_resolve_undefined_parameter() {
        let variable = ast::expression::Variable::Identifier("undefined_param".to_string());
        let local_vars = HashSet::new();

        let result = resolve_variable(
            &variable,
            &local_vars,
            &create_test_parameter_info(),
            &create_test_submodel_info(),
            &create_test_module_info(),
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
        let variable = ast::expression::Variable::Identifier("error_param".to_string());
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
            &create_test_module_info(),
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
        let inner_var = ast::expression::Variable::Identifier("parameter".to_string());
        let variable = ast::expression::Variable::Accessor {
            parent: "undefined_submodel".to_string(),
            component: Box::new(inner_var),
        };

        let local_vars = HashSet::new();

        let result = resolve_variable(
            &variable,
            &local_vars,
            &create_test_parameter_info(),
            &create_test_submodel_info(),
            &create_test_module_info(),
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
        let inner_var = ast::expression::Variable::Identifier("parameter".to_string());
        let variable = ast::expression::Variable::Accessor {
            parent: "error_submodel".to_string(),
            component: Box::new(inner_var),
        };

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
            &create_test_module_info(),
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            VariableResolutionError::SubmodelHasError(ident) => {
                assert_eq!(ident, Identifier::new("error_submodel"));
            }
            error => panic!("Expected submodel has error, got {:?}", error),
        }
    }

    #[test]
    fn test_resolve_nested_accessor() {
        // Create a nested variable: submodel.parameter
        let inner_var = ast::expression::Variable::Identifier("parameter".to_string());
        let variable = ast::expression::Variable::Accessor {
            parent: "submodel".to_string(),
            component: Box::new(inner_var),
        };

        let local_vars = HashSet::new();

        // Create submodel info with the submodel
        let mut submodel_map = HashMap::new();
        let submodel_path = ModulePath::new("test_submodel");
        let submodel_id = Identifier::new("submodel");
        submodel_map.insert(&submodel_id, &submodel_path);
        let submodel_info = SubmodelInfo::new(submodel_map, HashSet::new());

        // Create module info with the submodel module
        let mut module_map = HashMap::new();
        let submodel_module = create_module_with_parameter("parameter");
        module_map.insert(&submodel_path, &submodel_module);
        let module_info = ModuleInfo::new(module_map, HashSet::new());

        let result = resolve_variable(
            &variable,
            &local_vars,
            &create_test_parameter_info(),
            &submodel_info,
            &module_info,
        );

        assert!(result.is_ok());
        match result.unwrap() {
            oneil_module::expr::Expr::Variable(oneil_module::expr::Variable::Parameter(ident)) => {
                assert_eq!(ident, Identifier::new("parameter"));
            }
            error => panic!("Expected parameter variable expression, got {:?}", error),
        }
    }

    #[test]
    fn test_resolve_deeply_nested_accessor() {
        // Create a deeply nested variable: parameter.submodel2.submodel1
        let parameter_var = ast::expression::Variable::Identifier("parameter".to_string());
        let submodel2_var = ast::expression::Variable::Accessor {
            parent: "submodel2".to_string(),
            component: Box::new(parameter_var),
        };
        let variable = ast::expression::Variable::Accessor {
            parent: "submodel1".to_string(),
            component: Box::new(submodel2_var),
        };

        let local_vars = HashSet::new();

        // Create submodel info
        let mut submodel_map = HashMap::new();
        let submodel1_path = ModulePath::new("test_submodel1");
        let submodel1_id = Identifier::new("submodel1");
        submodel_map.insert(&submodel1_id, &submodel1_path);
        let submodel_info = SubmodelInfo::new(submodel_map, HashSet::new());

        // Create module info with nested modules
        let mut module_map = HashMap::new();
        let submodel2_path = ModulePath::new("test_submodel2");
        let submodel2_module = create_module_with_parameter("parameter");
        let submodel1_module = create_module_with_submodel("submodel2", submodel2_path.clone());
        module_map.insert(&submodel1_path, &submodel1_module);
        module_map.insert(&submodel2_path, &submodel2_module);
        let module_info = ModuleInfo::new(module_map, HashSet::new());

        let result = resolve_variable(
            &variable,
            &local_vars,
            &create_test_parameter_info(),
            &submodel_info,
            &module_info,
        );

        assert!(result.is_ok());
        match result.unwrap() {
            oneil_module::expr::Expr::Variable(oneil_module::expr::Variable::Parameter(ident)) => {
                assert_eq!(ident, Identifier::new("parameter"));
            }
            error => panic!("Expected parameter variable expression, got {:?}", error),
        }
    }

    #[test]
    fn test_resolve_undefined_parameter_in_submodel() {
        let inner_var = ast::expression::Variable::Identifier("undefined_param".to_string());
        let variable = ast::expression::Variable::Accessor {
            parent: "submodel".to_string(),
            component: Box::new(inner_var),
        };

        let local_vars = HashSet::new();

        // Create submodel info
        let mut submodel_map = HashMap::new();
        let submodel_path = ModulePath::new("test_submodel");
        let submodel_id = Identifier::new("submodel");
        submodel_map.insert(&submodel_id, &submodel_path);
        let submodel_info = SubmodelInfo::new(submodel_map, HashSet::new());

        // Create module info with empty submodel module
        let mut module_map = HashMap::new();
        let submodel_module = Module::new(
            HashSet::new(),
            HashMap::new(),
            oneil_module::parameter::ParameterCollection::new(HashMap::new()),
            HashMap::new(),
            Vec::new(),
        );
        module_map.insert(&submodel_path, &submodel_module);
        let module_info = ModuleInfo::new(module_map, HashSet::new());

        let result = resolve_variable(
            &variable,
            &local_vars,
            &create_test_parameter_info(),
            &submodel_info,
            &module_info,
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
        let inner_var = ast::expression::Variable::Identifier("parameter".to_string());
        let variable = ast::expression::Variable::Accessor {
            parent: "undefined_submodel".to_string(),
            component: Box::new(inner_var),
        };
        let variable = ast::expression::Variable::Accessor {
            parent: "submodel".to_string(),
            component: Box::new(variable),
        };

        let local_vars = HashSet::new();

        // Create submodel info
        let mut submodel_map = HashMap::new();
        let submodel_path = ModulePath::new("test_submodel");
        let submodel_id = Identifier::new("submodel");
        submodel_map.insert(&submodel_id, &submodel_path);
        let submodel_info = SubmodelInfo::new(submodel_map, HashSet::new());

        // Create module info with empty submodel module
        let mut module_map = HashMap::new();
        let submodel_module = Module::new(
            HashSet::new(),
            HashMap::new(),
            oneil_module::parameter::ParameterCollection::new(HashMap::new()),
            HashMap::new(),
            Vec::new(),
        );
        module_map.insert(&submodel_path, &submodel_module);
        let module_info = ModuleInfo::new(module_map, HashSet::new());

        let result = resolve_variable(
            &variable,
            &local_vars,
            &create_test_parameter_info(),
            &submodel_info,
            &module_info,
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
    fn test_resolve_module_with_error() {
        let inner_var = ast::expression::Variable::Identifier("parameter".to_string());
        let variable = ast::expression::Variable::Accessor {
            parent: "submodel".to_string(),
            component: Box::new(inner_var),
        };

        let local_vars = HashSet::new();

        // Create submodel info
        let mut submodel_map = HashMap::new();
        let submodel_path = ModulePath::new("test_submodel");
        let submodel_id = Identifier::new("submodel");
        submodel_map.insert(&submodel_id, &submodel_path);
        let submodel_info = SubmodelInfo::new(submodel_map, HashSet::new());

        // Create module info with error
        let mut module_with_errors = HashSet::new();
        module_with_errors.insert(&submodel_path);
        let module_info = ModuleInfo::new(HashMap::new(), module_with_errors);

        let result = resolve_variable(
            &variable,
            &local_vars,
            &create_test_parameter_info(),
            &submodel_info,
            &module_info,
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            VariableResolutionError::ModuleHasError(path) => {
                assert_eq!(path, submodel_path);
            }
            error => panic!("Expected module has error, got {:?}", error),
        }
    }

    #[test]
    fn test_local_variable_takes_precedence_over_parameter() {
        let variable = ast::expression::Variable::Identifier("conflict".to_string());
        let mut local_vars = HashSet::new();
        local_vars.insert(Identifier::new("conflict"));

        let mut param_map = HashMap::new();
        let param = Parameter::new(
            HashSet::new(),
            Identifier::new("conflict"),
            oneil_module::parameter::ParameterValue::simple(
                oneil_module::expr::Expr::literal(oneil_module::expr::Literal::number(42.0)),
                None,
            ),
            oneil_module::parameter::Limits::default(),
            false,
            oneil_module::debug_info::TraceLevel::None,
        );
        let conflict_id = Identifier::new("conflict");
        param_map.insert(&conflict_id, &param);
        let param_info = ParameterInfo::new(param_map, HashSet::new());

        let result = resolve_variable(
            &variable,
            &local_vars,
            &param_info,
            &create_test_submodel_info(),
            &create_test_module_info(),
        );

        assert!(result.is_ok());
        match result.unwrap() {
            oneil_module::expr::Expr::Variable(oneil_module::expr::Variable::Local(ident)) => {
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
        let variable = ast::expression::Variable::Identifier("parameter".to_string());
        let local_vars = HashSet::new();

        let mut param_map = HashMap::new();
        let param = Parameter::new(
            HashSet::new(),
            Identifier::new("parameter"),
            oneil_module::parameter::ParameterValue::simple(
                oneil_module::expr::Expr::literal(oneil_module::expr::Literal::number(42.0)),
                None,
            ),
            oneil_module::parameter::Limits::default(),
            false,
            oneil_module::debug_info::TraceLevel::None,
        );
        let parameter_id = Identifier::new("parameter");
        param_map.insert(&parameter_id, &param);
        let param_info = ParameterInfo::new(param_map, HashSet::new());

        let result = resolve_variable(
            &variable,
            &local_vars,
            &param_info,
            &create_test_submodel_info(),
            &create_test_module_info(),
        );

        assert!(result.is_ok());
        match result.unwrap() {
            oneil_module::expr::Expr::Variable(oneil_module::expr::Variable::Parameter(ident)) => {
                assert_eq!(ident, Identifier::new("parameter"));
            }
            error => panic!("Expected parameter variable expression, got {:?}", error),
        }
    }
}
