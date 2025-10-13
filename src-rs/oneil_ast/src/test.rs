//! Test constructs for the AST

use crate::{debug_info::TraceLevelNode, expression::ExprNode, node::Node, note::NoteNode};

/// A test declaration in an Oneil program
#[derive(Debug, Clone, PartialEq)]
pub struct Test {
    trace_level: Option<TraceLevelNode>,
    expr: ExprNode,
    note: Option<NoteNode>,
}

/// A node containing a test definition
pub type TestNode = Node<Test>;

impl Test {
    /// Creates a new test with the given components
    #[must_use]
    pub const fn new(
        trace_level: Option<TraceLevelNode>,
        expr: ExprNode,
        note: Option<NoteNode>,
    ) -> Self {
        Self {
            trace_level,
            expr,
            note,
        }
    }

    /// Returns the trace level for this test, if any
    #[must_use]
    pub const fn trace_level(&self) -> Option<&TraceLevelNode> {
        self.trace_level.as_ref()
    }

    /// Returns the test expression
    #[must_use]
    pub const fn expr(&self) -> &ExprNode {
        &self.expr
    }

    /// Returns the note for this test, if any
    #[must_use]
    pub const fn note(&self) -> Option<&NoteNode> {
        self.note.as_ref()
    }
}
