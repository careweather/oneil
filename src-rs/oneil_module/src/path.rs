use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModulePath(PathBuf);

impl ModulePath {
    pub fn new(mut path: PathBuf) -> Self {
        match path.extension() {
            Some(ext) if ext == "on" => Self(path),
            Some(ext) => panic!(
                "Module paths must not have an extension other than .on: '{:?}'",
                ext
            ),
            None => {
                path.set_extension("on");
                Self(path)
            }
        }
    }

    /// Returns a path for a sibling module relative to the current module's path.
    ///
    /// Given a path to another module, this function returns a new path that represents
    /// that module as a sibling of the current module (i.e., in the same directory).
    ///
    /// For example, given that the path of `self` is `foo/bar/baz` and `other` is `qux`,
    /// the returned path will be `foo/bar/qux`.
    pub fn get_sibling_path(&self, other: impl AsRef<Path>) -> PathBuf {
        let parent = self.0.parent();

        let sibling_path = match parent {
            Some(parent) => parent.join(other.as_ref()),
            None => PathBuf::from(other.as_ref()),
        };

        sibling_path
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
    pub fn new(mut path: PathBuf) -> Self {
        match path.extension() {
            Some(ext) if ext == "py" => Self(path),
            Some(ext) => panic!(
                "Python paths must not have an extension other than .py: '{:?}'",
                ext
            ),
            None => {
                path.set_extension("py");
                Self(path)
            }
        }
    }
}

impl AsRef<Path> for PythonPath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}
