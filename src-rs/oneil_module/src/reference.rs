use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier(String);

impl Identifier {
    pub fn new(identifier: impl AsRef<str>) -> Self {
        Self(identifier.as_ref().to_string())
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModulePath(PathBuf);

impl ModulePath {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let mut path = path.as_ref().to_path_buf();

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
    pub fn get_sibling_path(&self, sibling_name: impl AsRef<Path>) -> PathBuf {
        let parent = self.0.parent();
        let sibling_name = sibling_name.as_ref();

        let sibling_path = match parent {
            Some(parent) => parent.join(sibling_name),
            None => PathBuf::from(sibling_name),
        };

        sibling_path
    }
}

impl AsRef<Path> for ModulePath {
    fn as_ref(&self) -> &Path {
        self.0.as_path()
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
        self.0.as_path()
    }
}
