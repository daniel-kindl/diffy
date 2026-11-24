// Throwaway S1 verdict checks for map question Q54. Never merged.
// Runs when DIFFY_S1_Q54=A or B. Each part records its facts and, where the
// charter has a rule, a pass or a fail. The owner reads the measurements.

import * as monaco from "monaco-editor/editor";
import { invoke } from "@tauri-apps/api/core";
import type { Block, BView, Canonical, Current, Option } from "./main";

type Api = {
  open: (option: Option, name: string) => Promise<{ ok: boolean; error?: string; phases: Record<string, number>; checks: Record<string, unknown> }>;
  current: () => Current | null;
  applyB: (l: monaco.editor.ICodeEditor, r: monaco.editor.ICodeEditor, blocks: Block[]) => BView;
  syncScroll: (a: monaco.editor.ICodeEditor, b: monaco.editor.ICodeEditor) => monaco.IDisposable[];
  padForScrollbar: (a: monaco.editor.ICodeEditor, b: monaco.editor.ICodeEditor) => monaco.IDisposable[];
  editorOptions: monaco.editor.IEditorOptions;
  percentile: (values: number[], p: number) => number | null;
};
type Config = { option: Option; label: string; parts: string; fixtures: string };

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));
const frame = () => new Promise<number>((r) => requestAnimationFrame((t) => r(t)));
const frames = async () => { await frame(); await frame(); };
const status = (t: string) => { document.getElementById("status")!.textContent = t; };
const now = () => performance.now();

const METHOD = [
  "Each part opens the fixture with the Q41 open path, then runs its checks. Times are performance.now() in the page.",
  "coordinates: the canonical row model comes from the oracle blocks. Each unchanged pair (left line, right line) must have the same top (getTopForLineNumber) on both sides. Each block must start at the same top on both sides. The changed lines on each side must be equal to the canonical changed lines: option A uses getLineChanges, option B uses its line decorations.",
  "dom_rows: at evenly spaced scroll positions and at the first block starts, each rendered row on both sides is hit-tested with getTargetAtClientPoint. A row passes when both sides show a canonical unchanged pair, or lines of the same canonical block, or a view zone next to a block line, or two view zones.",
  "findings: five finding widgets, each a view zone of 3 lines, at canonical coordinates: an unchanged right line, a removed left line, an added right line, right line 1 and the last left line. Option A adds the widget to one side and Monaco aligns the other side. Option B adds the widget and a spacer of the same height on the other side. The coordinate check and the row check run again, and each widget must start at the bottom of its line (1 px tolerance).",
  "scroll: setScrollTop on the right editor in each requestAnimationFrame callback, a fixed step each frame, so the speed follows the display refresh. This is programmatic scrolling, not wheel input. The frame interval is the time between rAF callbacks. A long frame is more than 1.5 times the median interval. Each frame checks that the two sides have the same scrollTop.",
  "live: the Rust side changes the live copy of the right side on disk (write a temporary file, then rename). The page reads the file, applies line edits to the right model, and recomputes the diff: option A with the Monaco worker, option B with the oracle on the live file. The settled time is from before the write to two animation frames after the new diff is shown. Run 1 is a warm-up. A file watcher is not part of the prototype; the time does not include a watcher delay.",
  "live anchor: before the edits, a tracked decoration marks the first visible line of the right editor. After the new diff is shown, the page scrolls so that this line has the same offset from the top as before. Monaco alone keeps scrollTop in pixels, so a line inserted above the viewport moves the content down.",
  "live checks: the scroll position is kept when the same content line stays at the top of the right editor (shifted by the lines inserted above it). The selection is kept when the selected text is the same.",
  "option B open path (Q54): when only one side shows a horizontal scroll bar, a view zone of the scroll bar height goes below the last line of the other side, so both sides have the same maximum scrollTop. Q42 found the defect, and the 10k-lines fixture has it (26 right lines of 83 characters).",
  "hscroll: an in-memory pair of 200 lines where right line 100 has 2,000 characters, so only the right side needs a horizontal scroll bar. The check scrolls to the bottom and compares the two sides.",
  "stress: a memory sampler thread reads the RSS of the process tree each 50 ms. CPU time is utime + stime of the process tree from /proc. A stall is a timer callback that runs more than 50 ms late, or a gap of more than 50 ms between animation frames.",
];

type Rec = { part: string; fixture?: string; pass?: boolean; facts: Record<string, unknown> };

// The canonical row model: each unchanged pair, and the block of each changed line.
type Model = {
  pairOfLeft: Int32Array; pairOfRight: Int32Array;
  blockOfLeft: Int32Array; blockOfRight: Int32Array;
  pairs: number; walkErrors: number;
};

