import { describe, expect, it } from "vitest";

import { parseTestReport } from "../src/schema.js";

describe("parseTestReport", () => {
  it("parses a well-formed report", () => {
    const report = parseTestReport(
      JSON.stringify({
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
                span: { start: { offset: 0, line: 1, column: 1 }, end: { offset: 9, line: 1, column: 10 } },
                result: "fail",
                dependencies: [{ name: "snr", value: { type: "number", value: 8.2, max: null } }],
              },
            ],
          },
        ],
      }),
      "test",
    );

    expect(report.success).toBe(false);
    expect(report.models[0]?.tests[0]?.result).toBe("fail");
  });

  it("throws a descriptive error on invalid JSON", () => {
    expect(() => parseTestReport("not json", "test context")).toThrow(/test context/);
  });

  it("round-trips special float values", () => {
    const report = parseTestReport(
      JSON.stringify({
        success: true,
        diagnostics: [],
        models: [
          {
            model_path: "m.on",
            test_count: 1,
            passed_count: 1,
            tests: [
              {
                expression: "x",
                span: { start: { offset: 0, line: 1, column: 1 }, end: { offset: 1, line: 1, column: 2 } },
                result: "pass",
                dependencies: [{ name: "x", value: { type: "number", value: { float_special: "INFINITY" }, max: null } }],
              },
            ],
          },
        ],
      }),
      "test",
    );

    const value = report.models[0]?.tests[0]?.dependencies[0]?.value;
    expect(value).toEqual({ type: "number", value: { float_special: "INFINITY" }, max: null });
  });
});
