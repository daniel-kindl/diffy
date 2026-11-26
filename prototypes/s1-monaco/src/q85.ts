// Throwaway check for map question Q85. Never merged.
// Runs when DIFFY_Q85_LABEL is set. Opens option B (ADR 0004) on the small
// fixture, sets Monaco accessibilitySupport, focuses one editor on a changed
// line and sets the document title to "Q85 ready". q85-atspi.py then reads the
// AT-SPI tree, which is what Orca reads.

import * as monaco from "monaco-editor/editor";
import { invoke } from "@tauri-apps/api/core";

type Open = (option: "A" | "B", name: string) => Promise<{ ok: boolean; error?: string }>;
type Config = { label: string; sr: "on" | "off" | "auto"; target: "left" | "right"; fix: boolean; fixAttr?: string };
type Block = { left_start: number; left_count: number; right_start: number; right_count: number };

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

export async function q85(open: Open): Promise<boolean> {
  const cfg = await invoke<Config | null>("q85_config");
  if (!cfg) return false;
  document.title = `Q85 ${cfg.label}`;
  try {
    await run(cfg, open);
  } catch (e) {
    await invoke<string>("save_result", { label: `${cfg.label}-error`, json: JSON.stringify({ error: String(e), stack: (e as Error).stack }) });
    document.title = `Q85 ready ${cfg.label} error`;
  }
  return true;
}

async function run(cfg: Config, open: Open) {
  const step = (name: string) => invoke<string>("save_result", { label: `${cfg.label}-step`, json: JSON.stringify({ step: name, t: performance.now() }) });
  await step("start");
  await invoke("focus_main").catch(() => {});
  await sleep(500);
  const raf = await Promise.race([
    new Promise<string>((r) => requestAnimationFrame(() => r("raf fired"))),
    sleep(3000).then(() => "raf timeout"),
  ]);
  await step(`focused, ${raf}, visibility ${document.visibilityState}, hasFocus ${document.hasFocus()}, open`);
  const opened = await open("B", "small");
  await step("opened");
  const eds = monaco.editor.getEditors() as monaco.editor.IStandaloneCodeEditor[];
  for (const ed of eds) ed.updateOptions({ accessibilitySupport: cfg.sr });
  const canon = await invoke<{ blocks: Block[] }>("canonical_hunks", { name: "small" });
  const ed = cfg.target === "left" ? eds[0] : eds[1];
  const block = canon.blocks.find((b) => (cfg.target === "left" ? b.left_count : b.right_count) > 0)!;
  const line = cfg.target === "left" ? block.left_start : block.right_start;
  if (cfg.fix) {
    // Candidate M0 fixes: mark each textarea read-only for AT-SPI, and announce
    // the change type of the caret line in a polite live region.
    for (const e of eds) e.getDomNode()!.querySelector("textarea")!.setAttribute(cfg.fixAttr ?? "aria-readonly", cfg.fixAttr === "readonly" ? "" : "true");
    const live = document.createElement("div");
    live.id = "q85-live";
    live.setAttribute("aria-live", "polite");
    live.style.cssText = "position:absolute;left:-10000px;width:1px;height:1px;overflow:hidden";
    document.body.append(live);
    const announce = (e: monaco.editor.IStandaloneCodeEditor) => {
      const n = e.getPosition()!.lineNumber;
      const cls = (e.getLineDecorations(n) ?? []).map((d) => d.options.className);
      const kind = cls.includes("line-added") ? "added" : cls.includes("line-removed") ? "removed" : "unchanged";
      live.textContent = `${e === eds[0] ? "Original" : "Modified"}, line ${n}, ${kind}`;
    };
    for (const e of eds) e.onDidChangeCursorPosition(() => announce(e));
  }
  ed.focus();
  ed.setPosition({ lineNumber: line, column: 1 });
  ed.revealLineInCenter(line);
  await sleep(800);
  const ta = ed.getDomNode()!.querySelector("textarea");
  const lineDecorations = (ed.getLineDecorations(line) ?? []).map((d) => d.options.className).filter(Boolean);
  const facts = {
    opened,
    target: cfg.target,
    accessibility_support: cfg.sr,
    line,
    line_text: ed.getModel()!.getLineContent(line),
    line_decorations: lineDecorations,
    editor_read_only: ed.getOption(monaco.editor.EditorOption.readOnly),
    textarea: ta && {
      focused: document.activeElement === ta,
      role: ta.getAttribute("role"),
      aria_label: ta.getAttribute("aria-label"),
      aria_roledescription: ta.getAttribute("aria-roledescription"),
      aria_readonly: ta.getAttribute("aria-readonly"),
      readonly: ta.hasAttribute("readonly"),
      aria_multiline: ta.getAttribute("aria-multiline"),
      value: ta.value,
      selection: [ta.selectionStart, ta.selectionEnd],
    },
    live_regions: [...document.querySelectorAll("[aria-live]")].map((e) => ({ live: e.getAttribute("aria-live"), text: e.textContent })),
  };
  await invoke<string>("save_result", { label: `${cfg.label}-dom`, json: JSON.stringify(facts, null, 2) });
  document.title = `Q85 ready ${cfg.label}`;
}
