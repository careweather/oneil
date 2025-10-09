//! Reference types for identifiers, paths, and imports in Oneil.

use std::path::{Path, PathBuf};

/// An identifier for a variable, parameter, or other named entity in Oneil.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier(String);

impl Identifier {
    /// Creates a new identifier from a string or string-like type.
    #[must_use]
    pub fn new(identifier: impl AsRef<str>) -> Self {
        Self(identifier.as_ref().to_string())
    }

    /// Returns the string value of this identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// An identifier with associated source location information.
pub type IdentifierWithSpan = Identifier;

/// A path to an Oneil model file.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModelPath(PathBuf);

// TODO: maybe allow to convert from a list of strings (without extension) or
//       from a path (with extension). Same for PythonPath.
impl ModelPath {
    /// Creates a new model path from a path or path-like type.
    ///
    /// # Panics
    ///
    /// Panics if the path has an extension.
    #[must_use]
    pub fn new(path: impl AsRef<Path>) -> Self {
        let mut path = path.as_ref().to_path_buf();

        match path.extension() {
            Some(ext) if ext != "on" => {
                panic!(
                    "Model paths must not have an extension other than .on: '{}'",
                    ext.display()
                )
            }
            _ => {
                path.set_extension("on");
                Self(path)
            }
        }
    }

    // TODO: this might be a bit confusing because it returns a path without the
    //       extension. Maybe we should return a path with the extension? See
    //       the TODO above for more thoughts.
    /// Returns a path for a sibling model relative to the current model's path.
    pub fn get_sibling_path(&self, sibling_name: impl AsRef<str>) -> PathBuf {
        let parent = self.0.parent();
        let sibling_name = sibling_name.as_ref();

        parent.map_or_else(
            || PathBuf::from(sibling_name),
            |parent| parent.join(sibling_name),
        )
    }
}

impl AsRef<Path> for ModelPath {
    fn as_ref(&self) -> &Path {
        self.0.as_path()
    }
}

/// A path to a Python module file.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PythonPath(PathBuf);

impl PythonPath {
    /// Creates a new Python path from a path buffer.
    ///
    /// # Panics
    ///
    /// Panics if the path has an extension.
    #[must_use]
    pub fn new(mut path: PathBuf) -> Self {
        match path.extension() {
            Some(ext) if ext != "py" => panic!(
                "Python paths must not have an extension other than .py: '{}'",
                ext.display()
            ),
            _ => {
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
