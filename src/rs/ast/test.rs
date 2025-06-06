use super::expression::Expr;
use super::parameter::TraceLevel;

#[derive(Debug, Clone, PartialEq)]
pub struct Test {
    trace_level: TraceLevel,
    inputs: Vec<String>,
    expr: Expr,
}
