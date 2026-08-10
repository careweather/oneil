import { describe, expect, it } from "vitest";

import { run } from "../src/process.js";

describe("run", () => {
  it("captures stdout and a zero exit code", async () => {
    const result = await run(process.execPath, ["-e", "process.stdout.write('hi')"], {
      cwd: process.cwd(),
      timeoutMs: 5000,
    });

    expect(result).toEqual({ stdout: "hi", stderr: "", exitCode: 0, timedOut: false });
  });

  it("captures stdout even on a non-zero exit code, instead of throwing", async () => {
    const result = await run(
      process.execPath,
      ["-e", "process.stdout.write('partial'); process.exit(1)"],
      { cwd: process.cwd(), timeoutMs: 5000 },
    );

    expect(result.exitCode).toBe(1);
    expect(result.stdout).toBe("partial");
    expect(result.timedOut).toBe(false);
  });

  it("reports timedOut when the process outlives timeoutMs", async () => {
    const result = await run(process.execPath, ["-e", "setTimeout(() => {}, 5000)"], {
      cwd: process.cwd(),
      timeoutMs: 100,
    });

    expect(result.timedOut).toBe(true);
    expect(result.exitCode).toBeNull();
  });
});
