use std::path::Path;

use oneil_ast as ast;

pub mod builder;

pub trait FileLoader {
    type ParseError;
    fn parse_ast(&self, path: impl AsRef<Path>) -> Result<ast::Model, Self::ParseError>;
    fn file_exists(&self, path: impl AsRef<Path>) -> bool;
}

pub struct Stack<T: PartialEq> {
    items: Vec<T>,
}

impl<T: PartialEq> Stack<T> {
    pub fn new() -> Self {
        Self { items: vec![] }
    }

    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    pub fn contains(&self, item: &T) -> bool {
        self.items.contains(item)
    }
}
