//! Section metadata in the intermediate representation.
//!
//! Sections group parameters and tests under a named header with an optional note.
//! They preserve source-declaration order across both item types without embedding
//! full parameter or test data — items are referenced by ID only.

use oneil_shared::symbols::{ParameterName, TestIndex};

use crate::Note;

/// A reference to a parameter or test within a section.
///
/// Sections record items by ID so the caller can look up the full data from
/// `InstancedModel::parameters()` / `InstancedModel::tests()` without duplication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SectionItem {
    /// A parameter, identified by name.
    Parameter(ParameterName),
    /// A test, identified by its position-based index.
    Test(TestIndex),
}

/// A named section within a model, with an optional note and an ordered item list.
///
/// Items are in source-declaration order, interleaving parameters and tests as
/// they appeared in the source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    note: Option<Note>,
    items: Vec<SectionItem>,
}

impl Section {
    /// Creates a new section with the given note and item list.
    #[must_use]
    pub const fn new(note: Option<Note>, items: Vec<SectionItem>) -> Self {
        Self { note, items }
    }

    /// Returns the optional documentation note for this section.
    #[must_use]
    pub const fn note(&self) -> Option<&Note> {
        self.note.as_ref()
    }

    /// Returns the ordered list of parameter/test items in this section.
    #[must_use]
    pub fn items(&self) -> &[SectionItem] {
        &self.items
    }

    /// Mutable view of the ordered item list.
    pub const fn items_mut(&mut self) -> &mut Vec<SectionItem> {
        &mut self.items
    }
}
