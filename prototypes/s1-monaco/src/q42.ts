// Throwaway S2 checks for map question Q42. Never merged.
// Runs when DIFFY_S2_CHECK=A or B. Each check records pass or fail with the facts.
// Real key events come from xdotool (X11) or ydotool (Wayland). A different
// application (xclip or wl-clipboard) is the other side of the clipboard.

import * as monaco from "monaco-editor/editor";
import { invoke } from "@tauri-apps/api/core";
// The Q41 build uses the core editor, which has no find widget and no tab focus
// mode. Q42 adds both, so that the shortcut checks test the WebKitGTK key path.
import "monaco-editor/editor/contrib/find/browser/findController.js";
import "monaco-editor/editor/contrib/toggleTabFocusMode/browser/toggleTabFocusMode.js";

type Option = "A" | "B";
type Open = (option: Option, name: string) => Promise<{ ok: boolean; error?: string; checks: Record<string, unknown> }>;
type Config = { option: Option; label: string; backend: "x11" | "wayland" };
type Check = { name: string; pass: boolean; facts: Record<string, unknown> };

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));
const frame = () => new Promise<void>((r) => requestAnimationFrame(() => r()));
const status = (t: string) => { document.getElementById("status")!.textContent = t; };

const events: { t: number; type: string; detail: string }[] = [];
function listen() {
  const t0 = performance.now();
  const add = (type: string, detail: (e: Event) => string) =>
    window.addEventListener(type, (e) => { if (events.length < 2000) events.push({ t: Math.round(performance.now() - t0), type, detail: detail(e) }); }, true);
  add("keydown", (e) => { const k = e as KeyboardEvent; return `${k.ctrlKey ? "ctrl+" : ""}${k.shiftKey ? "shift+" : ""}${k.key} code=${k.code} composing=${k.isComposing}`; });
  add("compositionstart", () => "");
  add("compositionupdate", (e) => (e as CompositionEvent).data);
  add("compositionend", (e) => (e as CompositionEvent).data);
  add("beforeinput", (e) => { const i = e as InputEvent; return `${i.inputType} ${JSON.stringify(i.data)} composing=${i.isComposing}`; });
  add("copy", () => "");
  add("paste", () => "");
  add("focusin", (e) => describe(e.target as Element));
}

function describe(el: Element | null): string {
  if (!el) return "none";
  if (el.closest(".monaco-editor")) {
    const eds = monaco.editor.getEditors();
    const i = eds.findIndex((ed) => ed.getContainerDomNode().contains(el));
    return `editor#${i}:${el.tagName.toLowerCase()}${el.closest(".find-widget") ? ".find" : ""}`;
  }
  return el.id ? `#${el.id}` : el.tagName.toLowerCase();
}

// Wait until the window has the keyboard focus. Do not send keys otherwise:
// on Wayland, ydotool sends them to the focused window, whichever it is.
async function ensureFocus(): Promise<boolean> {
  for (let i = 0; i < 30; i++) {
    if (document.hasFocus()) return true;
    if (i % 10 === 0) await invoke("focus_main").catch(() => {});
    await sleep(100);
  }
  return false;
}

async function key(step: string) {
  if (!(await ensureFocus())) throw new Error("the window does not have the focus; no keys sent");
  await invoke("input", { step });
  await sleep(250);
}

function normalize(s: string) { return s.replace(/\r\n/g, "\n"); }

