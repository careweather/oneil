//! Variable resolution for the Oneil model loader

use oneil_ast as ast;
use oneil_ir as ir;

use crate::{
    BuiltinRef,
    error::VariableResolutionError,
    util::context::{
        ParameterContext, ParameterContextResult, ReferenceContext, ReferenceContextResult,
    },
};

/// Resolves a variable expression to its corresponding model expression.
pub fn resolve_variable(
    variable: &ast::VariableNode,
    builtin_ref: &impl BuiltinRef,
    reference_context: &ReferenceContext<'_, '_>,
    parameter_context: &ParameterContext<'_>,
) -> Result<ir::Expr, VariableResolutionError> {
    match &**variable {
        ast::Variable::Identifier(identifier) => {
            let var_identifier = ir::ParameterName::new(identifier.as_str().to_string());
            let variable_span = variable.span();
            let identifier_span = identifier.span();

            match parameter_context.lookup_parameter(&var_identifier) {
                ParameterContextResult::Found(_parameter) => {
                    let expr = ir::Expr::parameter_variable(
                        variable_span,
                        identifier_span,
                        var_identifier,
                    );
                    Ok(expr)
                }
                ParameterContextResult::HasError => Err(
                    VariableResolutionError::parameter_has_error(var_identifier, identifier_span),
                ),
                ParameterContextResult::NotFound => {
                    let builtin_identifier = ir::Identifier::new(identifier.as_str().to_string());
                    if builtin_ref.has_builtin_value(&builtin_identifier) {
                        let expr = ir::Expr::builtin_variable(
                            variable_span,
                            identifier_span,
                            builtin_identifier,
                        );
                        Ok(expr)
                    } else {
                        Err(VariableResolutionError::undefined_parameter(
                            var_identifier,
                            identifier_span,
                        ))
                    }
                }
            }
        }
        ast::Variable::ModelParameter {
            reference_model,
            parameter,
        } => {
            let reference_name = ir::ReferenceName::new(reference_model.as_str().to_string());
            let reference_name_span = reference_model.span();
            let variable_span = variable.span();

            let (model, reference_path) = match reference_context.lookup_reference(&reference_name)
            {
                ReferenceContextResult::ReferenceHasResolutionError => {
                    return Err(VariableResolutionError::reference_resolution_failed(
                        reference_name,
                        reference_name_span,
                    ));
                }
                ReferenceContextResult::ReferenceNotFound => {
                    return Err(VariableResolutionError::undefined_reference(
                        reference_name,
                        reference_name_span,
                    ));
                }
                ReferenceContextResult::ModelHasResolutionError(reference_path) => {
                    return Err(VariableResolutionError::model_has_error(
                        reference_path.clone(),
                        reference_name_span,
                    ));
                }
                ReferenceContextResult::ModelNotFound(_reference_path) => {
                    unreachable!("reference should have been visited already")
                }
                ReferenceContextResult::Found(model, reference_path) => (model, reference_path),
            };

            // ensure that the parameter is defined in the reference model
            let var_identifier = ir::ParameterName::new(parameter.as_str().to_string());
            let var_identifier_span = parameter.span();
            if model.get_parameter(&var_identifier).is_none() {
                return Err(VariableResolutionError::undefined_parameter_in_reference(
                    reference_path.clone(),
                    var_identifier,
                    var_identifier_span,
                ));
            }

            let expr = ir::Expr::external_variable(
                variable_span,
                reference_path.clone(),
                reference_name,
                reference_name_span,
                var_identifier,
                var_identifier_span,
            );
            Ok(expr)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test::{
        TestBuiltinRef,
        construct::{ParameterContextBuilder, ReferenceContextBuilder, test_ast, test_ir},
    };

    use super::*;

    use oneil_ir as ir;

    macro_rules! assert_var_is_builtin {
        ($variable:expr, $expected_ident:expr $(,)?) => {
            let variable: ir::Expr = $variable;
            let expected_ident: &str = $expected_ident;

            let ir::Expr::Variable {
                span: _,
                variable:
                    ir::Variable::Builtin {
                        ident: actual_ident,
                        ..
                    },
            } = variable
            else {
                panic!("expected builtin variable, got {variable:?}");
            };

            assert_eq!(
                actual_ident.as_str(),
                expected_ident,
                "actual ident does not match expected ident"
            );
        };
    }

    macro_rules! assert_var_is_parameter {
        ($variable:expr, $expected_ident:expr $(,)?) => {
            let variable: ir::Expr = $variable;
            let expected_ident: &str = $expected_ident;

            let ir::Expr::Variable {
                span: _,
                variable:
                    ir::Variable::Parameter {
                        parameter_name: actual_ident,
                        ..
                    },
            } = variable
            else {
                panic!("expected parameter variable, got {variable:?}");
            };

            assert_eq!(
                actual_ident.as_str(),
                expected_ident,
                "actual ident does not match expected ident"
            );
        };
    }

    macro_rules! assert_var_is_external {
        ($variable:expr, $expected_model_path:expr, $expected_parameter_name:expr $(,)?) => {
            let variable: ir::Expr = $variable;
            let expected_model_path: ir::ModelPath = ir::ModelPath::new($expected_model_path);
            let expected_parameter_name: &str = $expected_parameter_name;

            let ir::Expr::Variable {
                span: _,
                variable:
                    ir::Variable::External {
                        model_path: actual_model_path,
                        parameter_name: actual_parameter_name,
                        ..
                    },
            } = variable
            else {
                panic!("expected external variable, got {variable:?}");
            };

            assert_eq!(
                actual_model_path, expected_model_path,
                "actual model path does not match expected model path"
            );

            assert_eq!(
                actual_parameter_name.as_str(),
                expected_parameter_name,
                "actual ident does not match expected ident"
            );
        };
    }

    #[test]
    fn resolve_builtin_variable() {
        // create a local variable
        let variable = test_ast::identifier_variable_node("pi");

        // create the context and builtin ref
        let builtin_ref = TestBuiltinRef::new().with_builtin_variables(["pi"]);

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        // resolve the variable
        let result = resolve_variable(
            &variable,
            &builtin_ref,
            &reference_context,
            &parameter_context,
        );

        // check the result
        let var = result.expect("variable should be resolved");
        assert_var_is_builtin!(var, "pi");
    }

    #[test]
    fn resolve_parameter_variable() {
        // create a parameter variable
        let variable = test_ast::identifier_variable_node("temperature");

        // create the context and builtin ref
        let builtin_ref = TestBuiltinRef::new().with_builtin_variables(["temperature"]);

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new().with_parameter_context([
            test_ir::ParameterBuilder::new()
                .with_name_str("temperature")
                .with_simple_number_value(42.0)
                .build(),
        ]);
        let parameter_context = parameter_context_builder.build();

        // resolve the variable
        let result = resolve_variable(
            &variable,
            &builtin_ref,
            &reference_context,
            &parameter_context,
        );

        // check the result
        let var = result.expect("variable should be resolved");
        assert_var_is_parameter!(var, "temperature");
    }

    #[test]
    fn resolve_undefined_parameter() {
        // create a variable for undefined parameter
        let variable = test_ast::identifier_variable_node("undefined_param");

        // create the context and builtin ref
        let builtin_ref = TestBuiltinRef::new();

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        // resolve the variable
        let result = resolve_variable(
            &variable,
            &builtin_ref,
            &reference_context,
            &parameter_context,
        );

        // check the result
        let Err(VariableResolutionError::UndefinedParameter {
            model_path,
            parameter_name,
            reference_span: _,
        }) = result
        else {
            panic!("expected undefined parameter error, got {result:?}");
        };

        assert_eq!(model_path, None);
        assert_eq!(
            parameter_name,
            ir::ParameterName::new("undefined_param".to_string())
        );
    }

    #[test]
    fn resolve_parameter_with_error() {
        // create a variable for parameter with error
        let variable = test_ast::identifier_variable_node("error_param");

        // create the context and builtin ref
        let builtin_ref = TestBuiltinRef::new();

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new()
            .with_parameter_error([ir::ParameterName::new("error_param".to_string())]);
        let parameter_context = parameter_context_builder.build();

        // resolve the variable
        let result = resolve_variable(
            &variable,
            &builtin_ref,
            &reference_context,
            &parameter_context,
        );

        // check the result
        let Err(VariableResolutionError::ParameterHasError {
            parameter_name,
            reference_span: _,
        }) = result
        else {
            panic!("expected parameter has error, got {result:?}");
        };

        assert_eq!(
            parameter_name,
            ir::ParameterName::new("error_param".to_string())
        );
    }

    #[test]
    fn resolve_undefined_reference() {
        // create an accessor variable for undefined reference
        let variable = test_ast::model_parameter_variable_node("undefined_reference", "parameter");

        // create the context and builtin ref
        let builtin_ref = TestBuiltinRef::new();

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        // resolve the variable
        let result = resolve_variable(
            &variable,
            &builtin_ref,
            &reference_context,
            &parameter_context,
        );

        // check the result
        let Err(VariableResolutionError::UndefinedReference {
            reference,
            reference_span: _,
        }) = result
        else {
            panic!("expected undefined reference error, got {result:?}");
        };

        assert_eq!(
            reference,
            ir::ReferenceName::new("undefined_reference".to_string())
        );
    }

    #[test]
    fn resolve_reference_with_error() {
        // create an accessor variable for reference with error
        let variable = test_ast::model_parameter_variable_node("error_reference", "parameter");

        // create the context and builtin ref
        let builtin_ref = TestBuiltinRef::new();

        let reference_context_builder = ReferenceContextBuilder::new()
            .with_reference_errors([ir::ReferenceName::new("error_reference".to_string())]);
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        // resolve the variable
        let result = resolve_variable(
            &variable,
            &builtin_ref,
            &reference_context,
            &parameter_context,
        );

        // check the result
        let Err(VariableResolutionError::ReferenceResolutionFailed {
            identifier,
            reference_span: _,
        }) = result
        else {
            panic!("expected reference resolution failed, got {result:?}");
        };

        assert_eq!(
            identifier,
            ir::ReferenceName::new("error_reference".to_string())
        );
    }

    #[test]
    fn resolve_nested_accessor() {
        // create a model parameter variable: parameter.reference
        let variable = test_ast::model_parameter_variable_node("reference", "parameter");

        // create the context and builtin ref
        let builtin_ref = TestBuiltinRef::new();

        let reference_path = ir::ModelPath::new("test_reference");
        let reference_name = test_ir::reference_name("reference");
        let reference_model = test_ir::ModelBuilder::new()
            .with_literal_number_parameter("parameter", 42.0)
            .build();

        let reference_context_builder = ReferenceContextBuilder::new().with_reference_context([(
            reference_name,
            reference_path,
            reference_model,
        )]);
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        // resolve the variable
        let result = resolve_variable(
            &variable,
            &builtin_ref,
            &reference_context,
            &parameter_context,
        );

        // check the result
        let var = result.expect("variable should be resolved");
        assert_var_is_external!(var, "test_reference", "parameter");
    }

    #[test]
    fn resolve_undefined_parameter_in_reference() {
        // create an accessor variable for undefined parameter in reference
        // undefined_param.reference
        let variable = test_ast::model_parameter_variable_node("reference", "undefined_param");

        // create the context and builtin ref
        let builtin_ref = TestBuiltinRef::new();

        let reference_path = ir::ModelPath::new("test_reference");
        let reference_name = test_ir::reference_name("reference");
        let reference_model = test_ir::empty_model();

        let reference_context_builder = ReferenceContextBuilder::new().with_reference_context([(
            reference_name,
            reference_path.clone(),
            reference_model,
        )]);
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        // resolve the variable
        let result = resolve_variable(
            &variable,
            &builtin_ref,
            &reference_context,
            &parameter_context,
        );

        // check the result
        let Err(VariableResolutionError::UndefinedParameter {
            model_path,
            parameter_name,
            reference_span: _,
        }) = result
        else {
            panic!("expected undefined parameter error, got {result:?}");
        };

        assert_eq!(model_path, Some(reference_path));
        assert_eq!(
            parameter_name,
            ir::ParameterName::new("undefined_param".to_string())
        );
    }

    #[test]
    fn resolve_model_with_error() {
        // create an model parameter variable for model with error
        // parameter.reference
        let variable = test_ast::model_parameter_variable_node("reference", "parameter");

        // create context and builtin ref
        let builtin_ref = TestBuiltinRef::new();

        // create submodel info
        let reference_path = ir::ModelPath::new("test_reference");
        let reference_name = test_ir::reference_name("reference");
        let reference_model = test_ir::empty_model();

        let reference_context_builder = ReferenceContextBuilder::new()
            .with_reference_context([(reference_name, reference_path.clone(), reference_model)])
            .with_model_error([reference_path.clone()]);
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        // resolve the variable
        let result = resolve_variable(
            &variable,
            &builtin_ref,
            &reference_context,
            &parameter_context,
        );

        // check the result
        let Err(VariableResolutionError::ModelHasError {
            path,
            reference_span: _,
        }) = result
        else {
            panic!("expected model has error, got {result:?}");
        };

        assert_eq!(path, reference_path);
    }

    #[test]
    fn parameter_takes_precedence_over_builtin() {
        // create a variable that conflicts between builtin and parameter
        let variable = test_ast::identifier_variable_node("conflict");

        // create the context and builtin ref
        let builtin_ref = TestBuiltinRef::new().with_builtin_variables(["conflict"]);

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new().with_parameter_context([
            test_ir::ParameterBuilder::new()
                .with_name_str("conflict")
                .with_simple_number_value(42.0)
                .build(),
        ]);
        let parameter_context = parameter_context_builder.build();

        // resolve the variable
        let result = resolve_variable(
            &variable,
            &builtin_ref,
            &reference_context,
            &parameter_context,
        );

        // check the result - parameter should take precedence
        let var = result.expect("variable should be resolved");
        assert_var_is_parameter!(var, "conflict");
    }
}
