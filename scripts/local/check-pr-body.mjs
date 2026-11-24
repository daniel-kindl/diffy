#!/usr/bin/env node
// Check the body of a pull request: an issue link with a closing keyword, an
// "Evidence" section of level 2 and one "Observability:" line with a value in
// that section. The rules are in the dkstack write-pr skill and the issue rules:
// https://raw.githubusercontent.com/daniel-kindl/dkstack/v1/skills/write-pr/SKILL.md#evidence
// https://raw.githubusercontent.com/daniel-kindl/dkstack/v1/workflow/issues.md
// The issue link needs a closing keyword ("Closes #12"), because the merge
// closes the issue. A link with no keyword does not close the issue.
// This is a project script, not a dkstack copy. dkstack v1.6.0 has no body
// check. Replace this script with the dkstack copy when a dkstack release has one.
// A pull request from a bot account is skipped. See the exception in AGENTS.md.
// The script prints only its diagnostics. It never prints the body or the author.
// Usage in CI: node scripts/local/check-pr-body.mjs
//   (reads the pull_request event from GITHUB_EVENT_PATH)
// Usage locally: node scripts/local/check-pr-body.mjs <body file>

import { readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";
import { FixedError, readEvent, safeMessage } from "./event.mjs";

const KEYWORD = String.raw`\b(?:close[sd]?|fix(?:e[sd])?|resolve[sd]?):?\s+`;
const ISSUE_URL = String.raw`https:\/\/github\.com\/[\w.-]+\/[\w.-]+\/issues\/\d+\/?`;
const ISSUE_REF = String.raw`(?:[\w.-]+\/[\w.-]+)?#\d+`;
// A keyword, then "#12", "owner/repo#12" or an issue URL. linkProse() turns
// a Markdown link to an issue URL into that URL first.
const ISSUE_LINK = new RegExp(`${KEYWORD}(?:${ISSUE_REF}(?![\\w\\]])|${ISSUE_URL}(?![\\w/]))`, "i");
const WHOLE_ISSUE_URL = new RegExp(`^${ISSUE_URL}$`);
// A code span ends in its paragraph, so it does not cross a blank line.
const INLINE_CODE = /(`+)(?:(?!\n[ \t]*\n)[\s\S])*?\1/g;
const MARKDOWN_LINK = /!?\[[^\]]*\]\(\s*<?([^\s)>]*)>?(?:\s+"[^"]*")?\s*\)/g;
const OBSERVABILITY = /^[ \t]*(?:[-*+][ \t]+)?Observability:[ \t]*(.*)$/;
const PLACEHOLDER = /<[^>]*>/;
const ATX = /^ {0,3}(#{1,6})(?:[ \t]+(.*?))?(?:[ \t]+#+)?[ \t]*$/;
const SETEXT = /^ {0,3}(=+|-+)[ \t]*$/;
const LIST_ITEM = /^ {0,3}(?:[-*+]|\d+[.)])(?:[ \t]|$)/;
const FENCE = /^ {0,3}(`{3,}|~{3,})/;
const BLOCK_START = /^ {0,3}(?:>|#{1,6}(?:[ \t]|$))/;

// Remove HTML comments and fenced code blocks. Example lines in them are not part of the body.
function stripExamples(text) {
  const lines = text.replace(/<!--[\s\S]*?-->/g, "").split(/\r?\n/);
  const out = [];
  let fence = null;
  for (const line of lines) {
    const m = FENCE.exec(line);
    if (fence === null && m) {
      fence = m[1];
      out.push("");
      continue;
    }
    if (fence !== null) {
      if (m && m[1][0] === fence[0] && m[1].length >= fence.length && line.trim() === m[1]) fence = null;
      out.push("");
      continue;
    }
    out.push(line);
  }
  return out;
}

// Return the text where the closing reference can be: no inline code, and
// each Markdown link replaced by its destination when that is an issue URL.
// The label of a link is not a closing reference. A link to another page
// is removed.
export function linkProse(lines) {
  return lines
    .join("\n")
    .replace(INLINE_CODE, " ")
    .replace(MARKDOWN_LINK, (_, destination) => (WHOLE_ISSUE_URL.test(destination) ? ` ${destination} ` : " "));
}

const isParagraphLine = (line) =>
  line.trim() !== "" && !ATX.test(line) && !SETEXT.test(line) && !LIST_ITEM.test(line) && !BLOCK_START.test(line);

// Return the headings as { line, level, text }. ATX headings ("## Text") and
// setext headings ("Text" over a line of "=" or "-") count. A "-" line after
// a list item or a blank line is a list or a thematic break, not a heading.
// The text of a setext heading is the whole paragraph over the underline.
export function headings(lines) {
  const result = [];
  for (let i = 0; i < lines.length; i++) {
    const atx = ATX.exec(lines[i]);
    if (atx) {
      result.push({ line: i, level: atx[1].length, text: (atx[2] ?? "").trim() });
      continue;
    }
    const setext = SETEXT.exec(lines[i]);
    if (!setext || i === 0) continue;
    const previous = lines[i - 1];
    if (previous.trim() === "" || LIST_ITEM.test(previous) || BLOCK_START.test(previous)) continue;
    if (/^(?: {4}|\t)/.test(previous) || SETEXT.test(previous)) continue;
    let first = i - 1;
    while (first > 0 && isParagraphLine(lines[first - 1])) first--;
    const text = lines.slice(first, i).map((line) => line.trim()).join(" ");
    result.push({ line: first, end: i, level: setext[1][0] === "=" ? 1 : 2, text });
  }
  return result;
}

// Return the lines of the level 2 "Evidence" section, or null when it is missing.
// The section ends at the next heading of level 1 or 2.
function evidenceLines(lines) {
  const all = headings(lines);
  const index = all.findIndex((h) => h.level === 2 && h.text === "Evidence");
  if (index === -1) return null;
  const start = all[index];
  const end = all.slice(index + 1).find((h) => h.level <= 2);
  // A setext heading ends at its underline.
  const first = (start.end ?? start.line) + 1;
  return lines.slice(first, end ? end.line : lines.length);
}

const observabilityValues = (lines) =>
  lines.map((line) => OBSERVABILITY.exec(line)).filter(Boolean).map((m) => m[1].trim());

// Return the list of problems. An empty list means a valid body.
export function checkPrBody(rawBody, author = "") {
  if (author.endsWith("[bot]")) return [];
  const lines = stripExamples(rawBody ?? "");
  const errors = [];
  if (!ISSUE_LINK.test(linkProse(lines))) {
    errors.push(
      'the body must link an issue with a closing keyword, for example "Closes #12". The merge then closes the issue',
    );
  }
  const evidence = evidenceLines(lines);
  if (evidence === null) {
    errors.push('the body must have an "## Evidence" section');
    return errors;
  }
  const all = observabilityValues(lines);
  const inEvidence = observabilityValues(evidence);
  if (all.length !== 1 || inEvidence.length !== 1) {
    errors.push(
      `the body must have one "Observability:" line, in the Evidence section. It has ${all.length}, and ${inEvidence.length} in the Evidence section`,
    );
  } else if (inEvidence[0] === "" || PLACEHOLDER.test(inEvidence[0])) {
    errors.push('the "Observability:" line must have a value, for example "none, docs only"');
  }
  return errors;
}

// Return { body, author } from a body file, or from the pull_request event.
export function readInput(args, env) {
  if (args.length > 0) return { body: readFileSync(args[0], "utf8"), author: "" };
  const pr = readEvent(env).pull_request;
  if (!pr) throw new FixedError("the event has no pull_request");
  return { body: pr.body ?? "", author: pr.user?.login ?? "" };
}

function main() {
  let input;
  try {
    input = readInput(process.argv.slice(2), process.env);
  } catch (error) {
    console.error(`PR body: ${safeMessage(error, "cannot read the body")}`);
    return 2;
  }
  if (input.author.endsWith("[bot]")) {
    console.log("A bot opened this pull request. The body check is skipped (see AGENTS.md).");
    return 0;
  }
  const errors = checkPrBody(input.body, input.author);
  for (const error of errors) console.error(`PR body: ${error}`);
  if (errors.length > 0) return 1;
  console.log("The PR body links an issue and has the Evidence section with one Observability line.");
  return 0;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  process.exitCode = main();
}
