// Read the GitHub Actions event from GITHUB_EVENT_PATH. The scripts read the
// pull request title and body from this file, not from the environment, so
// the CI log does not show them.
// Each error has a fixed message. No message contains text from the event.

import { readFileSync } from "node:fs";

// An error whose message is fixed text, safe to print in the CI log.
export class FixedError extends Error {}

export function readEvent(env) {
  if (!env.GITHUB_EVENT_PATH) {
    throw new FixedError("set GITHUB_EVENT_PATH, or give the input as an argument");
  }
  let text;
  try {
    text = readFileSync(env.GITHUB_EVENT_PATH, "utf8");
  } catch {
    throw new FixedError("cannot read the event file in GITHUB_EVENT_PATH");
  }
  try {
    return JSON.parse(text);
  } catch {
    throw new FixedError("the event file in GITHUB_EVENT_PATH is not valid JSON");
  }
}

// Return a message that is safe to print for any error.
export function safeMessage(error, fallback) {
  return error instanceof FixedError ? error.message : fallback;
}
