use crate::path::ModulePath;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Reference {
    Identifier(Identifier),
    Accessor {
        parent: Identifier,
        component: Box<Reference>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier(String);

impl Identifier {
    pub fn new(ident: String) -> Self {
        Self(ident)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleReference {
    path: ModulePath,
    subcomponents: Vec<Identifier>,
}

impl ModuleReference {
    pub fn new(path: ModulePath, subcomponents: Vec<Identifier>) -> Self {
        Self {
            path,
            subcomponents,
        }
    }
}
