import { describe, expect, it } from "vitest";

import { compareTestReports } from "../src/compare.js";
import type { TestOutcome, TestReport } from "../src/schema.js";

const SPAN = { start: { offset: 0, line: 1, column: 1 }, end: { offset: 1, line: 1, column: 2 } };

function report(models: Array<{ path: string; tests: Array<{ expr: string; result: TestOutcome }> }>): TestReport {
  return {
    success: models.every((model) => model.tests.every((test) => test.result === "pass")),
    diagnostics: [],
    models: models.map((model) => ({
      model_path: model.path,
      test_count: model.tests.length,
      passed_count: model.tests.filter((test) => test.result === "pass").length,
      tests: model.tests.map((test) => ({
        expression: test.expr,
        span: SPAN,
        result: test.result,
        dependencies: [],
      })),
    })),
  };
}

describe("compareTestReports", () => {
  it("reports its own results with no problems when there's no base", () => {
    const head = report([{ path: "m.on", tests: [{ expr: "a > 0", result: "pass" }] }]);

    const comparison = compareTestReports(head, null);

    expect(comparison.baseSuccess).toBeNull();
    expect(comparison.hasProblems).toBe(false);
    expect(comparison.entries).toEqual([
      { modelPath: "m.on", expression: "a > 0", status: "stable_pass", headResult: "pass", baseResult: null },
    ]);
  });

  it("flags a head-only failure as a problem when there's no base", () => {
    const head = report([{ path: "m.on", tests: [{ expr: "a > 0", result: "fail" }] }]);

    const comparison = compareTestReports(head, null);

    expect(comparison.hasProblems).toBe(true);
  });

  it("detects a regression (passed on base, fails on head)", () => {
    const base = report([{ path: "m.on", tests: [{ expr: "a > 0", result: "pass" }] }]);
    const head = report([{ path: "m.on", tests: [{ expr: "a > 0", result: "fail" }] }]);

    const comparison = compareTestReports(head, base);

    expect(comparison.hasProblems).toBe(true);
    expect(comparison.regressed).toHaveLength(1);
    expect(comparison.regressed[0]?.expression).toBe("a > 0");
    expect(comparison.fixed).toHaveLength(0);
  });

  it("detects a fix (failed on base, passes on head) and doesn't flag it as a problem", () => {
    const base = report([{ path: "m.on", tests: [{ expr: "a > 0", result: "fail" }] }]);
    const head = report([{ path: "m.on", tests: [{ expr: "a > 0", result: "pass" }] }]);

    const comparison = compareTestReports(head, base);

    expect(comparison.hasProblems).toBe(false);
    expect(comparison.fixed).toHaveLength(1);
    expect(comparison.regressed).toHaveLength(0);
  });

  it("treats a newly added failing test as a problem, distinct from a regression", () => {
    const base = report([{ path: "m.on", tests: [] }]);
    const head = report([{ path: "m.on", tests: [{ expr: "b < 0", result: "fail" }] }]);

    const comparison = compareTestReports(head, base);

    expect(comparison.hasProblems).toBe(true);
    expect(comparison.newFailing).toHaveLength(1);
    expect(comparison.regressed).toHaveLength(0);
  });

  it("tracks removed tests without treating them as a problem", () => {
    const base = report([{ path: "m.on", tests: [{ expr: "c == 1", result: "pass" }] }]);
    const head = report([{ path: "m.on", tests: [] }]);

    const comparison = compareTestReports(head, base);

    expect(comparison.hasProblems).toBe(false);
    expect(comparison.removed).toHaveLength(1);
  });

  it("tracks a test failing on both base and head as stillFailing, not a regression or a problem", () => {
    const base = report([{ path: "m.on", tests: [{ expr: "a > 0", result: "fail" }] }]);
    const head = report([{ path: "m.on", tests: [{ expr: "a > 0", result: "fail" }] }]);

    const comparison = compareTestReports(head, base);

    expect(comparison.hasProblems).toBe(false);
    expect(comparison.stillFailing).toHaveLength(1);
    expect(comparison.regressed).toHaveLength(0);
  });

  it("tracks a head-only failure as stillFailing too, when there's no base", () => {
    const head = report([{ path: "m.on", tests: [{ expr: "a > 0", result: "fail" }] }]);

    const comparison = compareTestReports(head, null);

    expect(comparison.stillFailing).toHaveLength(1);
  });

  it("flags a new diagnostic on head as a problem, even with all tests passing", () => {
    const base = report([{ path: "m.on", tests: [{ expr: "a > 0", result: "pass" }] }]);
    const head: TestReport = {
      ...report([{ path: "m.on", tests: [{ expr: "a > 0", result: "pass" }] }]),
      diagnostics: [{ kind: "warning", path: "m.on", message: "deprecated unit", line: 3, column: 1 }],
    };

    const comparison = compareTestReports(head, base);

    expect(comparison.hasProblems).toBe(true);
    expect(comparison.newDiagnostics).toHaveLength(1);
    expect(comparison.stillPresentDiagnostics).toHaveLength(0);
  });

  it("tracks a diagnostic present on both base and head as stillPresent, not a problem", () => {
    const diagnostic = { kind: "error" as const, path: "compass.on", message: "expected parameter or test", line: 37, column: 5 };
    const base: TestReport = {
      ...report([{ path: "compass.on", tests: [] }]),
      success: false,
      diagnostics: [diagnostic],
    };
    const head: TestReport = {
      ...report([{ path: "compass.on", tests: [] }]),
      success: false,
      diagnostics: [diagnostic],
    };

    const comparison = compareTestReports(head, base);

    expect(comparison.hasProblems).toBe(false);
    expect(comparison.newDiagnostics).toHaveLength(0);
    expect(comparison.stillPresentDiagnostics).toHaveLength(1);
    expect(comparison.stillPresentDiagnostics[0]?.path).toBe("compass.on");
  });

  it("lists head diagnostics as stillPresent when there's no base", () => {
    const head: TestReport = {
      ...report([{ path: "altimeter.on", tests: [] }]),
      success: false,
      diagnostics: [{ kind: "error", path: "altimeter.on", message: "parameter AntElOff is not defined", line: 40, column: 66 }],
    };

    const comparison = compareTestReports(head, null);

    expect(comparison.stillPresentDiagnostics).toHaveLength(1);
    expect(comparison.newDiagnostics).toHaveLength(0);
  });
});
