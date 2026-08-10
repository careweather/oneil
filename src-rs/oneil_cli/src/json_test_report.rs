//! Machine-readable JSON output for `oneil test --format json`.
//!
//! This gives CI tooling (e.g. the `model-test-report` GitHub Action) a
//! stable, structured alternative to parsing the human-readable text output
//! with regular expressions. The top-level *envelope* here ([`TestReport`]
//! and friends) is intentionally independent from `oneil_lsp`'s
//! `RenderedTree` payload (used by the VS Code rendered view): that payload
//! is an internal webview transport that can evolve freely, while this one
//! is a CI-facing contract. Leaf value/diagnostic-kind types are shared with
//! `oneil_lsp` (`oneil_output::EvaluatedValue`,
//! `oneil_shared::error::DiagnosticKind`) rather than re-declared here — see
//! `docs/CODING_STANDARDS.md` (JSON Wire Types).
//!
//! # Example
//!
//! ```json
//! {
//!   "success": false,
//!   "diagnostics": [],
//!   "models": [
//!     {
//!       "model_path": "model/radar.on",
//!       "test_count": 2,
//!       "passed_count": 1,
//!       "tests": [
//!         {
//!           "expression": "gain > 0",
//!           "span": { "start": { "offset": 10, "line": 2, "column": 5 }, "end": { "offset": 18, "line": 2, "column": 13 } },
//!           "result": "pass",
//!           "dependencies": []
//!         },
//!         {
//!           "expression": "snr >= 10",
//!           "span": { "start": { "offset": 40, "line": 4, "column": 5 }, "end": { "offset": 49, "line": 4, "column": 14 } },
//!           "result": "fail",
//!           "dependencies": [ { "name": "snr", "value": { "type": "number", "value": 8.2 } } ]
//!         }
//!       ]
//!     }
//!   ]
//! }
//! ```

use std::{collections::HashMap, path::PathBuf};

use indexmap::IndexSet;
use oneil_runtime::output::{
    self, DebugInfo, EvaluatedValue, Span, TestResult, reference::ModelReference,
};
use oneil_shared::error::{DiagnosticKind, OneilDiagnostic};
use oneil_shared::paths::ModelPath;
use serde::Serialize;

/// Top-level JSON payload for `oneil test --format json`.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "ts-bindings", derive(ts_rs::TS))]
pub struct TestReport {
    /// `true` if there were no error diagnostics and every test in
    /// [`Self::models`] passed.
    pub success: bool,
    /// Parse/resolution/eval diagnostics produced while loading and
    /// evaluating the model, independent of test pass/fail.
    pub diagnostics: Vec<ReportDiagnostic>,
    /// Per-model test results.
    ///
    /// Always contains the target model (even with zero tests). Contains
    /// submodels too when built with `recursive: true`, but only submodels
    /// that declare at least one test.
    pub models: Vec<ModelTestReport>,
}

/// A single diagnostic in [`TestReport::diagnostics`].
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "ts-bindings", derive(ts_rs::TS))]
pub struct ReportDiagnostic {
    /// Whether this diagnostic is fatal (`error`) or informational (`warning`).
    pub kind: DiagnosticKind,
    /// Path of the file the diagnostic occurred in.
    pub path: String,
    /// Human-readable message, without any source snippet or ANSI styling.
    pub message: String,
    /// 1-indexed line number, when the diagnostic has a known source location.
    pub line: Option<usize>,
    /// 1-indexed column number, when the diagnostic has a known source location.
    pub column: Option<usize>,
}

impl From<&OneilDiagnostic> for ReportDiagnostic {
    fn from(diagnostic: &OneilDiagnostic) -> Self {
        let (line, column) = diagnostic.location().map_or((None, None), |location| {
            (Some(location.line()), Some(location.column()))
        });

        Self {
            kind: diagnostic.kind(),
            path: diagnostic.path().display().to_string(),
            message: diagnostic.message().to_string(),
            line,
            column,
        }
    }
}

/// Test results for a single model file.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "ts-bindings", derive(ts_rs::TS))]
pub struct ModelTestReport {
    /// Path of the model file these tests were declared in.
    pub model_path: String,
    /// Total number of tests in this model.
    pub test_count: usize,
    /// Number of tests in this model that passed.
    pub passed_count: usize,
    /// The tests themselves, in source declaration order.
    pub tests: Vec<TestReportEntry>,
}

