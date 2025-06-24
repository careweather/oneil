use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModulePath(PathBuf);

impl ModulePath {
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }

    pub fn join(&self, other: &str) -> PathBuf {
        self.0.join(other)
    }
}

impl AsRef<Path> for ModulePath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PythonPath(PathBuf);

impl PythonPath {
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }
}

impl AsRef<Path> for PythonPath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}
