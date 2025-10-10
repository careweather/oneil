use std::ops::Deref;

use oneil_ast as ast;
use oneil_ir as ir;

/// Resolves an AST trace level into a model trace level.
pub fn resolve_trace_level(trace_level: Option<&ast::TraceLevelNode>) -> ir::TraceLevel {
    match trace_level.map(ast::Node::deref) {
        Some(ast::TraceLevel::Trace) => ir::TraceLevel::Trace,
        Some(ast::TraceLevel::Debug) => ir::TraceLevel::Debug,
        None => ir::TraceLevel::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_none_trace_level() {
        let expected_model_level = ir::TraceLevel::None;

        let result = resolve_trace_level(None);
        assert_eq!(
            result, expected_model_level,
            "Expected None to resolve to {expected_model_level:?}, but got {result:?}"
        );
    }

    #[test]
    fn resolve_trace_trace_level() {
        let expected_model_level = ir::TraceLevel::Trace;

        // Create a trace level node
        let trace_level_node = crate::test::construct::test_ast::trace_level_trace_node();
        let result = resolve_trace_level(Some(&trace_level_node));
        assert_eq!(
            result,
            expected_model_level,
            "Expected {:?} to resolve to {:?}, but got {:?}",
            ast::TraceLevel::Trace,
            expected_model_level,
            result
        );
    }

    #[test]
    fn resolve_debug_trace_level() {
        {
            let expected_model_level = ir::TraceLevel::Debug;

            // Create a trace level node
            let trace_level_node = crate::test::construct::test_ast::trace_level_debug_node();
            let result = resolve_trace_level(Some(&trace_level_node));
            assert_eq!(
                result,
                expected_model_level,
                "Expected {:?} to resolve to {:?}, but got {:?}",
                ast::TraceLevel::Debug,
                expected_model_level,
                result
            );
        };
    }
}