/// A single evaluated test.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "ts-bindings", derive(ts_rs::TS))]
pub struct TestReportEntry {
    /// The test expression's source text, e.g. `"gain > 0"`.
    ///
    /// `null` if the source file couldn't be re-read to slice the
    /// expression text out (e.g. it was deleted after evaluation).
    pub expression: Option<String>,
    /// Source location of the test expression.
    pub span: Span,
    /// Whether the test passed or failed.
    pub result: TestOutcome,
    /// Dependency values at the time the test was evaluated.
    ///
    /// Always empty for passing tests: only failures carry this debug
    /// information (mirrors the text output's "FAILING TESTS" section).
    pub dependencies: Vec<TestDependency>,
}

/// See [`TestReportEntry::result`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts-bindings", derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
pub enum TestOutcome {
    /// The test passed.
    Pass,
    /// The test failed.
    Fail,
}

/// One named dependency value, e.g. a parameter the test expression read.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "ts-bindings", derive(ts_rs::TS))]
pub struct TestDependency {
    /// The dependency's name, e.g. `"snr"` or, for external dependencies,
    /// `"<parameter>.<reference>"`.
    pub name: String,
    /// The dependency's value at test-evaluation time.
    pub value: EvaluatedValue,
}

/// Builds the full [`TestReport`] for `oneil test --format json`.
///
/// `diagnostics` are the parse/resolution/eval diagnostics from evaluating
/// the model (regardless of whether evaluation produced a model to test).
/// `model` is the evaluated model to test, if evaluation got far enough to
/// produce one. `recursive` mirrors `oneil test --recursive`: when set,
/// submodels that declare tests are included in [`TestReport::models`] too.
#[must_use]
pub fn build_report(
    diagnostics: &[&OneilDiagnostic],
    model: Option<ModelReference<'_>>,
    recursive: bool,
    show_internal_diagnostics: bool,
) -> TestReport {
    let report_diagnostics: Vec<ReportDiagnostic> = diagnostics
        .iter()
        .copied()
        .filter(|diagnostic| show_internal_diagnostics || !diagnostic.is_internal_diagnostic())
        .map(ReportDiagnostic::from)
        .collect();

    let has_error_diagnostic = report_diagnostics
        .iter()
        .any(|diagnostic| diagnostic.kind == DiagnosticKind::Error);

    let mut models = Vec::new();

    if let Some(model_ref) = model {
        let mut visited = IndexSet::new();
        let mut file_cache = HashMap::new();

        collect_model_reports(
            model_ref,
            true,
            recursive,
            &mut visited,
            &mut file_cache,
            &mut models,
        );
    }

    let all_tests_passed = models
        .iter()
        .all(|model| model.passed_count == model.test_count);

    TestReport {
        success: !has_error_diagnostic && all_tests_passed,
        diagnostics: report_diagnostics,
        models,
    }
}

/// Recursively walks `model_ref`, appending one [`ModelTestReport`] per
/// visited model that either is the root (`is_root`) or declares at least
/// one test. Mirrors `print_model_result::print_all_tests`'s traversal.
fn collect_model_reports<'runtime>(
    model_ref: ModelReference<'runtime>,
    is_root: bool,
    recursive: bool,
    visited: &mut IndexSet<&'runtime ModelPath>,
    file_cache: &mut HashMap<PathBuf, String>,
    reports: &mut Vec<ModelTestReport>,
) {
    let model_path = model_ref.path();

    if visited.contains(model_path) {
        return;
    }
    visited.insert(model_path);

    let tests = model_ref.tests();

    if is_root || !tests.is_empty() {
        let test_entries: Vec<TestReportEntry> = tests
            .values()
            .map(|test| build_test_entry(test, file_cache))
            .collect();
        let passed_count = test_entries
            .iter()
            .filter(|entry| entry.result == TestOutcome::Pass)
            .count();

        reports.push(ModelTestReport {
            model_path: model_path.as_path().display().to_string(),
            test_count: test_entries.len(),
            passed_count,
            tests: test_entries,
        });
    }

    if recursive {
        for reference in model_ref.references().values() {
            collect_model_reports(*reference, false, recursive, visited, file_cache, reports);
        }
    }
}

fn build_test_entry(
    test: &output::Test,
    file_cache: &mut HashMap<PathBuf, String>,
) -> TestReportEntry {
    let expression = slice_expression(&test.expr_span, file_cache);

    match &test.result {
        TestResult::Passed => TestReportEntry {
            expression,
            span: test.expr_span.clone(),
            result: TestOutcome::Pass,
            dependencies: Vec::new(),
        },
        TestResult::Failed { debug_info } => TestReportEntry {
            expression,
            span: test.expr_span.clone(),
            result: TestOutcome::Fail,
            dependencies: collect_dependencies(debug_info),
        },
    }
}

