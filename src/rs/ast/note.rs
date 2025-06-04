/// A documentation note in the AST
///
/// Notes can be attached to various AST nodes to provide documentation,
/// explanations, or other comments. They can be either single-line notes
/// starting with `~` or multi-line notes delimited by `~~~`.
#[derive(Debug, Clone, PartialEq)]
pub struct Note(pub String);
