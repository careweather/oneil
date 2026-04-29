//! Symbol types (identifiers and names for program entities).

/// A name for a built-in value (e.g. "pi", "e").
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BuiltinValueName(String);

impl BuiltinValueName {
    /// Creates a new built-in value name.
    #[must_use]
    pub const fn new(value: String) -> Self {
        Self(value)
    }

    /// Returns the name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns this name as a string.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl From<String> for BuiltinValueName {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&String> for BuiltinValueName {
    fn from(value: &String) -> Self {
        Self::new(value.clone())
    }
}

impl From<&str> for BuiltinValueName {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

/// A name for a built-in function (e.g. "sin", "max").
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BuiltinFunctionName(String);

impl BuiltinFunctionName {
    /// Creates a new built-in function name.
    #[must_use]
    pub const fn new(value: String) -> Self {
        Self(value)
    }

    /// Returns the name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns this name as a string.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl From<String> for BuiltinFunctionName {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&String> for BuiltinFunctionName {
    fn from(value: &String) -> Self {
        Self::new(value.clone())
    }
}

impl From<&str> for BuiltinFunctionName {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

/// A name for a Python function (from an imported module).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PyFunctionName(String);

impl PyFunctionName {
    /// Creates a new Python function name.
    #[must_use]
    pub const fn new(value: String) -> Self {
        Self(value)
    }

    /// Returns the name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns this name as a string.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl From<String> for PyFunctionName {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&String> for PyFunctionName {
    fn from(value: &String) -> Self {
        Self::new(value.clone())
    }
}

impl From<&str> for PyFunctionName {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

/// A name for a parameter in a model.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ParameterName(String);

impl ParameterName {
    /// Creates a new parameter name with the given name.
    #[must_use]
    pub const fn new(name: String) -> Self {
        Self(name)
    }

    /// Returns the parameter name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns this parameter name as a string.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl From<String> for ParameterName {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&String> for ParameterName {
    fn from(value: &String) -> Self {
        Self::new(value.clone())
    }
}

impl From<&str> for ParameterName {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

/// A name for a reference to another model.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReferenceName(String);

impl ReferenceName {
    /// Creates a new reference name with the given name.
    #[must_use]
    pub const fn new(name: String) -> Self {
        Self(name)
    }

    /// Returns the reference name as a string slice.
    #[must_use]
    pub const fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Returns this reference name as a string.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl From<String> for ReferenceName {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&String> for ReferenceName {
    fn from(value: &String) -> Self {
        Self::new(value.clone())
    }
}

impl From<&str> for ReferenceName {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

/// A name for a submodel.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SubmodelName(String);

impl SubmodelName {
    /// Creates a new submodel name with the given name.
    #[must_use]
    pub const fn new(name: String) -> Self {
        Self(name)
    }

    /// Returns the submodel name as a string slice.
    #[must_use]
    pub const fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Returns this submodel name as a string.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl From<String> for SubmodelName {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&String> for SubmodelName {
    fn from(value: &String) -> Self {
        Self::new(value.clone())
    }
}

impl From<&str> for SubmodelName {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

/// A full unit name (e.g. "m", "kg", "km", "dBW").
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnitName(String);

impl UnitName {
    /// Creates a new unit name with the given string value.
    #[must_use]
    pub const fn new(value: String) -> Self {
        Self(value)
    }

    /// Returns the unit name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns this unit name as a string.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl From<String> for UnitName {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&String> for UnitName {
    fn from(value: &String) -> Self {
        Self::new(value.clone())
    }
}

impl From<&str> for UnitName {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

/// The base part of a unit name without prefix (e.g. "m" in "km", "W" in "dBW").
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnitBaseName(String);

impl UnitBaseName {
    /// Creates a new unit base name with the given string value.
    #[must_use]
    pub const fn new(value: String) -> Self {
        Self(value)
    }

    /// Returns the unit base name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns this unit base name as a string.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl From<String> for UnitBaseName {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&String> for UnitBaseName {
    fn from(value: &String) -> Self {
        Self::new(value.clone())
    }
}

impl From<&str> for UnitBaseName {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

/// A prefix for a unit name (e.g. "k" in "km", "m" in "ms").
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnitPrefix(String);

impl UnitPrefix {
    /// Creates a new unit prefix with the given string value.
    #[must_use]
    pub const fn new(value: String) -> Self {
        Self(value)
    }

    /// Returns the unit prefix as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns this unit prefix as a string.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl From<String> for UnitPrefix {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for UnitPrefix {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

/// An index for identifying tests (0-based position in the model).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TestIndex(usize);

impl TestIndex {
    /// Creates a new test index from a numeric value.
    #[must_use]
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the index as a `usize`.
    #[must_use]
    pub const fn into_usize(&self) -> usize {
        self.0
    }
}

impl From<usize> for TestIndex {
    fn from(value: usize) -> Self {
        Self::new(value)
    }
}