function rowModel(blocks: Block[], leftLines: number, rightLines: number): Model {
  const m: Model = {
    pairOfLeft: new Int32Array(leftLines + 2), pairOfRight: new Int32Array(rightLines + 2),
    blockOfLeft: new Int32Array(leftLines + 2).fill(-1), blockOfRight: new Int32Array(rightLines + 2).fill(-1),
    pairs: 0, walkErrors: 0,
  };
  let li = 1, ri = 1;
  const pairRun = (n: number) => {
    for (let k = 0; k < n; k++) { m.pairOfLeft[li] = ri; m.pairOfRight[ri] = li; li++; ri++; m.pairs++; }
  };
  blocks.forEach((b, i) => {
    const n = b.left_start - li;
    if (b.right_start - ri !== n || n < 0) m.walkErrors++;
    pairRun(Math.max(0, n));
    for (let k = 0; k < b.left_count; k++) m.blockOfLeft[b.left_start + k] = i;
    for (let k = 0; k < b.right_count; k++) m.blockOfRight[b.right_start + k] = i;
    li = b.left_start + b.left_count;
    ri = b.right_start + b.right_count;
  });
  const n = leftLines - li + 1;
  if (rightLines - ri + 1 !== n) m.walkErrors++;
  pairRun(Math.max(0, n));
  return m;
}

const lh = (ed: monaco.editor.ICodeEditor) => ed.getOption(monaco.editor.EditorOption.lineHeight);

function blockTop(ed: monaco.editor.ICodeEditor, start: number, count: number) {
  if (count > 0) return ed.getTopForLineNumber(start);
  return start - 1 >= 1 ? ed.getTopForLineNumber(start - 1) + lh(ed) : 0;
}

function changedLines(c: Current): { left: Uint8Array; right: Uint8Array } {
  const left = new Uint8Array(c.orig.getModel()!.getLineCount() + 2);
  const right = new Uint8Array(c.mod.getModel()!.getLineCount() + 2);
  const mark = (arr: Uint8Array, s: number, e: number) => { for (let i = s; i <= e; i++) arr[i] = 1; };
  if (c.option === "A") {
    for (const ch of c.diff!.getLineChanges() ?? []) {
      if (ch.originalEndLineNumber > 0) mark(left, ch.originalStartLineNumber, ch.originalEndLineNumber);
      if (ch.modifiedEndLineNumber > 0) mark(right, ch.modifiedStartLineNumber, ch.modifiedEndLineNumber);
    }
  } else {
    const collect = (ed: monaco.editor.ICodeEditor, cls: string, arr: Uint8Array) => {
      for (const d of ed.getModel()!.getAllDecorations()) {
        if (d.options.className === cls) mark(arr, d.range.startLineNumber, d.range.endLineNumber);
      }
    };
    collect(c.orig, "line-removed", left);
    collect(c.mod, "line-added", right);
  }
  return { left, right };
}

function coordinates(c: Current): Rec["facts"] & { pass: boolean } {
  const t0 = now();
  const lm = c.orig.getModel()!, rm = c.mod.getModel()!;
  const m = rowModel(c.canon.blocks, lm.getLineCount(), rm.getLineCount());
  let misaligned = 0;
  const examples: unknown[] = [];
  for (let l = 1; l <= lm.getLineCount(); l++) {
    const r = m.pairOfLeft[l];
    if (!r) continue;
    const a = c.orig.getTopForLineNumber(l), b = c.mod.getTopForLineNumber(r);
    if (a !== b) { misaligned++; if (examples.length < 10) examples.push({ left: l, right: r, top_left: a, top_right: b }); }
  }
  let blockMisaligned = 0;
  const blockExamples: unknown[] = [];
  for (const b of c.canon.blocks) {
    const a = blockTop(c.orig, b.left_start, b.left_count), d = blockTop(c.mod, b.right_start, b.right_count);
    if (a !== d) { blockMisaligned++; if (blockExamples.length < 10) blockExamples.push({ block: b, top_left: a, top_right: d }); }
  }
  // The changed lines on each side.
  const shown = changedLines(c);
  let leftWrong = 0, rightWrong = 0;
  const wrongExamples: unknown[] = [];
  for (let l = 1; l <= lm.getLineCount(); l++) {
    const canon = m.blockOfLeft[l] >= 0 ? 1 : 0;
    if (canon !== shown.left[l]) { leftWrong++; if (wrongExamples.length < 10) wrongExamples.push({ side: "left", line: l, canonical_changed: !!canon }); }
  }
  for (let r = 1; r <= rm.getLineCount(); r++) {
    const canon = m.blockOfRight[r] >= 0 ? 1 : 0;
    if (canon !== shown.right[r]) { rightWrong++; if (wrongExamples.length < 20) wrongExamples.push({ side: "right", line: r, canonical_changed: !!canon }); }
  }
  const facts = {
    canonical_blocks: c.canon.blocks.length,
    canonical_pairs: m.pairs,
    walk_errors: m.walkErrors,
    misaligned_pairs: misaligned,
    misaligned_pair_examples: examples,
    misaligned_block_starts: blockMisaligned,
    misaligned_block_examples: blockExamples,
    left_lines_marked_wrong: leftWrong,
    right_lines_marked_wrong: rightWrong,
    wrong_mark_examples: wrongExamples,
    presentation_changes: c.option === "A" ? (c.diff!.getLineChanges() ?? []).length : undefined,
    check_ms: now() - t0,
  };
  const pass = m.walkErrors === 0 && misaligned === 0 && blockMisaligned === 0 && leftWrong === 0 && rightWrong === 0;
  return { ...facts, pass };
}

