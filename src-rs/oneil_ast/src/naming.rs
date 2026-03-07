//! Naming constructs for the AST

pub use oneil_shared::naming::{Identifier, Label};

use crate::node::Node;

/// A node containing an identifier
pub type IdentifierNode = Node<Identifier>;

/// A node containing a label
pub type LabelNode = Node<Label>;

/// A directory name in the Oneil language
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Directory {
    /// A single directory name
    Name(String),
    /// The parent directory
    Parent,
    /// The current directory
    Current,
}

/// A node containing a directory
pub type DirectoryNode = Node<Directory>;

impl Directory {
    /// Creates a new directory with the given string value
    #[must_use]
    pub const fn name(value: String) -> Self {
        Self::Name(value)
    }

    /// Creates a new parent directory
    #[must_use]
    pub const fn parent() -> Self {
        Self::Parent
    }

    /// Creates a new current directory
    #[must_use]
    pub const fn current() -> Self {
        Self::Current
    }

    /// Returns the directory as a string slice
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Name(name) => name,
            Self::Parent => "..",
            Self::Current => ".",
        }
    }
}

impl From<String> for Directory {
    fn from(value: String) -> Self {
        Self::name(value)
    }
}
