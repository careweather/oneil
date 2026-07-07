//! Variable resolution for the Oneil model loader

use oneil_ast as ast;
use oneil_ir as ir;
use oneil_shared::{
    span::Span,
    symbols::{BuiltinValueName, ParameterName},
};

use crate::{
    ExternalResolutionContext, ResolutionContext, context::ParameterResult,
    error::VariableResolutionError,
};

// Neither bare-name nor reference-name *existence* is checked at
// file-resolution time:
// - Bare names: an unknown identifier is lowered to `Variable::Parameter`.
//   The instance graph walk re-classifies it (parameter / builtin /
//   unresolved) against each instance's binding scope.
// - Reference names: lowered to `Variable::External` unconditionally.
//   Validation checks reference and parameter existence after the full
//   instance graph is built (when all design contributions are known).

/// Resolves a variable expression to its corresponding model expression.
#[expect(clippy::result_large_err)]
pub fn resolve_variable<E>(
    variable: &ast::VariableNode,
    resolution_context: &ResolutionContext<'_, E>,
) -> Result<ir::Expr, VariableResolutionError>
where
    E: ExternalResolutionContext,
{
    match &**variable {
        ast::Variable::Identifier(identifier) => {
            resolve_identifier_variable(variable, identifier, resolution_context)
        }
        ast::Variable::ModelParameter {
            reference_model,
            parameter,
        } => Ok(resolve_model_parameter_variable(
            reference_model,
            parameter,
            variable.span().clone(),
        )),
    }
}

/// Resolves a bare identifier: an active-model parameter or a builtin value.
#[expect(clippy::result_large_err)]
fn resolve_identifier_variable<E>(
    variable: &ast::VariableNode,
    identifier: &ast::IdentifierNode,
    resolution_context: &ResolutionContext<'_, E>,
) -> Result<ir::Expr, VariableResolutionError>
where
    E: ExternalResolutionContext,
{
    let var_identifier = ParameterName::from(identifier.as_str());
    let variable_span = variable.span().clone();
    let identifier_span = identifier.span().clone();

    match resolution_context.lookup_parameter_in_active_model(&var_identifier) {
        ParameterResult::Found(_parameter) => {
            let expr = ir::Expr::parameter_variable(variable_span, identifier_span, var_identifier);
            Ok(expr)
        }
        ParameterResult::HasError => Err(VariableResolutionError::parameter_has_error(
            var_identifier,
            identifier_span,
        )),
        ParameterResult::NotFound => {
            if resolution_context.has_builtin_value(identifier) {
                let builtin_identifier = BuiltinValueName::new(identifier.as_str().to_string());
                let expr =
                    ir::Expr::builtin_variable(variable_span, identifier_span, builtin_identifier);
                Ok(expr)
            } else {
                // Unknown bare name: lower as `Variable::Parameter` with no
                // pinned instance and let the walk re-classify (and emit any
                // diagnostic) against the per-instance binding scope. See the
                // module-level comment.
                let expr =
                    ir::Expr::parameter_variable(variable_span, identifier_span, var_identifier);
                Ok(expr)
            }
        }
    }
}

/// Resolves `parameter.reference_model` (subscript-style: parameter
/// first, reference / submodel second) to `Variable::External`.
///
/// Reference and parameter *existence* are **not** checked here; both are
/// deferred to the post-build validation pass so that designs applied at
/// instance time can contribute references and parameters that are not
/// visible in the static file.
fn resolve_model_parameter_variable(
    reference_model: &ast::ReferenceNameNode,
    parameter: &ast::ParameterNameNode,
    variable_span: Span,
) -> ir::Expr {
    let reference_name = reference_model.clone().take_value();
    let reference_name_span = reference_model.span().clone();
    let var_identifier = parameter.clone().take_value();
    let var_identifier_span = parameter.span().clone();

    ir::Expr::external_variable(
        variable_span,
        reference_name,
        reference_name_span,
        var_identifier,
        var_identifier_span,
    )
}

#[cfg(test)]
mod tests {
    use crate::test::{
        external_context::TestExternalContext, resolution_context::ResolutionContextBuilder,
        test_ast, test_ir, test_model_path,
    };

    use super::*;

    use oneil_ir as ir;

    /// Asserts that an expression is a builtin variable with the expected identifier.
    ///
    /// # Panics
    ///
    /// Panics if the expression is not a builtin variable or the identifier does not match.
    #[track_caller]
    fn assert_var_is_builtin(variable: ir::Expr, expected_ident: &str) {
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
    }

    /// Asserts that an expression is a parameter variable with the expected identifier.
    ///
    /// # Panics
    ///
    /// Panics if the expression is not a parameter variable or the identifier does not match.
    #[track_caller]
    fn assert_var_is_parameter(variable: ir::Expr, expected_ident: &str) {
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
    }

    /// Asserts that an expression is an external variable with the expected names.
    ///
    /// # Panics
    ///
    /// Panics if the expression is not an external variable or the names do not match.
    #[track_caller]
    fn assert_var_is_external(
        variable: ir::Expr,
        expected_reference_name: &str,
        expected_parameter_name: &str,
    ) {
        let ir::Expr::Variable {
            span: _,
            variable:
                ir::Variable::External {
                    reference_name: actual_reference_name,
                    parameter_name: actual_parameter_name,
                    ..
                },
        } = variable
        else {
            panic!("expected external variable, got {variable:?}");
        };

        assert_eq!(
            actual_reference_name.as_str(),
            expected_reference_name,
            "actual reference name does not match expected reference name"
        );

        assert_eq!(
            actual_parameter_name.as_str(),
            expected_parameter_name,
            "actual parameter name does not match expected parameter name"
        );
    }

    #[test]
    fn resolve_builtin_variable() {
        // build the variable
        let variable = test_ast::identifier_variable_node("pi");

        // build the context
        let active_path = test_model_path("main");
        let mut external = TestExternalContext::new().with_builtin_variables(["pi"]);
        let resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // run the variable resolution
        let result = resolve_variable(&variable, &resolution_context);

        // check the result
        let var = result.expect("variable should be resolved");
        assert_var_is_builtin(var, "pi");
    }

    #[test]
    fn resolve_parameter_variable() {
        // build the variable
        let variable = test_ast::identifier_variable_node("temperature");

        // build the context
        let active_path = test_model_path("main");
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
        assert_var_is_parameter(var, "temperature");
    }

    #[test]
    fn resolve_parameter_with_error() {
        // build the variable
        let variable = test_ast::identifier_variable_node("error_param");

        // build the context
        let active_path = test_model_path("main");
        let mut external = TestExternalContext::new();
        let parameter_errors = [ParameterName::from("error_param")];
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

        assert_eq!(parameter_name, ParameterName::from("error_param"));
    }

    #[test]
    fn resolve_nested_accessor() {
        let variable = test_ast::model_parameter_variable_node("reference", "parameter");

        let active_path = test_model_path("main");
        let mut external = TestExternalContext::new();
        let resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        let result = resolve_variable(&variable, &resolution_context);

        let var = result.expect("variable should be resolved");
        assert_var_is_external(var, "reference", "parameter");
    }

    #[test]
    fn parameter_takes_precedence_over_builtin() {
        // build the variable
        let variable = test_ast::identifier_variable_node("conflict");

        // build the context
        let active_path = test_model_path("main");
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
        assert_var_is_parameter(var, "conflict");
    }
}
