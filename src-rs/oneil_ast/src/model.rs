use crate::{atom::LabelNode, declaration::DeclNode, node::Node, note::NoteNode};

/// A model definition in an Oneil program
///
/// Models are the primary organizational unit in Oneil, containing declarations
/// and optionally divided into labeled sections.
#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    pub note: Option<NoteNode>,
    pub decls: Vec<DeclNode>,
    pub sections: Vec<SectionNode>,
}

impl Model {
    pub fn new(note: Option<NoteNode>, decls: Vec<DeclNode>, sections: Vec<SectionNode>) -> Self {
        Self {
            note,
            decls,
            sections,
        }
    }
}

pub type ModelNode = Node<Model>;

/// A labeled section within a model
#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    pub label: LabelNode,
    pub note: Option<NoteNode>,
    pub decls: Vec<DeclNode>,
}

impl Section {
    pub fn new(label: LabelNode, note: Option<NoteNode>, decls: Vec<DeclNode>) -> Self {
        Self { label, note, decls }
    }
}

pub type SectionNode = Node<Section>;
