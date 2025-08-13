//! Test constructs for the AST
//!
//! This module contains structures for representing test declarations
//! in Oneil programs.

use std::ops::Deref;

use crate::{
    debug_info::TraceLevelNode, expression::ExprNode, naming::IdentifierNode, node::Node,
    note::NoteNode,
};

/// A test declaration in an Oneil program
///
/// Tests are used to verify the behavior of models and expressions
/// with specific inputs and expected outputs.
#[derive(Debug, Clone, PartialEq)]
pub struct Test {
    trace_level: Option<TraceLevelNode>,
    inputs: Option<TestInputsNode>,
    expr: ExprNode,
    note: Option<NoteNode>,
}

/// A node containing a test definition
pub type TestNode = Node<Test>;

impl Test {
    /// Creates a new test with the given components
    pub fn new(
        trace_level: Option<TraceLevelNode>,
        inputs: Option<TestInputsNode>,
        expr: ExprNode,
        note: Option<NoteNode>,
    ) -> Self {
        Self {
            trace_level,
            inputs,
            expr,
            note,
        }
    }

    /// Returns the trace level for this test, if any
    pub fn trace_level(&self) -> Option<&TraceLevelNode> {
        self.trace_level.as_ref()
    }

    /// Returns the test inputs, if any
    pub fn inputs(&self) -> Option<&TestInputsNode> {
        self.inputs.as_ref()
    }

    /// Returns the test expression
    pub fn expr(&self) -> &ExprNode {
        &self.expr
    }

    /// Returns the note for this test, if any
    pub fn note(&self) -> Option<&NoteNode> {
        self.note.as_ref()
    }
}

/// A collection of test input identifiers
///
/// Test inputs specify which variables should be treated as inputs
/// when evaluating the test expression.
#[derive(Debug, Clone, PartialEq)]
pub struct TestInputs {
    inputs: Vec<IdentifierNode>,
}

/// A node containing test inputs
pub type TestInputsNode = Node<TestInputs>;

impl TestInputs {
    /// Creates a new test inputs collection
    pub fn new(inputs: Vec<IdentifierNode>) -> Self {
        Self { inputs }
    }
}

impl Deref for TestInputs {
    type Target = Vec<IdentifierNode>;

    fn deref(&self) -> &Self::Target {
        &self.inputs
    }
}
