// Throwaway S1 prototype for map question Q41. Never merged.
// Option A: the Monaco DiffEditor. It calculates its own presentation diff.
// Option B: two Monaco editors that show the canonical hunks from Rust.
// The harness measures each open and records the method with the results.

import * as monaco from "monaco-editor/editor";
import EditorWorker from "monaco-editor/editor/editor.worker?worker";
import { invoke } from "@tauri-apps/api/core";
declare const __MONACO_VERSION__: string;

(self as unknown as { MonacoEnvironment: monaco.Environment }).MonacoEnvironment = {
  getWorker: () => new EditorWorker(),
};

type Side = { file: string; bytes: number; lines: number; sha256: string };
type Fixture = { name: string; note: string; seed: number; left: Side; right: Side };
type Block = { left_start: number; left_count: number; right_start: number; right_count: number };
type Canonical = {
  source: string; git_version: string; command: string; oracle_ms: number; cached: boolean;
  hunks: Block[]; blocks: Block[]; added: number; removed: number;
};
type Option = "A" | "B";

type OpenResult = {
  option: Option;
  fixture: string;
  ok: boolean;
  error?: string;
  phases: Record<string, number>;
  checks: Record<string, unknown>;
  memory?: unknown;
};

const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;
const host = $<HTMLDivElement>("host");
const status = (text: string) => { $("status").textContent = text; };
const now = () => performance.now();
const frames = () => new Promise<void>((r) => requestAnimationFrame(() => requestAnimationFrame(() => r())));
const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

const TIMEOUT_MS = 180_000;
function withTimeout<T>(p: Promise<T>, what: string): Promise<T> {
  return Promise.race([p, sleep(TIMEOUT_MS).then(() => { throw new Error(`${what}: timeout after ${TIMEOUT_MS} ms`); })]);
}

let fixtures: Fixture[] = [];
let dispose: (() => void) | null = null;

function clear() {
  dispose?.();
  dispose = null;
  host.replaceChildren();
}

async function readSide(name: string, side: "left" | "right"): Promise<string> {
  const buf = await invoke<ArrayBuffer>("read_side", { name, side });
  return new TextDecoder("utf-8", { fatal: true }).decode(buf);
}

// The view lines that Monaco rendered in a container.
const renderedLines = (el: HTMLElement) => el.querySelectorAll(".view-line").length;

// Monaco gives a model one more line than the file when the file ends with "\n".
const modelLines = (f: Side) => f.lines + 1;

const EDITOR_OPTIONS: monaco.editor.IEditorOptions = {
  readOnly: true,
  automaticLayout: true,
  minimap: { enabled: false },
  scrollBeyondLastLine: false,
  folding: false,
  renderLineHighlight: "none",
  fontSize: 13,
};

function key(b: { os: number; oe: number; ms: number; me: number }) {
  return `${b.os},${b.oe},${b.ms},${b.me}`;
}

// A canonical block as a Monaco ILineChange. An empty side has end 0 and the
// start is the line before the block.
function blockAsLineChange(b: Block) {
  return {
    os: b.left_count === 0 ? b.left_start - 1 : b.left_start,
    oe: b.left_count === 0 ? 0 : b.left_start + b.left_count - 1,
    ms: b.right_count === 0 ? b.right_start - 1 : b.right_start,
    me: b.right_count === 0 ? 0 : b.right_start + b.right_count - 1,
  };
}

