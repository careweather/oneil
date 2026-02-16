//! Trees used for expressing relationships between parameters
//! including dependencies and references.

use std::path::PathBuf;

use oneil_output::Value;
use oneil_shared::span::Span;

/// A tree of values with children.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tree<T> {
    value: T,
    children: Vec<Self>,
}

impl<T> Tree<T> {
    /// Creates a new tree with the given value and children.
    #[must_use]
    pub const fn new(value: T, children: Vec<Self>) -> Self {
        Self { value, children }
    }

    /// Returns the value of the tree.
    #[must_use]
    pub const fn value(&self) -> &T {
        &self.value
    }

    /// Returns the children of the tree.
    #[must_use]
    pub const fn children(&self) -> &[Self] {
        self.children.as_slice()
    }
}

/// A value in a dependency tree
#[derive(Debug, Clone, PartialEq)]
pub struct DependencyTreeValue {
    /// The reference name used to access an external model, if this is an external dependency.
    ///
    /// This is `None` for builtin dependencies and parameters within the same model as the original parameter.
    pub reference_name: Option<String>,
    /// The name of the parameter or builtin value.
    pub parameter_name: String,
    /// The evaluated value of the parameter or builtin.
    pub parameter_value: Value,
    /// Display information for the parameter, containing the model path and source span.
    ///
    /// This is `None` for builtin dependencies, which don't have a source location.
    pub display_info: Option<(PathBuf, Span)>,
}

/// A value in a reference tree
#[derive(Debug, Clone, PartialEq)]
pub struct ReferenceTreeValue {
    /// The path to the model containing the parameter
    pub model_path: PathBuf,
    /// The name of the parameter
    pub parameter_name: String,
    /// The evaluated value of the parameter
    pub parameter_value: Value,
    /// Display information for the parameter, containing the model path and source span.
    pub display_info: (PathBuf, Span),
}