async function checkClipboard(cfg: Config, src: monaco.editor.ICodeEditor, dst: monaco.editor.ICodeEditor): Promise<Check[]> {
  const out: Check[] = [];
  // App to other application.
  await invoke("ext_clipboard_write", { text: "q42-sentinel" });
  src.focus();
  src.setSelection(new monaco.Range(2, 1, 4, 1));
  const expected = normalize(src.getModel()!.getValueInRange(src.getSelection()!));
  let facts: Record<string, unknown> = { shortcut: "ctrl+c", source: "read-only editor" };
  try {
    await key("key ctrl+c");
    await sleep(300);
    const readers = cfg.backend === "x11" ? ["x11", "wayland"] : ["wayland"];
    // Poll for up to 3 s: the clipboard owner can change after the copy event.
    const t0 = performance.now();
    for (const via of readers) {
      let got = "";
      for (let i = 0; i < 30; i++) {
        got = normalize(await invoke<string>("ext_clipboard_read", { via }).catch((e) => `ERROR ${e}`));
        if (got === expected) break;
        await sleep(100);
      }
      facts[`read_by_${via}`] = got;
      facts[`ms_until_${via}_read`] = Math.round(performance.now() - t0);
    }
    facts.expected = expected;
    out.push({ name: "clipboard: copy to another application", pass: readers.every((v) => facts[`read_by_${v}`] === expected), facts });
  } catch (e) { out.push({ name: "clipboard: copy to another application", pass: false, facts: { ...facts, error: String(e) } }); }

  // Other application to app.
  const marker = `q42-paste-${Math.random().toString(36).slice(2, 10)}`;
  facts = { shortcut: "ctrl+v", target: "editable editor", writer: cfg.backend === "x11" ? "xclip" : "wl-copy", paste_marker: marker };
  try {
    await invoke("ext_clipboard_write", { text: marker });
    await sleep(200);
    dst.focus();
    dst.setPosition({ lineNumber: 1, column: 1 });
    await key("key ctrl+v");
    const line1 = dst.getModel()!.getLineContent(1);
    facts.line1 = line1.slice(0, 80);
    const pasted = line1.startsWith(marker);
    await key("key ctrl+z");
    facts.undo_restored = !dst.getModel()!.getLineContent(1).startsWith(marker);
    out.push({ name: "clipboard: paste from another application", pass: pasted, facts });
  } catch (e) { out.push({ name: "clipboard: paste from another application", pass: false, facts: { ...facts, error: String(e) } }); }
  return out;
}

async function checkShortcuts(ed: monaco.editor.ICodeEditor, which: string): Promise<Check[]> {
  const out: Check[] = [];
  const model = ed.getModel()!;
  const run = async (name: string, step: string, test: () => boolean, facts: () => Record<string, unknown> = () => ({})) => {
    try {
      await key(step);
      await sleep(150);
      out.push({ name: `keyboard (${which}): ${name}`, pass: test(), facts: { step, ...facts() } });
    } catch (e) { out.push({ name: `keyboard (${which}): ${name}`, pass: false, facts: { step, error: String(e) } }); }
  };
  const findVisible = () => !!ed.getContainerDomNode().querySelector(".find-widget.visible");
  ed.focus();
  ed.setPosition({ lineNumber: 1, column: 1 });
  await run("ctrl+a selects all", "key ctrl+a", () => ed.getSelection()!.equalsRange(model.getFullModelRange()), () => ({ selection: ed.getSelection()?.toString() }));
  await run("ctrl+End moves to the end", "key ctrl+End", () => ed.getPosition()!.lineNumber === model.getLineCount());
  await run("ctrl+Home moves to the start", "key ctrl+Home", () => ed.getPosition()!.lineNumber === 1);
  await run("Next (PageDown) scrolls", "key Next", () => ed.getScrollTop() > 0, () => ({ scrollTop: ed.getScrollTop() }));
  const actions = ed.getSupportedActions().map((a) => a.id);
  await run("ctrl+f opens the find widget", "key ctrl+f", findVisible, () => ({ has_find_action: actions.includes("actions.find") }));
  if (!findVisible()) {
    ed.focus();
    ed.trigger("q42", "actions.find", null);
    await sleep(300);
    out[out.length - 1].facts.visible_after_programmatic_trigger = findVisible();
    if (findVisible()) { ed.trigger("q42", "closeFindWidget", null); await sleep(200); }
  }
  if (findVisible()) {
    try {
      await key("type return");
      await key("key Return");
      const sel = ed.getSelection()!;
      const text = model.getValueInRange(sel);
      out.push({ name: `keyboard (${which}): typed query and Return select a match`, pass: text.toLowerCase() === "return", facts: { selected: text, at: sel.toString() } });
    } catch (e) { out.push({ name: `keyboard (${which}): typed query and Return select a match`, pass: false, facts: { error: String(e) } }); }
    await run("Escape closes the find widget", "key Escape", () => !findVisible());
  }
  return out;
}

