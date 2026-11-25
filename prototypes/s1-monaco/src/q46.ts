// Throwaway S8 scale checks for map question Q46. Never merged.
// Runs when DIFFY_Q46 names the parts: list, cancel-stats, cancel-open. Option B only (ADR 0004).

import * as monaco from "monaco-editor/editor";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Block, BView } from "./main";

type Api = {
  applyB: (l: monaco.editor.ICodeEditor, r: monaco.editor.ICodeEditor, blocks: Block[]) => BView;
  syncScroll: (a: monaco.editor.ICodeEditor, b: monaco.editor.ICodeEditor) => monaco.IDisposable[];
  padForScrollbar: (a: monaco.editor.ICodeEditor, b: monaco.editor.ICodeEditor) => monaco.IDisposable[];
  editorOptions: monaco.editor.IEditorOptions;
  percentile: (values: number[], p: number) => number | null;
};
type Config = { parts: string; label: string; fixtures: string; runs: number; threads: number };
type Repo = { name: string; repo: string; base_files: number; base_bytes: number; changed_files: number; shortstat: string };
type Item = { s: string; p: string; o?: string };
type StatsBatch = { id: number; token: number; items: [number, number, number][] };
type StatsDone = { id: number; token: number; cancelled: boolean; counted: number; files: number; threads: number; ms: number; error: string | null };

const METHOD = [
  "Option B only, the renderer of ADR 0004. Times are performance.now() in the page. The window is the Q54 window.",
  "Fixture repositories: fixtures/gen-repos.mjs writes bare repositories with git fast-import. The comparison is the tree of branch base to the tree of branch head.",
  "Change list: the Rust command opens the repository with gix as ADR 0005 says (isolated, no filter section), and diffs the two trees with rename detection (gix Rewrites::default()). It returns one text line for each changed file. The page parses the lines and renders a virtual list: only the visible rows are in the DOM.",
  "list_ms: from the request to two animation frames after the list shows its first rows. Each run makes a new comparison; the Rust side drops the earlier one. Run 1 is in the summary, and the p95 is over each run.",
  "Line counts: after the list shows, a pool of DIFFY_Q46_THREADS worker threads (default 4) counts the added and removed lines of each file with the canonical diff (gix-imara-diff 0.3.4 with the Q60 patch, histogram, postprocess_lines). A sender thread emits a batch event each 50 ms. The page updates the visible rows in the next animation frame. stats_ms is from the start to the done event.",
  "Line count check: the sums of the counts must equal the insertions and deletions of `git diff --shortstat --histogram -M base head`.",
  "List scroll: setScrollTop on the list in each animation frame, a fixed step of 400 px, from the top to the end. A long frame is more than 1.5 times the median interval.",
  "File open: 3 files (the first, the middle and the last of the list) open in option B: two q46_blob calls and one q46_blocks call, then two Monaco models, two editors, the decorations and view zones of applyB. open_ms is to two animation frames after the editors are ready.",
  "Memory: the Q54 sampler reads the RSS of the process tree each 50 ms. The list phase has three samplers, one after the other: the change list runs, the line counts and the list scroll. RSS that the allocator keeps after a phase counts in the next phase too. The open phase is the 3 file opens. The Rust process is diffy-s1-monaco, the WebView is WebKitWebProcess. Each fixture runs in a new app process.",
  "Bounded memory: files-1000 and files-1000-large have the same 1,000 changed files and 2,000 files in each tree. Each file of the large fixtures has 50 times the lines. files-2000-large has twice the files of files-1000-large, at the same size. If the list phase holds no file content, its peak memory does not grow from files-1000-large to files-2000-large. The sampler records the RSS, and its anonymous (RssAnon) and file-backed (RssFile) parts.",
  "DIFFY_Q46_RENAMES=0 turns off rename detection. CPU: utime + stime of the process tree from /proc, for each phase. A stall is a timer callback (each 10 ms) that runs more than 50 ms late, or a gap of more than 50 ms between animation frames.",
  "cancel-stats: files-5000. The line counts start, and 300 ms later a cancel action runs. ack_ms is from the planned time of the cancel action to two animation frames after the list shows the cancelled state. The page does not wait for the q46_cancel command (async in Rust); rust_ack_ms is when that command returns. stop_ms is to the done event: the workers stop before their next file. A batch that arrives after the cancel is not applied.",
  "cancel-open: stress-50mb. The 50 MB file opens once in full. Then it opens again, and a cancel action is planned at 5, 30, 60, 75 or 90 % of the full open time. ack_ms is from the planned time to two animation frames after the page shows the cancelled state, with no wait for q46_cancel (rust_ack_ms). The open checks the cancel flag after each step and disposes what it made. The Rust q46_blocks diff cannot stop in the middle; the token drops its result.",
];

