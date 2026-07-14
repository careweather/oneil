#!/usr/bin/env node
/**
 * Run a one-shot Cursor agent that reviews the PR diff against
 * docs/CODING_STANDARDS.md and writes markdown feedback to disk.
 */

import { execFileSync } from "node:child_process";
import { writeFileSync } from "node:fs";
import { Agent, CursorAgentError } from "@cursor/sdk";

/** Paths whose changes warrant a coding-standards agent review. */
const REVIEW_PATH_PREFIXES = ["src-rs/"];

/** Skip the agent (and post a short note) when more than this many reviewable files change. */
const MAX_CHANGED_FILES = 60;

const WORKSPACE = process.env.GITHUB_WORKSPACE ?? process.cwd();
const OUTPUT_PATH =
  process.env.REVIEW_OUTPUT_PATH ?? `${WORKSPACE}/coding-standards-review.md`;

/**
 * Run a git command and return trimmed stdout.
 * @param {string[]} args
 * @returns {string}
 */
function git(args) {
  return execFileSync("git", args, {
    cwd: WORKSPACE,
    encoding: "utf8",
    maxBuffer: 32 * 1024 * 1024,
  }).trim();
}

/**
 * Whether a path should be reviewed against coding standards.
 * @param {string} path
 * @returns {boolean}
 */
function isReviewablePath(path) {
  return REVIEW_PATH_PREFIXES.some(
    (prefix) => path === prefix.slice(0, -1) || path.startsWith(prefix),
  );
}

/**
 * Parse `git diff --name-status` lines into path entries.
 * @param {string} nameStatus
 * @returns {{ status: string, path: string }[]}
 */
function parseNameStatus(nameStatus) {
  return nameStatus
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean)
    .map((line) => {
      const parts = line.split("\t");
      const status = parts[0] ?? "";
      // Renames: R100\told\tnew — review the new path.
      const path = parts.length >= 3 ? parts[parts.length - 1] : (parts[1] ?? "");
      return { status, path };
    })
    .filter((entry) => entry.path.length > 0);
}

/**
 * Collect reviewable changed files for the PR range.
 * @param {string} baseSha
 * @param {string} headSha
 * @returns {{ range: string, changedFiles: string, fileCount: number }}
 */
function collectChangedFiles(baseSha, headSha) {
  const range = `${baseSha}...${headSha}`;
  const entries = parseNameStatus(git(["diff", "--name-status", range])).filter(
    (entry) => isReviewablePath(entry.path),
  );

  const changedFiles = entries
    .map((entry) => `${entry.status}\t${entry.path}`)
    .join("\n");

  return { range, changedFiles, fileCount: entries.length };
}

/**
 * Build the review prompt for the Cursor agent.
 * The agent computes the diff locally; we only pass SHAs and the file list.
 * @param {{ baseSha: string, headSha: string, range: string, changedFiles: string }} payload
 * @returns {string}
 */
function buildPrompt({ baseSha, headSha, range, changedFiles }) {
  return `You are reviewing a pull request for compliance with this repository's coding standards.

## Instructions

1. Read \`docs/CODING_STANDARDS.md\` thoroughly (use the Read tool).
2. Review **only the PR changes** for the files listed below. Do not review unchanged code except when needed for context.
3. Compute the diff yourself in the workspace (cwd is the repo root), for example:
   \`git diff --unified=3 ${range}\` (base \`${baseSha}\`, head \`${headSha}\`).
   You may also \`git show\` / Read individual files. Prefer reading file contents for large or truncated hunks.
4. Give concise, actionable feedback keyed to these sections from the standards:
   - Error handling (recoverable \`Result\` vs unrecoverable \`expect\` / \`assert!\` / \`unreachable!\` / \`panic!\`)
   - TODOs and unimplemented features (\`// TODO\`, \`todo!\`, \`unimplemented!\`)
   - Prefer flat code
   - Prefer readable code over terse code
   - Avoid writing declarative macros
   - Use the type system (newtypes, invalid states unrepresentable)
   - Testing (3-step unit tests, snapshots, property tests) when the diff touches tests
5. Skip nits that CI already enforces: \`cargo fmt\`, \`cargo clippy\`, and whether tests were run.
6. Skip praise and filler. If the diff follows the standards well, say so briefly and list residual issues only.
7. **Do not edit, create, delete, or otherwise modify any files.** This is a read-only review. Do not run shell commands that change the working tree.
8. Your final assistant message must be markdown suitable for a GitHub PR comment, using **exactly** this structure (do **not** include a top-level \`## Coding standards review\` heading — that is added later):

\`\`\`
### Summary
(1–3 sentences)

### Findings
- **Section name** — \`path/to/file.rs\`: issue and suggested fix
(or "No issues found." if clean)

### Notes
(optional short caveats)
\`\`\`

## Changed files (reviewable)

\`\`\`
${changedFiles}
\`\`\`
`;
}

