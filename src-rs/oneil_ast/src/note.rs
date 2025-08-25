//! Documentation note constructs for the AST
//!
//! This module contains structures for representing documentation notes
//! that can be attached to various AST nodes.

use crate::node::Node;

/// A documentation note in the AST
///
/// Notes can be attached to various AST nodes to provide documentation,
/// explanations, or other comments. They can be either single-line notes
/// starting with `~` or multi-line notes delimited by `~~~`.
#[derive(Debug, Clone, PartialEq)]
pub struct Note(String);

/// A node containing a documentation note
pub type NoteNode = Node<Note>;

impl Note {
    /// Creates a new note with the given string value
    pub fn new(value: String) -> Self {
        Self(value)
    }

    /// Returns the note content as a string slice
    pub fn value(&self) -> &str {
        &self.0
    }
}
