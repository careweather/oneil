//! Documentation note constructs for the AST

use crate::node::Node;

/// A documentation note in the AST
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note(String);

/// A node containing a documentation note
pub type NoteNode = Node<Note>;

impl Note {
    /// Creates a new note with the given string value
    #[must_use]
    pub const fn new(value: String) -> Self {
        Self(value)
    }

    /// Returns the note content as a string slice
    #[must_use]
    pub fn value(&self) -> &str {
        &self.0
    }
}

impl From<String> for Note {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}
