/**
 * Entrypoint for the `model-test-report` action: expects `oneil` already on
 * `PATH` (the composite `action.yml` installs it via `install-oneil`), runs
 * `oneil test --format json` against one or two checkouts, and (when both are
 * given) diffs base vs. head for regressions/fixes.
 *
 * Checking out the model repo(s) and optional Python setup are the calling
 * workflow's job — see `README.md` for a full example.
 */

import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";

import * as core from "@actions/core";

import { compareTestReports } from "./compare.js";
import { ensureOneilAvailable, runAllModels } from "./oneil.js";
import { renderMarkdownReport } from "./report.js";

function splitList(value: string): string[] {
  return value
    .split(",")
    .map((entry) => entry.trim())
    .filter((entry) => entry.length > 0);
}

async function run(): Promise<void> {
  const headDir = core.getInput("head-dir") || ".";
  const baseDir = core.getInput("base-dir") || null;
  const modelDir = core.getInput("model-dir") || "model";
  const models = splitList(core.getInput("models"));
  const skipModels = splitList(core.getInput("skip-models"));
  const timeoutMs = Number(core.getInput("timeout-seconds") || "120") * 1000;
  const reportPath = core.getInput("report-path") || null;
  const failOnProblems = (core.getInput("fail-on-problems") || "true") !== "false";
  const headLabel = core.getInput("head-label") || "head";
  const baseLabel = core.getInput("base-label") || (baseDir === null ? null : "base");

  await ensureOneilAvailable();

  const discoverOptions = { modelDir, models, skip: skipModels };

  core.startGroup(`Running oneil test under ${headDir}`);
  const headReport = await runAllModels(headDir, { ...discoverOptions, oneilBinary: "oneil", timeoutMs });
  core.endGroup();

  const baseReport =
    baseDir === null
      ? null
      : await core.group(`Running oneil test under ${baseDir}`, () =>
          runAllModels(baseDir, { ...discoverOptions, oneilBinary: "oneil", timeoutMs }),
        );

  const comparison = compareTestReports(headReport, baseReport);
  const report = renderMarkdownReport(headReport, comparison, headLabel, baseLabel);

  core.info(report);
  await core.summary.addRaw(report).write();

  if (reportPath !== null) {
    await mkdir(path.dirname(reportPath), { recursive: true });
    await writeFile(reportPath, report, "utf8");
    core.setOutput("report-path", reportPath);
  }

  core.setOutput("has-problems", comparison.hasProblems);
  core.setOutput("report", report);

  if (comparison.hasProblems) {
    const message = `oneil model tests have problems (see the ${reportPath === null ? "step summary" : `report at \`${reportPath}\``} above)`;
    if (failOnProblems) {
      core.setFailed(message);
    } else {
      core.warning(message);
    }
  }
}

run().catch((error: unknown) => {
  core.setFailed(error instanceof Error ? error.message : String(error));
});