type Row = { kind: "line"; line: number } | { kind: "zone" } | { kind: "other"; type: number };

function rowAt(ed: monaco.editor.ICodeEditor, x: number, y: number): Row {
  const t = ed.getTargetAtClientPoint(x, y);
  if (!t) return { kind: "other", type: -1 };
  const T = monaco.editor.MouseTargetType;
  if ((t.type === T.CONTENT_TEXT || t.type === T.CONTENT_EMPTY) && t.position) return { kind: "line", line: t.position.lineNumber };
  if (t.type === T.CONTENT_VIEW_ZONE) return { kind: "zone" };
  return { kind: "other", type: t.type };
}

// Hit-test each rendered row of the viewport on both sides.
function rowsNow(c: Current, m: Model, out: { rows: number; bad: number; examples: unknown[] }) {
  const h = lh(c.mod);
  const rl = c.orig.getDomNode()!.getBoundingClientRect(), rr = c.mod.getDomNode()!.getBoundingClientRect();
  const xl = rl.left + c.orig.getLayoutInfo().contentLeft + 8, xr = rr.left + c.mod.getLayoutInfo().contentLeft + 8;
  const bottom = Math.min(rl.top + c.orig.getLayoutInfo().height, rr.top + c.mod.getLayoutInfo().height)
    - c.orig.getLayoutInfo().horizontalScrollbarHeight;
  // Sample the middle of each row, measured from the scroll position of the right side.
  const offset = (h - (c.mod.getScrollTop() % h)) % h;
  for (let y = rr.top + offset + h / 2; y < bottom - 1; y += h) {
    const a = rowAt(c.orig, xl, y), b = rowAt(c.mod, xr, y);
    out.rows++;
    let ok = false;
    if (a.kind === "line" && b.kind === "line") {
      ok = m.pairOfLeft[a.line] === b.line || (m.blockOfLeft[a.line] >= 0 && m.blockOfLeft[a.line] === m.blockOfRight[b.line]);
    } else if (a.kind === "zone" && b.kind === "line") ok = m.blockOfRight[b.line] >= 0;
    else if (a.kind === "line" && b.kind === "zone") ok = m.blockOfLeft[a.line] >= 0;
    else if (a.kind === "zone" && b.kind === "zone") ok = true;
    if (!ok) { out.bad++; if (out.examples.length < 10) out.examples.push({ scroll_top: c.mod.getScrollTop(), left: a, right: b }); }
  }
}

async function domRows(c: Current, extraTops: number[] = []) {
  const t0 = now();
  const m = rowModel(c.canon.blocks, c.orig.getModel()!.getLineCount(), c.mod.getModel()!.getLineCount());
  const height = c.mod.getScrollHeight() - c.mod.getLayoutInfo().height;
  const tops: number[] = [];
  for (let i = 0; i <= 24; i++) tops.push(Math.round((height * i) / 24));
  for (const b of c.canon.blocks.slice(0, 10)) tops.push(Math.max(0, blockTop(c.mod, b.right_start, b.right_count) - 3 * lh(c.mod)));
  tops.push(...extraTops);
  const out = { rows: 0, bad: 0, examples: [] as unknown[] };
  let scrollMismatch = 0;
  for (const top of tops) {
    c.mod.setScrollTop(top);
    await frames();
    if (c.orig.getScrollTop() !== c.mod.getScrollTop()) scrollMismatch++;
    rowsNow(c, m, out);
  }
  return { positions: tops.length, rows: out.rows, bad_rows: out.bad, bad_row_examples: out.examples, scroll_top_mismatch: scrollMismatch, check_ms: now() - t0, pass: out.bad === 0 && scrollMismatch === 0 };
}

