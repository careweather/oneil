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
pub struct Label(String);

impl Label {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

pub type LabelNode = Node<Label>;

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

impl Str {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

pub type StrNode = Node<Str>;

#[derive(Debug, Clone, PartialEq)]
pub struct Boolean(bool);

impl Boolean {
    pub fn new(value: bool) -> Self {
        Self(value)
    }

    pub fn value(&self) -> bool {
        self.0
    }
}

pub type BooleanNode = Node<Boolean>;
