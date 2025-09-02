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

use oneil_ast as ast;
use oneil_ir::{
    expr::{Expr, ExprWithSpan},
    reference::{Identifier, ModelPath},
    span::{Span, WithSpan},
};

use crate::{
    BuiltinRef,
    error::VariableResolutionError,
    util::{
        context::{LookupResult, ModelContext, ModelImportsContext, ParameterContext},
        get_span_from_ast_span,
    },
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
    builtin_ref: &impl BuiltinRef,
    context: &(impl ModelContext + ModelImportsContext + ParameterContext),
) -> Result<ExprWithSpan, VariableResolutionError> {
    match variable.node_value() {
        ast::expression::Variable::Identifier(identifier) => {
            let span = get_span_from_ast_span(variable.node_span());
            let var_identifier = Identifier::new(identifier.as_str());
            let var_identifier_span = get_span_from_ast_span(identifier.node_span());

            match context.lookup_parameter(&var_identifier) {
                LookupResult::Found(_parameter) => {
                    let span = get_span_from_ast_span(variable.node_span());
                    let expr = Expr::parameter_variable(var_identifier);
                    Ok(WithSpan::new(expr, span))
                }
                LookupResult::HasError => Err(VariableResolutionError::parameter_has_error(
                    var_identifier,
                    var_identifier_span,
                )),
                LookupResult::NotFound => {
                    if builtin_ref.has_builtin_value(&var_identifier) {
                        let expr = Expr::builtin_variable(var_identifier);
                        Ok(WithSpan::new(expr, span))
                    } else {
                        Err(VariableResolutionError::undefined_parameter(
                            var_identifier,
                            var_identifier_span,
                        ))
                    }
                }
            }
        }
        ast::expression::Variable::Accessor { parent, component } => {
            let parent_identifier = Identifier::new(parent.as_str());
            let parent_identifier_span = get_span_from_ast_span(parent.node_span());
            let submodel_path = match context.lookup_submodel(&parent_identifier) {
                LookupResult::Found((submodel_path, _span)) => submodel_path,
                LookupResult::HasError => {
                    return Err(VariableResolutionError::submodel_resolution_failed(
                        parent_identifier,
                        parent_identifier_span,
                    ));
                }
                LookupResult::NotFound => {
                    return Err(VariableResolutionError::undefined_submodel(
                        parent_identifier,
                        parent_identifier_span,
                    ));
                }
            };

            let (model_path, ident) = resolve_variable_recursive(
                submodel_path,
                component,
                parent_identifier_span,
                context,
            )?;

            let span = get_span_from_ast_span(variable.node_span());
            let expr = Expr::external_variable(model_path, ident);
            Ok(WithSpan::new(expr, span))
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
#[allow(
    clippy::panic_in_result_fn,
    reason = "panic enforces a function invariant"
)]
fn resolve_variable_recursive(
    submodel_path: &ModelPath,
    variable: &ast::expression::Variable,
    parent_identifier_span: Span,
    context: &impl ModelContext,
) -> Result<(ModelPath, Identifier), VariableResolutionError> {
    let model = match context.lookup_model(submodel_path) {
        LookupResult::Found(model) => model,
        LookupResult::HasError => {
            return Err(VariableResolutionError::model_has_error(
                submodel_path.clone(),
                parent_identifier_span,
            ));
        }
        LookupResult::NotFound => panic!("submodel should have been visited already"),
    };

    match variable {
        // if the variable is an identifier, this means that the variable refers to a parameter
        ast::expression::Variable::Identifier(identifier) => {
            let var_identifier = Identifier::new(identifier.as_str());
            let var_identifier_span = get_span_from_ast_span(identifier.node_span());
            if model.get_parameter(&var_identifier).is_some() {
                Ok((submodel_path.clone(), var_identifier))
            } else {
                Err(VariableResolutionError::undefined_parameter_in_submodel(
                    submodel_path.clone(),
                    var_identifier,
                    var_identifier_span,
                ))
            }
        }

        // if the variable is an accessor, this means that the variable refers to a submodel
        ast::expression::Variable::Accessor { parent, component } => {
            let parent_identifier = Identifier::new(parent.as_str());
            let parent_identifier_span = get_span_from_ast_span(parent.node_span());
            let Some((submodel_path, _)) = model.get_submodel(&parent_identifier) else {
                let source = VariableResolutionError::undefined_submodel_in_submodel(
                    submodel_path.clone(),
                    parent_identifier,
                    parent_identifier_span,
                );
                return Err(source);
            };

            resolve_variable_recursive(submodel_path, component, parent_identifier_span, context)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test::{TestBuiltinRef, TestContext};

    use super::*;
    use oneil_ast as ast;
    use oneil_ir::{
        model::Model,
        parameter::Parameter,
        reference::{Identifier, ModelPath},
    };
    use std::collections::{HashMap, HashSet};

    // TODO: write tests that test the span of the test inputs

    mod helper {
        use super::*;

        /// Helper function to create a test span
        pub fn test_ast_span(start: usize, end: usize) -> ast::Span {
            ast::Span::new(start, end - start, 0)
        }

        /// Helper function to create a test IR span
        pub fn test_ir_span(start: usize, end: usize) -> oneil_ir::span::Span {
            oneil_ir::span::Span::new(start, end - start)
        }

        /// Helper function to create an identifier node
        pub fn create_identifier_node(name: &str, start: usize) -> ast::naming::IdentifierNode {
            let identifier = ast::naming::Identifier::new(name.to_string());
            ast::node::Node::new(&test_ast_span(start, start + name.len()), identifier)
        }

        /// Helper function to create a variable node
        fn create_variable_node(
            variable: ast::expression::Variable,
            start: usize,
            end: usize,
        ) -> ast::expression::VariableNode {
            ast::node::Node::new(&test_ast_span(start, end), variable)
        }

        /// Helper function to create a simple identifier variable
        pub fn create_identifier_variable(name: &str) -> ast::expression::VariableNode {
            let identifier_node = create_identifier_node(name, 0);
            let variable = ast::expression::Variable::Identifier(identifier_node);
            create_variable_node(variable, 0, name.len())
        }

        /// Helper function to create an accessor variable
        pub fn create_accessor_variable(
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

        /// Helper function to create a model with a parameter
        pub fn create_model_with_parameter(param_name: &str) -> Model {
            let mut param_map = HashMap::new();
            let param_name = WithSpan::new(
                Identifier::new(param_name),
                test_ir_span(0, param_name.len()),
            );

            let param = Parameter::new(
                HashSet::new(),
                param_name.clone(),
                oneil_ir::parameter::ParameterValue::simple(
                    WithSpan::new(
                        oneil_ir::expr::Expr::literal(oneil_ir::expr::Literal::number(42.0)),
                        test_ir_span(0, 1),
                    ),
                    None,
                ),
                oneil_ir::parameter::Limits::default(),
                false,
                oneil_ir::debug_info::TraceLevel::None,
            );
            param_map.insert(param_name.value().clone(), param);
            let param_collection = oneil_ir::parameter::ParameterCollection::new(param_map);

            Model::new(
                HashMap::new(),
                HashMap::new(),
                param_collection,
                HashMap::new(),
            )
        }

        /// Helper function to create a model with a submodel
        pub fn create_modelwith_submodel(
            submodel_name: &str,
            submodel_path: (ModelPath, Span),
        ) -> Model {
            let mut submodel_map = HashMap::new();
            submodel_map.insert(Identifier::new(submodel_name), submodel_path);

            Model::new(
                HashMap::new(),
                submodel_map,
                oneil_ir::parameter::ParameterCollection::new(HashMap::new()),
                HashMap::new(),
            )
        }

        /// Helper function for getting the span of an accessor parent identifier
        pub fn get_accessor_parent_identifier_span(
            accessor: &ast::expression::VariableNode,
        ) -> oneil_ir::span::Span {
            let parent_span = match accessor.node_value() {
                ast::expression::Variable::Accessor { parent, .. } => parent.node_span(),
                ast::expression::Variable::Identifier(_) => {
                    panic!("accessor should be an accessor variable")
                }
            };
            get_span_from_ast_span(parent_span)
        }
    }

    #[test]
    fn test_resolve_builtin_variable() {
        // create a local variable
        let variable = helper::create_identifier_variable("pi");

        // create the context and builtin ref
        let builtin_ref = TestBuiltinRef::new().with_builtin_variables(["pi"]);
        let context = TestContext::new();

        // resolve the variable
        let result = resolve_variable(&variable, &builtin_ref, &context);

        // check the result
        assert!(result.is_ok());
        let result = result.expect("result should be ok");

        assert_eq!(result.span(), get_span_from_ast_span(variable.node_span()));
        match result.value() {
            oneil_ir::expr::Expr::Variable(oneil_ir::expr::Variable::Builtin(ident)) => {
                assert_eq!(ident, &Identifier::new("pi"));
            }
            error => panic!("Expected builtin variable expression, got {error:?}"),
        }
    }

    #[test]
    fn test_resolve_parameter_variable() {
        // create a parameter variable
        let variable = helper::create_identifier_variable("temperature");

        // create the context and builtin ref
        let builtin_ref = TestBuiltinRef::new().with_builtin_variables(["temperature"]);

        let temp_param_id = Identifier::new("temperature");
        let temp_param = Parameter::new(
            HashSet::new(),
            WithSpan::new(Identifier::new("temperature"), helper::test_ir_span(0, 10)),
            oneil_ir::parameter::ParameterValue::simple(
                WithSpan::new(
                    oneil_ir::expr::Expr::literal(oneil_ir::expr::Literal::number(42.0)),
                    helper::test_ir_span(0, 1),
                ),
                None,
            ),
            oneil_ir::parameter::Limits::default(),
            false,
            oneil_ir::debug_info::TraceLevel::None,
        );

        let context = TestContext::new().with_parameter_context([(temp_param_id, temp_param)]);

        // resolve the variable
        let result = resolve_variable(&variable, &builtin_ref, &context);

        // check the result
        assert!(result.is_ok());
        let result = result.expect("result should be ok");

        assert_eq!(result.span(), get_span_from_ast_span(variable.node_span()));
        match result.value() {
            oneil_ir::expr::Expr::Variable(oneil_ir::expr::Variable::Parameter(ident)) => {
                assert_eq!(ident, &Identifier::new("temperature"));
            }
            error => panic!("Expected parameter variable expression, got {error:?}"),
        }
    }

    #[test]
    fn test_resolve_undefined_parameter() {
        // create a variable for undefined parameter
        let variable = helper::create_identifier_variable("undefined_param");

        // create the context and builtin ref
        let builtin_ref = TestBuiltinRef::new();
        let context = TestContext::new();

        // resolve the variable
        let result = resolve_variable(&variable, &builtin_ref, &context);

        // check the result
        match result {
            Err(VariableResolutionError::UndefinedParameter {
                model_path,
                parameter,
                reference_span,
            }) => {
                assert_eq!(model_path, None);

                let span = get_span_from_ast_span(variable.node_span());
                assert_eq!(reference_span, span);
                assert_eq!(parameter, Identifier::new("undefined_param"));
            }
            result => panic!("Expected undefined parameter error, got {result:?}"),
        }
    }

    #[test]
    fn test_resolve_parameter_with_error() {
        // create a variable for parameter with error
        let variable = helper::create_identifier_variable("error_param");
        let variable_span = get_span_from_ast_span(variable.node_span());

        // create the context and builtin ref
        let builtin_ref = TestBuiltinRef::new();

        let error_param_id = Identifier::new("error_param");
        let context = TestContext::new().with_parameter_errors([error_param_id]);

        // resolve the variable
        let result = resolve_variable(&variable, &builtin_ref, &context);

        // check the result
        match result {
            Err(VariableResolutionError::ParameterHasError {
                identifier,
                reference_span,
            }) => {
                assert_eq!(identifier, Identifier::new("error_param"));
                assert_eq!(reference_span, variable_span);
            }
            result => panic!("Expected parameter has error, got {result:?}"),
        }
    }

    #[test]
    fn test_resolve_undefined_submodel() {
        // create an accessor variable for undefined submodel
        let inner_var = helper::create_identifier_variable("parameter");
        let variable = helper::create_accessor_variable("undefined_submodel", inner_var);
        let undefined_submodel_span = helper::get_accessor_parent_identifier_span(&variable);

        // create the context and builtin ref
        let builtin_ref = TestBuiltinRef::new();
        let context = TestContext::new();

        // resolve the variable
        let result = resolve_variable(&variable, &builtin_ref, &context);

        // check the result
        match result {
            Err(VariableResolutionError::UndefinedSubmodel {
                model_path,
                submodel,
                reference_span,
            }) => {
                assert_eq!(model_path, None);

                assert_eq!(reference_span, undefined_submodel_span);
                assert_eq!(submodel, Identifier::new("undefined_submodel"));
            }
            result => panic!("Expected undefined submodel error, got {result:?}"),
        }
    }

    #[test]
    fn test_resolve_submodel_with_error() {
        // create an accessor variable for submodel with error
        let inner_var = helper::create_identifier_variable("parameter");
        let variable = helper::create_accessor_variable("error_submodel", inner_var);
        let error_submodel_span = helper::get_accessor_parent_identifier_span(&variable);

        // create the context and builtin ref
        let builtin_ref = TestBuiltinRef::new();

        let error_submodel_id = Identifier::new("error_submodel");
        let context = TestContext::new().with_submodel_errors([error_submodel_id]);

        // resolve the variable
        let result = resolve_variable(&variable, &builtin_ref, &context);

        // check the result
        match result {
            Err(VariableResolutionError::SubmodelResolutionFailed {
                identifier,
                reference_span,
            }) => {
                assert_eq!(identifier, Identifier::new("error_submodel"));
                assert_eq!(reference_span, error_submodel_span);
            }
            result => panic!("Expected submodel has error, got {result:?}"),
        }
    }

    #[test]
    fn test_resolve_nested_accessor() {
        // create a nested accessor variable: submodel.parameter
        let inner_var = helper::create_identifier_variable("parameter");
        let variable = helper::create_accessor_variable("submodel", inner_var);

        // create the context and builtin ref
        let builtin_ref = TestBuiltinRef::new();

        let submodel_path = ModelPath::new("test_submodel");
        let submodel_path_span = helper::test_ir_span(0, 10);
        let submodel_id = Identifier::new("submodel");

        let submodel_model = helper::create_model_with_parameter("parameter");

        let context = TestContext::new()
            .with_submodel_context([(submodel_id, (submodel_path.clone(), submodel_path_span))])
            .with_model_context([(submodel_path, submodel_model)]);

        // resolve the variable
        let result = resolve_variable(&variable, &builtin_ref, &context);

        // check the result
        match result {
            Ok(result) => match result.value() {
                oneil_ir::expr::Expr::Variable(oneil_ir::expr::Variable::External {
                    model,
                    ident,
                }) => {
                    assert_eq!(model, &ModelPath::new("test_submodel"));
                    assert_eq!(ident, &Identifier::new("parameter"));
                }
                result => panic!("Expected external variable expression, got {result:?}"),
            },
            Err(error) => panic!("Expected external variable expression, got {error:?}"),
        }
    }

    #[test]
    fn test_resolve_deeply_nested_accessor() {
        // create a deeply nested accessor variable: submodel1.submodel2.parameter
        let parameter_var = helper::create_identifier_variable("parameter");
        let submodel2_var = helper::create_accessor_variable("submodel2", parameter_var);
        let variable = helper::create_accessor_variable("submodel1", submodel2_var);

        // create the context and builtin ref
        let builtin_ref = TestBuiltinRef::new();

        let submodel1_path = ModelPath::new("test_submodel1");
        let submodel1_path_span = helper::test_ir_span(0, 10);
        let submodel1_id = Identifier::new("submodel1");

        let submodel2_path = ModelPath::new("test_submodel2");
        let submodel2_path_span = helper::test_ir_span(0, 10);
        let submodel2_model = helper::create_model_with_parameter("parameter");
        let submodel1_model = helper::create_modelwith_submodel(
            "submodel2",
            (submodel2_path.clone(), submodel2_path_span),
        );

        let context = TestContext::new()
            .with_submodel_context([(submodel1_id, (submodel1_path.clone(), submodel1_path_span))])
            .with_model_context([
                (submodel1_path, submodel1_model),
                (submodel2_path, submodel2_model),
            ]);

        // resolve the variable
        let result = resolve_variable(&variable, &builtin_ref, &context);

        // check the result
        match result {
            Ok(result) => match result.value() {
                oneil_ir::expr::Expr::Variable(oneil_ir::expr::Variable::External {
                    model,
                    ident,
                }) => {
                    assert_eq!(model, &ModelPath::new("test_submodel2"));
                    assert_eq!(ident, &Identifier::new("parameter"));
                }
                result => panic!("Expected external variable expression, got {result:?}"),
            },
            Err(error) => panic!("Expected external variable expression, got {error:?}"),
        }
    }

    #[test]
    fn test_resolve_undefined_parameter_in_submodel() {
        // create an accessor variable for undefined parameter in submodel
        let inner_var = helper::create_identifier_variable("undefined_param");
        let inner_var_span = get_span_from_ast_span(inner_var.node_span());
        let variable = helper::create_accessor_variable("submodel", inner_var);

        // create the context and builtin ref
        let builtin_ref = TestBuiltinRef::new();

        let submodel_path = ModelPath::new("test_submodel");
        let submodel_path_span = helper::test_ir_span(0, 10);
        let submodel_id = Identifier::new("submodel");

        let submodel_model = Model::new(
            HashMap::new(),
            HashMap::new(),
            oneil_ir::parameter::ParameterCollection::new(HashMap::new()),
            HashMap::new(),
        );

        let context = TestContext::new()
            .with_submodel_context([(submodel_id, (submodel_path.clone(), submodel_path_span))])
            .with_model_context([(submodel_path.clone(), submodel_model)]);

        // resolve the variable
        let result = resolve_variable(&variable, &builtin_ref, &context);

        // check the result
        match result {
            Err(VariableResolutionError::UndefinedParameter {
                model_path,
                parameter,
                reference_span,
            }) => {
                assert_eq!(model_path, Some(submodel_path));

                assert_eq!(reference_span, inner_var_span);
                assert_eq!(parameter, Identifier::new("undefined_param"));
            }
            result => panic!("Expected undefined parameter in submodel error, got {result:?}"),
        }
    }

    #[test]
    fn test_resolve_undefined_submodel_in_submodel() {
        // create a nested accessor variable for undefined submodel in submodel
        let inner_var = helper::create_identifier_variable("parameter");
        let undefined_submodel = helper::create_accessor_variable("undefined_submodel", inner_var);
        let undefined_submodel_span =
            helper::get_accessor_parent_identifier_span(&undefined_submodel);
        let variable = helper::create_accessor_variable("submodel", undefined_submodel);

        // create the context and builtin ref
        let builtin_ref = TestBuiltinRef::new();

        // create submodel info
        let submodel_path = ModelPath::new("test_submodel");
        let submodel_path_span = helper::test_ir_span(0, 10);
        let submodel_id = Identifier::new("submodel");

        let submodel_model = Model::new(
            HashMap::new(),
            HashMap::new(),
            oneil_ir::parameter::ParameterCollection::new(HashMap::new()),
            HashMap::new(),
        );

        let context = TestContext::new()
            .with_submodel_context([(submodel_id, (submodel_path.clone(), submodel_path_span))])
            .with_model_context([(submodel_path.clone(), submodel_model)]);

        // resolve the variable
        let result = resolve_variable(&variable, &builtin_ref, &context);

        // check the result
        match result {
            Err(VariableResolutionError::UndefinedSubmodel {
                model_path,
                submodel,
                reference_span,
            }) => {
                assert_eq!(model_path, Some(submodel_path));

                assert_eq!(reference_span, undefined_submodel_span);
                assert_eq!(submodel, Identifier::new("undefined_submodel"));
            }
            result => panic!("Expected undefined submodel in submodel error, got {result:?}"),
        }
    }

    #[test]
    fn test_resolve_modelwith_error() {
        // create an accessor variable for model with error
        let inner_var = helper::create_identifier_variable("parameter");
        let variable = helper::create_accessor_variable("submodel", inner_var);
        let variable_span = helper::get_accessor_parent_identifier_span(&variable);

        // create context and builtin ref
        let builtin_ref = TestBuiltinRef::new();

        // create submodel info
        let submodel_path = ModelPath::new("test_submodel");
        let submodel_path_span = helper::test_ir_span(0, 10);
        let submodel_id = Identifier::new("submodel");

        let context = TestContext::new()
            .with_submodel_context([(submodel_id, (submodel_path.clone(), submodel_path_span))])
            .with_model_errors([submodel_path.clone()]);

        // resolve the variable
        let result = resolve_variable(&variable, &builtin_ref, &context);

        // check the result
        match result {
            Err(VariableResolutionError::ModelHasError {
                path,
                reference_span,
            }) => {
                assert_eq!(path, submodel_path);
                assert_eq!(reference_span, variable_span);
            }
            result => panic!("Expected model has error, got {result:?}"),
        }
    }

    #[test]
    fn test_parameter_takes_precedence_over_builtin() {
        // create a variable that conflicts between builtin and parameter
        let variable = helper::create_identifier_variable("conflict");
        let builtin_ref = TestBuiltinRef::new().with_builtin_variables(["conflict"]);

        let conflict_id = Identifier::new("conflict");
        let param = Parameter::new(
            HashSet::new(),
            WithSpan::new(Identifier::new("conflict"), helper::test_ir_span(0, 10)),
            oneil_ir::parameter::ParameterValue::simple(
                WithSpan::new(
                    oneil_ir::expr::Expr::literal(oneil_ir::expr::Literal::number(42.0)),
                    helper::test_ir_span(0, 1),
                ),
                None,
            ),
            oneil_ir::parameter::Limits::default(),
            false,
            oneil_ir::debug_info::TraceLevel::None,
        );

        let context = TestContext::new().with_parameter_context([(conflict_id, param)]);

        // resolve the variable
        let result = resolve_variable(&variable, &builtin_ref, &context);

        // check the result - parameter should take precedence
        match result {
            Ok(result) => match result.value() {
                oneil_ir::expr::Expr::Variable(oneil_ir::expr::Variable::Parameter(ident)) => {
                    assert_eq!(ident, &Identifier::new("conflict"));
                }
                result => panic!(
                    "Expected parameter variable expression (should take precedence), got {result:?}"
                ),
            },
            Err(error) => panic!(
                "Expected parameter variable expression (should take precedence), got {error:?}"
            ),
        }
    }

    #[test]
    fn test_empty_local_variables() {
        // create a parameter variable with empty local variables
        let variable = helper::create_identifier_variable("parameter");

        // create the context and builtin ref
        let builtin_ref = TestBuiltinRef::new();

        // create parameter
        let parameter_id = Identifier::new("parameter");
        let param = Parameter::new(
            HashSet::new(),
            WithSpan::new(Identifier::new("parameter"), helper::test_ir_span(0, 10)),
            oneil_ir::parameter::ParameterValue::simple(
                WithSpan::new(
                    oneil_ir::expr::Expr::literal(oneil_ir::expr::Literal::number(42.0)),
                    helper::test_ir_span(0, 1),
                ),
                None,
            ),
            oneil_ir::parameter::Limits::default(),
            false,
            oneil_ir::debug_info::TraceLevel::None,
        );

        let context = TestContext::new().with_parameter_context([(parameter_id, param)]);

        // resolve the variable
        let result = resolve_variable(&variable, &builtin_ref, &context);

        // check the result
        assert!(result.is_ok());
        let result = result.expect("result should be ok");
        match result.value() {
            oneil_ir::expr::Expr::Variable(oneil_ir::expr::Variable::Parameter(ident)) => {
                assert_eq!(ident, &Identifier::new("parameter"));
            }
            error => panic!("Expected parameter variable expression, got {error:?}"),
        }
    }
}