async function openA(f: Fixture): Promise<OpenResult> {
  const phases: Record<string, number> = {};
  const t0 = now();
  const [left, right] = await Promise.all([readSide(f.name, "left"), readSide(f.name, "right")]);
  const t1 = now();
  phases.load_ms = t1 - t0;
  const original = monaco.editor.createModel(left, "plaintext");
  const modified = monaco.editor.createModel(right, "plaintext");
  const t2 = now();
  phases.model_ms = t2 - t1;
  const el = document.createElement("div");
  el.className = "pane";
  host.append(el);
  const diff = monaco.editor.createDiffEditor(el, {
    ...EDITOR_OPTIONS,
    originalEditable: false,
    renderSideBySide: true,
    useInlineViewWhenSpaceIsLimited: false,
    ignoreTrimWhitespace: false,
    maxComputationTime: 0,
    hideUnchangedRegions: { enabled: false },
  });
  dispose = () => { diff.dispose(); original.dispose(); modified.dispose(); };
  const updated = new Promise<void>((resolve) => {
    const sub = diff.onDidUpdateDiff(() => { sub.dispose(); resolve(); });
  });
  diff.setModel({ original, modified });
  await withTimeout(updated, "the presentation diff");
  const t3 = now();
  phases.diff_ms = t3 - t2;
  await frames();
  const t4 = now();
  phases.paint_ms = t4 - t3;
  phases.total_ms = t4 - t0;

  // Checks, after the timed part.
  const changes = diff.getLineChanges() ?? [];
  const canon = await invoke<Canonical>("canonical_hunks", { name: f.name });
  const canonKeys = new Set(canon.blocks.map((b) => key(blockAsLineChange(b))));
  const presentation = changes.map((c) => key({
    os: c.originalStartLineNumber, oe: c.originalEndLineNumber, ms: c.modifiedStartLineNumber, me: c.modifiedEndLineNumber,
  }));
  const same = presentation.filter((k) => canonKeys.has(k)).length;
  const quitEarly = (diff as unknown as { getDiffComputationResult?: () => { quitEarly?: boolean } | null })
    .getDiffComputationResult?.()?.quitEarly;
  const checks = {
    left_model_lines: original.getLineCount(),
    right_model_lines: modified.getLineCount(),
    expected_left_lines: modelLines(f.left),
    expected_right_lines: modelLines(f.right),
    rendered_view_lines: renderedLines(el),
    presentation_changes: changes.length,
    canonical_blocks: canon.blocks.length,
    presentation_changes_equal_to_a_canonical_block: same,
    quit_early: quitEarly ?? "unknown",
  };
  const ok = checks.left_model_lines === checks.expected_left_lines
    && checks.right_model_lines === checks.expected_right_lines
    && checks.rendered_view_lines > 0 && changes.length > 0;
  return { option: "A", fixture: f.name, ok, phases, checks };
}

function align(editor: monaco.editor.IStandaloneCodeEditor, zones: { after: number; lines: number }[]) {
  editor.changeViewZones((acc) => {
    for (const z of zones) {
      const dom = document.createElement("div");
      dom.className = "pad";
      acc.addZone({ afterLineNumber: z.after, heightInLines: z.lines, domNode: dom });
    }
  });
}

function decorate(editor: monaco.editor.IStandaloneCodeEditor, ranges: [number, number][], className: string) {
  return editor.createDecorationsCollection(ranges.map(([start, end]) => ({
    range: new monaco.Range(start, 1, end, 1),
    options: { isWholeLine: true, className },
  })));
}

function syncScroll(a: monaco.editor.IStandaloneCodeEditor, b: monaco.editor.IStandaloneCodeEditor) {
  let busy = false;
  const link = (from: monaco.editor.IStandaloneCodeEditor, to: monaco.editor.IStandaloneCodeEditor) =>
    from.onDidScrollChange((e) => {
      if (busy || (!e.scrollTopChanged && !e.scrollLeftChanged)) return;
      busy = true;
      to.setScrollPosition({ scrollTop: e.scrollTop, scrollLeft: e.scrollLeft });
      busy = false;
    });
  return [link(a, b), link(b, a)];
}

