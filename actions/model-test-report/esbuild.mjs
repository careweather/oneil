/**
 * Bundles the action into a single CommonJS file (`dist/index.js`), the
 * standard packaging for a `runs: using: node24` GitHub Action so consumers
 * don't need to run `npm install` themselves.
 *
 * Mirrors `vscode/esbuild.mjs`'s structure.
 */

import esbuild from "esbuild";

const production = process.argv.includes("--production");

await esbuild.build({
  entryPoints: ["src/main.ts"],
  bundle: true,
  // `.cjs`, not `.js`: this package is `"type": "module"` for the TS
  // source, but the bundle is CommonJS (esbuild's `format: "cjs"` below) —
  // Node picks module type from file extension over package.json for `.cjs`,
  // so this runs correctly regardless of the nearest package.json's `type`.
  outfile: "dist/index.cjs",
  platform: "node",
  target: "node24",
  format: "cjs",
  sourcemap: !production,
  minify: production,
  logLevel: "info",
});
