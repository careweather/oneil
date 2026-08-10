/**
 * Installing `oneil` and running `oneil test --format json` against a
 * directory of model files, aggregated into one {@link TestReport}-shaped
 * result per checkout (head or base).
 *
 * Toolchain setup (Rust, Python) is intentionally *not* this module's job —
 * the calling workflow sets those up with `dtolnay/rust-toolchain` /
 * `actions/setup-python`, exactly as it would for any other Rust/Python CI
 * step. This module only knows how to install a specific `oneil` ref and run
 * its `test` subcommand.
 */

import { readdir, readFile } from "node:fs/promises";
import path from "node:path";

import * as core from "@actions/core";

import { parseTestReport, type ReportDiagnostic, type TestReport } from "./schema.js";
import { run } from "./process.js";

export const ONEIL_REPOSITORY = "https://github.com/careweather/oneil.git";

/** Clones `oneilRef` into `cloneDir` and installs it via its own `install.sh`, unless a matching version is already on `PATH`. */
export async function installOneil(oneilRef: string, cloneDir: string): Promise<void> {
  const installed = await currentOneilVersion();
  const normalizedRef = oneilRef.replace(/^v/, "");

  if (installed !== null && installed.includes(normalizedRef)) {
    core.info(`Using already-installed oneil ${installed}`);
    return;
  }

  core.info(`Installing oneil ${oneilRef}...`);
  const clone = await run("git", ["clone", "--depth", "1", "--branch", oneilRef, ONEIL_REPOSITORY, cloneDir], {
    cwd: process.cwd(),
    timeoutMs: 5 * 60 * 1000,
  });
  if (clone.exitCode !== 0) {
    throw new Error(`failed to clone oneil@${oneilRef}:\n${clone.stderr}`);
  }

  const install = await run("bash", [path.join(cloneDir, "install.sh")], {
    cwd: cloneDir,
    timeoutMs: 15 * 60 * 1000,
  });
  if (install.exitCode !== 0) {
    throw new Error(`failed to install oneil@${oneilRef}:\n${install.stdout}\n${install.stderr}`);
  }

  const version = await currentOneilVersion();
  core.info(`Installed oneil ${version ?? "(unknown version)"}`);
}

async function currentOneilVersion(): Promise<string | null> {
  try {
    const result = await run("oneil", ["--version"], { cwd: process.cwd(), timeoutMs: 10_000 });
    return result.exitCode === 0 ? result.stdout.trim() : null;
  } catch {
    return null;
  }
}

export interface DiscoverOptions {
  /** Directory (relative to `cwd`) to scan for `.on` / `.one` source files. */
  modelDir: string;
  /** Explicit filenames (relative to `modelDir`) to use instead of discovery. */
  models: string[];
  /** Filenames (relative to `modelDir`) to exclude from discovery. Ignored when `models` is non-empty. */
  skip: string[];
}

/** Returns whether `name` is an Oneil model (`.on`) or design (`.one`) source file. */
function isOneilSourceFile(name: string): boolean {
  return name.endsWith(".on") || name.endsWith(".one");
}

/**
 * Resolves which `.on` / `.one` files under `modelDir` to run `oneil test` against.
 *
 * With no explicit `models`, discovers top-level model and design files in
 * `modelDir` that declare at least one `test:` block — most model directories
 * also contain submodel files with no tests of their own, and there's no
 * point spinning up a subprocess for those (`oneil test --recursive`
 * already covers submodel tests reached from an entry-point model). Design
 * files (`.one`) with their own tests are included the same way.
 */
export async function discoverModels(cwd: string, options: DiscoverOptions): Promise<string[]> {
  if (options.models.length > 0) {
    return options.models;
  }

  const modelDirPath = path.join(cwd, options.modelDir);
  const entries = await readdir(modelDirPath, { withFileTypes: true });
  const skip = new Set(options.skip);

  const candidates = entries
    .filter((entry) => entry.isFile() && isOneilSourceFile(entry.name) && !skip.has(entry.name))
    .map((entry) => entry.name)
    .sort();

  const withTests: string[] = [];
  for (const name of candidates) {
    const contents = await readFile(path.join(modelDirPath, name), "utf8");
    if (/^\s*test\s*:/m.test(contents)) {
      withTests.push(name);
    }
  }

  return withTests;
}

/** Runs `oneil test <modelDir>/<model> --recursive --format json` for one model file. */
async function runOneilTestOnModel(
  oneilBinary: string,
  cwd: string,
  modelDir: string,
  model: string,
  timeoutMs: number,
): Promise<TestReport> {
  const modelPath = path.join(modelDir, model);
  const result = await run(oneilBinary, ["test", modelPath, "--recursive", "--format", "json"], {
    cwd,
    timeoutMs,
  });

  if (result.timedOut) {
    return {
      success: false,
      diagnostics: [timeoutDiagnostic(modelPath, timeoutMs)],
      models: [],
    };
  }

  try {
    return parseTestReport(result.stdout, `oneil test ${modelPath}`);
  } catch (cause) {
    // `oneil test` failed before it could even produce JSON (e.g. it
    // crashed, or this is an oneil version that doesn't support
    // `--format json` yet) — surface that as a diagnostic instead of
    // aborting the whole run over one model.
    return {
      success: false,
      diagnostics: [
        {
          kind: "error",
          path: modelPath,
          message: `${String(cause)}\n${result.stderr}`.trim(),
          line: null,
          column: null,
        },
      ],
      models: [],
    };
  }
}

function timeoutDiagnostic(modelPath: string, timeoutMs: number): ReportDiagnostic {
  return {
    kind: "error",
    path: modelPath,
    message: `\`oneil test\` did not complete within ${timeoutMs}ms`,
    line: null,
    column: null,
  };
}

export interface RunAllOptions extends DiscoverOptions {
  oneilBinary: string;
  timeoutMs: number;
}

/**
 * Runs `oneil test --format json` for every discovered/explicit model under
 * `cwd`, aggregating the per-model reports into a single {@link TestReport}.
 */
export async function runAllModels(cwd: string, options: RunAllOptions): Promise<TestReport> {
  const models = await discoverModels(cwd, options);

  if (models.length === 0) {
    core.warning(`No model files with tests found under ${path.join(cwd, options.modelDir)}`);
  }

  const reports: TestReport[] = [];
  for (const model of models) {
    core.info(`Running oneil test ${path.join(options.modelDir, model)}...`);
    reports.push(await runOneilTestOnModel(options.oneilBinary, cwd, options.modelDir, model, options.timeoutMs));
  }

  return {
    success: reports.every((report) => report.success),
    diagnostics: reports.flatMap((report) => report.diagnostics),
    models: reports.flatMap((report) => report.models),
  };
}