/// Slices the test expression's source text out of its containing file,
/// caching file reads across tests/models. Returns `None` if the file can't
/// be read or the span's offsets fall outside its current contents.
fn slice_expression(span: &Span, file_cache: &mut HashMap<PathBuf, String>) -> Option<String> {
    let path = span.path().to_path_buf();
    // Soft-fail on I/O errors the same as a missing span: both yield
    // `expression: null` rather than failing the whole report.
    let contents = file_cache
        .entry(path.clone())
        .or_insert_with(|| std::fs::read_to_string(&path).unwrap_or_default());

    contents
        .get(span.start().offset..span.end().offset)
        .map(str::to_string)
}

fn collect_dependencies(debug_info: &DebugInfo) -> Vec<TestDependency> {
    let builtin = debug_info
        .builtin_dependency_values
        .iter()
        .map(|(name, value)| TestDependency {
            name: name.as_str().to_string(),
            value: EvaluatedValue::from(value),
        });

    let parameter = debug_info
        .parameter_dependency_values
        .iter()
        .map(|(name, value)| TestDependency {
            name: name.as_str().to_string(),
            value: EvaluatedValue::from(value),
        });

    let external = debug_info.external_dependency_values.iter().map(
        |((reference_name, parameter_name), value)| TestDependency {
            name: format!("{}.{}", parameter_name.as_str(), reference_name.as_str()),
            value: EvaluatedValue::from(value),
        },
    );

    builtin.chain(parameter).chain(external).collect()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use oneil_shared::error::{AsOneilDiagnostic, DiagnosticKind, OneilDiagnostic};

    use super::*;

    // Value-serialization behavior (scalars vs. intervals, tagging, etc.) is
    // covered by `oneil_output::evaluated_value`'s own tests now that
    // `EvaluatedValue` is shared rather than redefined here. Full model/test
    // traversal is covered by `oneil_cli/tests/json_test_report.rs`.

    struct TestDiagnostic {
        kind: DiagnosticKind,
        message: &'static str,
        is_internal: bool,
    }

    impl AsOneilDiagnostic for TestDiagnostic {
        fn kind(&self) -> DiagnosticKind {
            self.kind
        }

        fn message(&self) -> String {
            self.message.to_string()
        }

        fn is_internal_diagnostic(&self) -> bool {
            self.is_internal
        }
    }

    /// Builds an [`OneilDiagnostic`] for unit tests of report filtering.
    fn diagnostic(
        kind: DiagnosticKind,
        message: &'static str,
        is_internal: bool,
    ) -> OneilDiagnostic {
        OneilDiagnostic::from_error(
            &TestDiagnostic {
                kind,
                message,
                is_internal,
            },
            PathBuf::from("fixture.on"),
        )
    }

    #[test]
    fn build_report_hides_internal_diagnostics_by_default() {
        let user_error = diagnostic(DiagnosticKind::Error, "user-facing", false);
        let internal = diagnostic(DiagnosticKind::Error, "internal", true);
        let warning = diagnostic(DiagnosticKind::Warning, "warn", false);
        let diagnostics = [&user_error, &internal, &warning];

        let report = build_report(&diagnostics, None, false, false);

        assert!(
            !report.success,
            "a non-internal error diagnostic should make the report fail"
        );
        assert_eq!(report.diagnostics.len(), 2);
        assert!(
            report
                .diagnostics
                .iter()
                .all(|entry| entry.message != "internal"),
            "internal diagnostics should be filtered out"
        );
        assert!(report.models.is_empty());
    }

    #[test]
    fn build_report_includes_internal_diagnostics_when_requested() {
        let user_error = diagnostic(DiagnosticKind::Error, "user-facing", false);
        let internal = diagnostic(DiagnosticKind::Error, "internal", true);
        let diagnostics = [&user_error, &internal];

        let report = build_report(&diagnostics, None, false, true);

        assert!(!report.success);
        assert_eq!(report.diagnostics.len(), 2);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|entry| entry.message == "internal")
        );
    }

    #[test]
    fn build_report_is_success_when_only_warnings_are_present() {
        let warning = diagnostic(DiagnosticKind::Warning, "warn", false);
        let report = build_report(&[&warning], None, false, false);

        assert!(
            report.success,
            "warnings alone shouldn't make the report fail"
        );
        assert_eq!(report.diagnostics.len(), 1);
        assert_eq!(report.diagnostics[0].kind, DiagnosticKind::Warning);
    }
}
