use super::declaration::Decl;
use super::note::Note;

#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    pub note: Option<Note>,
    pub decls: Vec<Decl>,
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    pub label: String,
    pub note: Option<Note>,
    pub decls: Vec<Decl>,
}
