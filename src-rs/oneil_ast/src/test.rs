use crate::{debug_info::TraceLevelNode, expression::ExprNode, naming::IdentifierNode, node::Node};

#[derive(Debug, Clone, PartialEq)]
pub struct Test {
    trace_level: Option<TraceLevelNode>,
    inputs: Option<TestInputsNode>,
    expr: ExprNode,
}

pub type TestNode = Node<Test>;

impl Test {
    pub fn new(
        trace_level: Option<TraceLevelNode>,
        inputs: Option<TestInputsNode>,
        expr: ExprNode,
    ) -> Self {
        Self {
            trace_level,
            inputs,
            expr,
        }
    }

    pub fn trace_level(&self) -> Option<&TraceLevelNode> {
        self.trace_level.as_ref()
    }

    pub fn inputs(&self) -> Option<&TestInputsNode> {
        self.inputs.as_ref()
    }

    pub fn expr(&self) -> &ExprNode {
        &self.expr
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestInputs {
    inputs: Vec<IdentifierNode>,
}

pub type TestInputsNode = Node<TestInputs>;

impl TestInputs {
    pub fn new(inputs: Vec<IdentifierNode>) -> Self {
        Self { inputs }
    }
}