const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;
const now = () => performance.now();
const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));
const frames = () => new Promise<void>((r) => requestAnimationFrame(() => requestAnimationFrame(() => r())));
const status = (text: string) => { $("status").textContent = text; };

// Stalls: timer callbacks that run late, and gaps between animation frames.
class Stalls {
  timer: number[] = [];
  frames: number[] = [];
  private stop = false;
  start() {
    let expected = now() + 10;
    const tick = () => {
      if (this.stop) return;
      const t = now();
      const late = t - expected;
      if (late > 50) this.timer.push(late);
      expected = t + 10;
      setTimeout(tick, 10);
    };
    setTimeout(tick, 10);
    let last = now();
    const frame = (t: number) => {
      if (this.stop) return;
      if (t - last > 50) this.frames.push(t - last);
      last = t;
      requestAnimationFrame(frame);
    };
    requestAnimationFrame(frame);
  }
  take() {
    const d = (v: number[]) => ({ count: v.length, total_ms: v.reduce((a, b) => a + b, 0), max_ms: v.length ? Math.max(...v) : 0 });
    const out = { timer: d(this.timer), frames: d(this.frames) };
    this.timer = []; this.frames = [];
    return out;
  }
}

// A virtual list: only the visible rows are DOM nodes.
const ROW = 20;
class ChangeList {
  el = document.createElement("div");
  private inner = document.createElement("div");
  private pool: HTMLDivElement[] = [];
  items: Item[] = [];
  added = new Int32Array(0);
  removed = new Int32Array(0);
  state = "";
  private queued = false;
  constructor(parent: HTMLElement) {
    this.el.className = "q46-list";
    this.inner.className = "q46-inner";
    this.el.append(this.inner);
    parent.append(this.el);
    this.el.addEventListener("scroll", () => this.schedule());
  }
  set(items: Item[]) {
    this.items = items;
    this.added = new Int32Array(items.length).fill(-2);
    this.removed = new Int32Array(items.length).fill(-2);
    this.inner.style.height = `${items.length * ROW}px`;
    this.el.scrollTop = 0;
    this.render();
  }
  stats(batch: [number, number, number][]) {
    for (const [i, a, r] of batch) { this.added[i] = a; this.removed[i] = r; }
    this.schedule();
  }
  schedule() {
    if (this.queued) return;
    this.queued = true;
    requestAnimationFrame(() => { this.queued = false; this.render(); });
  }
  render() {
    const first = Math.floor(this.el.scrollTop / ROW);
    const count = Math.min(this.items.length - first, Math.ceil(this.el.clientHeight / ROW) + 1);
    while (this.pool.length < count) {
      const row = document.createElement("div");
      row.className = "q46-row";
      this.inner.append(row);
      this.pool.push(row);
    }
    for (let k = 0; k < this.pool.length; k++) {
      const row = this.pool[k];
      const i = first + k;
      if (k >= count) { row.style.display = "none"; continue; }
      const it = this.items[i];
      const a = this.added[i], r = this.removed[i];
      const counts = a === -2 ? "" : a === -1 ? " binary" : ` +${a} -${r}`;
      row.style.display = "";
      row.style.transform = `translateY(${i * ROW}px)`;
      row.textContent = `${it.s} ${it.o ? `${it.o} -> ` : ""}${it.p}${counts}`;
    }
  }
  visibleRows() { return this.pool.filter((r) => r.style.display !== "none" && r.textContent).length; }
}

function parseList(buf: ArrayBuffer) {
  const text = new TextDecoder().decode(buf);
  const nl = text.indexOf("\n");
  const header = JSON.parse(text.slice(0, nl));
  const items: Item[] = [];
  for (const line of text.slice(nl + 1).split("\n")) {
    if (!line) continue;
    const [s, p, o] = line.split("\t");
    items.push(o === undefined ? { s, p } : { s, p, o });
  }
  return { header, items };
}

let nextId = 1;
const handlers = new Map<number, { batch: (b: StatsBatch) => void; done: (d: StatsDone) => void }>();

