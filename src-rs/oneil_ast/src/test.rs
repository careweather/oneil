use crate::{atom::IdentifierNode, debug_info::TraceLevelNode, expression::ExprNode, node::Node};

#[derive(Debug, Clone, PartialEq)]
pub struct Test {
    pub trace_level: TraceLevelNode,
    pub inputs: Vec<IdentifierNode>,
    pub expr: ExprNode,
}

pub type TestNode = Node<Test>;

impl Test {
    pub fn new(trace_level: TraceLevelNode, inputs: Vec<IdentifierNode>, expr: ExprNode) -> Self {
        Self {
            trace_level,
            inputs,
            expr,
        }
    }
}