async function checkTabOrder(): Promise<Check> {
  const order: string[] = [];
  const facts: Record<string, unknown> = {};
  try {
    (document.getElementById("option") as HTMLElement).focus();
    order.push(describe(document.activeElement));
    for (let i = 0; i < 6; i++) { await key("key Tab"); order.push(describe(document.activeElement)); }
    facts.forward = [...order];
    const back: string[] = [];
    (document.getElementById("bench") as HTMLElement).focus();
    for (let i = 0; i < 4; i++) { await key("key shift+Tab"); back.push(describe(document.activeElement)); }
    facts.backward = back;
    const expected = ["#option", "#fixture", "#open", "#runs", "#bench"];
    const pass = expected.every((id, i) => order[i] === id) && order[5].startsWith("editor#")
      && ["#runs", "#open", "#fixture", "#option"].every((id, i) => back[i] === id);
    facts.expected_prefix = expected;
    return { name: "keyboard: Tab focus order", pass, facts };
  } catch (e) { return { name: "keyboard: Tab focus order", pass: false, facts: { ...facts, order, error: String(e) } }; }
}

async function checkTabTrap(ed: monaco.editor.ICodeEditor): Promise<Check> {
  // In an editable Monaco editor Tab indents. ctrl+m turns on the tab focus mode, so Tab moves the focus.
  const facts: Record<string, unknown> = {};
  try {
    ed.focus();
    ed.setPosition({ lineNumber: 1, column: 1 });
    const before = ed.getModel()!.getLineContent(1);
    await key("key Tab");
    facts.tab_indents = ed.getModel()!.getLineContent(1) !== before;
    facts.has_toggle_tab_focus_action = ed.getSupportedActions().some((a) => a.id === "editor.action.toggleTabFocusMode");
    await key("key ctrl+z");
    await key("key ctrl+m");
    await key("key Tab");
    facts.focus_after_ctrl_m_tab = describe(document.activeElement);
    const left = !ed.hasTextFocus();
    ed.focus();
    await key("key ctrl+m");
    return { name: "keyboard: leave an editable editor with ctrl+m then Tab", pass: left, facts };
  } catch (e) { return { name: "keyboard: leave an editable editor with ctrl+m then Tab", pass: false, facts: { ...facts, error: String(e) } }; }
}

const PLAIN_ENGINE = "xkb:us::eng";

async function checkIme(ed: monaco.editor.ICodeEditor): Promise<Check[]> {
  const out: Check[] = [];
  const engine = (name: string) => invoke<string>("ime_engine", { name }).catch((e) => `ERROR ${e}`);
  const attempt = async (name: string, steps: string[], expect: string, target: "editor" | "textarea" = "editor") => {
    const from = events.length;
    let ta: HTMLTextAreaElement | null = null;
    try {
      if (target === "textarea") {
        ta = document.createElement("textarea");
        ta.style.cssText = "position:fixed;right:8px;bottom:8px;width:200px;height:40px;z-index:10";
        document.body.append(ta);
        ta.focus();
      } else {
        ed.focus();
        ed.setPosition({ lineNumber: 1, column: 1 });
      }
      for (const s of steps) await key(s);
      await sleep(300);
      const value = ta ? ta.value : ed.getModel()!.getLineContent(1);
      const comp = events.slice(from).filter((e) => e.type.startsWith("composition"));
      const inputs = events.slice(from).filter((e) => e.type === "beforeinput").map((e) => e.detail);
      out.push({ name, pass: value.startsWith(expect), facts: { target, steps, expect, value: value.slice(0, 40), composition_events: comp.length, composition: comp.map((e) => `${e.type}:${e.detail}`), beforeinput: inputs } });
      if (!ta && value.startsWith(expect)) await key("key ctrl+z");
    } catch (e) { out.push({ name, pass: false, facts: { target, steps, error: String(e) } }); }
    ta?.remove();
  };
  const composed = (name: string) => out.push({ name, pass: (out[out.length - 1].facts.composition_events as number) > 0, facts: { from: out[out.length - 1].name } });

  // The m17n latn-post engine: "e" then "'" composes "é".
  out.push({ name: "input method: engine m17n:t:latn-post", pass: (await engine("m17n:t:latn-post")) === "m17n:t:latn-post", facts: {} });
  await attempt("input method: ibus m17n latn-post composes é", ["key e", "key apostrophe", "key space"], "é");
  // Unicode entry: ctrl+shift+u e 9 space.
  await attempt("input method: ctrl+shift+u e9 enters é", ["key ctrl+shift+u", "key e", "key 9", "key space"], "é");

  // libpinyin has a preedit, so it tests the composition path: "nihao" then space commits 你好.
  out.push({ name: "input method: engine libpinyin", pass: (await engine("libpinyin")) === "libpinyin", facts: {} });
  await sleep(300);
  await attempt("input method: libpinyin commits 你好 (wry default)", ["type nihao", "key space"], "你好");
  composed("input method: the editor gets composition events (wry default: preedit off)");
  await attempt("input method: libpinyin commits 你好 in a plain textarea (wry default)", ["type nihao", "key space"], "你好", "textarea");
  // Workaround candidate: turn the preedit on again.
  await invoke("set_preedit", { enable: true });
  await sleep(200);
  await attempt("input method: libpinyin commits 你好 (preedit on)", ["type nihao", "key space"], "你好");
  composed("input method: the editor gets composition events (workaround: preedit on)");
  await attempt("input method: libpinyin commits 你好 in a plain textarea (preedit on)", ["type nihao", "key space"], "你好", "textarea");
  composed("input method: a plain textarea gets composition events (workaround: preedit on)");
  await invoke("set_preedit", { enable: false });
  await engine(PLAIN_ENGINE);
  return out;
}

