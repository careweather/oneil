use std::collections::HashMap;

use oneil_module::reference::{Identifier, ModulePath};

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleErrorMap {
    errors: HashMap<ModulePath, Vec<()>>,
}

impl ModuleErrorMap {
    pub fn new() -> Self {
        Self {
            errors: HashMap::new(),
        }
    }

    pub fn add_error(&mut self, module_path: ModulePath, error: ()) {
        self.errors.entry(module_path).or_insert(vec![]).push(error);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParameterErrorMap {
    errors: HashMap<Identifier, ()>,
}

impl ParameterErrorMap {
    pub fn new() -> Self {
        Self {
            errors: HashMap::new(),
        }
    }

    pub fn add_error(&mut self, identifier: Identifier, error: ()) {
        self.errors.insert(identifier, error);
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
}
