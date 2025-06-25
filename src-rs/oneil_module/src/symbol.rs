use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Deref;

use crate::reference::{Identifier, ModuleReference, Reference};
use oneil_ast as ast;

#[derive(Debug, Clone, PartialEq)]
pub enum Symbol {
    Parameter {
        dependencies: HashSet<Reference>,
        parameter: ast::Parameter,
    },
    Import(ModuleReference),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SymbolMap(HashMap<Identifier, Symbol>);

impl SymbolMap {
    pub fn new(symbols: HashMap<Identifier, Symbol>) -> Self {
        Self(symbols)
    }

    pub fn empty() -> Self {
        Self::new(HashMap::new())
    }
}

impl Deref for SymbolMap {
    type Target = HashMap<Identifier, Symbol>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
