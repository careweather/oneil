#[derive(Debug, Clone, PartialEq)]
pub enum TraceLevel {
    None,
    Trace,
    Debug,
}

impl Default for TraceLevel {
    fn default() -> Self {
        Self::None
    }
}
