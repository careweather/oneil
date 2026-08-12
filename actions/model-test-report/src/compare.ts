/**
 * Diffs two {@link TestReport}s (head vs. base) by `(model, test expression)`
 * key, to find regressions and fixes rather than just a raw pass/fail count.
 */

import type { ReportDiagnostic, TestOutcome, TestReport, TestReportEntry } from "./schema.js";

export interface TestEntryRef {
  modelPath: string;
  /** The test's source expression, or a synthetic label if it couldn't be read from source. */
  expression: string;
}

export type ComparisonStatus = "regressed" | "fixed" | "stable_pass" | "stable_fail" | "new" | "removed";

export interface ComparisonEntry extends TestEntryRef {
  status: ComparisonStatus;
  headResult: TestOutcome | null;
  baseResult: TestOutcome | null;
}

export interface Comparison {
  headSuccess: boolean;
  /** `null` when no base report was compared against (single-report mode). */
  baseSuccess: boolean | null;
  entries: ComparisonEntry[];
  regressed: ComparisonEntry[];
  fixed: ComparisonEntry[];
  newFailing: ComparisonEntry[];
  newPassing: ComparisonEntry[];
  removed: ComparisonEntry[];
  /** Failing on both base and head (or just failing, in single-report mode): not a regression, but still worth surfacing. */
  stillFailing: ComparisonEntry[];
  /** Diagnostics present on head that weren't present on base (by path + message). */
  newDiagnostics: ReportDiagnostic[];
  /**
   * Diagnostics present on both base and head (by path + message), or — in
   * single-report mode — every head diagnostic. Models that fail to evaluate
   * (parse/resolve errors) produce diagnostics with no test results; without
   * this list they would vanish from the report whenever base has the same
   * breakage.
   */
  stillPresentDiagnostics: ReportDiagnostic[];
  /** Whether this comparison should fail CI: any regression, new failure, or new diagnostic, or (with no base) a plain head failure. */
  hasProblems: boolean;
}

function testKey(ref: TestEntryRef): string {
  return `${ref.modelPath}\u0000${ref.expression}`;
}

/** Flattens a {@link TestReport} into a map keyed by `(model, expression)`. */
function flattenTests(report: TestReport): Map<string, { ref: TestEntryRef; result: TestOutcome }> {
  const flattened = new Map<string, { ref: TestEntryRef; result: TestOutcome }>();

  for (const model of report.models) {
    model.tests.forEach((test: TestReportEntry, index: number) => {
      // Fall back to a positional label so tests with an unreadable
      // expression (see `TestReportEntry.expression`) still get a stable,
      // if less friendly, comparison key instead of colliding on `null`.
      const expression = test.expression ?? `<test #${index + 1} at ${test.span.start.line}:${test.span.start.column}>`;
      const ref: TestEntryRef = { modelPath: model.model_path, expression };
      flattened.set(testKey(ref), { ref, result: test.result });
    });
  }

  return flattened;
}

function diagnosticKey(diagnostic: ReportDiagnostic): string {
  return `${diagnostic.path}\u0000${diagnostic.message}`;
}

/**
 * Compares `head` against `base`. Pass `base: null` to just evaluate `head`
 * on its own (no regression/fix detection, but still a structured result).
 */
export function compareTestReports(head: TestReport, base: TestReport | null): Comparison {
  const headTests = flattenTests(head);
  const entries: ComparisonEntry[] = [];

  if (base === null) {
    for (const { ref, result } of headTests.values()) {
      entries.push({ ...ref, status: result === "pass" ? "stable_pass" : "stable_fail", headResult: result, baseResult: null });
    }

    return finalizeComparison(head.success, null, entries, [], head.diagnostics);
  }

  const baseTests = flattenTests(base);
  const seen = new Set<string>();

  for (const [key, { ref, result: headResult }] of headTests) {
    seen.add(key);
    const baseEntry = baseTests.get(key);

    if (baseEntry === undefined) {
      entries.push({ ...ref, status: "new", headResult, baseResult: null });
      continue;
    }

    entries.push({ ...ref, status: transitionStatus(baseEntry.result, headResult), headResult, baseResult: baseEntry.result });
  }

  for (const [key, { ref, result: baseResult }] of baseTests) {
    if (!seen.has(key)) {
      entries.push({ ...ref, status: "removed", headResult: null, baseResult });
    }
  }

  const baseDiagnosticKeys = new Set(base.diagnostics.map(diagnosticKey));
  const newDiagnostics = head.diagnostics.filter(
    (diagnostic: ReportDiagnostic) => !baseDiagnosticKeys.has(diagnosticKey(diagnostic)),
  );
  const stillPresentDiagnostics = head.diagnostics.filter((diagnostic: ReportDiagnostic) =>
    baseDiagnosticKeys.has(diagnosticKey(diagnostic)),
  );

  return finalizeComparison(head.success, base.success, entries, newDiagnostics, stillPresentDiagnostics);
}

function transitionStatus(baseResult: TestOutcome, headResult: TestOutcome): ComparisonStatus {
  if (baseResult === "pass" && headResult === "fail") return "regressed";
  if (baseResult === "fail" && headResult === "pass") return "fixed";
  return headResult === "pass" ? "stable_pass" : "stable_fail";
}

function finalizeComparison(
  headSuccess: boolean,
  baseSuccess: boolean | null,
  entries: ComparisonEntry[],
  newDiagnostics: ReportDiagnostic[],
  stillPresentDiagnostics: ReportDiagnostic[],
): Comparison {
  const byStatus = (status: ComparisonStatus) => entries.filter((entry) => entry.status === status);

  const regressed = byStatus("regressed");
  const fixed = byStatus("fixed");
  const newFailing = byStatus("new").filter((entry) => entry.headResult === "fail");
  const newPassing = byStatus("new").filter((entry) => entry.headResult === "pass");
  const removed = byStatus("removed");
  const stillFailing = byStatus("stable_fail");

  // Unchanged diagnostics (like stillFailing tests) are surfaced in the
  // report but do not fail CI on their own — only newly introduced ones do.
  const hasProblems =
    regressed.length > 0 ||
    newFailing.length > 0 ||
    newDiagnostics.length > 0 ||
    (baseSuccess === null && !headSuccess);

  return {
    headSuccess,
    baseSuccess,
    entries,
    regressed,
    fixed,
    newFailing,
    newPassing,
    removed,
    stillFailing,
    newDiagnostics,
    stillPresentDiagnostics,
    hasProblems,
  };
}
