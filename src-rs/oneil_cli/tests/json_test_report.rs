//! Integration test for `oneil_cli::json_test_report`, exercising the full
//! parse → resolve → eval pipeline against a real fixture, rather than
//! hand-built `output::Model` values.

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use oneil_cli::json_test_report::{self, TestOutcome};
    use oneil_runtime::{CacheReadPolicy, CacheWritePolicy, Runtime};
    use oneil_shared::paths::ModelPath;
    use serde_json::json;

    fn fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name)
    }

    /// Replaces absolute `model_path` values with their file names so the
    /// CI JSON contract can be asserted without baking in host paths.
    fn with_portable_model_paths(mut value: serde_json::Value) -> serde_json::Value {
        if let Some(models) = value
            .get_mut("models")
            .and_then(|models| models.as_array_mut())
        {
            for model in models {
                if let Some(path) = model.get("model_path").and_then(|path| path.as_str()) {
                    let file_name = PathBuf::from(path).file_name().map_or_else(
                        || path.to_string(),
                        |name| name.to_string_lossy().into_owned(),
                    );
                    model["model_path"] = json!(file_name);
                }
            }
        }
        value
    }

    #[test]
    fn build_report_reflects_a_mix_of_passing_and_failing_tests() {
        let path = ModelPath::from_path_with_ext(&fixture_path("mixed_tests.on"));
        let mut runtime = Runtime::new(CacheReadPolicy::Never, CacheWritePolicy::Never);
        let (model_opt, errors) = runtime.eval_model(&path);
        let errors_vec = errors.to_vec();

        let report = json_test_report::build_report(&errors_vec, model_opt, false, false);

        assert!(
            !report.success,
            "one test fails, so the report shouldn't be a success"
        );
        assert!(
            report.diagnostics.is_empty(),
            "the fixture has no parse/eval errors"
        );
        assert_eq!(report.models.len(), 1);

        let model_report = &report.models[0];
        assert_eq!(model_report.test_count, 2);
        assert_eq!(model_report.passed_count, 1);
        assert_eq!(model_report.tests.len(), 2);

        let passing = model_report
            .tests
            .iter()
            .find(|entry| entry.expression.as_deref() == Some("f > t"))
            .expect("fixture declares a passing `f > t` test");
        assert_eq!(passing.result, TestOutcome::Pass);
        assert!(
            passing.dependencies.is_empty(),
            "passing tests don't carry debug info"
        );

        let failing = model_report
            .tests
            .iter()
            .find(|entry| entry.expression.as_deref() == Some("f < t"))
            .expect("fixture declares a failing `f < t` test");
        assert_eq!(failing.result, TestOutcome::Fail);
        assert!(
            !failing.dependencies.is_empty(),
            "failing tests should carry their dependency values"
        );
        assert!(
            failing
                .dependencies
                .iter()
                .any(|dependency| dependency.name == "f"),
            "the failing test depends on `f`"
        );

        // Guard the CI-facing wire contract (`oneil test --format json`):
        // serialize exactly as the CLI does, then compare the full schema.
        let actual = with_portable_model_paths(
            serde_json::to_value(&report).expect("test report only contains JSON-safe types"),
        );
        let expected = json!({
            "success": false,
            "diagnostics": [],
            "models": [
                {
                    "model_path": "mixed_tests.on",
                    "test_count": 2,
                    "passed_count": 1,
                    "tests": [
                        {
                            "expression": "f > t",
                            "span": {
                                "start": { "offset": 238, "line": 8, "column": 7 },
                                "end": { "offset": 243, "line": 8, "column": 12 }
                            },
                            "result": "pass",
                            "dependencies": []
                        },
                        {
                            "expression": "f < t",
                            "span": {
                                "start": { "offset": 250, "line": 9, "column": 7 },
                                "end": { "offset": 255, "line": 9, "column": 12 }
                            },
                            "result": "fail",
                            "dependencies": [
                                {
                                    "name": "f",
                                    "value": {
                                        "type": "measured_number",
                                        "value": 49.050_000_000_000_004,
                                        "max": null,
                                        "unit": "N"
                                    }
                                },
                                {
                                    "name": "t",
                                    "value": {
                                        "type": "measured_number",
                                        "value": 10.0,
                                        "max": null,
                                        "unit": "N"
                                    }
                                }
                            ]
                        }
                    ]
                }
            ]
        });
        assert_eq!(actual, expected);
    }

    #[test]
    fn build_report_is_a_success_when_every_test_passes() {
        let path = ModelPath::from_path_with_ext(&fixture_path("all_passing.on"));
        let mut runtime = Runtime::new(CacheReadPolicy::Never, CacheWritePolicy::Never);
        let (model_opt, errors) = runtime.eval_model(&path);
        let errors_vec = errors.to_vec();

        let report = json_test_report::build_report(&errors_vec, model_opt, false, false);

        assert!(report.success);
        assert_eq!(report.models.len(), 1);
        assert_eq!(report.models[0].passed_count, report.models[0].test_count);
    }
}