async function findings(c: Current) {
  const lm = c.orig.getModel()!, rm = c.mod.getModel()!;
  const m = rowModel(c.canon.blocks, lm.getLineCount(), rm.getLineCount());
  const blocks = c.canon.blocks;
  const removedBlock = blocks.find((b) => b.left_count > 0)!;
  const addedBlock = [...blocks].reverse().find((b) => b.right_count > 0)!;
  let midPair = Math.floor(rm.getLineCount() / 2);
  while (!m.pairOfRight[midPair] && midPair < rm.getLineCount()) midPair++;
  type F = { id: string; side: "left" | "right"; line: number };
  const list: F[] = [
    { id: "F1-unchanged", side: "right", line: midPair },
    { id: "F2-removed", side: "left", line: removedBlock.left_start + Math.floor(removedBlock.left_count / 2) },
    { id: "F3-added", side: "right", line: addedBlock.right_start + Math.floor(addedBlock.right_count / 2) },
    { id: "F4-first", side: "right", line: 1 },
    { id: "F5-last", side: "left", line: lm.getLineCount() },
  ];
  const H = 3;
  const doms: { f: F; dom: HTMLElement }[] = [];
  const add = (ed: monaco.editor.ICodeEditor, after: number, dom: HTMLElement) =>
    ed.changeViewZones((acc) => { acc.addZone({ afterLineNumber: after, heightInLines: H, domNode: dom }); });
  // The line on the other side after which option B puts the spacer.
  const otherAfter = (f: F) => {
    const pairOf = f.side === "left" ? m.pairOfLeft : m.pairOfRight;
    const blockOf = f.side === "left" ? m.blockOfLeft : m.blockOfRight;
    if (pairOf[f.line]) return pairOf[f.line];
    const b = blocks[blockOf[f.line]];
    return f.side === "left" ? (b.right_count > 0 ? b.right_start + b.right_count - 1 : b.right_start - 1)
      : (b.left_count > 0 ? b.left_start + b.left_count - 1 : b.left_start - 1);
  };
  for (const f of list) {
    const dom = document.createElement("div");
    dom.className = "finding";
    dom.textContent = `Finding ${f.id} at ${f.side} line ${f.line}`;
    dom.style.cssText = "background:#fff3c4;color:#000;border-left:3px solid #c80;font:12px sans-serif;padding:2px 6px;box-sizing:border-box";
    const ed = f.side === "left" ? c.orig : c.mod;
    add(ed, f.line, dom);
    if (c.option === "B") {
      const spacer = document.createElement("div");
      spacer.className = "pad";
      add(f.side === "left" ? c.mod : c.orig, otherAfter(f), spacer);
    }
    doms.push({ f, dom });
  }
  await frames();
  await sleep(100);
  const coords = coordinates(c);
  // Each widget must start at the bottom of its line.
  const placement: unknown[] = [];
  let placementBad = 0;
  const extraTops: number[] = [];
  for (const { f, dom } of doms) {
    const ed = f.side === "left" ? c.orig : c.mod;
    const lineTop = ed.getTopForLineNumber(f.line);
    c.mod.setScrollTop(Math.max(0, lineTop - 5 * lh(ed)));
    extraTops.push(Math.max(0, lineTop - 5 * lh(ed)));
    await frames();
    const pos = ed.getScrolledVisiblePosition({ lineNumber: f.line, column: 1 });
    const edTop = ed.getDomNode()!.getBoundingClientRect().top;
    const rect = dom.getBoundingClientRect();
    const expected = pos ? edTop + pos.top + lh(ed) : NaN;
    const ok = Math.abs(rect.top - expected) <= 1 && rect.height >= H * lh(ed) - 1;
    if (!ok) placementBad++;
    placement.push({ ...f, widget_top: rect.top, expected_top: expected, widget_height: rect.height, ok });
  }
  const rows = await domRows(c, extraTops);
  return {
    widget_lines: H,
    aligned_by: c.option === "A" ? "Monaco DiffEditor (one zone on the finding side)" : "prototype code (a zone and a spacer)",
    placement, placement_bad: placementBad,
    coordinates_after: coords, dom_rows_after: rows,
    pass: coords.pass && rows.pass && placementBad === 0,
  };
}