export async function q46(api: Api): Promise<boolean> {
  const config = await invoke<Config | null>("q46_config");
  if (!config) return false;
  await listen<StatsBatch>("q46-stats", (e) => handlers.get(e.payload.token)?.batch(e.payload));
  await listen<StatsDone>("q46-stats-done", (e) => handlers.get(e.payload.token)?.done(e.payload));
  const repos = (await invoke<{ fixtures: Repo[] }>("q46_repos")).fixtures;
  const host = $<HTMLDivElement>("host");
  host.replaceChildren();
  const list = new ChangeList(host);
  const panes = document.createElement("div");
  panes.className = "q46-panes";
  host.append(panes);
  const stalls = new Stalls();
  stalls.start();

  let dispose: (() => void) | null = null;
  const closeFile = () => { dispose?.(); dispose = null; panes.replaceChildren(); };

  const loadList = async (name: string) => {
    const id = nextId++;
    const t0 = now();
    const buf = await invoke<ArrayBuffer>("q46_change_list", { name, id });
    const t1 = now();
    const { header, items } = parseList(buf);
    const t2 = now();
    list.set(items);
    await frames();
    const t3 = now();
    return { id, items, header, phases: { list_ms: t3 - t0, ipc_ms: t1 - t0, parse_ms: t2 - t1, render_ms: t3 - t2, rust_ms: header.rust_ms, gix_open_ms: header.open_ms, tree_diff_ms: header.diff_ms }, visible_rows: list.visibleRows() };
  };

  const startStats = (id: number, onBatch?: () => boolean) => {
    const token = nextId++;
    let applied = 0, dropped = 0;
    const done = new Promise<StatsDone & { applied_batches: number; dropped_batches: number; done_at: number }>((resolve) => {
      handlers.set(token, {
        batch: (b) => {
          if (onBatch && !onBatch()) { dropped++; return; }
          list.stats(b.items); applied++;
        },
        done: (d) => { handlers.delete(token); resolve({ ...d, applied_batches: applied, dropped_batches: dropped, done_at: now() }); },
      });
    });
    const started = now();
    void invoke("q46_stats_start", { id, token });
    return { token, done, started };
  };

  // Opens one file in option B. `cancelled()` is checked after each step.
  const openFile = async (id: number, index: number, token: number, cancelled: () => boolean) => {
    closeFile();
    const phases: Record<string, number> = {};
    const t0 = now();
    const step = (name: string, since: number) => { phases[name] = now() - since; return cancelled(); };
    const [lb, rb, blocks] = await Promise.all([
      invoke<ArrayBuffer>("q46_blob", { id, index, side: "left", token }),
      invoke<ArrayBuffer>("q46_blob", { id, index, side: "right", token }),
      invoke<{ blocks: Block[]; rust_ms: number }>("q46_blocks", { id, index, token }),
    ]).catch((e) => { throw new Error(String(e)); });
    if (step("load_ms", t0)) return { cancelled_after: "load", phases };
    const t1 = now();
    const dec = new TextDecoder();
    const lm = monaco.editor.createModel(dec.decode(lb), "plaintext");
    if (cancelled()) { lm.dispose(); return { cancelled_after: "left model", phases }; }
    const rm = monaco.editor.createModel(dec.decode(rb), "plaintext");
    if (step("model_ms", t1)) { lm.dispose(); rm.dispose(); return { cancelled_after: "models", phases }; }
    const t2 = now();
    const le = document.createElement("div"), re = document.createElement("div");
    le.className = re.className = "pane";
    panes.append(le, re);
    const l = monaco.editor.create(le, { ...api.editorOptions, model: lm });
    const r = monaco.editor.create(re, { ...api.editorOptions, model: rm });
    const view = api.applyB(l, r, blocks.blocks);
    const subs = [...api.syncScroll(l, r), ...api.padForScrollbar(l, r)];
    dispose = () => { subs.forEach((s) => s.dispose()); view.clear(); l.dispose(); r.dispose(); lm.dispose(); rm.dispose(); };
    if (step("setup_ms", t2)) { closeFile(); return { cancelled_after: "setup", phases }; }
    const t3 = now();
    await frames();
    phases.paint_ms = now() - t3;
    phases.open_ms = now() - t0;
    phases.blocks_rust_ms = blocks.rust_ms;
    if (cancelled()) { closeFile(); return { cancelled_after: "paint", phases }; }
    return { cancelled_after: null, phases, blocks: blocks.blocks.length, left_lines: lm.getLineCount(), right_lines: rm.getLineCount() };
  };

  const tree = () => invoke<{ total_cpu_ms: number; total_rss_kb: number; processes: { name: string; rss_kb: number; cpu_ms: number }[] }>("tree_stats");
  const cpuBy = (a: Awaited<ReturnType<typeof tree>>, b: Awaited<ReturnType<typeof tree>>) => {
    const sum = (t: typeof a, n: string) => t.processes.filter((p) => p.name === n).reduce((s, p) => s + p.cpu_ms, 0);
    const names = [...new Set(b.processes.map((p) => p.name))];
    return { total_ms: b.total_cpu_ms - a.total_cpu_ms, by_process: Object.fromEntries(names.map((n) => [n, sum(b, n) - sum(a, n)])) };
  };
  const p = (v: number[], q: number) => api.percentile(v, q);
  const results: unknown[] = [];
  const fixtures = config.fixtures.split(",");

  for (const part of config.parts.split(",")) {
    for (const name of fixtures) {
      const repo = repos.find((r) => r.name === name);
      if (!repo) throw new Error(`unknown repository fixture ${name}`);
      status(`Q46 ${part}: ${name}`);
      if (part === "list") {
        const c0 = await tree();
        stalls.take();
        await invoke("sampler_start", { intervalMs: 50 });
        const runs = [];
        let last = null;
        for (let k = 0; k < config.runs; k++) { last = await loadList(name); runs.push(last); }
        const listStalls = stalls.take();
        const memRuns = await invoke("sampler_stop");
        await invoke("sampler_start", { intervalMs: 50 });
        const stats = startStats(last!.id);
        const done = await stats.done;
        await frames();
        const sums = { added: 0, removed: 0, binary: 0, missing: 0 };
        for (let i = 0; i < list.items.length; i++) {
          if (list.added[i] === -2) sums.missing++;
          else if (list.added[i] === -1) sums.binary++;
          else { sums.added += list.added[i]; sums.removed += list.removed[i]; }
        }
        const m = /(\d+) insertions?\(\+\)/.exec(repo.shortstat), d = /(\d+) deletions?\(-\)/.exec(repo.shortstat);
        const expected = { added: m ? Number(m[1]) : 0, removed: d ? Number(d[1]) : 0 };
        const statsStalls = stalls.take();
        const memStats = await invoke("sampler_stop");
        await invoke("sampler_start", { intervalMs: 50 });
        // List scroll.
        const gaps: number[] = [];
        await new Promise<void>((resolve) => {
          let prev = 0;
          const tick = (t: number) => {
            if (prev) gaps.push(t - prev);
            prev = t;
            const max = list.el.scrollHeight - list.el.clientHeight;
            if (list.el.scrollTop >= max) { resolve(); return; }
            list.el.scrollTop = Math.min(max, list.el.scrollTop + 400);
            requestAnimationFrame(tick);
          };
          requestAnimationFrame(tick);
        });
        const median = p(gaps, 50) ?? 0;
        const scroll = { frames: gaps.length, median_ms: median, p95_ms: p(gaps, 95), long_frames: gaps.filter((g) => g > 1.5 * median).length, max_ms: Math.max(...gaps) };
        const scrollStalls = stalls.take();
        const memScroll = await invoke("sampler_stop");
        const c1 = await tree();
        // File opens.
        await invoke("sampler_start", { intervalMs: 50 });
        const opens = [];
        for (const index of [0, Math.floor(list.items.length / 2), list.items.length - 1]) {
          const token = nextId++;
          opens.push({ index, path: list.items[index].p, status: list.items[index].s, ...(await openFile(last!.id, index, token, () => false)) });
        }
        const openStalls = stalls.take();
        const memOpen = await invoke("sampler_stop");
        const c2 = await tree();
        closeFile();
        const listMs = runs.map((r) => r.phases.list_ms);
        results.push({
          part, fixture: name, repo,
          files_in_list: last!.items.length,
          files_ok: last!.items.length === repo.changed_files,
          visible_rows: last!.visible_rows,
          list: { runs: runs.map((r) => r.phases), run1_ms: listMs[0], p50_ms: p(listMs, 50), p95_ms: p(listMs, 95), max_ms: Math.max(...listMs), stalls: listStalls },
          stats: { ...done, wall_ms: done.done_at - stats.started, sums, expected, counts_ok: sums.added === expected.added && sums.removed === expected.removed && sums.missing === 0, stalls: statsStalls },
          list_scroll: { ...scroll, stalls: scrollStalls },
          memory_list_phase: { list_runs: memRuns, line_counts: memStats, list_scroll: memScroll },
          cpu_list_phase: cpuBy(c0, c1),
          opens, open_stalls: openStalls,
          memory_open_phase: memOpen,
          cpu_open_phase: cpuBy(c1, c2),
        });
      } else if (part === "cancel-stats") {
        const runs = [];
        for (let k = 0; k < config.runs; k++) {
          const { id } = await loadList(name);
          await frames();
          let cancelledAt = Infinity, rustAck = 0;
          const stats = startStats(id, () => now() < cancelledAt);
          const planned = stats.started + 300;
          const ack = await new Promise<number>((resolve) => {
            setTimeout(async () => {
              cancelledAt = now();
              const sent = invoke("q46_cancel", { token: stats.token }).then(() => { rustAck = now(); });
              list.state = "cancelled";
              status(`Q46 cancel-stats ${k + 1}: cancelled`);
              await frames();
              resolve(now());
              await sent;
            }, Math.max(0, planned - now()));
          });
          const done = await stats.done;
          runs.push({ ack_ms: ack - planned, rust_ack_ms: rustAck - planned, timer_late_ms: cancelledAt - planned, stop_ms: done.done_at - planned, rust_stop_ms: done.ms - 300, cancelled: done.cancelled, counted: done.counted, files: done.files, dropped_batches: done.dropped_batches });
          await sleep(200);
        }
        const ack = runs.map((r) => r.ack_ms), stop = runs.map((r) => r.stop_ms);
        results.push({ part, fixture: name, runs, ack_p50_ms: p(ack, 50), ack_p95_ms: p(ack, 95), ack_max_ms: Math.max(...ack), stop_p95_ms: p(stop, 95), stop_max_ms: Math.max(...stop), each_cancelled: runs.every((r) => r.cancelled && r.counted < r.files) });
      } else if (part === "cancel-open") {
        const { id, items } = await loadList(name);
        const index = items.findIndex((it) => it.p.startsWith("data/"));
        // A full open first, for the phase times.
        const full = await openFile(id, index, nextId++, () => false);
        closeFile();
        await sleep(500);
        const runs = [];
        // The cancel delays are parts of the full open time, so each step of the open gets a cancel.
        for (const share of [0.05, 0.3, 0.6, 0.75, 0.9]) {
          const delay = Math.round(share * (full.phases.open_ms ?? 1000));
          for (let k = 0; k < Math.max(1, Math.round(config.runs / 5)); k++) {
            const token = nextId++;
            let flag = false;
            const t0 = now();
            const planned = t0 + delay;
            let ackAt = 0, firedAt = 0, rustAt = 0;
            const timer = new Promise<void>((resolve) => setTimeout(async () => {
              firedAt = now();
              flag = true;
              const sent = invoke("q46_cancel", { token }).then(() => { rustAt = now(); });
              status(`Q46 cancel-open: cancelled`);
              await frames();
              ackAt = now();
              await sent;
              resolve();
            }, delay));
            const result = await openFile(id, index, token, () => flag).catch((e) => ({ cancelled_after: `error: ${e}`, phases: {} }));
            const openEnd = now();
            await timer;
            const shown = panes.childElementCount > 0;
            runs.push({ delay_ms: delay, ack_ms: ackAt - planned, rust_ack_ms: rustAt - planned, timer_late_ms: firedAt - planned, open_end_ms: openEnd - t0, cancelled_after: result.cancelled_after, editors_left: shown });
            closeFile();
            await sleep(500);
          }
        }
        const ack = runs.map((r) => r.ack_ms);
        results.push({ part, fixture: name, full_open: full, runs, ack_p50_ms: p(ack, 50), ack_p95_ms: p(ack, 95), ack_max_ms: Math.max(...ack), no_editor_left: runs.every((r) => !r.editors_left || r.cancelled_after === null) });
      }
    }
  }

  const doc = {
    prototype: "Diffy S1 prototype, Q46 S8 scale checks",
    created: new Date().toISOString(),
    system: await invoke("system_info"),
    user_agent: navigator.userAgent,
    device_pixel_ratio: window.devicePixelRatio,
    window: { width: window.innerWidth, height: window.innerHeight },
    config,
    method: METHOD,
    results,
  };
  const path = await invoke<string>("save_result", { label: config.label, json: JSON.stringify(doc, null, 2) });
  status(`Q46 done: ${path}`);
  console.log(`q46 result: ${path}`);
  await sleep(300);
  await invoke("quit", { code: 0 });
  return true;
}
