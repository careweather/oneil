use crate::node::Node;

/// A documentation note in the AST
///
/// Notes can be attached to various AST nodes to provide documentation,
/// explanations, or other comments. They can be either single-line notes
/// starting with `~` or multi-line notes delimited by `~~~`.
#[derive(Debug, Clone, PartialEq)]
pub struct Note(String);

pub type NoteNode = Node<Note>;

impl Note {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}