async function scrollRun(c: Current, step: number, count: number) {
  c.mod.setScrollTop(0);
  await frames();
  const max = c.mod.getScrollHeight() - c.mod.getLayoutInfo().height;
  const times: number[] = [];
  let mismatch = 0, top = 0;
  for (let i = 0; i < count; i++) {
    times.push(await frame());
    if (i > 0 && c.orig.getScrollTop() !== c.mod.getScrollTop()) mismatch++;
    top = top + step > max ? 0 : top + step;
    c.mod.setScrollTop(top);
  }
  times.push(await frame());
  const iv = times.slice(1).map((t, i) => t - times[i]);
  const med = iv.length ? [...iv].sort((a, b) => a - b)[Math.floor(iv.length / 2)] : 0;
  const long = iv.filter((v) => v > 1.5 * med);
  return {
    frames: iv.length, step_px: step,
    fps: (iv.length / (times[times.length - 1] - times[0])) * 1000,
    interval_ms_median: med,
    interval_ms_p95: api!.percentile(iv, 95), interval_ms_p99: api!.percentile(iv, 99), interval_ms_max: Math.max(...iv),
    long_frames: long.length, long_frames_over_33ms: iv.filter((v) => v > 33.4).length,
    scroll_top_mismatch_frames: mismatch,
    pass_alignment: mismatch === 0,
  };
}

function firstLine(ed: monaco.editor.ICodeEditor) {
  return ed.getVisibleRanges()[0]?.startLineNumber ?? 0;
}

async function applyText(c: Current, text: string) {
  const model = c.mod.getModel()!;
  const oldLines = model.getLinesContent();
  const newLines = text.split("\n");
  const edits: monaco.editor.IIdentifiedSingleEditOperation[] = [];
  if (oldLines.length === newLines.length) {
    for (let i = 0; i < newLines.length; i++) {
      if (oldLines[i] !== newLines[i]) edits.push({ range: new monaco.Range(i + 1, 1, i + 1, oldLines[i].length + 1), text: newLines[i] });
    }
  } else {
    let p = 0;
    while (p < oldLines.length && p < newLines.length && oldLines[p] === newLines[p]) p++;
    let s = 0;
    while (s < oldLines.length - p && s < newLines.length - p && oldLines[oldLines.length - 1 - s] === newLines[newLines.length - 1 - s]) s++;
    const oldMid = oldLines.slice(p, oldLines.length - s), newMid = newLines.slice(p, newLines.length - s);
    // Replace whole lines: from the start of line p+1 to the start of the first suffix line.
    const endLine = oldLines.length - s + 1;
    const range = endLine <= oldLines.length
      ? new monaco.Range(p + 1, 1, endLine, 1)
      : new monaco.Range(p + 1, 1, oldLines.length, oldLines[oldLines.length - 1].length + 1);
    const inserted = newMid.join("\n") + (endLine <= oldLines.length && newMid.length ? "\n" : "");
    if (oldMid.length || newMid.length) edits.push({ range, text: inserted });
  }
  model.applyEdits(edits);
  return edits.length;
}

