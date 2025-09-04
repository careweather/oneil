use std::collections::HashMap;

use oneil_ir::{
    reference::{ModelPath, PythonPath},
    span::IrSpan,
};

use crate::{
    FileLoader,
    error::resolution::ImportResolutionError,
    util::{builder::ModelCollectionBuilder, get_span_from_ast_span},
};

type ValidatedImports = HashMap<PythonPath, IrSpan>;
type ImportErrors = HashMap<PythonPath, ImportResolutionError>;

/// Validates a list of Python import declarations for a given model.
///
/// This function processes import declarations from an Oneil model and validates
/// that the referenced Python files exist. It uses a functional approach with `fold`
/// to accumulate results across all imports.
///
/// # Arguments
///
/// * `model_path` - The path of the model containing the imports
/// * `builder` - A builder for constructing the model collection
/// * `imports` - A vector of import declarations to validate
/// * `file_loader` - A file loader implementation for validating Python imports
///
/// # Returns
///
/// A tuple containing:
/// * `HashSet<WithSpan<PythonPath>>` - Successfully validated Python import paths
/// * `HashMap<PythonPath, ImportResolutionError>` - Failed imports with their errors
/// * `ModelCollectionBuilder<F::ParseError, F::PythonError>` - Updated builder
///
/// # Notes
///
/// - Each import path is converted to a Python path relative to the model's location
/// - Successful imports are added to the returned set of valid Python imports
/// - Failed imports are recorded in both the error map and the builder
pub fn validate_imports<F>(
    model_path: &ModelPath,
    builder: ModelCollectionBuilder<F::ParseError, F::PythonError>,
    imports: Vec<&oneil_ast::declaration::ImportNode>,
    file_loader: &F,
) -> (
    ValidatedImports,
    ImportErrors,
    ModelCollectionBuilder<F::ParseError, F::PythonError>,
)
where
    F: FileLoader,
{
    imports.into_iter().fold(
        (HashMap::new(), HashMap::new(), builder),
        |(mut python_imports, mut import_resolution_errors, mut builder), import| {
            let python_path = model_path.get_sibling_path(import.path().node_value());
            let python_path = PythonPath::new(python_path);
            let python_path_span = get_span_from_ast_span(import.path().node_span());

            // check for duplicate imports
            let original_import_span = python_imports.get(&python_path);
            if let Some(original_import_span) = original_import_span {
                import_resolution_errors.insert(
                    python_path.clone(),
                    ImportResolutionError::duplicate_import(
                        *original_import_span,
                        python_path_span,
                        python_path,
                    ),
                );

                return (python_imports, import_resolution_errors, builder);
            }

            let result = file_loader.validate_python_import(&python_path);
            match result {
                Ok(()) => {
                    python_imports.insert(python_path, python_path_span);
                    (python_imports, import_resolution_errors, builder)
                }
                Err(error) => {
                    builder.add_import_error(python_path.clone(), error);
                    import_resolution_errors.insert(
                        python_path.clone(),
                        ImportResolutionError::failed_validation(python_path_span, python_path),
                    );
                    (python_imports, import_resolution_errors, builder)
                }
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use oneil_ast::declaration::ImportNode;

    use super::*;
    use crate::test::TestPythonValidator;

    mod helper {
        use std::collections::HashSet;

        use oneil_ast::{AstSpan, declaration::Import, node::Node};

        use super::*;

        pub fn get_model_path() -> ModelPath {
            ModelPath::new(PathBuf::from("test_model"))
        }

        pub fn get_empty_builder() -> ModelCollectionBuilder<(), ()> {
            ModelCollectionBuilder::new(HashSet::new())
        }

        pub fn build_import(path: &str) -> ImportNode {
            // for simplicity's sake, we'll use a span that's the length of the path
            let span = AstSpan::new(0, path.len(), 0);
            let import = Import::new(Node::new(&span, path.to_string()));
            Node::new(&span, import)
        }

        pub fn build_import_with_span(
            path: &str,
            start: usize,
            end: usize,
            line: usize,
        ) -> ImportNode {
            let span = AstSpan::new(start, end, line);
            let import = Import::new(Node::new(&span, path.to_string()));
            Node::new(&span, import)
        }
    }

    #[test]
    fn test_validate_imports_empty_list() {
        // set up the context
        let file_loader = TestPythonValidator::validate_all();
        let model_path = helper::get_model_path();
        let builder = helper::get_empty_builder();

        // set up the imports
        let imports = vec![];

        // validate the imports
        let (valid_imports, errors, _builder) =
            validate_imports(&model_path, builder, imports, &file_loader);

        // check the imports
        assert!(valid_imports.is_empty());

        // check the errors
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_imports_single_valid_import() {
        // set up the context
        let file_loader = TestPythonValidator::validate_all();
        let model_path = helper::get_model_path();
        let builder = helper::get_empty_builder();

        // set up the imports
        let imports = [helper::build_import("my_python")];
        let import_refs = imports.iter().collect();

        // validate the imports
        let (valid_imports, errors, _builder) =
            validate_imports(&model_path, builder, import_refs, &file_loader);

        // check the imports
        assert_eq!(valid_imports.len(), 1);

        let valid_path = PythonPath::new(PathBuf::from("my_python"));
        let valid_path_span = get_span_from_ast_span(imports[0].node_span());
        assert_eq!(valid_imports.get(&valid_path), Some(&valid_path_span));

        // check the errors
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_imports_single_invalid_import() {
        // set up the context
        let file_loader = TestPythonValidator::validate_none();
        let model_path = helper::get_model_path();
        let builder = helper::get_empty_builder();

        // set up the imports
        let imports = [helper::build_import("nonexistent")];
        let import_refs = imports.iter().collect();

        // validate the imports
        let (valid_imports, errors, _builder) =
            validate_imports(&model_path, builder, import_refs, &file_loader);

        // check the imports
        assert!(valid_imports.is_empty());

        // check the errors
        assert_eq!(errors.len(), 1);

        let error_path = PythonPath::new(PathBuf::from("nonexistent"));
        let error_path_span = get_span_from_ast_span(imports[0].node_span());
        assert_eq!(
            errors.get(&error_path),
            Some(&ImportResolutionError::failed_validation(
                error_path_span,
                error_path
            ))
        );
    }

    #[test]
    fn test_validate_imports_mixed_valid_and_invalid() {
        // set up the context
        let file_loader = TestPythonValidator::validate_some(vec!["my_python.py".into()]);
        let model_path = helper::get_model_path();
        let builder = helper::get_empty_builder();

        // set up the imports
        let imports = [
            helper::build_import("my_python"),
            helper::build_import("nonexistent"),
        ];
        let import_refs = imports.iter().collect();

        // validate the imports
        let (valid_imports, errors, _builder) =
            validate_imports(&model_path, builder, import_refs, &file_loader);

        // check the imports
        assert_eq!(valid_imports.len(), 1);

        let valid_path = PythonPath::new(PathBuf::from("my_python"));
        let valid_path_span = get_span_from_ast_span(imports[0].node_span());
        assert_eq!(valid_imports.get(&valid_path), Some(&valid_path_span));

        // check the errors
        assert_eq!(errors.len(), 1);

        let error_path = PythonPath::new(PathBuf::from("nonexistent"));
        let error_path_span = get_span_from_ast_span(imports[1].node_span());
        assert_eq!(
            errors.get(&error_path),
            Some(&ImportResolutionError::failed_validation(
                error_path_span,
                error_path
            ))
        );
    }

    #[test]
    fn test_validate_imports_multiple_valid_imports() {
        // set up the context
        let file_loader = TestPythonValidator::validate_all();
        let model_path = helper::get_model_path();
        let builder = helper::get_empty_builder();

        // set up the imports
        let imports = [
            helper::build_import("my_python1"),
            helper::build_import("my_python2"),
            helper::build_import("my_python3"),
        ];
        let import_refs = imports.iter().collect();

        // validate the imports
        let (valid_imports, errors, _builder) =
            validate_imports(&model_path, builder, import_refs, &file_loader);

        // check the imports
        assert_eq!(valid_imports.len(), 3);
        let valid_path1 = PythonPath::new(PathBuf::from("my_python1"));
        let valid_path1_span = get_span_from_ast_span(imports[0].node_span());
        assert_eq!(valid_imports.get(&valid_path1), Some(&valid_path1_span));

        let valid_path2 = PythonPath::new(PathBuf::from("my_python2"));
        let valid_path2_span = get_span_from_ast_span(imports[1].node_span());
        assert_eq!(valid_imports.get(&valid_path2), Some(&valid_path2_span));

        let valid_path3 = PythonPath::new(PathBuf::from("my_python3"));
        let valid_path3_span = get_span_from_ast_span(imports[2].node_span());
        assert_eq!(valid_imports.get(&valid_path3), Some(&valid_path3_span));

        // check the errors
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_imports_all_invalid() {
        // set up the context
        let file_loader = TestPythonValidator::validate_none();
        let model_path = helper::get_model_path();
        let builder = helper::get_empty_builder();

        // set up the imports
        let imports = [
            helper::build_import("nonexistent1"),
            helper::build_import("nonexistent2"),
        ];
        let import_refs = imports.iter().collect();

        // validate the imports
        let (valid_imports, errors, _builder) =
            validate_imports(&model_path, builder, import_refs, &file_loader);

        // check the imports
        assert!(valid_imports.is_empty());

        // check the errors
        assert_eq!(errors.len(), 2);

        let error_path1 = PythonPath::new(PathBuf::from("nonexistent1"));
        let error_path1_span = get_span_from_ast_span(imports[0].node_span());
        assert_eq!(
            errors.get(&error_path1),
            Some(&ImportResolutionError::failed_validation(
                error_path1_span,
                error_path1
            ))
        );

        let error_path2 = PythonPath::new(PathBuf::from("nonexistent2"));
        let error_path2_span = get_span_from_ast_span(imports[1].node_span());
        assert_eq!(
            errors.get(&error_path2),
            Some(&ImportResolutionError::failed_validation(
                error_path2_span,
                error_path2
            ))
        );
    }

    #[test]
    fn test_validate_imports_builder_error_tracking() {
        // set up the context
        let file_loader = TestPythonValidator::validate_none();
        let model_path = helper::get_model_path();
        let builder = helper::get_empty_builder();

        // set up the imports
        let imports = [helper::build_import("nonexistent")];
        let import_refs = imports.iter().collect();

        // validate the imports
        let (_valid_imports, _errors, builder) =
            validate_imports(&model_path, builder, import_refs, &file_loader);

        // check the builder
        assert!(
            builder
                .get_imports_with_errors()
                .contains(&PythonPath::new(PathBuf::from("nonexistent")))
        );
    }

    #[test]
    fn test_validate_imports_path_conversion() {
        // set up the context
        let file_loader = TestPythonValidator::validate_some(vec!["subdir/my_python.py".into()]);
        let model_path = ModelPath::new(PathBuf::from("subdir/test_model"));
        let builder = helper::get_empty_builder();

        // set up the imports
        let imports = [helper::build_import("my_python")];
        let import_refs = imports.iter().collect();

        // validate the imports
        let (valid_imports, errors, _builder) =
            validate_imports(&model_path, builder, import_refs, &file_loader);

        // check the imports
        assert_eq!(valid_imports.len(), 1);

        let valid_path = PythonPath::new(PathBuf::from("subdir/my_python"));
        let valid_path_span = get_span_from_ast_span(imports[0].node_span());
        assert_eq!(valid_imports.get(&valid_path), Some(&valid_path_span));

        // check the errors
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_imports_duplicate_imports() {
        // set up the context
        let file_loader = TestPythonValidator::validate_all();
        let model_path = helper::get_model_path();
        let builder = helper::get_empty_builder();

        // set up the imports with different spans to simulate different positions in the file
        let imports = [
            helper::build_import_with_span("my_python", 0, 9, 1), // first import at line 1
            helper::build_import_with_span("my_python", 10, 19, 2), // duplicate import at line 2
        ];
        let import_refs = imports.iter().collect();

        // validate the imports
        let (valid_imports, errors, _builder) =
            validate_imports(&model_path, builder, import_refs, &file_loader);

        // check the imports - only the first one should be valid
        assert_eq!(valid_imports.len(), 1);

        let valid_path = PythonPath::new(PathBuf::from("my_python"));
        let valid_path_span = get_span_from_ast_span(imports[0].node_span());
        assert_eq!(valid_imports.get(&valid_path), Some(&valid_path_span));

        // check the errors - should have one duplicate import error
        assert_eq!(errors.len(), 1);

        let error_path = PythonPath::new(PathBuf::from("my_python"));
        let duplicate_span = get_span_from_ast_span(imports[1].node_span());
        let expected_error = ImportResolutionError::duplicate_import(
            valid_path_span,
            duplicate_span,
            error_path.clone(),
        );
        assert_eq!(errors.get(&error_path), Some(&expected_error));
    }

    #[test]
    fn test_validate_imports_multiple_duplicate_imports() {
        // set up the context
        let file_loader = TestPythonValidator::validate_all();
        let model_path = helper::get_model_path();
        let builder = helper::get_empty_builder();

        // set up the imports with multiple duplicates
        let imports = [
            helper::build_import_with_span("my_python", 0, 9, 1), // first import
            helper::build_import_with_span("other_python", 10, 22, 2), // different import
            helper::build_import_with_span("my_python", 23, 32, 3), // duplicate of first
            helper::build_import_with_span("other_python", 33, 45, 4), // duplicate of second
        ];
        let import_refs = imports.iter().collect();

        // validate the imports
        let (valid_imports, errors, _builder) =
            validate_imports(&model_path, builder, import_refs, &file_loader);

        // check the imports - only the first occurrence of each should be valid
        assert_eq!(valid_imports.len(), 2);

        let valid_path1 = PythonPath::new(PathBuf::from("my_python"));
        let valid_path1_span = get_span_from_ast_span(imports[0].node_span());
        assert_eq!(valid_imports.get(&valid_path1), Some(&valid_path1_span));

        let valid_path2 = PythonPath::new(PathBuf::from("other_python"));
        let valid_path2_span = get_span_from_ast_span(imports[1].node_span());
        assert_eq!(valid_imports.get(&valid_path2), Some(&valid_path2_span));

        // check the errors - should have two duplicate import errors
        assert_eq!(errors.len(), 2);

        let duplicate_span1 = get_span_from_ast_span(imports[2].node_span());
        let expected_error1 = ImportResolutionError::duplicate_import(
            valid_path1_span,
            duplicate_span1,
            valid_path1.clone(),
        );
        assert_eq!(errors.get(&valid_path1), Some(&expected_error1));

        let duplicate_span2 = get_span_from_ast_span(imports[3].node_span());
        let expected_error2 = ImportResolutionError::duplicate_import(
            valid_path2_span,
            duplicate_span2,
            valid_path2.clone(),
        );
        assert_eq!(errors.get(&valid_path2), Some(&expected_error2));
    }

    #[test]
    fn test_validate_imports_duplicate_imports_with_invalid_imports() {
        // set up the context
        let file_loader = TestPythonValidator::validate_some(vec!["my_python.py".into()]);
        let model_path = helper::get_model_path();
        let builder = helper::get_empty_builder();

        // set up the imports with duplicates and invalid imports
        let imports = [
            helper::build_import_with_span("my_python", 0, 9, 1), // valid import
            helper::build_import_with_span("nonexistent", 10, 21, 2), // invalid import
            helper::build_import_with_span("my_python", 22, 31, 3), // duplicate of first
            helper::build_import_with_span("another_nonexistent", 32, 50, 4), // another invalid import
        ];
        let import_refs = imports.iter().collect();

        // validate the imports
        let (valid_imports, errors, _builder) =
            validate_imports(&model_path, builder, import_refs, &file_loader);

        // check the imports - only the first valid import should be present
        assert_eq!(valid_imports.len(), 1);

        let valid_path = PythonPath::new(PathBuf::from("my_python"));
        let valid_path_span = get_span_from_ast_span(imports[0].node_span());
        assert_eq!(valid_imports.get(&valid_path), Some(&valid_path_span));

        // check the errors - should have 3 errors: 1 duplicate + 2 invalid imports
        assert_eq!(errors.len(), 3);

        // Check duplicate import error
        let duplicate_span = get_span_from_ast_span(imports[2].node_span());
        let expected_duplicate_error = ImportResolutionError::duplicate_import(
            valid_path_span,
            duplicate_span,
            valid_path.clone(),
        );
        assert_eq!(errors.get(&valid_path), Some(&expected_duplicate_error));

        // Check invalid import errors
        let invalid_path1 = PythonPath::new(PathBuf::from("nonexistent"));
        let invalid_span1 = get_span_from_ast_span(imports[1].node_span());
        let expected_invalid_error1 =
            ImportResolutionError::failed_validation(invalid_span1, invalid_path1.clone());
        assert_eq!(errors.get(&invalid_path1), Some(&expected_invalid_error1));

        let invalid_path2 = PythonPath::new(PathBuf::from("another_nonexistent"));
        let invalid_span2 = get_span_from_ast_span(imports[3].node_span());
        let expected_invalid_error2 =
            ImportResolutionError::failed_validation(invalid_span2, invalid_path2.clone());
        assert_eq!(errors.get(&invalid_path2), Some(&expected_invalid_error2));
    }
}
