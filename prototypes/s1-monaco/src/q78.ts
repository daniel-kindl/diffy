// Throwaway check for map question Q78. Never merged.
// Runs when DIFFY_Q78_LABEL is set. fcitx5 with Mozc, the preedit on (Q57),
// option B (ADR 0004). Real keys come from xdotool (X11) or ydotool (Wayland).
// KWin gives the screen geometry of each window, so the check can compare the
// candidate window with the caret.

import * as monaco from "monaco-editor/editor";
import { invoke } from "@tauri-apps/api/core";

type Open = (option: "A" | "B", name: string) => Promise<{ ok: boolean; error?: string }>;
type Config = { label: string; backend: "x11" | "wayland" };
type Win = { cls: string; caption: string; x: number; y: number; w: number; h: number; cx: number; cy: number; popup: boolean; active: boolean; pid: number; managed?: boolean };
type Check = { name: string; pass: boolean; facts: Record<string, unknown> };

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));
const status = (t: string) => { document.getElementById("status")!.textContent = t; };
const events: { t: number; type: string; detail: string }[] = [];

function listen() {
  const t0 = performance.now();
  const add = (type: string, detail: (e: Event) => string) =>
    window.addEventListener(type, (e) => { if (events.length < 2000) events.push({ t: Math.round(performance.now() - t0), type, detail: detail(e) }); }, true);
  add("keydown", (e) => { const k = e as KeyboardEvent; return `${k.key} code=${k.code} composing=${k.isComposing}`; });
  add("compositionstart", () => "");
  add("compositionupdate", (e) => (e as CompositionEvent).data);
  add("compositionend", (e) => (e as CompositionEvent).data);
  add("beforeinput", (e) => { const i = e as InputEvent; return `${i.inputType} ${JSON.stringify(i.data)} composing=${i.isComposing}`; });
}

async function ensureFocus(): Promise<boolean> {
  for (let i = 0; i < 30; i++) {
    if (document.hasFocus()) return true;
    if (i % 10 === 0) await invoke("focus_main").catch(() => {});
    await sleep(100);
  }
  return false;
}

// Keys go to the focused window. Send none unless this window has the focus.
async function key(step: string) {
  if (!(await ensureFocus())) throw new Error("the window does not have the focus; no keys sent");
  await invoke("input", { step });
}

const geometry = () => invoke<string>("q78_geometry").then((s) => JSON.parse(s) as Win[]);

// The caret of an editor or a textarea, in CSS pixels of the page.
function caretRect(ed: monaco.editor.ICodeEditor | null, ta: HTMLTextAreaElement | null) {
  if (ta) { const r = ta.getBoundingClientRect(); return { x: r.left + 4, y: r.top, h: 18 }; }
  const pos = ed!.getPosition()!;
  const v = ed!.getScrolledVisiblePosition(pos)!;
  const r = ed!.getDomNode()!.getBoundingClientRect();
  return { x: r.left + v.left, y: r.top + v.top, h: v.height };
}

