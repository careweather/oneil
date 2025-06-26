use super::declaration::Decl;
use super::note::Note;
use crate::parser::error::CanBeEmpty;

/// A model definition in an Oneil program
///
/// Models are the primary organizational unit in Oneil, containing declarations
/// and optionally divided into labeled sections.
#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    pub note: Option<Note>,
    pub decls: Vec<Decl>,
    pub sections: Vec<Section>,
}

impl CanBeEmpty for Model {
    fn empty() -> Self {
        Model {
            note: None,
            decls: Vec::new(),
            sections: Vec::new(),
        }
    }
}

/// A labeled section within a model
#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    pub label: String,
    pub note: Option<Note>,
    pub decls: Vec<Decl>,
}

impl CanBeEmpty for Section {
    fn empty() -> Self {
        Section {
            label: String::new(),
            note: None,
            decls: Vec::new(),
        }
    }
}
