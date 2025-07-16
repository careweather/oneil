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
    header: SectionHeaderNode,
    note: Option<NoteNode>,
    decls: Vec<DeclNode>,
}

pub type SectionNode = Node<Section>;

impl Section {
    pub fn new(header: SectionHeaderNode, note: Option<NoteNode>, decls: Vec<DeclNode>) -> Self {
        Self {
            header,
            note,
            decls,
        }
    }

    pub fn header(&self) -> &SectionHeaderNode {
        &self.header
    }

    pub fn note(&self) -> Option<&NoteNode> {
        self.note.as_ref()
    }

    pub fn decls(&self) -> &[DeclNode] {
        &self.decls
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SectionHeader {
    label: LabelNode,
}

pub type SectionHeaderNode = Node<SectionHeader>;

impl SectionHeader {
    pub fn new(label: LabelNode) -> Self {
        Self { label }
    }

    pub fn label(&self) -> &LabelNode {
        &self.label
    }
}
