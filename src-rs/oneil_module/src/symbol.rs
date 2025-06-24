use std::collections::HashMap;
use std::collections::HashSet;

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
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add_symbol(&mut self, ident: Identifier, symbol: Symbol) {
        self.0.insert(ident, symbol);
    }
}