// The DOM lines of each editor inside its viewport, as the snapshot check wants them.
function lineBoxes() {
  const boxes: { x: number; y: number; w: number; h: number; text: boolean }[] = [];
  for (const ed of monaco.editor.getEditors()) {
    const root = ed.getContainerDomNode().getBoundingClientRect();
    const layout = ed.getLayoutInfo();
    const left = root.left + layout.contentLeft;
    const right = root.left + layout.contentLeft + layout.contentWidth - layout.verticalScrollbarWidth;
    for (const el of ed.getContainerDomNode().querySelectorAll<HTMLElement>(".view-lines .view-line")) {
      const r = el.getBoundingClientRect();
      if (r.bottom <= root.top || r.top >= root.bottom) continue;
      const text = el.textContent ?? "";
      // Measure the text width so that a short line is not checked past its end.
      const span = el.firstElementChild as HTMLElement | null;
      const w = Math.min(right, r.left + (span?.getBoundingClientRect().width ?? r.width)) - left;
      const top = Math.max(r.top, root.top);
      const bottom = Math.min(r.bottom, root.bottom);
      boxes.push({ x: left, y: top, w: Math.max(0, w), h: bottom - top, text: text.trim().length > 0 && bottom - top >= r.height });
    }
  }
  return boxes;
}

// The text area of each editor, without the line numbers and the scroll bar.
function regions() {
  return monaco.editor.getEditors().map((ed) => {
    const root = ed.getContainerDomNode().getBoundingClientRect();
    const l = ed.getLayoutInfo();
    return { x: root.left + l.contentLeft, y: root.top, w: l.contentWidth - l.verticalScrollbarWidth, h: root.height };
  });
}

// The longest run of empty lines in the 10 MB fixture is 8. A longer run with no ink is a blank area.
const MAX_BLANK_LINES = 9;

async function snap(save?: string) {
  const lineHeight = monaco.editor.getEditors()[0]?.getOption(monaco.editor.EditorOption.lineHeight) ?? 18;
  return invoke<{ text_lines: number; text_lines_without_ink: number; empty_lines_with_ink: number; longest_blank_run_lines: number; saved?: string; scale: number }>(
    "snapshot_check", { lines: lineBoxes(), regions: regions(), lineHeight, cssWidth: window.innerWidth, save: save ?? null });
}

