use crate::node::Node;

#[derive(Debug, Clone, PartialEq)]
pub struct Identifier(String);

pub type IdentifierNode = Node<Identifier>;

impl Identifier {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Label(String);

pub type LabelNode = Node<Label>;

impl Label {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Number(f64);

pub type NumberNode = Node<Number>;

impl Number {
    pub fn new(value: f64) -> Self {
        Self(value)
    }

    pub fn value(&self) -> f64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Str(std::string::String);

pub type StrNode = Node<Str>;

impl Str {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Boolean(bool);

pub type BooleanNode = Node<Boolean>;

impl Boolean {
    pub fn new(value: bool) -> Self {
        Self(value)
    }

    pub fn value(&self) -> bool {
        self.0
    }
}
