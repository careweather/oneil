//! Naming constructs for the AST
//!
//! This module contains structures for representing identifiers and labels
//! used throughout the Oneil language AST.

use crate::node::Node;

/// An identifier in the Oneil language
///
/// Identifiers are used to name variables, functions, models, and other
/// program elements. They are represented as strings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier(String);

/// A node containing an identifier
pub type IdentifierNode = Node<Identifier>;

impl Identifier {
    /// Creates a new identifier with the given string value
    #[must_use]
    pub const fn new(value: String) -> Self {
        Self(value)
    }

    /// Returns the identifier as a string slice
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A label in the Oneil language
///
/// Labels are used to provide human-readable names for parameters,
/// sections, and other labeled elements in the language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label(String);

/// A node containing a label
pub type LabelNode = Node<Label>;

impl Label {
    /// Creates a new label with the given string value
    #[must_use]
    pub const fn new(value: String) -> Self {
        Self(value)
    }

    /// Returns the label as a string slice
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A directory name in the Oneil language
///
/// Directories are used to organize models and other resources into logical groups.
/// This represents a single directory name, not a full path.
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
