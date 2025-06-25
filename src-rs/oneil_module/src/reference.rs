use crate::path::ModulePath;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier(String);

impl Identifier {
    pub fn new(ident: String) -> Self {
        Self(ident)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for Identifier {
    fn from(ident: String) -> Self {
        Self(ident)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Reference {
    Identifier(Identifier),
    Accessor {
        parent: Identifier,
        component: Box<Reference>,
    },
}

impl Reference {
    pub fn identifier(ident: Identifier) -> Self {
        Self::Identifier(ident)
    }

    pub fn accessor(parent: Identifier, component: Reference) -> Self {
        Self::Accessor {
            parent,
            component: Box::new(component),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleReference {
    path: ModulePath,
    reference: Option<Reference>,
}

impl ModuleReference {
    pub fn new(path: ModulePath, reference: Option<Reference>) -> Self {
        Self { path, reference }
    }

    pub fn module_path(&self) -> &ModulePath {
        &self.path
    }

    pub fn reference(&self) -> Option<&Reference> {
        self.reference.as_ref()
    }
}
