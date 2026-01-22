//! Trees used for expressing relationships between parameters
//! including dependencies and references.

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
