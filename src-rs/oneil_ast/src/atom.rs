use crate::node::Node;

#[derive(Debug, Clone, PartialEq)]
pub struct Identifier(String);

impl Identifier {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

pub type IdentifierNode = Node<Identifier>;

#[derive(Debug, Clone, PartialEq)]
pub struct Number(f64);

impl Number {
    pub fn new(value: f64) -> Self {
        Self(value)
    }

    pub fn value(&self) -> f64 {
        self.0
    }
}

pub type NumberNode = Node<Number>;

#[derive(Debug, Clone, PartialEq)]
pub struct Str(std::string::String);

pub type StrNode = Node<Str>;

#[derive(Debug, Clone, PartialEq)]
pub struct Boolean(bool);

pub type BooleanNode = Node<Boolean>;
