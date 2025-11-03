use std::ops::Deref;

use oneil_shared::span::Span;

use crate::reference::ModelPath;

/// A name for a submodel.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubmodelName(String);

impl SubmodelName {
    /// Creates a new submodel name with the given name.
    #[must_use]
    pub const fn new(name: String) -> Self {
        Self(name)
    }
}

impl Deref for SubmodelName {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// An import for a submodel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmodelImport {
    name: SubmodelName,
    name_span: Span,
    path: ModelPath,
}

impl SubmodelImport {
    /// Creates a new submodel import with the given name and path.
    #[must_use]
    pub const fn new(name: SubmodelName, name_span: Span, path: ModelPath) -> Self {
        Self {
            name,
            name_span,
            path,
        }
    }

    /// Returns the name of the submodel.
    #[must_use]
    pub const fn name(&self) -> &SubmodelName {
        &self.name
    }

    /// Returns the span of the name of the submodel.
    #[must_use]
    pub const fn name_span(&self) -> &Span {
        &self.name_span
    }

    /// Returns the path of the submodel.
    #[must_use]
    pub const fn path(&self) -> &ModelPath {
        &self.path
    }
}

/// A name for a reference.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReferenceName(String);

impl ReferenceName {
    /// Creates a new reference name with the given name.
    #[must_use]
    pub const fn new(name: String) -> Self {
        Self(name)
    }
}

impl Deref for ReferenceName {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// An import for a reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceImport {
    name: ReferenceName,
    name_span: Span,
    path: ModelPath,
}

impl ReferenceImport {
    /// Creates a new reference import with the given name and path.
    #[must_use]
    pub const fn new(name: ReferenceName, name_span: Span, path: ModelPath) -> Self {
        Self {
            name,
            name_span,
            path,
        }
    }

    /// Returns the name of the reference.
    #[must_use]
    pub const fn name(&self) -> &ReferenceName {
        &self.name
    }

    /// Returns the span of the name of the reference.
    #[must_use]
    pub const fn name_span(&self) -> &Span {
        &self.name_span
    }

    /// Returns the path of the reference.
    #[must_use]
    pub const fn path(&self) -> &ModelPath {
        &self.path
    }
}
