use oneil_ir as ir;

/// A trait for checking if identifiers refer to builtin values or functions.
///
/// This trait provides methods to check whether a given identifier refers to a builtin
/// value (like a constant or variable) or a builtin function in the Oneil language.
/// Implementations of this trait maintain the set of valid builtin identifiers and
/// provide lookup functionality.
///
/// # Examples
///
/// ```ignore
/// use oneil_ir::reference::Identifier;
///
/// struct MyBuiltins;
///
/// impl BuiltinRef for MyBuiltins {
///     fn has_builtin_value(&self, id: &Identifier) -> bool {
///         // Check if id is a builtin value
///         false
///     }
///
///     fn has_builtin_function(&self, id: &Identifier) -> bool {
///         // Check if id is a builtin function
///         false  
///     }
/// }
/// ```
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
