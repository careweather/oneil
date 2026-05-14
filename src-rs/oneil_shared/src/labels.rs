//! Label types (human-readable display names).

/// A label for a parameter (human-readable display name).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct ParameterLabel(String);

impl ParameterLabel {
    /// Creates a new parameter label with the given string value.
    #[must_use]
    pub const fn new(value: String) -> Self {
        Self(value)
    }

    /// Returns the label as a string slice.
    #[must_use]
    pub const fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Returns this label as a string.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl From<String> for ParameterLabel {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&String> for ParameterLabel {
    fn from(value: &String) -> Self {
        Self::new(value.clone())
    }
}

impl From<&str> for ParameterLabel {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

/// An optional LaTeX render-name for a parameter (e.g. `\hat{v}`).
///
/// When present, the frontend uses this raw LaTeX string to display the parameter
/// symbol instead of deriving one from the identifier via `mathName`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct RenderName(String);

impl RenderName {
    /// Creates a new render-name with the given LaTeX string.
    #[must_use]
    pub const fn new(value: String) -> Self {
        Self(value)
    }

    /// Returns the LaTeX string as a string slice.
    #[must_use]
    pub const fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Returns this render-name as a string.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl From<String> for RenderName {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for RenderName {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

/// A label for a section header.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct SectionLabel(String);

impl SectionLabel {
    /// Creates a new section label with the given string value.
    #[must_use]
    pub const fn new(value: String) -> Self {
        Self(value)
    }

    /// Returns the label as a string slice.
    #[must_use]
    pub const fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Returns this label as a string.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl From<String> for SectionLabel {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&String> for SectionLabel {
    fn from(value: &String) -> Self {
        Self::new(value.clone())
    }
}

impl From<&str> for SectionLabel {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}
