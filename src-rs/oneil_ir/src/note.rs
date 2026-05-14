//! Documentation notes in the intermediate representation.

/// Optional documentation attached to a parameter or model in the IR.
///
/// In source, this corresponds to a tilde-delimited note (`~` …) on a parameter
/// declaration. The stored string is the note body after trimming and removing
/// delimiters (single-line and multi-line forms both end up here).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(transparent)]
pub struct Note {
    content: String,
}

impl Note {
    /// Wraps the note body as it appears after parsing (trimmed, delimiters removed).
    #[must_use]
    pub const fn new(content: String) -> Self {
        Self { content }
    }

    /// Returns the note content as a string slice
    #[must_use]
    pub const fn content(&self) -> &str {
        self.content.as_str()
    }
}
