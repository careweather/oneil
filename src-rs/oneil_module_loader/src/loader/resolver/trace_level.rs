use oneil_ast as ast;

pub fn resolve_trace_level(
    trace_level: &ast::parameter::TraceLevel,
) -> oneil_module::debug_info::TraceLevel {
    match trace_level {
        ast::parameter::TraceLevel::None => oneil_module::debug_info::TraceLevel::None,
        ast::parameter::TraceLevel::Trace => oneil_module::debug_info::TraceLevel::Trace,
        ast::parameter::TraceLevel::Debug => oneil_module::debug_info::TraceLevel::Debug,
    }
}
