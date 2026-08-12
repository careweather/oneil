/** Renders a {@link Comparison} (see `compare.ts`) as a Markdown report. */

import type { Comparison, ComparisonEntry } from "./compare.js";
import type { FloatValue, ReportDiagnostic, TestReport, TestValue } from "./schema.js";

/** Formats a {@link FloatValue}, preserving special floats (`NaN`, `±Infinity`). */
function formatFloat(value: FloatValue): string {
  if (typeof value === "number") {
    return String(value);
  }
  switch (value.float_special) {
    case "NAN":
      return "NaN";
    case "INFINITY":
      return "Infinity";
    case "NEGATIVE_INFINITY":
      return "-Infinity";
  }
}

/** Formats a {@link TestValue} for display in a report, e.g. `49.05 N` or `[1, 3]`. */
export function formatTestValue(value: TestValue): string {
  switch (value.type) {
    case "boolean":
      return String(value.value);
    case "string":
      return JSON.stringify(value.value);
    case "number":
      return value.max === null ? formatFloat(value.value) : `[${formatFloat(value.value)}, ${formatFloat(value.max)}]`;
    case "measured_number": {
      const unitSuffix = value.unit.length > 0 ? ` ${value.unit}` : "";
      return value.max === null
        ? `${formatFloat(value.value)}${unitSuffix}`
        : `[${formatFloat(value.value)}, ${formatFloat(value.max)}]${unitSuffix}`;
    }
  }
}

function findDependencies(report: TestReport, entry: ComparisonEntry): string[] {
  const model = report.models.find((candidate) => candidate.model_path === entry.modelPath);
  const test = model?.tests.find((candidate) => (candidate.expression ?? "") === entry.expression);
  return (test?.dependencies ?? []).map((dependency) => `\`${dependency.name}\` = ${formatTestValue(dependency.value)}`);
}

function renderEntryList(title: string, entries: ComparisonEntry[], head: TestReport): string[] {
  if (entries.length === 0) {
    return [];
  }

  const lines = [`### ${title}`, ""];
  for (const entry of entries) {
    lines.push(`- **${entry.modelPath}**: \`${entry.expression}\``);
    for (const dependency of findDependencies(head, entry)) {
      lines.push(`  - ${dependency}`);
    }
  }
  lines.push("");
  return lines;
}

function summaryLine(comparison: Comparison): string {
  const total = comparison.entries.filter((entry) => entry.headResult !== null).length;
  const passing = comparison.entries.filter((entry) => entry.headResult === "pass").length;

  if (comparison.hasProblems) {
    return `❌ **${passing}/${total} tests passing** — this run has problems (see below).`;
  }
  return `✅ **${passing}/${total} tests passing**, no regressions.`;
}

/**
 * Renders `comparison` (produced by comparing `head` against `base`, or
 * `head` alone) as a Markdown report suitable for a PR comment or step
 * summary.
 */
export function renderMarkdownReport(
  head: TestReport,
  comparison: Comparison,
  headLabel: string,
  baseLabel: string | null,
): string {
  const lines: string[] = ["## Oneil model test report", ""];

  if (baseLabel !== null) {
    lines.push(`Comparing \`${headLabel}\` against \`${baseLabel}\`.`, "");
  } else {
    lines.push(`Results for \`${headLabel}\` (no base to compare against).`, "");
  }

  lines.push(summaryLine(comparison), "");

  lines.push(...renderEntryList("🔴 Regressions (passed on base, now failing)", comparison.regressed, head));
  lines.push(...renderEntryList("🟢 Fixed (failed on base, now passing)", comparison.fixed, head));
  lines.push(...renderEntryList("🆕 New failing tests", comparison.newFailing, head));
  lines.push(...renderEntryList("🆕 New passing tests", comparison.newPassing, head));

  const stillFailingTitle = baseLabel === null ? "❌ Failing tests" : "❌ Still failing (unchanged from base)";
  lines.push(...renderEntryList(stillFailingTitle, comparison.stillFailing, head));

  if (comparison.removed.length > 0) {
    lines.push(`### 🗑️ Removed tests`, "");
    for (const entry of comparison.removed) {
      lines.push(`- **${entry.modelPath}**: \`${entry.expression}\``);
    }
    lines.push("");
  }

  lines.push(...renderDiagnosticList("⚠️ New diagnostics", comparison.newDiagnostics));

  const stillPresentTitle = baseLabel === null ? "⚠️ Diagnostics" : "⚠️ Still present diagnostics (unchanged from base)";
  lines.push(...renderDiagnosticList(stillPresentTitle, comparison.stillPresentDiagnostics));

  const anySectionRendered =
    comparison.regressed.length > 0 ||
    comparison.fixed.length > 0 ||
    comparison.newFailing.length > 0 ||
    comparison.newPassing.length > 0 ||
    comparison.stillFailing.length > 0 ||
    comparison.removed.length > 0 ||
    comparison.newDiagnostics.length > 0 ||
    comparison.stillPresentDiagnostics.length > 0;

  if (!anySectionRendered) {
    // Nothing else was rendered above; make that explicit rather than
    // leaving a report that's just a title and a summary line.
    lines.push("No changes in test results.", "");
  }

  return lines.join("\n");
}

function renderDiagnosticList(title: string, diagnostics: ReportDiagnostic[]): string[] {
  if (diagnostics.length === 0) {
    return [];
  }

  const lines = [`### ${title}`, ""];
  for (const diagnostic of diagnostics) {
    const location = diagnostic.line === null ? "" : `:${diagnostic.line}:${diagnostic.column ?? ""}`;
    lines.push(`- **${diagnostic.kind}** at \`${diagnostic.path}${location}\`: ${diagnostic.message}`);
  }
  lines.push("");
  return lines;
}
