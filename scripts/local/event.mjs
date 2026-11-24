// Read the GitHub Actions event from GITHUB_EVENT_PATH. The scripts read the
// pull request title and body from this file, not from the environment, so
// the CI log does not show them.

import { readFileSync } from "node:fs";

export function readEvent(env) {
  if (!env.GITHUB_EVENT_PATH) throw new Error("set GITHUB_EVENT_PATH, or give the input as an argument");
  return JSON.parse(readFileSync(env.GITHUB_EVENT_PATH, "utf8"));
}
