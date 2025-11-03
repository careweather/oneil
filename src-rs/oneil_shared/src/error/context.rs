/// Represents different types of contextual information that can be associated with errors.
///
/// The `Context` enum provides a way to attach additional information to error messages,
/// helping users understand the context in which an error occurred and how to resolve it.
/// This is used throughout the Oneil compiler and parser to provide rich, helpful error messages.
///
/// # Examples
///
/// ```rust
/// use oneil_shared::error::Context;
///
/// // Adding a note to provide additional context
/// let note = Context::Note("Variable 'x' was declared here".to_string());
///
/// // Adding help text to suggest a solution
/// let help = Context::Help("Try using 'let x = 42;' to declare a variable".to_string());
///
/// // Using in error reporting
/// let contexts = vec![note, help];
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Context {
    /// Additional information or context about the error.
    ///
    /// Notes provide supplementary details that help users understand the error
    /// better. They might include:
    /// - References to related code locations
    /// - Explanations of what the code was trying to do
    /// - Context about the current state when the error occurred
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oneil_shared::error::Context;
    ///
    /// let note = Context::Note("Unclosed parenthesis found here".to_string());
    /// ```
    Note(String),

    /// Helpful suggestions for resolving the error.
    ///
    /// Help text provides actionable advice on how to fix the error. This might include:
    /// - Code examples showing correct usage
    /// - Step-by-step instructions for fixing the issue
    /// - References to documentation or best practices
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oneil_shared::error::Context;
    ///
    /// let help = Context::Help("Strings in Oneil use single quotes; try using `'` instead of `\"`".to_string());
    /// ```
    Help(String),
}