async function openB(f: Fixture): Promise<OpenResult> {
  const phases: Record<string, number> = {};
  const t0 = now();
  const [left, right, canon] = await Promise.all([
    readSide(f.name, "left"), readSide(f.name, "right"), invoke<Canonical>("canonical_hunks", { name: f.name }),
  ]);
  const t1 = now();
  phases.load_ms = t1 - t0;
  phases.oracle_ms = canon.cached ? 0 : canon.oracle_ms;
  const lm = monaco.editor.createModel(left, "plaintext");
  const rm = monaco.editor.createModel(right, "plaintext");
  const t2 = now();
  phases.model_ms = t2 - t1;
  const le = document.createElement("div");
  const re = document.createElement("div");
  le.className = re.className = "pane";
  host.append(le, re);
  const l = monaco.editor.create(le, { ...EDITOR_OPTIONS, model: lm });
  const r = monaco.editor.create(re, { ...EDITOR_OPTIONS, model: rm });
  const leftZones: { after: number; lines: number }[] = [];
  const rightZones: { after: number; lines: number }[] = [];
  const removed: [number, number][] = [];
  const added: [number, number][] = [];
  for (const b of canon.blocks) {
    if (b.left_count > 0) removed.push([b.left_start, b.left_start + b.left_count - 1]);
    if (b.right_count > 0) added.push([b.right_start, b.right_start + b.right_count - 1]);
    const d = b.left_count - b.right_count;
    if (d > 0) rightZones.push({ after: b.right_start + b.right_count - 1, lines: d });
    if (d < 0) leftZones.push({ after: b.left_start + b.left_count - 1, lines: -d });
  }
  const decorations = [decorate(l, removed, "line-removed"), decorate(r, added, "line-added")];
  align(l, leftZones);
  align(r, rightZones);
  const subs = syncScroll(l, r);
  dispose = () => {
    subs.forEach((s) => s.dispose());
    decorations.forEach((d) => d.clear());
    l.dispose(); r.dispose(); lm.dispose(); rm.dispose();
  };
  const t3 = now();
  phases.setup_ms = t3 - t2;
  await frames();
  const t4 = now();
  phases.paint_ms = t4 - t3;
  phases.total_ms = t4 - t0;

  // The rows on both sides must be equal after the padding.
  const leftRows = lm.getLineCount() + leftZones.reduce((s, z) => s + z.lines, 0);
  const rightRows = rm.getLineCount() + rightZones.reduce((s, z) => s + z.lines, 0);
  const checks = {
    left_model_lines: lm.getLineCount(),
    right_model_lines: rm.getLineCount(),
    expected_left_lines: modelLines(f.left),
    expected_right_lines: modelLines(f.right),
    rendered_view_lines_left: renderedLines(le),
    rendered_view_lines_right: renderedLines(re),
    canonical_blocks: canon.blocks.length,
    canonical_hunks: canon.hunks.length,
    canonical_added: canon.added,
    canonical_removed: canon.removed,
    aligned_rows_left: leftRows,
    aligned_rows_right: rightRows,
    content_height_left: l.getContentHeight(),
    content_height_right: r.getContentHeight(),
    oracle_cached: canon.cached,
  };
  const ok = checks.left_model_lines === checks.expected_left_lines
    && checks.right_model_lines === checks.expected_right_lines
    && checks.rendered_view_lines_left > 0 && checks.rendered_view_lines_right > 0
    && leftRows === rightRows && checks.content_height_left === checks.content_height_right;
  return { option: "B", fixture: f.name, ok, phases, checks };
}

async function open(option: Option, name: string): Promise<OpenResult> {
  clear();
  await frames();
  const f = fixtures.find((x) => x.name === name);
  if (!f) return { option, fixture: name, ok: false, error: "unknown fixture", phases: {}, checks: {} };
  try {
    const result = option === "A" ? await openA(f) : await openB(f);
    result.memory = await invoke("process_memory");
    return result;
  } catch (e) {
    return { option, fixture: name, ok: false, error: String(e), phases: {}, checks: {} };
  }
}

// Nearest-rank percentile.
function percentile(values: number[], p: number) {
  if (values.length === 0) return null;
  const sorted = [...values].sort((a, b) => a - b);
  return sorted[Math.min(sorted.length - 1, Math.ceil((p / 100) * sorted.length) - 1)];
}

