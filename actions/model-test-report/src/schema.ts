/**
 * Types and helpers for the JSON produced by `oneil test --format json` (see
 * `oneil_cli::json_test_report` and
 * `docs/decisions/2026-07-28-structured-test-output-for-ci.md`).
 *
 * Wire types come from `oneil-ts-interfaces` (`packages/ts-interfaces`). See
 * `docs/CODING_STANDARDS.md`. This Action does **not** runtime-validate the
 * payload: callers pin `oneil-ref` (and the Action itself) to a known Oneil
 * version.
 */

export type {
  DiagnosticKind,
  EvaluatedValue,
  FloatValue,
  ModelTestReport,
  ReportDiagnostic,
  SourceLocation,
  Span,
  TestDependency,
  TestOutcome,
  TestReport,
  TestReportEntry,
} from "oneil-ts-interfaces";

import type { EvaluatedValue, TestReport } from "oneil-ts-interfaces";

/** Alias kept for call sites that historically mirrored the old `TestValue` name. */
export type TestValue = EvaluatedValue;

/**
 * Parses a raw `oneil test --format json` payload.
 *
 * @throws {Error} if `raw` is not valid JSON.
 */
export function parseTestReport(raw: string, context: string): TestReport {
  try {
    return JSON.parse(raw) as TestReport;
  } catch (cause) {
    throw new Error(
      `${context}: expected JSON from \`oneil test --format json\`, but the output wasn't valid JSON: ${String(cause)}\n---\n${raw}`,
      { cause },
    );
  }
}
