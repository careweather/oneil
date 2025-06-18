use super::expression::Expr;
use super::parameter::TraceLevel;

#[derive(Debug, Clone, PartialEq)]
pub struct Test {
    pub trace_level: TraceLevel,
    pub inputs: Vec<String>,
    pub expr: Expr,
}
