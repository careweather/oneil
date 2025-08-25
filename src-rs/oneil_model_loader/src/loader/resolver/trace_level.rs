use oneil_ast::{self as ast, node::Node};

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
    trace_level: Option<&ast::debug_info::TraceLevelNode>,
) -> oneil_ir::debug_info::TraceLevel {
    match trace_level.map(Node::node_value) {
        Some(ast::debug_info::TraceLevel::Trace) => oneil_ir::debug_info::TraceLevel::Trace,
        Some(ast::debug_info::TraceLevel::Debug) => oneil_ir::debug_info::TraceLevel::Debug,
        None => oneil_ir::debug_info::TraceLevel::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_none_trace_level() {
        {
            let expected_model_level = oneil_ir::debug_info::TraceLevel::None;

            let result = resolve_trace_level(None);
            assert_eq!(
                result, expected_model_level,
                "Expected None to resolve to {expected_model_level:?}, but got {result:?}"
            );
        };
    }

    #[test]
    fn test_resolve_trace_trace_level() {
        {
            let ast_level = ast::debug_info::TraceLevel::Trace;
            let expected_model_level = oneil_ir::debug_info::TraceLevel::Trace;

            // Create a trace level node
            let trace_level_node = ast::node::Node::new(&ast::Span::new(0, 0, 0), ast_level);
            let result = resolve_trace_level(Some(&trace_level_node));
            assert_eq!(
                result,
                expected_model_level,
                "Expected {:?} to resolve to {:?}, but got {:?}",
                ast::debug_info::TraceLevel::Trace,
                expected_model_level,
                result
            );
        };
    }

    #[test]
    fn test_resolve_debug_trace_level() {
        {
            let ast_level = ast::debug_info::TraceLevel::Debug;
            let expected_model_level = oneil_ir::debug_info::TraceLevel::Debug;

            // Create a trace level node
            let trace_level_node = ast::node::Node::new(&ast::Span::new(0, 0, 0), ast_level);
            let result = resolve_trace_level(Some(&trace_level_node));
            assert_eq!(
                result,
                expected_model_level,
                "Expected {:?} to resolve to {:?}, but got {:?}",
                ast::debug_info::TraceLevel::Debug,
                expected_model_level,
                result
            );
        };
    }
}
