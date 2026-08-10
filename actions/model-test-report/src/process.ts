/**
 * Thin wrapper around `node:child_process.execFile`, used instead of
 * `@actions/exec` for `oneil test` specifically: we need per-call timeouts
 * and we must capture stdout even on a non-zero exit code (which is now the
 * *expected* outcome for a model with failing tests, not an exceptional one
 * — see `docs/decisions/2026-07-28-structured-test-output-for-ci.md`).
 */

import { execFile } from "node:child_process";

export interface ProcessResult {
  stdout: string;
  stderr: string;
  /** `null` if the process was killed (e.g. by `timeoutMs`) rather than exiting normally. */
  exitCode: number | null;
  timedOut: boolean;
}

export interface RunOptions {
  cwd: string;
  timeoutMs: number;
}

/** Runs `command` with `args`, never rejecting on a non-zero exit code. */
export function run(command: string, args: string[], options: RunOptions): Promise<ProcessResult> {
  return new Promise((resolve, reject) => {
    const child = execFile(
      command,
      args,
      { cwd: options.cwd, timeout: options.timeoutMs, maxBuffer: 64 * 1024 * 1024 },
      (error, stdout, stderr) => {
        if (error === null) {
          resolve({ stdout, stderr, exitCode: 0, timedOut: false });
          return;
        }

        // `execFile`'s callback receives an `error` both for a non-zero exit
        // code and for a timeout kill; `killed`/`code` tell us which.
        if (error.killed === true) {
          resolve({ stdout, stderr, exitCode: null, timedOut: true });
          return;
        }

        if (typeof error.code === "number") {
          resolve({ stdout, stderr, exitCode: error.code, timedOut: false });
          return;
        }

        // `error` is an `ExecFileException`, which extends `Error`, but
        // `instanceof` lets us satisfy `prefer-promise-reject-errors`
        // without assuming that invariant holds at runtime.
        reject(error instanceof Error ? error : new Error("execFile failed with a non-Error value", { cause: error }));
      },
    );

    child.on("error", reject);
  });
}
