use oneil_ast as ast;

/// Resolves an AST trace level into a model trace level.
///
/// This function converts between the AST representation of trace levels
/// (used during parsing) and the model representation (used during
/// model loading and execution).
///
/// # Arguments
///
/// * `trace_level` - The AST trace level to convert
///
/// # Returns
///
/// The corresponding model trace level
pub fn resolve_trace_level(
    trace_level: &ast::parameter::TraceLevel,
) -> oneil_ir::debug_info::TraceLevel {
    match trace_level {
        ast::parameter::TraceLevel::None => oneil_ir::debug_info::TraceLevel::None,
        ast::parameter::TraceLevel::Trace => oneil_ir::debug_info::TraceLevel::Trace,
        ast::parameter::TraceLevel::Debug => oneil_ir::debug_info::TraceLevel::Debug,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_none_trace_level() {
        {
            let ast_level = ast::parameter::TraceLevel::None;
            let expected_model_level = oneil_ir::debug_info::TraceLevel::None;

            let result = resolve_trace_level(&ast_level);
            assert_eq!(
                result, expected_model_level,
                "Expected {:?} to resolve to {:?}, but got {:?}",
                ast_level, expected_model_level, result
            );
        };
    }

    #[test]
    fn test_resolve_trace_trace_level() {
        {
            let ast_level = ast::parameter::TraceLevel::Trace;
            let expected_model_level = oneil_ir::debug_info::TraceLevel::Trace;

            let result = resolve_trace_level(&ast_level);
            assert_eq!(
                result, expected_model_level,
                "Expected {:?} to resolve to {:?}, but got {:?}",
                ast_level, expected_model_level, result
            );
        };
    }

    #[test]
    fn test_resolve_debug_trace_level() {
        {
            let ast_level = ast::parameter::TraceLevel::Debug;
            let expected_model_level = oneil_ir::debug_info::TraceLevel::Debug;

            let result = resolve_trace_level(&ast_level);
            assert_eq!(
                result, expected_model_level,
                "Expected {:?} to resolve to {:?}, but got {:?}",
                ast_level, expected_model_level, result
            );
        };
    }
}
