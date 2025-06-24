use std::path::Path;

use oneil_ast as ast;

pub trait FileLoader {
    type ParseError;
    fn parse_ast(&self, path: impl AsRef<Path>) -> Result<ast::Model, Self::ParseError>;
    fn file_exists(&self, path: impl AsRef<Path>) -> bool;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stack<T>
where
    T: PartialEq + Clone,
{
    stack: Vec<T>,
}

impl<T> Stack<T>
where
    T: PartialEq + Clone,
{
    pub fn new() -> Self {
        Self { stack: vec![] }
    }

    pub fn push(&mut self, path: T) {
        self.stack.push(path);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.stack.pop()
    }

    pub fn check_for_cyclical_dependency(&self, path: &T) -> Option<Vec<T>> {
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
