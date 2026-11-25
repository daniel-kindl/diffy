// Throwaway Q49 UI check. Never merged.
// Usage: node ui-check.mjs <OneDev URL> <pull request number> <result file> <screenshot prefix> <code comment id>
//
// Logs in to the OneDev web UI as `diffy-q49-reviewer` in headless Chromium, with the password
// that probe.mjs saved in ~/.config/diffy-q49/reviewer-password. It opens the activity page and the
// changes page of the pull request, and the changes page with the code comment open. On each page it checks the visible text and the HTML for the
// Q34 marker, and saves a screenshot.
import { spawn } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { homedir, tmpdir } from "node:os";
import { join } from "node:path";

const [base, prNumber, outFile, shotPrefix, commentId] = process.argv.slice(2);
if (!shotPrefix) throw new Error("usage: ui-check.mjs <OneDev URL> <pull request number> <result file> <screenshot prefix>");
const password = readFileSync(join(homedir(), ".config", "diffy-q49", "reviewer-password"), "utf8").trim();
const shell = "/usr/lib64/chromium-browser/headless_shell";
const profile = mkdtempSync(join(process.env.TMPDIR ?? tmpdir(), "q49-chrome-"));
const port = 9300 + Math.floor(Math.random() * 500);
const chrome = spawn(shell, [`--remote-debugging-port=${port}`, `--user-data-dir=${profile}`, "--no-sandbox", "--window-size=1400,1000", "about:blank"], { stdio: "ignore" });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

let ws;
let seq = 0;
const pending = new Map();
const events = [];
function send(method, params = {}, sessionId) {
  const id = ++seq;
  ws.send(JSON.stringify({ id, method, params, ...(sessionId ? { sessionId } : {}) }));
  return new Promise((resolve, reject) => pending.set(id, { resolve, reject }));
}

const result = { url: base, pullRequest: Number(prNumber), pages: [] };
try {
  let info;
  for (let i = 0; i < 50 && !info; i++) {
    await sleep(200);
    info = await fetch(`http://127.0.0.1:${port}/json/version`).then((r) => r.json()).catch(() => null);
  }
  result.browser = info.Browser;
  ws = new WebSocket(info.webSocketDebuggerUrl);
  await new Promise((r) => ws.addEventListener("open", r));
  ws.addEventListener("message", (m) => {
    const msg = JSON.parse(m.data);
    if (msg.id && pending.has(msg.id)) {
      const p = pending.get(msg.id);
      pending.delete(msg.id);
      msg.error ? p.reject(new Error(JSON.stringify(msg.error))) : p.resolve(msg.result);
    } else events.push(msg);
  });
  const { targetId } = await send("Target.createTarget", { url: "about:blank" });
  const { sessionId } = await send("Target.attachToTarget", { targetId, flatten: true });
  const s = (method, params) => send(method, params, sessionId);
  await s("Page.enable");
  await s("Runtime.enable");
  await s("Emulation.setDeviceMetricsOverride", { width: 1400, height: 1000, deviceScaleFactor: 1, mobile: false });
  const evaluate = async (expression) => (await s("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true })).result.value;
  const go = async (url) => {
    await s("Page.navigate", { url });
    for (let i = 0; i < 100; i++) {
      await sleep(200);
      if ((await evaluate("document.readyState")) === "complete") break;
    }
    await sleep(2500);
  };

  await go(`${base}/~login`);
  await evaluate(`(() => {
    document.querySelector('input[name=userName]').value = 'diffy-q49-reviewer';
    document.querySelector('input[name=password]').value = ${JSON.stringify(password)};
    document.querySelector('input[name=userName]').closest('form').submit();
  })()`);
  await sleep(3000);
  result.loggedIn = !(await evaluate("location.pathname")).startsWith("/~login");

  for (const [name, path] of [["activities", `/diffy-q49-sandbox/~pulls/${prNumber}/activities`], ["changes", `/diffy-q49-sandbox/~pulls/${prNumber}/changes`], ["inline-comment", `/diffy-q49-sandbox/~pulls/${prNumber}/changes?comment=${commentId}`]]) {
    await go(base + path);
    const page = await evaluate(`(() => {
      const text = document.body.innerText;
      const html = document.documentElement.outerHTML;
      return {
        path: location.pathname,
        title: document.title,
        textHasMarker: text.includes('diffy:f='),
        htmlHasMarker: html.includes('diffy:f='),
        htmlHasEscapedMarker: html.includes('&lt;!-- diffy'),
        textHasFinding: text.includes('the value of') || text.includes('a finding with no line') || text.includes('A reply.'),
        findingTexts: ['the value of', 'a finding with no line', 'A reply.'].filter((t) => text.includes(t)),
      };
    })()`);
    const shot = await s("Page.captureScreenshot", { format: "png", captureBeyondViewport: true });
    const file = `${shotPrefix}-${name}.png`;
    writeFileSync(file, Buffer.from(shot.data, "base64"));
    result.pages.push({ name, ...page, screenshot: file.split("/").pop() });
  }
} catch (e) {
  result.error = String(e.stack ?? e);
} finally {
  try {
    ws?.close();
  } catch {}
  chrome.kill("SIGTERM");
  await sleep(500);
  rmSync(profile, { recursive: true, force: true });
  writeFileSync(outFile, JSON.stringify(result, null, 2) + "\n");
  console.log(JSON.stringify(result, null, 2));
}
