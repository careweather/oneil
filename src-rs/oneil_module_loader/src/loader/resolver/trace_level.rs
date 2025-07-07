use oneil_ast as ast;

/// Resolves an AST trace level into a module trace level.
///
/// This function converts between the AST representation of trace levels
/// (used during parsing) and the module representation (used during
/// module loading and execution).
///
/// # Arguments
///
/// * `trace_level` - The AST trace level to convert
///
/// # Returns
///
/// The corresponding module trace level
pub fn resolve_trace_level(
    trace_level: &ast::parameter::TraceLevel,
) -> oneil_module::debug_info::TraceLevel {
    match trace_level {
        ast::parameter::TraceLevel::None => oneil_module::debug_info::TraceLevel::None,
        ast::parameter::TraceLevel::Trace => oneil_module::debug_info::TraceLevel::Trace,
        ast::parameter::TraceLevel::Debug => oneil_module::debug_info::TraceLevel::Debug,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_none_trace_level() {
        {
            let ast_level = ast::parameter::TraceLevel::None;
            let expected_module_level = oneil_module::debug_info::TraceLevel::None;

            let result = resolve_trace_level(&ast_level);
            assert_eq!(
                result, expected_module_level,
                "Expected {:?} to resolve to {:?}, but got {:?}",
                ast_level, expected_module_level, result
            );
        };
    }

    #[test]
    fn test_resolve_trace_trace_level() {
        {
            let ast_level = ast::parameter::TraceLevel::Trace;
            let expected_module_level = oneil_module::debug_info::TraceLevel::Trace;

            let result = resolve_trace_level(&ast_level);
            assert_eq!(
                result, expected_module_level,
                "Expected {:?} to resolve to {:?}, but got {:?}",
                ast_level, expected_module_level, result
            );
        };
    }

    #[test]
    fn test_resolve_debug_trace_level() {
        {
            let ast_level = ast::parameter::TraceLevel::Debug;
            let expected_module_level = oneil_module::debug_info::TraceLevel::Debug;

            let result = resolve_trace_level(&ast_level);
            assert_eq!(
                result, expected_module_level,
                "Expected {:?} to resolve to {:?}, but got {:?}",
                ast_level, expected_module_level, result
            );
        };
    }
}
