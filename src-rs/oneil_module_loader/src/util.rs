use std::path::Path;

use oneil_ast as ast;
use oneil_module::ModulePath;

pub trait FileLoader {
    type ParseError;
    fn parse_ast(&self, path: impl AsRef<Path>) -> Result<ast::Model, Self::ParseError>;
    fn file_exists(&self, path: impl AsRef<Path>) -> bool;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleStack {
    stack: Vec<ModulePath>,
}

impl ModuleStack {
    pub fn new() -> Self {
        Self { stack: vec![] }
    }

    pub fn push(&mut self, path: ModulePath) {
        self.stack.push(path);
    }

    pub fn pop(&mut self) -> Option<ModulePath> {
        self.stack.pop()
    }

    pub fn check_for_cyclical_dependency(&self, path: &ModulePath) -> Option<Vec<ModulePath>> {
        // Get the index of the last occurence of the path in the stack, if any exists
        let last_index = self.stack.iter().rposition(|p| p == path);

        match last_index {
            Some(index) => {
                let cyclical_deps = self.stack[index..].to_vec();
                Some(cyclical_deps)
            }
            None => None,
        }
    }
}
