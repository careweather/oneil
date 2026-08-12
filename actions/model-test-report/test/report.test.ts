import { describe, expect, it } from "vitest";

import { compareTestReports } from "../src/compare.js";
import { formatTestValue, renderMarkdownReport } from "../src/report.js";
import type { TestReport } from "../src/schema.js";

describe("formatTestValue", () => {
  it("formats booleans and strings", () => {
    expect(formatTestValue({ type: "boolean", value: true })).toBe("true");
    expect(formatTestValue({ type: "string", value: "hi" })).toBe('"hi"');
  });

  it("formats a scalar measured number with its unit", () => {
    expect(formatTestValue({ type: "measured_number", value: 49.05, max: null, unit: "N" })).toBe("49.05 N");
  });

  it("formats an interval as a range", () => {
    expect(formatTestValue({ type: "number", value: 1, max: 3 })).toBe("[1, 3]");
  });

  it("formats special float values", () => {
    expect(formatTestValue({ type: "number", value: { float_special: "INFINITY" }, max: null })).toBe("Infinity");
  });
});

describe("renderMarkdownReport", () => {
  const SPAN = { start: { offset: 0, line: 1, column: 1 }, end: { offset: 1, line: 1, column: 2 } };

  function reportWithFailure(): TestReport {
    return {
      success: false,
      diagnostics: [],
      models: [
        {
          model_path: "model/radar.on",
          test_count: 1,
          passed_count: 0,
          tests: [
            {
              expression: "snr >= 10",
              span: SPAN,
              result: "fail",
              dependencies: [{ name: "snr", value: { type: "number", value: 8.2, max: null } }],
            },
          ],
        },
      ],
    };
  }

  it("includes a regression with its dependency values", () => {
    const base: TestReport = { ...reportWithFailure(), success: true, models: [{ ...reportWithFailure().models[0]!, passed_count: 1, tests: [{ ...reportWithFailure().models[0]!.tests[0]!, result: "pass" }] }] };
    const head = reportWithFailure();

    const comparison = compareTestReports(head, base);
    const markdown = renderMarkdownReport(head, comparison, "pr-branch", "main");

    expect(markdown).toContain("Regressions");
    expect(markdown).toContain("snr >= 10");
    expect(markdown).toContain("`snr` = 8.2");
    expect(markdown).toContain("pr-branch");
    expect(markdown).toContain("main");
  });

  it("lists still-failing tests even in single-checkout mode (no base)", () => {
    const head = reportWithFailure();
    const comparison = compareTestReports(head, null);

    const markdown = renderMarkdownReport(head, comparison, "head", null);

    expect(markdown).toContain("Failing tests");
    expect(markdown).toContain("snr >= 10");
  });

  it("says there's nothing to report when nothing changed", () => {
    const head: TestReport = { success: true, diagnostics: [], models: [] };
    const comparison = compareTestReports(head, null);

    const markdown = renderMarkdownReport(head, comparison, "head", null);

    expect(markdown).toContain("No changes in test results.");
    expect(markdown).toContain("no base to compare against");
  });

  it("lists still-present diagnostics that are unchanged from base", () => {
    const diagnostic = {
      kind: "error" as const,
      path: "compass.on",
      message: "expected parameter or test",
      line: 37,
      column: 5,
    };
    const base: TestReport = { success: false, diagnostics: [diagnostic], models: [] };
    const head: TestReport = { success: false, diagnostics: [diagnostic], models: [] };

    const comparison = compareTestReports(head, base);
    const markdown = renderMarkdownReport(head, comparison, "pr-branch", "main");

    expect(markdown).toContain("Still present diagnostics");
    expect(markdown).toContain("compass.on:37:5");
    expect(markdown).toContain("expected parameter or test");
    expect(markdown).not.toContain("New diagnostics");
  });
});