async function attempt(cfg: Config, name: string, ed: monaco.editor.ICodeEditor | null, useTextarea: boolean): Promise<Check[]> {
  const out: Check[] = [];
  const from = events.length;
  let ta: HTMLTextAreaElement | null = null;
  try {
    if (useTextarea) {
      ta = document.createElement("textarea");
      ta.style.cssText = "position:fixed;left:520px;top:300px;width:220px;height:40px;z-index:10";
      document.body.append(ta);
      ta.focus();
    } else {
      ed!.focus();
      ed!.setPosition({ lineNumber: 3, column: 5 });
    }
    await sleep(200);
    // fcitx5 keeps the input method for each input context. Switch it after the focus.
    const im = await invoke<string>("q78_im", { name: "mozc" }).catch((e) => `ERROR ${e}`);
    const before = await geometry();
    await key("type nihon");
    await sleep(500);
    const preedit = events.slice(from).filter((e) => e.type.startsWith("composition"));
    // Space converts. A second space opens the candidate list.
    await key("key space");
    await sleep(400);
    await key("key space");
    await sleep(800);
    const during = await geometry();
    const caret = caretRect(ed, ta);
    await key("key Return");
    await sleep(400);
    const value = ta ? ta.value : ed!.getModel()!.getLineContent(3);
    const comp = events.slice(from).filter((e) => e.type.startsWith("composition"));
    const me = during.find((w) => /^diffy-s1-monaco$/i.test(w.cls) && !w.popup && w.caption !== "");
    const known = new Set(before.map((w) => `${w.pid}:${w.cls}:${w.x}:${w.y}:${w.w}:${w.h}`));
    const fresh = during.filter((w) => !known.has(`${w.pid}:${w.cls}:${w.x}:${w.y}:${w.w}:${w.h}`) );
    const dpr = window.devicePixelRatio;
    // KWin and CSS both use logical pixels at a page zoom of 1.
    const caretScreen = me ? { x: Math.round(me.cx + caret.x), y: Math.round(me.cy + caret.y), bottom: Math.round(me.cy + caret.y + caret.h) } : null;
    // The candidate window is a new popup. fcitx5-gtk draws it in the app on Wayland.
    const cand = fresh.find((w) => w.popup) ?? fresh[0];
    const dist = cand && caretScreen ? { dx: cand.x - caretScreen.x, dy: cand.y - caretScreen.bottom } : null;
    const near = !!dist && Math.abs(dist.dx) <= 60 && dist.dy >= -4 && dist.dy <= 60;
    const facts = { target: ta ? "textarea" : "editor", im, value: value.slice(0, 40), composition_events: comp.length,
      composition: comp.map((e) => `${e.type}:${e.detail}`), preedit_after_typing: preedit.map((e) => `${e.type}:${e.detail}`),
      beforeinput: events.slice(from).filter((e) => e.type === "beforeinput").map((e) => e.detail),
      app_window: me ?? null, caret_css: caret, device_pixel_ratio: dpr, caret_screen: caretScreen, new_windows: fresh, candidate: cand ?? null, offset: dist };
    out.push({ name: `${name}: commits Japanese text`, pass: /[぀-ヿ一-鿿]/.test(value), facts });
    out.push({ name: `${name}: gets composition events with the preedit`, pass: preedit.some((e) => e.type === "compositionupdate" && /[぀-ゟ]/.test(e.detail)), facts: { from: name } });
    out.push({ name: `${name}: the candidate window is next to the caret`, pass: near, facts: { from: name, rule: "candidate top-left within 60 px right of the caret and 0 to 60 px below the caret bottom", backend: cfg.backend } });
  } catch (e) { out.push({ name, pass: false, facts: { error: String(e) } }); }
  ta?.remove();
  return out;
}

export async function q78(open: Open): Promise<boolean> {
  const cfg = await invoke<Config | null>("q78_config");
  if (!cfg) return false;
  listen();
  document.title = `Q78 ${cfg.label}`;
  status(`Q78 check, ${cfg.backend}. Do not use the keyboard or the mouse.`);
  const meta: Record<string, unknown> = { system: await invoke("system_info"), user_agent: navigator.userAgent, im: await invoke("q78_im", { name: "" }).catch((e) => `ERROR ${e}`) };
  await invoke("focus_main").catch(() => {});
  meta.focus = await ensureFocus();
  const opened = await open("B", "small");
  const eds = monaco.editor.getEditors();
  const dst = eds[eds.length - 1];
  dst.updateOptions({ readOnly: false });
  await invoke("set_preedit", { enable: true });
  await sleep(500);
  const checks: Check[] = [{ name: "open the small fixture", pass: opened.ok, facts: { error: opened.error } }];
  checks.push(...await attempt(cfg, "fcitx5 mozc in the editor", dst, false));
  checks.push(...await attempt(cfg, "fcitx5 mozc in a plain textarea", null, true));
  meta.im_end = await invoke("q78_im", { name: "keyboard-us" }).catch((e) => `ERROR ${e}`);
  const doc = { prototype: "Diffy map question Q78. fcitx5 with the preedit on, on WebKitGTK.", created: new Date().toISOString(),
    backend: cfg.backend, ...meta, passed: checks.filter((c) => c.pass).length, failed: checks.filter((c) => !c.pass).map((c) => c.name), checks, events };
  const path = await invoke<string>("save_result", { label: cfg.label, json: JSON.stringify(doc, null, 2) });
  status(`Q78 done: ${doc.passed} of ${checks.length} passed. ${path}`);
  await sleep(500);
  await invoke("quit", { code: doc.failed.length === 0 ? 0 : 1 });
  return true;
}
