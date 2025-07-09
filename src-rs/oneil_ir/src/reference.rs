//! Reference types for identifiers, paths, and imports in Oneil.
//!
//! This module provides the fundamental types for referencing entities in Oneil,
//! including identifiers for variables and parameters, module paths for file
//! references, and Python paths for external imports.

use std::path::{Path, PathBuf};

/// An identifier for a variable, parameter, or other named entity in Oneil.
///
/// `Identifier` represents a string-based name that uniquely identifies
/// an entity within a scope. Identifiers are used throughout Oneil for
/// naming parameters, variables, submodels, and other components.
///
/// Identifiers are immutable and provide a safe wrapper around string values.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier(String);

impl Identifier {
    /// Creates a new identifier from a string or string-like type.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The string value for the identifier
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::reference::Identifier;
    ///
    /// let id1 = Identifier::new("radius");
    /// let id2 = Identifier::new(String::from("area"));
    ///
    /// assert_eq!(id1.value(), "radius");
    /// assert_eq!(id2.value(), "area");
    /// ```
    pub fn new(identifier: impl AsRef<str>) -> Self {
        Self(identifier.as_ref().to_string())
    }

    /// Returns the string value of this identifier.
    ///
    /// # Returns
    ///
    /// A string slice containing the identifier's value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::reference::Identifier;
    ///
    /// let id = Identifier::new("my_parameter");
    /// assert_eq!(id.value(), "my_parameter");
    /// ```
    pub fn value(&self) -> &str {
        &self.0
    }
}

/// A path to an Oneil module file.
///
/// `ModulePath` represents the file system path to an Oneil module (`.on` file).
/// It automatically handles file extensions, ensuring that all module paths
/// have the correct `.on` extension.
///
/// Module paths are used for:
/// - Importing submodels from other files
/// - Resolving module dependencies
/// - File system operations on Oneil modules
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModulePath(PathBuf);

// TODO: maybe allow to convert from a list of strings (without extension) or
//       from a path (with extension). Same for PythonPath.
impl ModulePath {
    /// Creates a new module path from a path or path-like type.
    ///
    /// This constructor automatically ensures the path has a `.on` extension.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the module file
    ///
    /// # Panics
    ///
    /// Panics if the path has an extension.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::reference::ModulePath;
    /// use std::path::PathBuf;
    ///
    /// // Path without extension gets .on added
    /// let path1 = ModulePath::new("math");
    /// assert_eq!(path1.as_ref().to_string_lossy(), "math.on");
    ///
    /// // Path with an extension panics
    /// // ModulePath::new("file.on"); // This would panic
    /// // ModulePath::new("file.txt"); // This would panic
    /// ```
    pub fn new(path: impl AsRef<Path>) -> Self {
        let mut path = path.as_ref().to_path_buf();

        match path.extension() {
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

    // TODO: this might be a bit confusing because it returns a path without the
    //       extension. Maybe we should return a path with the extension? See
    //       the TODO above for more thoughts.
    /// Returns a path for a sibling module relative to the current module's path.
    ///
    /// Given a path to another module, this function returns a new path that represents
    /// that module as a sibling of the current module (i.e., in the same directory).
    ///
    /// # Arguments
    ///
    /// * `sibling_name` - The name or path of the sibling module
    ///
    /// # Returns
    ///
    /// A `PathBuf` representing the sibling module's path.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::reference::ModulePath;
    ///
    /// let current = ModulePath::new("models/geometry/circle");
    /// let sibling = current.get_sibling_path("square");
    ///
    /// // Result: "models/geometry/square" (without .on extension)
    /// assert_eq!(sibling.to_string_lossy(), "models/geometry/square");
    ///
    /// // Works with nested paths too
    /// let sibling2 = current.get_sibling_path("shapes/triangle");
    /// assert_eq!(sibling2.to_string_lossy(), "models/geometry/shapes/triangle");
    /// ```
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

/// A path to a Python module file.
///
/// `PythonPath` represents the file system path to a Python module (`.py` file).
/// It automatically handles file extensions, ensuring that all Python paths
/// have the correct `.py` extension.
///
/// Python paths are used for:
/// - Importing external Python functionality into Oneil modules
/// - Resolving Python module dependencies
/// - File system operations on Python modules
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PythonPath(PathBuf);

impl PythonPath {
    /// Creates a new Python path from a path buffer.
    ///
    /// This constructor automatically ensures the path has a `.py` extension.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the Python module file
    ///
    /// # Panics
    ///
    /// Panics if the path has an extension.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::reference::PythonPath;
    /// use std::path::PathBuf;
    ///
    /// // Path without extension gets .py added
    /// let path1 = PythonPath::new(PathBuf::from("math"));
    /// assert_eq!(path1.as_ref().to_string_lossy(), "math.py");
    ///
    /// // Path with other extension panics
    /// // PythonPath::new(PathBuf::from("file.txt")); // This would panic
    /// ```
    pub fn new(mut path: PathBuf) -> Self {
        match path.extension() {
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