async function checkScroll(cfg: Config, open: Open): Promise<Check[]> {
  const out: Check[] = [];
  const opened = await open(cfg.option, "10mb");
  if (!opened.ok) return [{ name: "rendering: open the 10 MB fixture", pass: false, facts: { error: opened.error, checks: opened.checks } }];
  await sleep(500);
  const eds = monaco.editor.getEditors();
  const main = eds[eds.length - 1];
  const height = main.getContentHeight();
  // Jumps: 40 positions over the whole file. Snapshot one frame after the jump and again later.
  let early = 0, late = 0, lines = 0, worst = 0, blankRun = 0;
  const samples: unknown[] = [];
  for (let i = 0; i < 40; i++) {
    const top = Math.round((height - main.getLayoutInfo().height) * (i / 39));
    main.setScrollTop(top);
    if (cfg.option === "A") eds[0].setScrollTop(top);
    await frame();
    const a = await snap(i === 20 ? `${cfg.label}-10mb-jump20` : undefined);
    await sleep(250);
    const b = await snap();
    early += a.text_lines_without_ink;
    late += b.text_lines_without_ink;
    lines += b.text_lines;
    worst = Math.max(worst, a.text_lines_without_ink);
    blankRun = Math.max(blankRun, a.longest_blank_run_lines, b.longest_blank_run_lines);
    if (a.text_lines_without_ink || b.text_lines_without_ink) samples.push({ i, top, early: a.text_lines_without_ink, late: b.text_lines_without_ink, of: b.text_lines });
  }
  out.push({ name: "rendering: no blank lines 250 ms after a jump scroll (10 MB)", pass: late === 0, facts: { jumps: 40, text_lines_checked: lines, without_ink_after_250ms: late, samples } });
  out.push({ name: "rendering: no blank lines one frame after a jump scroll (10 MB)", pass: early === 0, facts: { without_ink_one_frame_after: early, worst_single_snapshot: worst } });
  out.push({ name: "rendering: no blank area after a jump scroll, image only (10 MB)", pass: blankRun <= MAX_BLANK_LINES, facts: { longest_blank_run_lines: blankRun, limit: MAX_BLANK_LINES } });

  // Real key input: 60 PageDown events while snapshots run.
  main.focus();
  main.setPosition({ lineNumber: 1, column: 1 });
  main.setScrollTop(0);
  const runs: number[] = [];
  let done = false;
  const keys = (async () => { if (await ensureFocus()) await invoke("input", { step: "repeat 60 Next" }); done = true; })();
  while (!done && runs.length < 200) {
    const s = await snap(runs.length === 5 ? `${cfg.label}-10mb-pagedown` : undefined);
    runs.push(Math.round(s.longest_blank_run_lines * 10) / 10);
  }
  await keys;
  await sleep(1000);
  const after = await snap(`${cfg.label}-10mb-after-pagedown`);
  const worstRun = Math.max(after.longest_blank_run_lines, ...runs);
  out.push({
    name: "rendering: no blank area during 60 PageDown key events, image only (10 MB)",
    pass: runs.length > 0 && worstRun <= MAX_BLANK_LINES && main.getScrollTop() > 0,
    facts: { snapshots: runs.length, longest_blank_run_lines_each: runs, after: after.longest_blank_run_lines, after_text_lines_without_ink: after.text_lines_without_ink, limit: MAX_BLANK_LINES, scrollTop: main.getScrollTop() },
  });
  return out;
}

