use crate::{atom::LabelNode, declaration::DeclNode, node::Node, note::NoteNode};

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

pub type ModelNode = Node<Model>;

impl Model {
    pub fn new(note: Option<NoteNode>, decls: Vec<DeclNode>, sections: Vec<SectionNode>) -> Self {
        Self {
            note,
            decls,
            sections,
        }
    }

    pub fn note(&self) -> Option<&NoteNode> {
        self.note.as_ref()
    }

    pub fn decls(&self) -> &[DeclNode] {
        &self.decls
    }

    pub fn sections(&self) -> &[SectionNode] {
        &self.sections
    }
}

/// A labeled section within a model
#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    label: LabelNode,
    note: Option<NoteNode>,
    decls: Vec<DeclNode>,
}

pub type SectionNode = Node<Section>;

impl Section {
    pub fn new(label: LabelNode, note: Option<NoteNode>, decls: Vec<DeclNode>) -> Self {
        Self { label, note, decls }
    }

    pub fn label(&self) -> &LabelNode {
        &self.label
    }

    pub fn note(&self) -> Option<&NoteNode> {
        self.note.as_ref()
    }

    pub fn decls(&self) -> &[DeclNode] {
        &self.decls
    }
}
