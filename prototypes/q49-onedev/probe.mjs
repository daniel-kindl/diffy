// Throwaway Q49 probe. Never merged.
// Usage: <admin token on stdin> | node probe.mjs <OneDev URL> <result file>
//
// The probe needs an administrator access token on standard input. It never prints a token.
// It creates or reuses the project `diffy-q49-sandbox` and the user `diffy-q49-reviewer`.
// Each run pushes a new branch, opens a new pull request as the administrator, and does each
// review action as the reviewer, with the Q34 marker in each comment body. After each write it
// runs the read call that confirms the write before a retry (ADR 0003). Then it runs the
// negative checks. At the end it deletes the reviewer token that it created.
import { spawnSync } from "node:child_process";
import { chmodSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { homedir, tmpdir } from "node:os";
import { join } from "node:path";
import { randomBytes } from "node:crypto";

const [base, outFile] = process.argv.slice(2);
if (!base || !outFile) throw new Error("usage: probe.mjs <OneDev URL> <result file>");
const adminToken = readFileSync(0, "utf8").trim();
if (!adminToken) throw new Error("no token on standard input");

const PROJECT = "diffy-q49-sandbox";
const REVIEWER = "diffy-q49-reviewer";
const secretDir = join(homedir(), ".config", "diffy-q49");
const runId = new Date().toISOString().replace(/[-:]/g, "").replace(/\..*/, "");
const calls = [];
const rows = [];
const secrets = [adminToken];

const scrub = (s) => secrets.reduce((t, x) => (x ? t.split(x).join("<token>") : t), String(s));

async function api(token, method, path, body, { expect = [200], record = true, raw } = {}) {
  const t0 = performance.now();
  const headers = { Authorization: `Bearer ${token}`, Accept: "application/json" };
  let payload;
  if (raw !== undefined) {
    headers["Content-Type"] = raw.type;
    payload = raw.body;
  } else if (body !== undefined) {
    headers["Content-Type"] = "application/json";
    payload = JSON.stringify(body);
  }
  const res = await fetch(base + path, { method, headers, body: payload });
  const text = await res.text();
  const ms = Math.round(performance.now() - t0);
  let json;
  try {
    json = text ? JSON.parse(text) : null;
  } catch {
    json = text;
  }
  if (record) calls.push({ method, path, status: res.status, ms, ...(expect.includes(res.status) ? {} : { body: scrub(text).slice(0, 400) }) });
  if (!expect.includes(res.status)) {
    const err = new Error(`${method} ${path}: ${res.status} ${scrub(text).slice(0, 400)}`);
    err.status = res.status;
    err.text = scrub(text);
    throw err;
  }
  return json;
}

function git(cwd, args, env = {}) {
  const r = spawnSync("git", args, { cwd, env: { ...process.env, GIT_CONFIG_NOSYSTEM: "1", GIT_CONFIG_GLOBAL: "/dev/null", ...env }, encoding: "utf8" });
  if (r.status !== 0) throw new Error(`git ${args[0]}: ${scrub(r.stderr)}`);
  return r.stdout.trim();
}

function row(action, endpoint, source, live, confirm) {
  rows.push({ action, endpoint, source, live, confirm });
}

const SRC = "https://github.com/theonedev/onedev/blob/v17.2.4/server-core/src/main/java/io/onedev/server/rest/resource/";
const marker = (f, a) => `<!-- diffy:f=${f} a=${a} -->`;

const result = { run: runId, url: base, started: new Date().toISOString() };
let reviewerTokenId = null;
let reviewerToken = null;
try {
  // Version and authentication.
  result.version = await api(adminToken, "GET", "/~api/server/version");
  const me = await api(adminToken, "GET", "/~api/users/me");
  result.admin = { id: me.id, name: me.name };

  // Project.
  let projectId;
  try {
    projectId = await api(adminToken, "GET", `/~api/projects/ids/${PROJECT}`, undefined, { expect: [200] });
  } catch (e) {
    if (e.status !== 404 && e.status !== 406) throw e;
  }
  if (!projectId) {
    projectId = await api(adminToken, "POST", "/~api/projects", {
      name: PROJECT,
      description: "Throwaway project for Diffy map question Q49. Safe to delete.",
      codeManagement: true,
      issueManagement: false,
      packManagement: false,
      gitPackConfig: {},
      codeIndexingSetting: {},
    });
  }
  result.projectId = projectId;

  // Reviewer user, password kept in a local file with mode 600.
  mkdirSync(secretDir, { recursive: true, mode: 0o700 });
  const pwFile = join(secretDir, "reviewer-password");
  let reviewerId;
  try {
    reviewerId = await api(adminToken, "GET", `/~api/users/ids/${REVIEWER}`);
  } catch (e) {
    if (e.status !== 404 && e.status !== 406) throw e;
  }
  if (!reviewerId) {
    const password = randomBytes(18).toString("base64url");
    writeFileSync(pwFile, password + "\n", { mode: 0o600 });
    chmodSync(pwFile, 0o600);
    reviewerId = await api(adminToken, "POST", "/~api/users", {
      type: "ORDINARY",
      name: REVIEWER,
      fullName: "Diffy Q49 reviewer",
      password,
      emailAddress: "diffy-q49-reviewer@example.invalid",
      notifyOwnEvents: false,
    });
  }
  if (!existsSync(pwFile)) result.note = "reviewer password file missing; the UI check cannot log in";
  result.reviewerId = reviewerId;

  // Code Writer role on the project for the reviewer, once.
  const roleId = await api(adminToken, "GET", `/~api/roles/ids/${encodeURIComponent("Code Writer")}`);
  const auths = await api(adminToken, "GET", `/~api/projects/${projectId}/user-authorizations`, undefined, { expect: [200, 404] }).catch(() => null);
  const hasAuth = Array.isArray(auths) && auths.some((a) => (a.userId ?? a.user) === reviewerId);
  if (!hasAuth) {
    await api(adminToken, "POST", "/~api/user-authorizations", { projectId, userId: reviewerId, roleId }, { expect: [200, 406, 409] }).catch(async () => {
      await api(adminToken, "POST", "/~api/user-authorizations", { project: projectId, user: reviewerId, role: roleId }, { expect: [200, 406, 409] });
    });
  }

  // Short-lived reviewer token. Only this process holds its value.
  const expire = new Date(Date.now() + 2 * 3600 * 1000).toISOString();
  const created = await api(adminToken, "POST", "/~api/access-tokens", {
    ownerId: reviewerId,
    description: `diffy-q49 run ${runId}`,
    hasOwnerPermissions: true,
    expireDate: expire,
  });
  reviewerTokenId = created.id;
  reviewerToken = created.value;
  secrets.push(reviewerToken);

  // Repository content: main with one file, and a run branch that edits two lines.
  const work = mkdtempSync(join(process.env.TMPDIR ?? tmpdir(), "q49-git-"));
  const askpass = join(work, "askpass.sh");
  writeFileSync(askpass, '#!/bin/sh\ncase "$1" in Username*) echo "$Q49_USER";; *) echo "$Q49_PASS";; esac\n', { mode: 0o700 });
  const repo = join(work, "repo");
  mkdirSync(repo);
  const genv = { GIT_ASKPASS: askpass, Q49_USER: me.name, Q49_PASS: adminToken, GIT_TERMINAL_PROMPT: "0", GIT_AUTHOR_NAME: "Q49", GIT_AUTHOR_EMAIL: "q49@example.invalid", GIT_COMMITTER_NAME: "Q49", GIT_COMMITTER_EMAIL: "q49@example.invalid" };
  const remote = `${base}/${PROJECT}`;
  git(repo, ["init", "-q", "-b", "main"]);
  const lines = Array.from({ length: 60 }, (_, i) => `fn f${i + 1}() -> u32 { ${i + 1} }`);
  const ls = spawnSync("git", ["ls-remote", remote, "refs/heads/main"], { env: { ...process.env, ...genv }, encoding: "utf8" });
  if (ls.status === 0 && ls.stdout.trim()) {
    git(repo, ["fetch", "-q", remote, "main"], genv);
    git(repo, ["reset", "-q", "--hard", "FETCH_HEAD"]);
  } else {
    mkdirSync(join(repo, "src"));
    writeFileSync(join(repo, "src", "lib.rs"), lines.join("\n") + "\n");
    git(repo, ["add", "."]);
    git(repo, ["commit", "-q", "-m", "base"], genv);
    git(repo, ["push", "-q", remote, "main"], genv);
  }
  const baseCommit = git(repo, ["rev-parse", "HEAD"]);
  const branch = `q49-run-${runId}`;
  git(repo, ["checkout", "-q", "-b", branch]);
  const edited = [...lines];
  edited[9] = "fn f10() -> u32 { 1000 }";
  edited[49] = "fn f50() -> u32 { 5000 }";
  writeFileSync(join(repo, "src", "lib.rs"), edited.join("\n") + "\n");
  git(repo, ["commit", "-q", "-am", `edit for run ${runId}`], genv);
  git(repo, ["push", "-q", remote, branch], genv);
  const headCommit = git(repo, ["rev-parse", "HEAD"]);
  rmSync(work, { recursive: true, force: true });
  result.commits = { base: baseCommit, head: headCommit, branch };

  // Pull request, opened by the administrator, with the reviewer as a reviewer.
  const prId = await api(adminToken, "POST", "/~api/pulls", {
    targetProjectId: projectId,
    sourceProjectId: projectId,
    targetBranch: "main",
    sourceBranch: branch,
    title: `Q49 run ${runId}`,
    description: "Throwaway pull request for Diffy map question Q49.",
    reviewerIds: [reviewerId],
  });
  result.pullRequestId = prId;

  // 1. Authenticate as the reviewer.
  const rme = await api(reviewerToken, "GET", "/~api/users/me");
  row("authenticate", "GET /~api/users/me (Authorization: Bearer <access token>)", `${SRC}UserResource.java#L153`, rme.name === REVIEWER ? "pass" : "fail", "n/a");

  // 2. Get the pull request.
  const pr = await api(reviewerToken, "GET", `/~api/pulls/${prId}`);
  result.pullRequest = { number: pr.number, status: pr.status, keys: Object.keys(pr).sort() };
  row("get a pull request", "GET /~api/pulls/{id}", `${SRC}PullRequestResource.java#L110`, pr.id === prId || pr.number ? "pass" : "fail", "n/a");

  // 3. Inline comment with the marker.
  const attempt = randomBytes(4).toString("hex");
  const m1 = marker("F1", attempt);
  const ctx = { oldCommitHash: baseCommit, newCommitHash: headCommit, pullRequestId: prId };
  const mark = (row0) => ({ commitHash: headCommit, path: "src/lib.rs", range: { fromRow: row0, fromColumn: -1, toRow: row0, toColumn: -1, tabWidth: 1 } });
  const content1 = `**major** correctness: the value of \`f10\` changed.\n\n${m1}`;
  const commentId = await api(reviewerToken, "POST", "/~api/code-comments", { projectId, content: content1, mark: mark(9), compareContext: ctx });
  const list1 = await api(reviewerToken, "GET", `/~api/pulls/${prId}/code-comments`);
  const found1 = list1.filter((c) => typeof c.content === "string" && c.content.includes(m1));
  const one = await api(reviewerToken, "GET", `/~api/code-comments/${commentId}`);
  result.codeComment = { id: commentId, keys: Object.keys(one).sort(), contentEndsWithMarker: one.content.endsWith(m1), mark: one.mark, compareContext: one.compareContext };
  row("inline comment", "POST /~api/code-comments", `${SRC}CodeCommentResource.java#L155`, found1.length === 1 ? "pass" : "fail", `GET /~api/pulls/{id}/code-comments, match the marker in content (found ${found1.length})`);

  // 4. Reply with the marker.
  const m2 = marker("F1", `${attempt}r`);
  const replyId = await api(reviewerToken, "POST", "/~api/code-comment-replies", { commentId, content: `A reply.\n\n${m2}` });
  const replies = await api(reviewerToken, "GET", `/~api/code-comments/${commentId}/replies`);
  const found2 = replies.filter((r) => typeof r.content === "string" && r.content.includes(m2));
  result.reply = { id: replyId, keys: replies[0] ? Object.keys(replies[0]).sort() : [] };
  row("reply", "POST /~api/code-comment-replies", `${SRC}CodeCommentReplyResource.java#L59`, found2.length === 1 ? "pass" : "fail", `GET /~api/code-comments/{id}/replies, match the marker in content (found ${found2.length})`);

  // 5. Resolve, unresolve, resolve again. The state is the `resolved` field of the comment.
  await api(reviewerToken, "POST", `/~api/code-comments/${commentId}/resolve`);
  const r1 = (await api(reviewerToken, "GET", `/~api/code-comments/${commentId}`)).resolved;
  await api(reviewerToken, "POST", `/~api/code-comments/${commentId}/resolve`);
  const r1b = (await api(reviewerToken, "GET", `/~api/code-comments/${commentId}`)).resolved;
  await api(reviewerToken, "POST", `/~api/code-comments/${commentId}/unresolve`);
  const r2 = (await api(reviewerToken, "GET", `/~api/code-comments/${commentId}`)).resolved;
  await api(reviewerToken, "POST", `/~api/code-comments/${commentId}/resolve`);
  const r3 = (await api(reviewerToken, "GET", `/~api/code-comments/${commentId}`)).resolved;
  const repliesAfter = await api(reviewerToken, "GET", `/~api/code-comments/${commentId}/replies`);
  result.resolve = { afterResolve: r1, afterSecondResolve: r1b, afterUnresolve: r2, afterResolveAgain: r3, repliesAfterStatusChanges: repliesAfter.length };
  // A status change with no body adds an empty reply. Try a JSON null body, then a note with a marker.
  const count = async () => (await api(reviewerToken, "GET", `/~api/code-comments/${commentId}/replies`)).length;
  const n0 = await count();
  await api(reviewerToken, "POST", `/~api/code-comments/${commentId}/unresolve`, undefined, { raw: { type: "application/json", body: "null" } });
  const n1 = await count();
  const m4 = marker("F1", `${attempt}s`);
  await api(reviewerToken, "POST", `/~api/code-comments/${commentId}/resolve`, undefined, { raw: { type: "application/json", body: `Resolved.\n\n${m4}` } });
  const afterNote = await api(reviewerToken, "GET", `/~api/code-comments/${commentId}/replies`);
  const noteReplies = afterNote.filter((r) => typeof r.content === "string" && r.content.includes(m4));
  result.resolve.emptyRepliesFromStatusChanges = repliesAfter.filter((r) => !r.content).length;
  result.resolve.repliesAddedByJsonNullBody = n1 - n0;
  result.resolve.jsonNullBodyReplyContent = afterNote.slice(-2, -1)[0]?.content ?? null;
  result.resolve.noteWithMarker = { found: noteReplies.length, content: noteReplies[0]?.content, resolved: (await api(reviewerToken, "GET", `/~api/code-comments/${commentId}`)).resolved };
  row("resolve", "POST /~api/code-comments/{id}/resolve and /unresolve", `${SRC}CodeCommentResource.java#L265`, r1 === true && r1b === true && r2 === false && r3 === true ? "pass" : "fail", "GET /~api/code-comments/{id}, field `resolved`");

  // 6. Pull request comment (not inline) with the marker.
  const m3 = marker("F2", attempt);
  const prCommentId = await api(reviewerToken, "POST", "/~api/pull-request-comments", { requestId: prId, userId: reviewerId, content: `**minor** docs: a finding with no line.\n\n${m3}` }, { expect: [200, 400, 406, 500] })
    .catch((e) => ({ error: e.status }));
  let prc = prCommentId;
  if (typeof prc !== "number") {
    prc = await api(reviewerToken, "POST", "/~api/pull-request-comments", { request: prId, user: reviewerId, content: `**minor** docs: a finding with no line.\n\n${m3}`, date: new Date().toISOString() });
  }
  const prComments = await api(reviewerToken, "GET", `/~api/pulls/${prId}/comments`);
  const found3 = prComments.filter((c) => typeof c.content === "string" && c.content.includes(m3));
  row("pull request comment", "POST /~api/pull-request-comments", `${SRC}PullRequestCommentResource.java#L50`, found3.length === 1 ? "pass" : "fail", `GET /~api/pulls/{id}/comments, match the marker in content (found ${found3.length})`);

  // 7. Approve, then request changes. The state is the reviewer's entry in the reviews.
  const reviewOf = async () => (await api(reviewerToken, "GET", `/~api/pulls/${prId}/reviews`)).find((r) => (r.userId ?? r.user) === reviewerId);
  const before = await reviewOf();
  await api(reviewerToken, "POST", `/~api/pulls/${prId}/approve`);
  const afterApprove = await reviewOf();
  await api(reviewerToken, "POST", `/~api/pulls/${prId}/approve`);
  const afterApprove2 = await reviewOf();
  result.reviews = { keys: afterApprove ? Object.keys(afterApprove).sort() : [], before, afterApprove, afterApprove2 };
  row("approve", "POST /~api/pulls/{id}/approve", `${SRC}PullRequestResource.java#L408`, afterApprove?.status === "APPROVED" ? "pass" : "fail", "GET /~api/pulls/{id}/reviews, the entry of the user, field `status`");
  await api(reviewerToken, "POST", `/~api/pulls/${prId}/request-for-changes`);
  const afterChanges = await reviewOf();
  result.reviews.afterRequestChanges = afterChanges;
  row("request changes", "POST /~api/pulls/{id}/request-for-changes", `${SRC}PullRequestResource.java#L425`, afterChanges?.status === "REQUESTED_FOR_CHANGES" ? "pass" : "fail", "GET /~api/pulls/{id}/reviews, the entry of the user, field `status`");
  const changes = await api(reviewerToken, "GET", `/~api/pulls/${prId}/changes`);
  result.reviewChanges = changes.filter((c) => (c.userId ?? c.user) === reviewerId).map((c) => ({ date: c.date, data: c.data?.["@class"] ?? c.data?.type ?? Object.keys(c.data ?? {}) }));

  // Negative checks.
  const neg = {};
  const tryCall = async (label, token, method, path, body) => {
    try {
      await api(token, method, path, body, { expect: [200] });
      neg[label] = { status: 200 };
    } catch (e) {
      neg[label] = { status: e.status, text: (e.text ?? "").slice(0, 200) };
    }
  };
  await tryCall("submitter approves", adminToken, "POST", `/~api/pulls/${prId}/approve`);
  await tryCall("inline comment outside a hunk (row 30)", reviewerToken, "POST", "/~api/code-comments", { projectId, content: `outside\n\n${marker("F3", attempt)}`, mark: mark(29), compareContext: ctx });
  await tryCall("inline comment on a context row (row 7)", reviewerToken, "POST", "/~api/code-comments", { projectId, content: `context\n\n${marker("F4", attempt)}`, mark: mark(6), compareContext: ctx });
  result.negative = neg;
  result.markers = { m1, m2, m3 };
} catch (e) {
  result.error = scrub(e.stack ?? e);
} finally {
  if (reviewerTokenId) {
    try {
      await api(adminToken, "DELETE", `/~api/access-tokens/${reviewerTokenId}`);
      result.reviewerTokenDeleted = true;
    } catch (e) {
      result.reviewerTokenDeleted = scrub(e.message);
    }
  }
  result.rows = rows;
  result.calls = calls;
  result.finished = new Date().toISOString();
  writeFileSync(outFile, scrub(JSON.stringify(result, null, 2)) + "\n");
  console.log(scrub(JSON.stringify({ version: result.version, rows: rows.map((r) => [r.action, r.live]), negative: result.negative, error: result.error }, null, 2)));
}
