//! Variable resolution for the Oneil model loader

use oneil_ast as ast;
use oneil_ir as ir;

use crate::{
    ExternalResolutionContext, ResolutionContext,
    error::VariableResolutionError,
    context::{ParameterResult, ReferencePathResult},
};

/// Resolves a variable expression to its corresponding model expression.
pub fn resolve_variable<E>(
    variable: &ast::VariableNode,
    resolution_context: &ResolutionContext<'_, E>,
) -> Result<ir::Expr, VariableResolutionError>
where
    E: ExternalResolutionContext,
{
    match &**variable {
        ast::Variable::Identifier(identifier) => {
            let var_identifier = ir::ParameterName::new(identifier.as_str().to_string());
            let variable_span = variable.span();
            let identifier_span = identifier.span();

            match resolution_context.lookup_parameter_in_active_model(&var_identifier) {
                ParameterResult::Found(_parameter) => {
                    let expr = ir::Expr::parameter_variable(
                        variable_span,
                        identifier_span,
                        var_identifier,
                    );
                    Ok(expr)
                }
                ParameterResult::HasError => Err(VariableResolutionError::parameter_has_error(
                    var_identifier,
                    identifier_span,
                )),
                ParameterResult::NotFound => {
                    let builtin_identifier = ir::Identifier::new(identifier.as_str().to_string());
                    if resolution_context.has_builtin_value(&builtin_identifier) {
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

            let (model, reference_path) =
                match resolution_context.lookup_reference_path_in_active_model(&reference_name) {
                    ReferencePathResult::ReferenceHasResolutionError => {
                        return Err(VariableResolutionError::reference_resolution_failed(
                            reference_name,
                            reference_name_span,
                        ));
                    }
                    ReferencePathResult::ReferenceNotFound => {
                        return Err(VariableResolutionError::undefined_reference(
                            reference_name,
                            reference_name_span,
                        ));
                    }
                    ReferencePathResult::ModelHasResolutionError(reference_path) => {
                        return Err(VariableResolutionError::model_has_error(
                            reference_path.clone(),
                            reference_name_span,
                        ));
                    }
                    ReferencePathResult::ModelNotFound(_reference_path) => {
                        unreachable!("reference should have been visited already")
                    }
                    ReferencePathResult::Found(model, reference_path) => (model, reference_path),
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
        external_context::TestExternalContext, resolution_context::ResolutionContextBuilder,
        test_ast, test_ir,
    };

    use super::*;

    use crate::error::ModelImportResolutionError;
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
        // build the variable
        let variable = test_ast::identifier_variable_node("pi");

        // build the context
        let active_path = ir::ModelPath::new("main");
        let mut external = TestExternalContext::new().with_builtin_variables(["pi"]);
        let resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // run the variable resolution
        let result = resolve_variable(&variable, &resolution_context);

        // check the result
        let var = result.expect("variable should be resolved");
        assert_var_is_builtin!(var, "pi");
    }

    #[test]
    fn resolve_parameter_variable() {
        // build the variable
        let variable = test_ast::identifier_variable_node("temperature");

        // build the context
        let active_path = ir::ModelPath::new("main");
        let params = [test_ir::ParameterBuilder::new()
            .with_name_str("temperature")
            .with_simple_number_value(42.0)
            .build()];
        let mut external = TestExternalContext::new();
        let resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_parameters(params)
            .with_external_context(&mut external)
            .build();

        // run the variable resolution
        let result = resolve_variable(&variable, &resolution_context);

        // check the result
        let var = result.expect("variable should be resolved");
        assert_var_is_parameter!(var, "temperature");
    }

    #[test]
    fn resolve_undefined_parameter() {
        // build the variable
        let variable = test_ast::identifier_variable_node("undefined_param");

        // build the context
        let active_path = ir::ModelPath::new("main");
        let mut external = TestExternalContext::new();
        let resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // run the variable resolution
        let result = resolve_variable(&variable, &resolution_context);

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
        // build the variable
        let variable = test_ast::identifier_variable_node("error_param");

        // build the context
        let active_path = ir::ModelPath::new("main");
        let mut external = TestExternalContext::new();
        let parameter_errors = [ir::ParameterName::new("error_param".to_string())];
        let resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_parameter_errors(parameter_errors)
            .with_external_context(&mut external)
            .build();

        // run the variable resolution
        let result = resolve_variable(&variable, &resolution_context);

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
        // build the variable
        let variable = test_ast::model_parameter_variable_node("undefined_reference", "parameter");

        // build the context
        let active_path = ir::ModelPath::new("main");
        let mut external = TestExternalContext::new();
        let resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // run the variable resolution
        let result = resolve_variable(&variable, &resolution_context);

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
        // build the variable
        let variable = test_ast::model_parameter_variable_node("error_reference", "parameter");

        // build the context
        let active_path = ir::ModelPath::new("main");
        let mut external = TestExternalContext::new();
        let ref_name = ir::ReferenceName::new("error_reference".to_string());
        let ref_errors = [(
            ref_name.clone(),
            ModelImportResolutionError::model_has_error(
                ir::ModelPath::new("dummy"),
                crate::test::unimportant_span(),
            ),
        )];
        let resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_reference_errors(ref_errors)
            .with_external_context(&mut external)
            .build();

        // run the variable resolution
        let result = resolve_variable(&variable, &resolution_context);

        // check the result
        let Err(VariableResolutionError::ReferenceResolutionFailed {
            identifier,
            reference_span: _,
        }) = result
        else {
            panic!("expected reference resolution failed, got {result:?}");
        };

        assert_eq!(identifier, ref_name);
    }

    #[test]
    fn resolve_nested_accessor() {
        // build the variable
        let variable = test_ast::model_parameter_variable_node("reference", "parameter");

        // build the context
        let active_path = ir::ModelPath::new("main");
        let mut external = TestExternalContext::new();
        let reference_path = ir::ModelPath::new("test_reference");
        let reference_name = test_ir::reference_name("reference");
        let reference_model = test_ir::ModelBuilder::new()
            .with_literal_number_parameter("parameter", 42.0)
            .build();
        let references = [(reference_name, reference_path, reference_model)];
        let resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_references(references)
            .with_external_context(&mut external)
            .build();

        // run the variable resolution
        let result = resolve_variable(&variable, &resolution_context);

        // check the result
        let var = result.expect("variable should be resolved");
        assert_var_is_external!(var, "test_reference", "parameter");
    }

    #[test]
    fn resolve_undefined_parameter_in_reference() {
        // build the variable
        let variable = test_ast::model_parameter_variable_node("reference", "undefined_param");

        // build the context
        let active_path = ir::ModelPath::new("main");
        let mut external = TestExternalContext::new();
        let reference_path = ir::ModelPath::new("test_reference");
        let reference_name = test_ir::reference_name("reference");
        let reference_model = test_ir::empty_model();
        let references = [(reference_name, reference_path.clone(), reference_model)];
        let resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_references(references)
            .with_external_context(&mut external)
            .build();

        // run the variable resolution
        let result = resolve_variable(&variable, &resolution_context);

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
        // build the variable
        let variable = test_ast::model_parameter_variable_node("reference", "parameter");

        // build the context
        let active_path = ir::ModelPath::new("main");
        let mut external = TestExternalContext::new();
        let reference_path = ir::ModelPath::new("test_reference");
        let reference_name = test_ir::reference_name("reference");
        let reference_model = test_ir::empty_model();
        let references = [(reference_name, reference_path.clone(), reference_model)];
        let model_errors = [reference_path.clone()];
        let resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_references(references)
            .with_model_errors(model_errors)
            .with_external_context(&mut external)
            .build();

        // run the variable resolution
        let result = resolve_variable(&variable, &resolution_context);

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
        // build the variable
        let variable = test_ast::identifier_variable_node("conflict");

        // build the context
        let active_path = ir::ModelPath::new("main");
        let mut external = TestExternalContext::new().with_builtin_variables(["conflict"]);
        let params = [test_ir::ParameterBuilder::new()
            .with_name_str("conflict")
            .with_simple_number_value(42.0)
            .build()];
        let resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_parameters(params)
            .with_external_context(&mut external)
            .build();

        // run the variable resolution
        let result = resolve_variable(&variable, &resolution_context);

        // check the result
        let var = result.expect("variable should be resolved");
        assert_var_is_parameter!(var, "conflict");
    }
}
