//! Integration test for `oneil_cli::json_test_report`, exercising the full
//! parse → resolve → eval pipeline against a real fixture, rather than
//! hand-built `output::Model` values.

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use oneil_cli::json_test_report::{self, TestOutcome};
    use oneil_runtime::{CacheReadPolicy, CacheWritePolicy, Runtime};
    use oneil_shared::paths::ModelPath;

    fn fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name)
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

        // Round-trips through `serde_json` without panicking or losing data
        // (this is the exact call the CLI makes for `oneil test --format json`).
        let json = serde_json::to_string_pretty(&report).expect("serialize");
        assert!(json.contains("\"result\": \"fail\""));
    }

    #[test]
    fn build_report_is_a_success_when_every_test_passes() {
        let path = ModelPath::from_path_with_ext(&fixture_path("all_passing.on"));
        let mut runtime = Runtime::new(CacheReadPolicy::Never, CacheWritePolicy::Never);
        let (model_opt, errors) = runtime.eval_model(&path);
        let errors_vec = errors.to_vec();

        let report = json_test_report::build_report(&errors_vec, model_opt, false, false);

        assert!(report.success);
        assert_eq!(report.models[0].passed_count, report.models[0].test_count);
    }
}
