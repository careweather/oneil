use oneil_ir as ir;

/// A trait for checking if identifiers refer to builtin values or functions.
pub trait BuiltinRef {
    /// Checks if the given identifier refers to a builtin value.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The identifier to check
    ///
    /// # Returns
    ///
    /// Returns `true` if the identifier refers to a builtin value, `false` otherwise.
    fn has_builtin_value(&self, identifier: &ir::Identifier) -> bool;

    /// Checks if the given identifier refers to a builtin function.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The identifier to check
    ///
    /// # Returns
    ///
    /// Returns `true` if the identifier refers to a builtin function, `false` otherwise.
    fn has_builtin_function(&self, identifier: &ir::Identifier) -> bool;
}
