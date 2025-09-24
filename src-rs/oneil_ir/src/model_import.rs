use std::{collections::HashMap, ops::Deref};

use crate::{reference::ModelPath, span::WithSpan};

/// A map of submodels with their names and imports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmodelMap(HashMap<SubmodelName, SubmodelImport>);

impl SubmodelMap {
    /// Creates a new submodel map with the given submodels.
    #[must_use]
    pub const fn new(submodels: HashMap<SubmodelName, SubmodelImport>) -> Self {
        Self(submodels)
    }
}

impl Deref for SubmodelMap {
    type Target = HashMap<SubmodelName, SubmodelImport>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A name for a submodel.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubmodelName(String);

/// A name for a submodel with a span.
pub type SubmodelNameWithSpan = WithSpan<SubmodelName>;

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
    name: SubmodelNameWithSpan,
    path: ModelPath,
}

impl SubmodelImport {
    /// Creates a new submodel import with the given name and path.
    #[must_use]
    pub const fn new(name: SubmodelNameWithSpan, path: ModelPath) -> Self {
        Self { name, path }
    }

    /// Returns the name of the submodel.
    #[must_use]
    pub const fn name(&self) -> &SubmodelNameWithSpan {
        &self.name
    }

    /// Returns the path of the submodel.
    #[must_use]
    pub const fn path(&self) -> &ModelPath {
        &self.path
    }
}

/// A map of references with their names and imports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceMap(HashMap<ReferenceName, ReferenceImport>);

impl ReferenceMap {
    /// Creates a new reference map with the given references.
    #[must_use]
    pub const fn new(references: HashMap<ReferenceName, ReferenceImport>) -> Self {
        Self(references)
    }
}

impl Deref for ReferenceMap {
    type Target = HashMap<ReferenceName, ReferenceImport>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A name for a reference.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReferenceName(String);

/// A name for a reference with a span.
pub type ReferenceNameWithSpan = WithSpan<ReferenceName>;

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
    name: ReferenceNameWithSpan,
    path: ModelPath,
}

impl ReferenceImport {
    /// Creates a new reference import with the given name and path.
    #[must_use]
    pub const fn new(name: ReferenceNameWithSpan, path: ModelPath) -> Self {
        Self { name, path }
    }

    /// Returns the name of the reference.
    #[must_use]
    pub const fn name(&self) -> &ReferenceNameWithSpan {
        &self.name
    }

    /// Returns the path of the reference.
    #[must_use]
    pub const fn path(&self) -> &ModelPath {
        &self.path
    }
}
