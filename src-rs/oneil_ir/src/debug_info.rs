//! Debug information and tracing capabilities for Oneil model IR.
//!
//! This module provides the data structures for controlling debug output
//! and tracing during model evaluation and testing. Trace levels allow
//! fine-grained control over the amount of debugging information produced.

/// Trace levels for controlling debug output in Oneil models.
///
/// `TraceLevel` determines how much debugging information is output
/// during model evaluation, parameter calculation, and test execution.
/// Higher trace levels produce more detailed output.
///
/// Trace levels are used throughout the Oneil system to provide
/// visibility into the evaluation process without overwhelming users
/// with unnecessary information.
#[derive(Debug, Clone, PartialEq)]
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
    ///
    /// # Returns
    ///
    /// The default trace level (`TraceLevel::None`).
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::debug_info::TraceLevel;
    ///
    /// let default_level = TraceLevel::default();
    /// assert_eq!(default_level, TraceLevel::None);
    /// ```
    fn default() -> Self {
        Self::None
    }
}
