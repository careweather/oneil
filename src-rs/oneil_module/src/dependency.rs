use crate::path::{ModulePath, PythonPath};

#[derive(Debug, Clone, PartialEq)]
pub struct ExternalImportList(Vec<PythonPath>);

impl ExternalImportList {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn add_import(&mut self, import_path: PythonPath) {
        self.0.push(import_path);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Dependency {
    Python(PythonPath),
    Module(ModulePath),
}