const METHOD = [
  "Each run disposes the previous editors and models, waits two animation frames, then opens the fixture.",
  "load_ms: the two read_side IPC calls and the UTF-8 decode. For option B it also includes the canonical_hunks IPC call.",
  "oracle_ms (option B): the git diff oracle time on the first run. Later runs use the cached hunks and record 0.",
  "model_ms: monaco.editor.createModel for both sides.",
  "diff_ms (option A): from setModel to the first onDidUpdateDiff event. Monaco calculates the presentation diff in a web worker.",
  "setup_ms (option B): editor creation, decorations, alignment view zones and scroll sync.",
  "paint_ms: two requestAnimationFrame callbacks after the editor is ready, as an estimate of the first paint.",
  "total_ms: from the open request to the end of paint_ms.",
  "Run 1 is a warm-up. The summary excludes it when there are 2 or more runs.",
  "memory: the resident set size of the app process and its descendants, for example WebKitWebProcess, after the open.",
];

async function bench(option: Option, name: string, runs: number) {
  const results: OpenResult[] = [];
  for (let i = 0; i < runs; i++) {
    status(`Option ${option}, ${name}: run ${i + 1} of ${runs}.`);
    results.push(await open(option, name));
    await sleep(100);
  }
  const measured = (runs > 1 ? results.slice(1) : results).filter((r) => r.ok).map((r) => r.phases.total_ms);
  return {
    option,
    fixture: fixtures.find((f) => f.name === name),
    runs,
    all_ok: results.every((r) => r.ok),
    summary: {
      measured_runs: measured.length,
      total_ms_p50: percentile(measured, 50),
      total_ms_p95: percentile(measured, 95),
      total_ms_max: measured.length ? Math.max(...measured) : null,
    },
    results,
  };
}

async function report(label: string, items: unknown[]) {
  const doc = {
    prototype: "Diffy S1, map question Q41 (build phase). Q54 gives the verdict.",
    created: new Date().toISOString(),
    system: await invoke("system_info"),
    user_agent: navigator.userAgent,
    device_pixel_ratio: window.devicePixelRatio,
    window: { width: window.innerWidth, height: window.innerHeight },
    monaco: __MONACO_VERSION__,
    method: METHOD,
    items,
  };
  return invoke<string>("save_result", { label, json: JSON.stringify(doc, null, 2) });
}

async function autorun() {
  const config = await invoke<{ spec: string; runs: number; label: string } | null>("autorun_config");
  if (!config) return false;
  const items = [];
  for (const part of config.spec.split(",")) {
    const [option, name] = part.split(":") as [Option, string];
    items.push(await bench(option, name, config.runs));
  }
  const path = await report(config.label, items);
  const ok = items.every((i) => i.all_ok);
  status(`Autorun ${ok ? "passed" : "failed"}. Result: ${path}`);
  console.log(`autorun result: ${path}`);
  await sleep(500);
  await invoke("quit", { code: ok ? 0 : 1 });
  return true;
}

function describe(r: OpenResult) {
  if (!r.ok) return `Option ${r.option}, ${r.fixture}: FAILED. ${r.error ?? JSON.stringify(r.checks)}`;
  const p = Object.entries(r.phases).map(([k, v]) => `${k} ${v.toFixed(1)}`).join(", ");
  return `Option ${r.option}, ${r.fixture}: open. ${p}`;
}

async function main() {
  const manifest = await invoke<{ fixtures: Fixture[] }>("list_fixtures").catch((e) => {
    status(String(e));
    throw e;
  });
  fixtures = manifest.fixtures;
  const select = $<HTMLSelectElement>("fixture");
  for (const f of fixtures) select.append(new Option(`${f.name} (${(f.left.bytes / 1e6).toFixed(2)} MB)`, f.name));
  status("Select an option and a fixture.");
  if (await autorun()) return;
  const option = () => $<HTMLSelectElement>("option").value as Option;
  $("open").addEventListener("click", async () => {
    status(`Opening ${select.value} with option ${option()}.`);
    status(describe(await open(option(), select.value)));
  });
  $("bench").addEventListener("click", async () => {
    const runs = Math.max(1, Number($<HTMLInputElement>("runs").value) || 1);
    const item = await bench(option(), select.value, runs);
    const label = `manual-${option()}-${select.value}-${Date.now()}`;
    const path = await report(label, [item]);
    status(`p50 ${item.summary.total_ms_p50?.toFixed(1)} ms, p95 ${item.summary.total_ms_p95?.toFixed(1)} ms. Saved ${path}`);
  });
}

main();
