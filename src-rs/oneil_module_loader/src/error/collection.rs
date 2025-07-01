use std::collections::HashMap;

use oneil_module::reference::ModulePath;

#[derive(Debug, Clone, PartialEq)]
pub struct LoadErrorMap {
    errors: HashMap<ModulePath, Vec<()>>,
}

impl LoadErrorMap {
    pub fn new() -> Self {
        Self {
            errors: HashMap::new(),
        }
    }

    pub fn add_error(&mut self, module_path: ModulePath, error: ()) {
        self.errors.entry(module_path).or_insert(vec![]).push(error);
    }
}
