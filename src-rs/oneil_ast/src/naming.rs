//! Naming constructs for the AST

use indexmap::Equivalent;
use oneil_shared::{
    labels::{ParameterLabel, RenderName, SectionLabel},
    symbols::{
        BuiltinFunctionName, BuiltinValueName, ParameterName, PyFunctionName, ReferenceName,
    },
};

use crate::node::Node;

/// A node containing a reference name
pub type ReferenceNameNode = Node<ReferenceName>;

/// A node containing a submodel name
pub type ParameterNameNode = Node<ParameterName>;

/// A node containing a parameter label
pub type ParameterLabelNode = Node<ParameterLabel>;

/// A node containing an optional LaTeX render-name
pub type RenderNameNode = Node<RenderName>;

/// A node containing a section label
pub type SectionLabelNode = Node<SectionLabel>;

/// An identifier for a parameter, builtin value, or function name
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier(String);

impl Identifier {
    /// Creates a new identifier from a string or string-like type.
    #[must_use]
    pub const fn new(identifier: String) -> Self {
        Self(identifier)
    }

    /// Returns the identifier as a string slice.
    #[must_use]
    pub const fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Returns this identifier as a string.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl From<String> for Identifier {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for Identifier {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

impl Equivalent<BuiltinValueName> for Identifier {
    fn equivalent(&self, key: &BuiltinValueName) -> bool {
        self.as_str() == key.as_str()
    }
}

impl Equivalent<BuiltinFunctionName> for Identifier {
    fn equivalent(&self, key: &BuiltinFunctionName) -> bool {
        self.as_str() == key.as_str()
    }
}

impl Equivalent<PyFunctionName> for Identifier {
    fn equivalent(&self, key: &PyFunctionName) -> bool {
        self.as_str() == key.as_str()
    }
}

impl Equivalent<ParameterName> for Identifier {
    fn equivalent(&self, key: &ParameterName) -> bool {
        self.as_str() == key.as_str()
    }
}

/// A node containing an identifier
pub type IdentifierNode = Node<Identifier>;

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