/**
 * Write the sticky-comment body to the output path.
 * @param {string} body Markdown under the top-level heading (Summary / Findings / …)
 */
function writeReview(body) {
  const marker = "<!-- coding-standards-review -->";
  const markdown = `${marker}\n## Coding standards review\n\n${body.trim()}\n`;
  writeFileSync(OUTPUT_PATH, markdown, "utf8");
  console.log(`Wrote review to ${OUTPUT_PATH}`);
}

/**
 * Write a short skip notice instead of running the agent.
 * @param {string} reason
 */
function writeSkip(reason) {
  writeReview(`### Summary\n\n${reason}\n\n### Findings\n\nNo issues found.\n`);
}

async function main() {
  const apiKey = process.env.CURSOR_API_KEY;
  if (!apiKey) {
    console.error("CURSOR_API_KEY is required");
    process.exit(1);
  }

  const baseSha = process.env.PR_BASE_SHA;
  const headSha = process.env.PR_HEAD_SHA;

  if (baseSha == null || headSha == null) {
    console.error("PR_BASE_SHA and PR_HEAD_SHA are required");
    process.exit(1);
  }

  const { range, changedFiles, fileCount } = collectChangedFiles(baseSha, headSha);

  console.log(
    `PR range ${baseSha.slice(0, 7)}...${headSha.slice(0, 7)}: ${fileCount} reviewable file(s)`,
  );

  if (fileCount === 0) {
    console.log("No reviewable changes; skipping agent");
    writeSkip(
      "No reviewable changes in this PR (nothing under `src-rs/`). Agent review skipped.",
    );
    return;
  }

  if (fileCount > MAX_CHANGED_FILES) {
    console.log(
      `Too many reviewable files (${fileCount} > ${MAX_CHANGED_FILES}); skipping agent`,
    );
    writeSkip(
      `This PR touches ${fileCount} reviewable files (limit ${MAX_CHANGED_FILES}). Agent review skipped to control cost; please request a human review or split the PR.`,
    );
    return;
  }

  const prompt = buildPrompt({ baseSha, headSha, range, changedFiles });

  try {
    const result = await Agent.prompt(prompt, {
      apiKey,
      model: { id: "composer-2.5" },
      local: { cwd: WORKSPACE, settingSources: [] },
    });

    console.log(`run id=${result.id} status=${result.status}`);

    if (result.status === "error") {
      console.error("Agent run failed:", result.error ?? result.id);
      process.exit(2);
    }

    if (result.status === "cancelled") {
      console.error("Agent run was cancelled:", result.id);
      process.exit(2);
    }

    const text = result.result?.trim();
    if (!text) {
      console.error("Agent finished with empty result; refusing blank review");
      process.exit(2);
    }

    writeReview(text);
  } catch (err) {
    if (err instanceof CursorAgentError) {
      console.error(
        `Agent startup failed: ${err.message} (retryable=${err.isRetryable})`,
      );
      process.exit(1);
    }
    throw err;
  }
}

main();