function fontFacts() {
  const el = document.querySelector(".monaco-editor .view-line span") as HTMLElement | null;
  const family = el ? getComputedStyle(el).fontFamily : "";
  const families = family.split(",").map((f) => f.trim().replace(/^["']|["']$/g, "")).filter(Boolean);
  // Monaco measures the rendered font itself.
  const info = monaco.editor.getEditors()[0]?.getOption(monaco.editor.EditorOption.fontInfo);
  return { family, families, monaco_font_info: info ? { isMonospace: info.isMonospace, typicalHalfwidthCharacterWidth: info.typicalHalfwidthCharacterWidth, typicalFullwidthCharacterWidth: info.typicalFullwidthCharacterWidth, lineHeight: info.lineHeight, fontSize: info.fontSize } : null };
}

export async function q42(open: Open): Promise<boolean> {
  const cfg = await invoke<Config | null>("s2_config");
  if (!cfg) return false;
  listen();
  const checks: Check[] = [];
  const meta: Record<string, unknown> = {};
  status(`Q42 checks: option ${cfg.option}, ${cfg.backend}. Do not use the keyboard or the mouse.`);
  meta.system = await invoke("system_info");
  meta.user_agent = navigator.userAgent;
  meta.device_pixel_ratio = window.devicePixelRatio;
  meta.window = { width: window.innerWidth, height: window.innerHeight };
  meta.gpu_report = await invoke<string>("gpu_report").catch((e) => `ERROR ${e}`);
  await invoke("focus_main").catch(() => {});
  meta.focus = await ensureFocus();
  meta.ime_engine_for_other_checks = await invoke<string>("ime_engine", { name: PLAIN_ENGINE }).catch((e) => `ERROR ${e}`);

  const opened = await open(cfg.option, "small");
  meta.editor_geometry = monaco.editor.getEditors().map((ed) => {
    const fi = ed.getOption(monaco.editor.EditorOption.fontInfo);
    const zones = [...ed.getContainerDomNode().querySelectorAll<HTMLElement>(".view-zones > div")].map((z) => z.style.height);
    return { lineHeight: ed.getOption(monaco.editor.EditorOption.lineHeight), fontLineHeight: fi.lineHeight, lines: ed.getModel()?.getLineCount(), contentHeight: ed.getContentHeight(), scrollHeight: ed.getScrollHeight(), zones, layoutHeight: ed.getLayoutInfo().height, horizontalScrollbar: ed.getLayoutInfo().horizontalScrollbarHeight };
  });
  checks.push({ name: "open the small fixture", pass: opened.ok, facts: { error: opened.error, checks: opened.checks } });
  await sleep(500);
  const eds = monaco.editor.getEditors();
  if (cfg.option === "B") {
    // Workaround candidate: a horizontal scroll bar on one side only adds its height to that side.
    const before = eds.map((e) => e.getContentHeight());
    eds.forEach((e) => e.updateOptions({ scrollbar: { horizontal: "visible" } }));
    await sleep(300);
    const after = eds.map((e) => e.getContentHeight());
    checks.push({ name: "option B: equal content heights with the horizontal scroll bar always visible (workaround)", pass: after[0] === after[1], facts: { before, after } });
  }
  const src = eds[0];
  const dst = eds[eds.length - 1];
  // The check needs an editable editor for paste and input. The modified (right) side is it.
  dst.updateOptions({ readOnly: false });
  if (cfg.option === "A") (eds[0] as monaco.editor.ICodeEditor).updateOptions({ readOnly: true });

  const fonts = fontFacts();
  meta.fonts = { ...fonts, fc_match: await invoke("font_match", { families: fonts.families }) };
  const first = await snap(`${cfg.label}-small`);
  checks.push({ name: "rendering: each text line of the small fixture has ink", pass: first.text_lines > 0 && first.text_lines_without_ink === 0, facts: first });

  checks.push(...await checkClipboard(cfg, src, dst));
  checks.push(...await checkShortcuts(src, "read-only"));
  checks.push(...await checkShortcuts(dst, "editable"));
  checks.push(await checkTabOrder());
  checks.push(await checkTabTrap(dst));
  checks.push(...await checkIme(dst));
  checks.push(...await checkScroll(cfg, open));

  const doc = {
    prototype: "Diffy S2, map question Q42. Checks of the Q41 prototype on WebKitGTK.",
    created: new Date().toISOString(),
    option: cfg.option,
    backend: cfg.backend,
    ...meta,
    passed: checks.filter((c) => c.pass).length,
    failed: checks.filter((c) => !c.pass).map((c) => c.name),
    checks,
    events,
  };
  const path = await invoke<string>("save_result", { label: cfg.label, json: JSON.stringify(doc, null, 2) });
  status(`Q42 done: ${doc.passed} of ${checks.length} passed. ${path}`);
  console.log(`q42 result: ${path}`);
  await sleep(500);
  await invoke("quit", { code: doc.failed.length === 0 ? 0 : 1 });
  return true;
}
