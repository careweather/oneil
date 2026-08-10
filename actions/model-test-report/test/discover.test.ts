import { mkdtemp, mkdir, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";

import { discoverModels } from "../src/oneil.js";

/**
 * Creates a temporary checkout with a `model/` directory containing the
 * given files, then runs discovery against it.
 */
async function discoverInTemp(
  files: Record<string, string>,
  options: { models?: string[]; skip?: string[] } = {},
): Promise<string[]> {
  const cwd = await mkdtemp(path.join(tmpdir(), "oneil-discover-"));
  const modelDir = path.join(cwd, "model");
  await mkdir(modelDir, { recursive: true });
  for (const [name, contents] of Object.entries(files)) {
    await writeFile(path.join(modelDir, name), contents, "utf8");
  }
  return discoverModels(cwd, {
    modelDir: "model",
    models: options.models ?? [],
    skip: options.skip ?? [],
  });
}

describe("discoverModels", () => {
  it("includes .on and .one files that declare a test: block", async () => {
    const found = await discoverInTemp({
      "rover.on": "mass: 1 kg\ntest: mass > 0\n",
      "overlay.one": "design rover\ntest: mass < 100\n",
      "helper.on": "mass: 1 kg\n",
      "notes.txt": "test: not a model\n",
    });

    expect(found).toEqual(["overlay.one", "rover.on"]);
  });

  it("honors skip for both extensions", async () => {
    const found = await discoverInTemp(
      {
        "rover.on": "test: 1 > 0\n",
        "overlay.one": "test: 1 > 0\n",
      },
      { skip: ["overlay.one"] },
    );

    expect(found).toEqual(["rover.on"]);
  });

  it("returns an explicit models list without scanning", async () => {
    const found = await discoverInTemp(
      { "ignored.on": "test: 1 > 0\n" },
      { models: ["custom.one", "other.on"] },
    );

    expect(found).toEqual(["custom.one", "other.on"]);
  });
});