async function live(c: Current, mode: "replace" | "insert-above", runs: number) {
  const name = c.fixture.name;
  await invoke("live_reset", { name });
  const rm = c.mod.getModel()!;
  const mid = Math.floor(rm.getLineCount() / 2);
  c.mod.setScrollTop(c.mod.getTopForLineNumber(mid));
  await frames();
  const sel = new monaco.Range(mid + 3, 1, mid + 5, 8);
  c.mod.setSelection(sel);
  const results: unknown[] = [];
  const settled: number[] = [];
  let scrollKept = 0, selectionKept = 0;
  for (let k = 0; k < runs; k++) {
    const topLine = firstLine(c.mod);
    const scrollTop = c.mod.getScrollTop();
    const selText = rm.getValueInRange(c.mod.getSelection()!);
    const edits = mode === "replace"
      ? [{ line: mid + 10 + (k % 5), text: `live edit ${k} in the viewport` }, { line: Math.min(rm.getLineCount() - 1, mid + 20000), text: `live edit ${k} far below` }]
      : [{ line: Math.max(1, topLine - 50), text: `live line ${k} inserted above the viewport`, insert: true }];
    const insertedAbove = mode === "insert-above" ? 1 : 0;
    const t0 = now();
    let updated: Promise<void> | null = null;
    if (c.option === "A") {
      updated = new Promise<void>((resolve) => { const s = c.diff!.onDidUpdateDiff(() => { s.dispose(); resolve(); }); });
    }
    const w = await invoke<{ write_ms: number }>("live_write", { name, edits });
    const t1 = now();
    const buf = await invoke<ArrayBuffer>("live_read", { name });
    const text = new TextDecoder("utf-8", { fatal: true }).decode(buf);
    const t2 = now();
    // Anchor the top line: a tracked decoration follows the line through the edits.
    const topOffset = scrollTop - c.mod.getTopForLineNumber(topLine);
    const anchor = c.mod.createDecorationsCollection([{ range: new monaco.Range(topLine, 1, topLine, 1), options: { stickiness: monaco.editor.TrackedRangeStickiness.NeverGrowsWhenTypingAtEdges } }]);
    const applied = await applyText(c, text);
    const t3 = now();
    let oracle_ms: number | undefined;
    if (c.option === "B") {
      const canon = await invoke<Canonical>("live_canonical", { name });
      oracle_ms = canon.oracle_ms;
      c.b!.clear();
      c.b = api!.applyB(c.orig, c.mod, canon.blocks);
      c.canon = canon;
    } else {
      await updated;
    }
    const anchored = anchor.getRange(0)?.startLineNumber ?? topLine;
    anchor.clear();
    c.mod.setScrollTop(c.mod.getTopForLineNumber(anchored) + topOffset);
    const t4 = now();
    await frames();
    const t5 = now();
    const total = t5 - t0;
    if (k > 0) settled.push(total);
    const keptScroll = firstLine(c.mod) === topLine + insertedAbove;
    const keptSel = !c.mod.getSelection()!.isEmpty() && rm.getValueInRange(c.mod.getSelection()!) === selText;
    if (k > 0 && keptScroll) scrollKept++;
    if (k > 0 && keptSel) selectionKept++;
    results.push({
      run: k + 1, write_ms: w.write_ms, ipc_write_ms: t1 - t0, read_ms: t2 - t1, apply_ms: t3 - t2, model_edits: applied,
      diff_ms: t4 - t3, oracle_ms, paint_ms: t5 - t4, settled_ms: total,
      first_line_before: topLine, first_line_after: firstLine(c.mod), scroll_top_before: scrollTop, scroll_top_after: c.mod.getScrollTop(),
      scroll_kept: keptScroll, selection_kept: keptSel,
    });
    await sleep(300);
  }
  // The coordinate check against the canonical diff of the live file.
  if (c.option === "A") c.canon = await invoke<Canonical>("live_canonical", { name });
  const coords = coordinates(c);
  await invoke("live_reset", { name });
  return {
    mode, runs, measured_runs: settled.length,
    settled_ms_p50: api!.percentile(settled, 50), settled_ms_p95: api!.percentile(settled, 95), settled_ms_max: Math.max(...settled),
    scroll_kept_runs: scrollKept, selection_kept_runs: selectionKept,
    coordinates_after: coords,
    results,
    pass: scrollKept === settled.length && selectionKept === settled.length && coords.pass,
  };
}

