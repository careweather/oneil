use crate::node::Node;

#[derive(Debug, Clone, PartialEq)]
pub enum TraceLevel {
    None,
    Trace,
    Debug,
}

pub type TraceLevelNode = Node<TraceLevel>;

impl TraceLevel {
    pub fn none() -> Self {
        Self::None
    }

    pub fn trace() -> Self {
        Self::Trace
    }

    pub fn debug() -> Self {
        Self::Debug
    }
}
