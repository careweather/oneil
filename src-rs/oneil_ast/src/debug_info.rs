//! Debug information and trace levels for the AST
//!
//! This module contains structures for handling debug information and trace levels
//! that can be attached to AST nodes for debugging and tracing purposes.

use crate::node::Node;

/// Represents different levels of tracing for debug information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceLevel {
    /// Basic tracing level for general debugging
    Trace,
    /// Detailed debugging level for in-depth analysis
    Debug,
}

/// A node containing trace level information
pub type TraceLevelNode = Node<TraceLevel>;

impl TraceLevel {
    /// Creates a new trace-level debug marker
    #[must_use]
    pub const fn trace() -> Self {
        Self::Trace
    }

    /// Creates a new debug-level debug marker
    #[must_use]
    pub const fn debug() -> Self {
        Self::Debug
    }
}
