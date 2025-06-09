//! Parser for model definitions in an Oneil program.
//!
//! # Examples
//!
//! ```
//! use oneil::parser::model::parse;
//! use oneil::parser::{Config, Span};
//!
//! // Parse a simple model
//! let input = Span::new_extra("import foo\n", Config::default());
//! let (_, model) = parse(input).unwrap();
//!
//! // Parse a model with sections
//! let input = Span::new_extra("import foo\nsection bar\nimport baz\n", Config::default());
//! let (_, model) = parse(input).unwrap();
//! ```

use nom::{
    Parser as _,
    combinator::{all_consuming, cut, opt},
    multi::many0,
};

use crate::ast::model::{Model, Section};

use super::{
    declaration::parse as parse_decl,
    note::parse as parse_note,
    token::{keyword::section, naming::identifier, structure::end_of_line},
    util::{Result, Span},
};

/// Parses a model definition
///
/// This function **may not consume the complete input**.
///
/// # Examples
///
/// ```
/// use oneil::parser::model::parse;
/// use oneil::parser::{Config, Span};
///
/// let input = Span::new_extra("import foo\n", Config::default());
/// let (rest, model) = parse(input).unwrap();
/// assert_eq!(rest.fragment(), &"");
/// ```
///
/// ```
/// use oneil::parser::model::parse;
/// use oneil::parser::{Config, Span};
///
/// let input = Span::new_extra("import foo\n<rest>", Config::default());
/// let (rest, model) = parse(input).unwrap();
/// assert_eq!(rest.fragment(), &"<rest>");
/// ```
pub fn parse(input: Span) -> Result<Model> {
    model(input)
}

/// Parses a model definition
///
/// This function **fails if the complete input is not consumed**.
///
/// # Examples
///
/// ```
/// use oneil::parser::model::parse_complete;
/// use oneil::parser::{Config, Span};
///
/// let input = Span::new_extra("import foo\n", Config::default());
/// let (rest, model) = parse_complete(input).unwrap();
/// assert_eq!(rest.fragment(), &"");
/// ```
///
/// ```
/// use oneil::parser::model::parse_complete;
/// use oneil::parser::{Config, Span};
///
/// let input = Span::new_extra("import foo\nrest", Config::default());
/// let result = parse_complete(input);
/// assert_eq!(result.is_err(), true);
/// ```
pub fn parse_complete(input: Span) -> Result<Model> {
    all_consuming(model).parse(input)
}

/// Parses a model definition
fn model(input: Span) -> Result<Model> {
    (
        opt(end_of_line),
        opt(parse_note),
        many0(parse_decl),
        many0(parse_section),
    )
        .map(|(_, note, decls, sections)| Model {
            note,
            decls,
            sections,
        })
        .parse(input)
}

/// Parses a section within a model
fn parse_section(input: Span) -> Result<Section> {
    (
        section,
        cut((identifier, end_of_line)),
        opt(parse_note),
        many0(parse_decl),
    )
        .map(|(_, (label, _), note, decls)| Section {
            label: label.to_string(),
            note,
            decls,
        })
        .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::declaration::Decl;
    use crate::parser::Config;

    #[test]
    fn test_empty_model() {
        let input = Span::new_extra("", Config::default());
        let (rest, model) = parse(input).unwrap();
        assert!(model.note.is_none());
        assert!(model.decls.is_empty());
        assert!(model.sections.is_empty());
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_model_with_note() {
        let input = Span::new_extra("~ This is a note\n", Config::default());
        let (rest, model) = parse(input).unwrap();
        assert!(model.note.is_some());
        assert!(model.decls.is_empty());
        assert!(model.sections.is_empty());
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_model_with_import() {
        let input = Span::new_extra("import foo\n", Config::default());
        let (rest, model) = parse(input).unwrap();
        assert!(model.note.is_none());
        assert_eq!(model.decls.len(), 1);
        match &model.decls[0] {
            Decl::Import { path } => assert_eq!(path, "foo"),
            _ => panic!("Expected import declaration"),
        }
        assert!(model.sections.is_empty());
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_model_with_section() {
        let input = Span::new_extra("section foo\nimport bar\n", Config::default());
        let (rest, model) = parse(input).unwrap();
        assert!(model.note.is_none());
        assert!(model.decls.is_empty());
        assert_eq!(model.sections.len(), 1);
        let section = &model.sections[0];
        assert_eq!(section.label, "foo");
        assert_eq!(section.decls.len(), 1);
        match &section.decls[0] {
            Decl::Import { path } => assert_eq!(path, "bar"),
            _ => panic!("Expected import declaration"),
        }
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_model_with_multiple_sections() {
        let input = Span::new_extra(
            "section foo\nimport bar\nsection baz\nimport qux\n",
            Config::default(),
        );
        let (rest, model) = parse(input).unwrap();
        assert!(model.note.is_none());
        assert!(model.decls.is_empty());
        assert_eq!(model.sections.len(), 2);

        let section1 = &model.sections[0];
        assert_eq!(section1.label, "foo");
        assert_eq!(section1.decls.len(), 1);
        match &section1.decls[0] {
            Decl::Import { path } => assert_eq!(path, "bar"),
            _ => panic!("Expected import declaration"),
        }

        let section2 = &model.sections[1];
        assert_eq!(section2.label, "baz");
        assert_eq!(section2.decls.len(), 1);
        match &section2.decls[0] {
            Decl::Import { path } => assert_eq!(path, "qux"),
            _ => panic!("Expected import declaration"),
        }

        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_parse_complete_empty_model_success() {
        let input = Span::new_extra("\n", Config::default());
        let (rest, model) = parse_complete(input).unwrap();
        assert!(model.note.is_none());
        assert!(model.decls.is_empty());
        assert!(model.sections.is_empty());
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_parse_complete_with_declarations_success() {
        let input = Span::new_extra("import foo\nimport bar\n", Config::default());
        let (rest, model) = parse_complete(input).unwrap();
        assert_eq!(model.decls.len(), 2);
        match &model.decls[0] {
            Decl::Import { path } => assert_eq!(path, "foo"),
            _ => panic!("Expected import declaration"),
        }
        match &model.decls[1] {
            Decl::Import { path } => assert_eq!(path, "bar"),
            _ => panic!("Expected import declaration"),
        }
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_parse_complete_with_remaining_input() {
        let input = Span::new_extra("import foo\n<rest>", Config::default());
        let result = parse_complete(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_empty_model() {
        let input = Span::new_extra("", Config::default());
        let (rest, model) = parse(input).unwrap();
        assert!(model.note.is_none());
        assert!(model.decls.is_empty());
        assert!(model.sections.is_empty());
        assert_eq!(rest.fragment(), &"");
    }
}
