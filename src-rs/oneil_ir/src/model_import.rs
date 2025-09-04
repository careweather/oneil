use std::{collections::HashMap, ops::Deref};

use crate::{reference::ModelPath, span::WithSpan};

#[derive(Debug, Clone, PartialEq)]
pub struct SubmodelMap(HashMap<SubmodelName, SubmodelImport>);

impl SubmodelMap {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubmodelName(String);

pub type SubmodelNameWithSpan = WithSpan<SubmodelName>;

impl SubmodelName {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmodelImport {
    name: SubmodelNameWithSpan,
    path: ModelPath,
}

impl SubmodelImport {
    #[must_use]
    pub const fn new(name: SubmodelNameWithSpan, path: ModelPath) -> Self {
        Self { name, path }
    }

    #[must_use]
    pub const fn name(&self) -> &SubmodelNameWithSpan {
        &self.name
    }

    #[must_use]
    pub const fn path(&self) -> &ModelPath {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReferenceMap(HashMap<ReferenceName, ReferenceImport>);

impl ReferenceMap {
    #[must_use]
    pub fn new(references: HashMap<ReferenceName, ReferenceImport>) -> Self {
        Self(references)
    }
}

impl Deref for ReferenceMap {
    type Target = HashMap<ReferenceName, ReferenceImport>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReferenceName(String);

pub type ReferenceNameWithSpan = WithSpan<ReferenceName>;

impl ReferenceName {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceImport {
    name: ReferenceNameWithSpan,
    path: ModelPath,
}

impl ReferenceImport {
    #[must_use]
    pub const fn new(name: ReferenceNameWithSpan, path: ModelPath) -> Self {
        Self { name, path }
    }

    #[must_use]
    pub const fn name(&self) -> &ReferenceNameWithSpan {
        &self.name
    }

    #[must_use]
    pub const fn path(&self) -> &ModelPath {
        &self.path
    }
}
