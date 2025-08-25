//! Model constructs for the AST
//!
//! This module contains structures for representing models in Oneil programs,
//! including model definitions, sections, and section headers.

use crate::{declaration::DeclNode, naming::LabelNode, node::Node, note::NoteNode};

/// A model definition in an Oneil program
///
/// Models are the primary organizational unit in Oneil, containing declarations
/// and optionally divided into labeled sections.
#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    note: Option<NoteNode>,
    decls: Vec<DeclNode>,
    sections: Vec<SectionNode>,
}

/// A node containing a model definition
pub type ModelNode = Node<Model>;

impl Model {
    /// Creates a new model with the given components
    pub fn new(note: Option<NoteNode>, decls: Vec<DeclNode>, sections: Vec<SectionNode>) -> Self {
        Self {
            note,
            decls,
            sections,
        }
    }

    /// Returns the optional note attached to this model
    pub fn note(&self) -> Option<&NoteNode> {
        self.note.as_ref()
    }

    /// Returns the list of declarations in this model
    pub fn decls(&self) -> &[DeclNode] {
        &self.decls
    }

    /// Returns the list of sections in this model
    pub fn sections(&self) -> &[SectionNode] {
        &self.sections
    }
}

/// A labeled section within a model
#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    header: SectionHeaderNode,
    note: Option<NoteNode>,
    decls: Vec<DeclNode>,
}

/// A node containing a section definition
pub type SectionNode = Node<Section>;

impl Section {
    /// Creates a new section with the given components
    pub fn new(header: SectionHeaderNode, note: Option<NoteNode>, decls: Vec<DeclNode>) -> Self {
        Self {
            header,
            note,
            decls,
        }
    }

    /// Returns the section header
    pub fn header(&self) -> &SectionHeaderNode {
        &self.header
    }

    /// Returns the optional note attached to this section
    pub fn note(&self) -> Option<&NoteNode> {
        self.note.as_ref()
    }

    /// Returns the list of declarations in this section
    pub fn decls(&self) -> &[DeclNode] {
        &self.decls
    }
}

/// A section header that contains a label
#[derive(Debug, Clone, PartialEq)]
pub struct SectionHeader {
    label: LabelNode,
}

/// A node containing a section header
pub type SectionHeaderNode = Node<SectionHeader>;

impl SectionHeader {
    /// Creates a new section header with the given label
    pub fn new(label: LabelNode) -> Self {
        Self { label }
    }

    /// Returns the label of this section header
    pub fn label(&self) -> &LabelNode {
        &self.label
    }
}
