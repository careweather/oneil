use std::path::Path;

use oneil_ast as ast;

pub mod builder;
pub mod info;

pub trait FileLoader {
    type ParseError: std::fmt::Debug;
    type PythonError: std::fmt::Debug;
    fn parse_ast(&self, path: impl AsRef<Path>) -> Result<ast::Model, Self::ParseError>;
    fn validate_python_import(&self, path: impl AsRef<Path>) -> Result<(), Self::PythonError>;
}

pub struct Stack<T: PartialEq + Clone> {
    items: Vec<T>,
}

impl<T: PartialEq + Clone> Stack<T> {
    pub fn new() -> Self {
        Self { items: vec![] }
    }

    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    pub fn find_circular_dependency(&self, item: &T) -> Option<Vec<T>> {
        let item_index = self.items.iter().position(|i| i == item)?;

        let mut circular_dependency = self.items[item_index..].to_vec();
        circular_dependency.push(item.clone());

        Some(circular_dependency)
    }
}
