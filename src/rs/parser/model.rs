//! Parser for model definitions in an Oneil program.
//!
//! # Examples
//!
//! ```
//! use oneil::parser::model::parse;
//! use oneil::parser::Span;
//!
//! // Parse a simple model
//! let input = Span::new("import foo\n");
//! let (_, model) = parse(input).unwrap();
//!
//! // Parse a model with sections
//! let input = Span::new("import foo\nsection bar\nimport baz\n");
//! let (_, model) = parse(input).unwrap();
//! ```

use nom::{
    Parser as _,
    combinator::{cut, opt},
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
pub fn parse(input: Span) -> Result<Model> {
    model(input)
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

    #[test]
    fn test_empty_model() {
        let input = Span::new("");
        let (rest, model) = parse(input).unwrap();
        assert!(model.note.is_none());
        assert!(model.decls.is_empty());
        assert!(model.sections.is_empty());
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_model_with_note() {
        let input = Span::new("~ This is a note\n");
        let (rest, model) = parse(input).unwrap();
        assert!(model.note.is_some());
        assert!(model.decls.is_empty());
        assert!(model.sections.is_empty());
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_model_with_import() {
        let input = Span::new("import foo\n");
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
        let input = Span::new("section foo\nimport bar\n");
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
        let input = Span::new("section foo\nimport bar\nsection baz\nimport qux\n");
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
}
