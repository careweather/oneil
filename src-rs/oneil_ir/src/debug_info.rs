//! Debug information and tracing capabilities for Oneil model IR.

/// Trace levels for controlling debug output in Oneil models.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceLevel {
    /// No debug output is produced
    None,
    /// Basic trace information is output
    Trace,
    /// Detailed debug information is output
    Debug,
}

impl Default for TraceLevel {
    /// Returns the default trace level.
    fn default() -> Self {
        Self::None
    }
}