async function hscroll(option: Option) {
  const host = document.getElementById("host")!;
  host.replaceChildren();
  const left = Array.from({ length: 200 }, (_, i) => `line ${i + 1}`);
  const right = [...left];
  right[99] = "x".repeat(2000);
  const canon = { blocks: [{ left_start: 100, left_count: 1, right_start: 100, right_count: 1 }] } as unknown as Canonical;
  const variants: { name: string; options: monaco.editor.IEditorOptions; pad?: boolean }[] = option === "A"
    ? [{ name: "A default", options: {} }]
    : [
      { name: "B default", options: {} },
      { name: "B scrollbar horizontal visible", options: { scrollbar: { horizontal: "visible" } } },
      { name: "B pad below the last line of the side with no horizontal scroll bar", options: {}, pad: true },
    ];
  const out: unknown[] = [];
  for (const v of variants) {
    host.replaceChildren();
    const lm = monaco.editor.createModel(left.join("\n") + "\n", "plaintext");
    const rm = monaco.editor.createModel(right.join("\n") + "\n", "plaintext");
    let c: Current;
    const disposers: (() => void)[] = [];
    if (option === "A") {
      const el = document.createElement("div");
      el.className = "pane";
      host.append(el);
      const diff = monaco.editor.createDiffEditor(el, { ...api!.editorOptions, ...v.options, originalEditable: false, renderSideBySide: true, useInlineViewWhenSpaceIsLimited: false, ignoreTrimWhitespace: false, hideUnchangedRegions: { enabled: false } } as monaco.editor.IDiffEditorConstructionOptions);
      const done = new Promise<void>((r) => { const s = diff.onDidUpdateDiff(() => { s.dispose(); r(); }); });
      diff.setModel({ original: lm, modified: rm });
      await done;
      c = { option, fixture: { name: "hscroll" } as never, canon, orig: diff.getOriginalEditor(), mod: diff.getModifiedEditor(), diff };
      disposers.push(() => diff.dispose());
    } else {
      const le = document.createElement("div"), re = document.createElement("div");
      le.className = re.className = "pane";
      host.append(le, re);
      const l = monaco.editor.create(le, { ...api!.editorOptions, ...v.options, model: lm });
      const r = monaco.editor.create(re, { ...api!.editorOptions, ...v.options, model: rm });
      const view = api!.applyB(l, r, canon.blocks);
      const subs = api!.syncScroll(l, r);
      if (v.pad) subs.push(...api!.padForScrollbar(l, r));
      c = { option, fixture: { name: "hscroll" } as never, canon, orig: l, mod: r, b: view };
      disposers.push(() => { subs.forEach((s) => s.dispose()); l.dispose(); r.dispose(); });
    }
    await frames();
    await sleep(200);
    // Monaco measures the width of a line only when it renders it.
    c.mod.setScrollTop(c.mod.getTopForLineNumber(95));
    await frames();
    await sleep(200);
    c.mod.setScrollTop(1e9);
    await frames();
    await sleep(200);
    const m = rowModel(canon.blocks, lm.getLineCount(), rm.getLineCount());
    const rows = { rows: 0, bad: 0, examples: [] as unknown[] };
    rowsNow(c, m, rows);
    // Scroll the left side to the bottom too.
    c.orig.setScrollTop(1e9);
    await frames();
    const fromLeft = { left: c.orig.getScrollTop(), right: c.mod.getScrollTop() };
    rowsNow(c, m, rows);
    const facts = {
      variant: v.name,
      content_height_left: c.orig.getContentHeight(), content_height_right: c.mod.getContentHeight(),
      scrollbar_height_left: c.orig.getLayoutInfo().horizontalScrollbarHeight, scrollbar_height_right: c.mod.getLayoutInfo().horizontalScrollbarHeight,
      scroll_width_left: c.orig.getScrollWidth(), scroll_width_right: c.mod.getScrollWidth(),
      content_width_left: c.orig.getLayoutInfo().contentWidth, content_width_right: c.mod.getLayoutInfo().contentWidth,
      horizontal_scrollbar_shown_left: c.orig.getScrollWidth() > c.orig.getLayoutInfo().contentWidth,
      horizontal_scrollbar_shown_right: c.mod.getScrollWidth() > c.mod.getLayoutInfo().contentWidth,
      bottom_from_right: { left: c.orig.getScrollTop(), right: c.mod.getScrollTop() },
      bottom_from_left: fromLeft,
      rows: rows.rows, bad_rows: rows.bad, bad_row_examples: rows.examples,
      last_line_top_left: c.orig.getTopForLineNumber(lm.getLineCount()), last_line_top_right: c.mod.getTopForLineNumber(rm.getLineCount()),
    };
    const pass = rows.bad === 0 && fromLeft.left === fromLeft.right && facts.bottom_from_right.left === facts.bottom_from_right.right;
    out.push({ ...facts, pass });
    disposers.forEach((d) => d());
    lm.dispose(); rm.dispose();
  }
  host.replaceChildren();
  return out;
}

async function stress(option: Option, name: string) {
  const stalls: number[] = [];
  const frameGaps: number[] = [];
  let running = true;
  // A timer that expects to run each 16 ms.
  (async () => {
    while (running) {
      const t = now();
      await sleep(16);
      const late = now() - t - 16;
      if (late > 50) stalls.push(late);
    }
  })();
  (async () => {
    let last = await frame();
    while (running) {
      const t = await frame();
      if (t - last > 50) frameGaps.push(t - last);
      last = t;
    }
  })();
  const cpu = async () => (await invoke<{ total_cpu_ms: number; total_rss_kb: number }>("tree_stats"));
  const s0 = await cpu();
  await invoke("sampler_start", { intervalMs: 50 });
  const t0 = now();
  const opened = await api!.open(option, name);
  const t1 = now();
  const s1 = await cpu();
  const openStalls = { timer: [...stalls], frames: [...frameGaps] };
  stalls.length = 0; frameGaps.length = 0;
  const c = api!.current();
  const scroll = c ? await scrollRun(c, 60, 360) : null;
  const s2 = await cpu();
  const scrollStalls = { timer: [...stalls], frames: [...frameGaps] };
  stalls.length = 0; frameGaps.length = 0;
  await sleep(2000);
  const s3 = await cpu();
  const idleStalls = { timer: [...stalls], frames: [...frameGaps] };
  running = false;
  const mem = await invoke("sampler_stop");
  const sum = (a: number[]) => a.reduce((x, y) => x + y, 0);
  const desc = (s: { timer: number[]; frames: number[] }) => ({
    timer_stalls: s.timer.length, timer_stall_ms_total: sum(s.timer), timer_stall_ms_max: s.timer.length ? Math.max(...s.timer) : 0,
    frame_gaps_over_50ms: s.frames.length, frame_gap_ms_max: s.frames.length ? Math.max(...s.frames) : 0,
  });
  return {
    open: { ok: opened.ok, error: opened.error, phases: opened.phases, wall_ms: t1 - t0, cpu_ms: s1.total_cpu_ms - s0.total_cpu_ms, stalls: desc(openStalls) },
    scroll: { result: scroll, cpu_ms: s2.total_cpu_ms - s1.total_cpu_ms, stalls: desc(scrollStalls) },
    idle_2s: { cpu_ms: s3.total_cpu_ms - s2.total_cpu_ms, stalls: desc(idleStalls) },
    rss_kb_before: s0.total_rss_kb, rss_kb_after_open: s1.total_rss_kb, rss_kb_end: s3.total_rss_kb,
    memory_sampler: mem,
    tree_end: await invoke("tree_stats"),
  };
}

