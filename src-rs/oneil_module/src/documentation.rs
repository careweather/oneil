use std::collections::HashMap;

use oneil_ast as ast;

use crate::{path::PythonPath, reference::Identifier, test::TestIndex};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SectionLabel {
    TopLevel,
    Subsection(String),
}

impl SectionLabel {
    pub fn new_top_level() -> Self {
        Self::TopLevel
    }

    pub fn new_subsection(label: String) -> Self {
        Self::Subsection(label)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SectionDecl {
    Test(TestIndex),
    Parameter(Identifier),
    InternalImport(Identifier),
    ExternalImport(PythonPath),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DocumentationMap {
    section_notes: HashMap<SectionLabel, ast::Note>,
    section_decls: HashMap<SectionLabel, Vec<SectionDecl>>,
}

impl DocumentationMap {
    pub fn new(
        section_notes: HashMap<SectionLabel, ast::Note>,
        section_decls: HashMap<SectionLabel, Vec<SectionDecl>>,
    ) -> Self {
        Self {
            section_notes,
            section_decls,
        }
    }

    pub fn section_notes(&self, section_label: &SectionLabel) -> Option<&ast::Note> {
        self.section_notes.get(section_label)
    }

    pub fn section_decls(&self, section_label: &SectionLabel) -> Option<&Vec<SectionDecl>> {
        self.section_decls.get(section_label)
    }
}
