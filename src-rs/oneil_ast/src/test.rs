use crate::{atom::IdentifierNode, debug_info::TraceLevelNode, expression::ExprNode, node::Node};

#[derive(Debug, Clone, PartialEq)]
pub struct Test {
    trace_level: TraceLevelNode,
    inputs: Vec<IdentifierNode>,
    expr: ExprNode,
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

    pub fn trace_level(&self) -> &TraceLevelNode {
        &self.trace_level
    }

    pub fn inputs(&self) -> &[IdentifierNode] {
        &self.inputs
    }

    pub fn expr(&self) -> &ExprNode {
        &self.expr
    }
}
