use std::borrow::Borrow;
use std::path::PathBuf;

use oneil_shared::{
    paths::PythonPath,
    symbols::{ParameterName, PyFunctionName, TestIndex},
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A path to a Python module.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CachedPythonPath(PythonPath);

impl Borrow<PythonPath> for CachedPythonPath {
    fn borrow(&self) -> &PythonPath {
        &self.0
    }
}

impl Serialize for CachedPythonPath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.as_path().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CachedPythonPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let path = PathBuf::deserialize(deserializer)?;
        if path.extension().map(|ext| ext.to_string_lossy()) != Some("py".into()) {
            return Err(serde::de::Error::custom(format!(
                "python module path must have .py extension, got {}",
                path.display()
            )));
        }
        Ok(Self(PythonPath::from_path_with_ext(&path)))
    }
}

impl From<PythonPath> for CachedPythonPath {
    fn from(value: PythonPath) -> Self {
        Self(value)
    }
}

impl From<CachedPythonPath> for PythonPath {
    fn from(value: CachedPythonPath) -> Self {
        value.0
    }
}

/// Compares to a [`PythonPath`] without wrapping it in [`CachedPythonPath`].
impl PartialEq<PythonPath> for CachedPythonPath {
    /// Returns whether the inner path equals `other`.
    fn eq(&self, other: &PythonPath) -> bool {
        self.0 == *other
    }
}

/// Compares to a [`CachedPythonPath`] by comparing inner paths.
impl PartialEq<CachedPythonPath> for PythonPath {
    /// Returns whether this path equals the inner path of `other`.
    fn eq(&self, other: &CachedPythonPath) -> bool {
        *self == other.0
    }
}

/// A name for a parameter in a model.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CachedParameterName(ParameterName);

impl Borrow<ParameterName> for CachedParameterName {
    fn borrow(&self) -> &ParameterName {
        &self.0
    }
}

impl Serialize for CachedParameterName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.as_str().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CachedParameterName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self(ParameterName::from(s)))
    }
}

impl From<ParameterName> for CachedParameterName {
    fn from(value: ParameterName) -> Self {
        Self(value)
    }
}

impl From<CachedParameterName> for ParameterName {
    fn from(value: CachedParameterName) -> Self {
        value.0
    }
}

/// Compares to a [`ParameterName`] without wrapping it in [`CachedParameterName`].
impl PartialEq<ParameterName> for CachedParameterName {
    /// Returns whether the inner name equals `other`.
    fn eq(&self, other: &ParameterName) -> bool {
        self.0 == *other
    }
}

/// Compares to a [`CachedParameterName`] by comparing inner names.
impl PartialEq<CachedParameterName> for ParameterName {
    /// Returns whether this name equals the inner name of `other`.
    fn eq(&self, other: &CachedParameterName) -> bool {
        *self == other.0
    }
}

/// A test index in a model.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CachedTestIndex(TestIndex);

impl Borrow<TestIndex> for CachedTestIndex {
    fn borrow(&self) -> &TestIndex {
        &self.0
    }
}

impl Serialize for CachedTestIndex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.into_usize().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CachedTestIndex {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let index = usize::deserialize(deserializer)?;
        Ok(Self(TestIndex::from(index)))
    }
}

impl From<TestIndex> for CachedTestIndex {
    fn from(value: TestIndex) -> Self {
        Self(value)
    }
}

impl From<CachedTestIndex> for TestIndex {
    fn from(value: CachedTestIndex) -> Self {
        value.0
    }
}

/// Compares to a [`TestIndex`] without wrapping it in [`CachedTestIndex`].
impl PartialEq<TestIndex> for CachedTestIndex {
    /// Returns whether the inner index equals `other`.
    fn eq(&self, other: &TestIndex) -> bool {
        self.0 == *other
    }
}

/// Compares to a [`CachedTestIndex`] by comparing inner indices.
impl PartialEq<CachedTestIndex> for TestIndex {
    /// Returns whether this index equals the inner index of `other`.
    fn eq(&self, other: &CachedTestIndex) -> bool {
        *self == other.0
    }
}

/// A name for a Python function.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CachedFunctionName(PyFunctionName);

impl Borrow<PyFunctionName> for CachedFunctionName {
    fn borrow(&self) -> &PyFunctionName {
        &self.0
    }
}

impl Serialize for CachedFunctionName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.as_str().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CachedFunctionName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self(PyFunctionName::from(s)))
    }
}

impl From<PyFunctionName> for CachedFunctionName {
    fn from(value: PyFunctionName) -> Self {
        Self(value)
    }
}

impl From<CachedFunctionName> for PyFunctionName {
    fn from(value: CachedFunctionName) -> Self {
        value.0
    }
}

/// Compares to a [`PyFunctionName`] without wrapping it in [`CachedFunctionName`].
impl PartialEq<PyFunctionName> for CachedFunctionName {
    /// Returns whether the inner name equals `other`.
    fn eq(&self, other: &PyFunctionName) -> bool {
        self.0 == *other
    }
}

/// Compares to a [`CachedFunctionName`] by comparing inner names.
impl PartialEq<CachedFunctionName> for PyFunctionName {
    /// Returns whether this name equals the inner name of `other`.
    fn eq(&self, other: &CachedFunctionName) -> bool {
        *self == other.0
    }
}
