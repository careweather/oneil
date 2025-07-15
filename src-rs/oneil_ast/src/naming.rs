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
