use oneil_ast as ast;
use oneil_ir as ir;

use crate::{ExternalResolutionContext, ResolutionContext, error::PythonImportResolutionError};

/// Validates a list of Python import declarations for a given model.
pub fn resolve_python_imports<E>(
    model_path: &ir::ModelPath,
    python_imports: Vec<&ast::ImportNode>,
    resolution_context: &mut ResolutionContext<'_, E>,
) where
    E: ExternalResolutionContext,
{
    for import in python_imports {
        let python_path = model_path.get_sibling_path(import.path().as_str());
        let python_path = ir::PythonPath::new(python_path);
        let python_path_span = import.path().span();

        #[cfg(feature = "python")]
        {
            // check for duplicate imports
            let original_import =
                resolution_context.get_python_import_from_active_model(&python_path);

            if let Some(original_import) = original_import {
                resolution_context.add_python_import_error_to_active_model(
                    python_path.clone(),
                    PythonImportResolutionError::duplicate_import(
                        *original_import.import_path_span(),
                        python_path_span,
                        python_path,
                    ),
                );

                continue;
            }

            resolution_context.load_python_import_to_active_model(&python_path, python_path_span);
        }

        #[cfg(not(feature = "python"))]
        {
            resolution_context.add_python_import_error_to_active_model(
                python_path,
                PythonImportResolutionError::python_unsupported(python_path_span),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::test::{
        external_context::TestExternalContext, resolution_context::ResolutionContextBuilder,
        test_ast,
    };

    fn python_path(s: &str) -> ir::PythonPath {
        ir::PythonPath::new(PathBuf::from(s))
    }

    #[test]
    fn resolve_python_imports_empty_list() {
        // build the imports
        let imports: Vec<&ast::ImportNode> = vec![];
        let model_path = ir::ModelPath::new("test_model");

        // build the context
        let active_path = ir::ModelPath::new("test_model");
        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // run the resolution
        resolve_python_imports(&model_path, imports, &mut resolution_context);

        // check the imports
        assert!(
            resolution_context
                .get_active_model_python_imports()
                .is_empty()
        );

        // check the errors
        assert!(
            resolution_context
                .get_active_model_python_import_errors()
                .is_empty()
        );
    }

    #[test]
    fn resolve_python_imports_single_valid_import() {
        // build the imports
        let imports = [test_ast::ImportPythonNodeBuilder::build("my_python")];
        let import_refs: Vec<&ast::ImportNode> = imports.iter().collect();
        let model_path = ir::ModelPath::new("test_model");

        // build the context (external allows "my_python")
        let active_path = ir::ModelPath::new("test_model");
        let mut external = TestExternalContext::new().with_python_imports_ok(["my_python"]);
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // run the resolution
        resolve_python_imports(&model_path, import_refs, &mut resolution_context);

        // check the imports
        let resolved = resolution_context.get_active_model_python_imports();
        assert_eq!(resolved.len(), 1);
        assert!(resolved.contains_key(&python_path("my_python")));

        // check the errors
        assert!(
            resolution_context
                .get_active_model_python_import_errors()
                .is_empty()
        );
    }

    #[test]
    fn resolve_python_imports_single_invalid_import() {
        // build the imports (external allows nothing)
        let imports = [test_ast::ImportPythonNodeBuilder::build("nonexistent")];
        let import_refs: Vec<&ast::ImportNode> = imports.iter().collect();
        let model_path = ir::ModelPath::new("test_model");

        // build the context
        let active_path = ir::ModelPath::new("test_model");
        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // run the resolution
        resolve_python_imports(&model_path, import_refs, &mut resolution_context);

        // check the imports
        assert!(
            resolution_context
                .get_active_model_python_imports()
                .is_empty()
        );

        // check the errors
        let errors = resolution_context.get_active_model_python_import_errors();
        assert_eq!(errors.len(), 1);

        // check the invalid error
        let error_path = python_path("nonexistent");
        let error = errors.get(&error_path).expect("error should be present");
        let PythonImportResolutionError::FailedValidation {
            python_path: error_path_actual,
            ..
        } = error
        else {
            panic!("error should be a failed validation error");
        };
        assert_eq!(error_path_actual, &error_path);
    }

    #[test]
    fn resolve_python_imports_mixed_valid_and_invalid() {
        // build the imports (external allows "my_python" only)
        let imports = [
            test_ast::ImportPythonNodeBuilder::build("my_python"),
            test_ast::ImportPythonNodeBuilder::build("nonexistent"),
        ];
        let import_refs: Vec<&ast::ImportNode> = imports.iter().collect();
        let model_path = ir::ModelPath::new("test_model");

        // build the context
        let active_path = ir::ModelPath::new("test_model");
        let mut external = TestExternalContext::new().with_python_imports_ok(["my_python"]);
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // run the resolution
        resolve_python_imports(&model_path, import_refs, &mut resolution_context);

        // check the imports
        let resolved = resolution_context.get_active_model_python_imports();
        assert_eq!(resolved.len(), 1);
        assert!(resolved.contains_key(&python_path("my_python")));

        // check the errors
        let errors = resolution_context.get_active_model_python_import_errors();
        assert_eq!(errors.len(), 1);

        // check the invalid error
        let error_path = python_path("nonexistent");
        let error = errors.get(&error_path).expect("error should be present");
        let PythonImportResolutionError::FailedValidation {
            python_path: error_path_actual,
            ..
        } = error
        else {
            panic!("error should be a failed validation error");
        };
        assert_eq!(error_path_actual, &error_path);
    }

    #[test]
    fn resolve_python_imports_multiple_valid_imports() {
        // build the imports
        let imports = [
            test_ast::ImportPythonNodeBuilder::build("my_python1"),
            test_ast::ImportPythonNodeBuilder::build("my_python2"),
            test_ast::ImportPythonNodeBuilder::build("my_python3"),
        ];
        let import_refs: Vec<&ast::ImportNode> = imports.iter().collect();
        let model_path = ir::ModelPath::new("test_model");

        // build the context
        let active_path = ir::ModelPath::new("test_model");
        let mut external = TestExternalContext::new().with_python_imports_ok([
            "my_python1",
            "my_python2",
            "my_python3",
        ]);
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // run the resolution
        resolve_python_imports(&model_path, import_refs, &mut resolution_context);

        // check the imports
        let resolved = resolution_context.get_active_model_python_imports();
        assert_eq!(resolved.len(), 3);
        assert!(resolved.contains_key(&python_path("my_python1")));
        assert!(resolved.contains_key(&python_path("my_python2")));
        assert!(resolved.contains_key(&python_path("my_python3")));

        // check the errors
        assert!(
            resolution_context
                .get_active_model_python_import_errors()
                .is_empty()
        );
    }

    #[expect(
        clippy::similar_names,
        reason = "the similar names should be clear from the test order"
    )]
    #[test]
    fn resolve_python_imports_all_invalid() {
        // build the imports
        let imports = [
            test_ast::ImportPythonNodeBuilder::build("nonexistent1"),
            test_ast::ImportPythonNodeBuilder::build("nonexistent2"),
        ];
        let import_refs: Vec<&ast::ImportNode> = imports.iter().collect();
        let model_path = ir::ModelPath::new("test_model");

        // build the context
        let active_path = ir::ModelPath::new("test_model");
        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // run the resolution
        resolve_python_imports(&model_path, import_refs, &mut resolution_context);

        // check the imports
        assert!(
            resolution_context
                .get_active_model_python_imports()
                .is_empty()
        );

        // check the errors
        let errors = resolution_context.get_active_model_python_import_errors();
        assert_eq!(errors.len(), 2);

        // check the first invalid error
        let error_path1 = python_path("nonexistent1");
        let error1 = errors.get(&error_path1).expect("error should be present");
        let PythonImportResolutionError::FailedValidation {
            python_path: actual,
            ..
        } = error1
        else {
            panic!("error should be a failed validation error");
        };
        assert_eq!(actual, &error_path1);

        // check the second invalid error
        let error_path2 = python_path("nonexistent2");
        let error2 = errors.get(&error_path2).expect("error should be present");
        let PythonImportResolutionError::FailedValidation {
            python_path: actual,
            ..
        } = error2
        else {
            panic!("error should be a failed validation error");
        };
        assert_eq!(actual, &error_path2);
    }

    #[test]
    fn resolve_python_imports_error_tracking() {
        // build the imports (invalid)
        let imports = [test_ast::ImportPythonNodeBuilder::build("nonexistent")];
        let import_refs: Vec<&ast::ImportNode> = imports.iter().collect();
        let model_path = ir::ModelPath::new("test_model");

        // build the context
        let active_path = ir::ModelPath::new("test_model");
        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // run the resolution
        resolve_python_imports(&model_path, import_refs, &mut resolution_context);

        // check the context has the error for this path
        let errors = resolution_context.get_active_model_python_import_errors();
        assert!(errors.contains_key(&python_path("nonexistent")));
    }

    #[test]
    fn resolve_python_imports_path_conversion() {
        // build the imports (model in subdir, import "my_python" -> subdir/my_python)
        let imports = [test_ast::ImportPythonNodeBuilder::build("my_python")];
        let import_refs: Vec<&ast::ImportNode> = imports.iter().collect();
        let model_path = ir::ModelPath::new(PathBuf::from("subdir/test_model"));

        // build the context (allow subdir/my_python)
        let active_path = ir::ModelPath::new(PathBuf::from("subdir/test_model"));
        let mut external = TestExternalContext::new().with_python_imports_ok(["subdir/my_python"]);
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // run the resolution
        resolve_python_imports(&model_path, import_refs, &mut resolution_context);

        // check the imports
        let resolved = resolution_context.get_active_model_python_imports();
        assert_eq!(resolved.len(), 1);
        assert!(resolved.contains_key(&python_path("subdir/my_python")));

        // check the errors
        assert!(
            resolution_context
                .get_active_model_python_import_errors()
                .is_empty()
        );
    }

    #[test]
    fn resolve_python_imports_duplicate_imports() {
        // build the imports (same path twice)
        let imports = [
            test_ast::ImportPythonNodeBuilder::build("my_python"),
            test_ast::ImportPythonNodeBuilder::build("my_python"),
        ];
        let import_refs: Vec<&ast::ImportNode> = imports.iter().collect();
        let model_path = ir::ModelPath::new("test_model");

        // build the context
        let active_path = ir::ModelPath::new("test_model");
        let mut external = TestExternalContext::new().with_python_imports_ok(["my_python"]);
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // run the resolution
        resolve_python_imports(&model_path, import_refs, &mut resolution_context);

        // check the imports - only the first one is stored
        let resolved = resolution_context.get_active_model_python_imports();
        assert_eq!(resolved.len(), 1);
        assert!(resolved.contains_key(&python_path("my_python")));

        // check the errors - one duplicate
        let errors = resolution_context.get_active_model_python_import_errors();
        assert_eq!(errors.len(), 1);

        // check the duplicate error
        let duplicate_error = errors
            .get(&python_path("my_python"))
            .expect("duplicate error");
        let PythonImportResolutionError::DuplicateImport {
            python_path: duplicate_error_path,
            ..
        } = duplicate_error
        else {
            panic!("duplicate error should be a duplicate import error");
        };
        assert_eq!(duplicate_error_path, &python_path("my_python"));
    }

    #[test]
    fn resolve_python_imports_multiple_duplicate_imports() {
        // build the imports (two paths, each duplicated)
        let imports = [
            test_ast::ImportPythonNodeBuilder::build("my_python"),
            test_ast::ImportPythonNodeBuilder::build("other_python"),
            test_ast::ImportPythonNodeBuilder::build("my_python"),
            test_ast::ImportPythonNodeBuilder::build("other_python"),
        ];
        let import_refs: Vec<&ast::ImportNode> = imports.iter().collect();
        let model_path = ir::ModelPath::new("test_model");

        // build the context
        let active_path = ir::ModelPath::new("test_model");
        let mut external =
            TestExternalContext::new().with_python_imports_ok(["my_python", "other_python"]);
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // run the resolution
        resolve_python_imports(&model_path, import_refs, &mut resolution_context);

        // check the imports
        let resolved = resolution_context.get_active_model_python_imports();
        assert_eq!(resolved.len(), 2);
        assert!(resolved.contains_key(&python_path("my_python")));
        assert!(resolved.contains_key(&python_path("other_python")));

        // check the errors - two duplicate errors
        let errors = resolution_context.get_active_model_python_import_errors();
        assert_eq!(errors.len(), 2);

        // check the first duplicate error
        let dup1 = errors
            .get(&python_path("my_python"))
            .expect("duplicate error");
        let PythonImportResolutionError::DuplicateImport {
            python_path: path1, ..
        } = dup1
        else {
            panic!("duplicate error expected");
        };
        assert_eq!(path1, &python_path("my_python"));

        // check the second duplicate error
        let dup2 = errors
            .get(&python_path("other_python"))
            .expect("duplicate error");
        let PythonImportResolutionError::DuplicateImport {
            python_path: path2, ..
        } = dup2
        else {
            panic!("duplicate error expected");
        };
        assert_eq!(path2, &python_path("other_python"));
    }

    #[test]
    fn resolve_python_imports_duplicate_imports_with_invalid_imports() {
        // build the imports: valid, invalid, duplicate of valid, another invalid
        let imports = [
            test_ast::ImportPythonNodeBuilder::build("my_python"),
            test_ast::ImportPythonNodeBuilder::build("nonexistent"),
            test_ast::ImportPythonNodeBuilder::build("my_python"),
            test_ast::ImportPythonNodeBuilder::build("another_nonexistent"),
        ];
        let import_refs: Vec<&ast::ImportNode> = imports.iter().collect();
        let model_path = ir::ModelPath::new("test_model");

        // build the context (only "my_python" allowed)
        let active_path = ir::ModelPath::new("test_model");
        let mut external = TestExternalContext::new().with_python_imports_ok(["my_python"]);
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // run the resolution
        resolve_python_imports(&model_path, import_refs, &mut resolution_context);

        // check the imports - only the first valid
        let resolved = resolution_context.get_active_model_python_imports();
        assert_eq!(resolved.len(), 1);
        assert!(resolved.contains_key(&python_path("my_python")));

        // check the errors - 1 duplicate + 2 invalid
        let errors = resolution_context.get_active_model_python_import_errors();
        assert_eq!(errors.len(), 3);

        // check the duplicate error
        let duplicate_error = errors
            .get(&python_path("my_python"))
            .expect("duplicate error");
        let PythonImportResolutionError::DuplicateImport {
            python_path: duplicate_error_path,
            ..
        } = duplicate_error
        else {
            panic!("duplicate error expected");
        };
        assert_eq!(duplicate_error_path, &python_path("my_python"));

        // check the first invalid error
        let invalid1 = errors
            .get(&python_path("nonexistent"))
            .expect("invalid error");
        let PythonImportResolutionError::FailedValidation { python_path: p, .. } = invalid1 else {
            panic!("failed validation expected");
        };
        assert_eq!(p, &python_path("nonexistent"));

        // check the second invalid error
        let invalid2 = errors
            .get(&python_path("another_nonexistent"))
            .expect("invalid error");
        let PythonImportResolutionError::FailedValidation { python_path: p, .. } = invalid2 else {
            panic!("failed validation expected");
        };
        assert_eq!(p, &python_path("another_nonexistent"));
    }
}