let api: Api | null = null;

export async function q54(a: Api): Promise<boolean> {
  const cfg = await invoke<Config | null>("q54_config");
  if (!cfg) return false;
  api = a;
  const option = cfg.option;
  const parts = new Set(cfg.parts === "all" ? ["coord", "findings", "scroll", "live", "hscroll"] : cfg.parts.split("+"));
  const names = cfg.fixtures.split(",");
  const recs: Rec[] = [];
  const push = (r: Rec) => { recs.push(r); console.log(`q54 ${r.part} ${r.fixture ?? ""}: ${r.pass === undefined ? "measured" : r.pass ? "pass" : "FAIL"}`); };
  const openOk = async (name: string) => {
    const r = await a.open(option, name);
    if (!r.ok) push({ part: "open", fixture: name, pass: false, facts: { error: r.error, checks: r.checks } });
    return r.ok ? a.current() : null;
  };
  try {
    if (parts.has("coord") || parts.has("findings")) {
      for (const name of names) {
        status(`Q54 option ${option}: coordinates, ${name}`);
        const c = await openOk(name);
        if (!c) continue;
        await frames();
        const coords = coordinates(c);
        push({ part: "coordinates", fixture: name, pass: coords.pass, facts: coords });
        const rows = await domRows(c);
        push({ part: "dom_rows", fixture: name, pass: rows.pass, facts: rows });
        if (parts.has("findings") && name !== "50mb-stress") {
          const f = await findings(c);
          push({ part: "findings", fixture: name, pass: f.pass, facts: f });
        }
      }
    }
    if (parts.has("scroll")) {
      for (const name of ["1mb", "10mb"].filter((n) => names.includes(n))) {
        status(`Q54 option ${option}: scroll, ${name}`);
        const c = await openOk(name);
        if (!c) continue;
        await sleep(500);
        for (const step of [20, 60]) push({ part: "scroll", fixture: name, facts: await scrollRun(c, step, 360) });
      }
    }
    if (parts.has("live")) {
      for (const name of ["small", "1mb", "10mb"].filter((n) => names.includes(n))) {
        for (const mode of ["replace", "insert-above"] as const) {
          status(`Q54 option ${option}: live refresh ${mode}, ${name}`);
          const c = await openOk(name);
          if (!c) continue;
          await sleep(300);
          const r = await live(c, mode, 11);
          push({ part: `live-${mode}`, fixture: name, pass: r.pass, facts: r });
        }
      }
    }
    if (parts.has("hscroll")) {
      status(`Q54 option ${option}: horizontal scroll bar`);
      for (const v of await hscroll(option)) push({ part: "hscroll", pass: (v as { pass: boolean }).pass, facts: v as Record<string, unknown> });
    }
    if (parts.has("stress")) {
      for (const name of names) {
        status(`Q54 option ${option}: stress, ${name}`);
        push({ part: "stress", fixture: name, facts: await stress(option, name) });
      }
    }
  } catch (e) {
    push({ part: "error", pass: false, facts: { error: String(e), stack: (e as Error).stack } });
  }
  const doc = {
    prototype: "Diffy S1, map question Q54 (verdict phase). Never merged.",
    created: new Date().toISOString(),
    option,
    system: await invoke("system_info"),
    user_agent: navigator.userAgent,
    device_pixel_ratio: window.devicePixelRatio,
    window: { width: window.innerWidth, height: window.innerHeight },
    method: METHOD,
    passed: recs.filter((r) => r.pass === true).length,
    failed: recs.filter((r) => r.pass === false).map((r) => `${r.part} ${r.fixture ?? ""}`.trim()),
    records: recs,
  };
  const path = await invoke<string>("save_result", { label: cfg.label, json: JSON.stringify(doc, null, 2) });
  status(`Q54 done: ${path}`);
  console.log(`q54 result: ${path}`);
  await sleep(300);
  await invoke("quit", { code: 0 });
  return true;
}
