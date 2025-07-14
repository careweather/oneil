use crate::{Expr, debug_info::TraceLevelNode};

#[derive(Debug, Clone, PartialEq)]
pub struct Test {
    pub trace_level: TraceLevelNode,
    pub inputs: Vec<String>,
    pub expr: Expr,
}
