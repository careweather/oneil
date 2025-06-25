use std::ops::Deref;

use crate::path::{ModulePath, PythonPath};

pub mod graph;

#[derive(Debug, Clone, PartialEq)]
pub struct ExternalImportList(Vec<PythonPath>);

impl ExternalImportList {
    pub fn new(imports: Vec<PythonPath>) -> Self {
        Self(imports)
    }

    pub fn empty() -> Self {
        Self::new(vec![])
    }
}

impl Deref for ExternalImportList {
    type Target = Vec<PythonPath>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Dependency {
    Python(PythonPath),
    Module(ModulePath),
}
