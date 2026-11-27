import "./style.css";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { setEditorTheme, showComparison, showUnified } from "./editor";
import { nextPreference, parsePreference, resolveTheme, type ThemePreference } from "./theme";
import { LEGACY_ENCODINGS } from "./encodings";
import { scrollToRestore, viewKey, type ViewMemory } from "./viewMemory";
import type { Hunk } from "./hunks";

// A page that fails to run leaves a trace on the standard error output of Rust.
function report(message: string): void {
  invoke("report_ui_error", { message }).catch(() => {
    // Nothing more can be done when the report itself fails.
  });
}
window.addEventListener("error", (e) => {
  report(`${e.message} at ${e.filename}:${String(e.lineno)}`);
});
document.addEventListener("securitypolicyviolation", (e) => {
  report(`blocked by the content security policy: ${e.violatedDirective} ${e.blockedURI}`);
});
window.addEventListener("unhandledrejection", (e) => {
  report(`rejection: ${String(e.reason)}`);
});

interface Side {
  label: string;
  encoding: string;
  encoding_override: string | null;
  first_error: number | null;
}

interface Comparison {
  base: string;
  head: string;
  hunks: Hunk[];
  sides: Side[];
}

// The frontend never names a file: Rust owns the inputs. With no input, a
// small example shows the renderer.
const sample = { base: "one\ntwo\nthree\n", head: "one\nTWO\nthree\nfour\n" };

async function load(): Promise<Comparison> {
  const current = await invoke<Comparison | null>("current_comparison");
  return current ?? invoke<Comparison>("compare_text", { input: sample });
}

function describe(side: Side | undefined, fallback: string): string {
  if (!side) return fallback;
  const error =
    side.first_error === null ? "" : ` – invalid bytes from offset ${String(side.first_error)}`;
  return `${side.label} (${side.encoding})${error}`;
}

const left = document.getElementById("left");
const right = document.getElementById("right");
const status = document.getElementById("status");
let view: { dispose(): void; scrollTop(): number; setScrollTop(top: number): void } | undefined;
let memory: ViewMemory | undefined;
let last: Comparison | undefined;
let unifiedMode = false;
let collapseMode = true;

function show(c: Comparison): void {
  if (!left || !right) return;
  last = c;
  // Remember where the user was, so a refresh keeps the place (L-04).
  if (view) memory = { key: memory?.key ?? "", scrollTop: view.scrollTop() };
  view?.dispose();
  if (status) {
    status.textContent = `${describe(c.sides[0], "Base")} ↔ ${describe(c.sides[1], "Head")}`;
  }
  for (const select of encodingSelectors) {
    const side = c.sides[select.dataset["side"] === "left" ? 0 : 1];
    select.disabled = side?.encoding_override == null;
    select.value = side?.encoding_override ?? "";
  }
  const unified = document.getElementById("unified");
  const app = document.getElementById("app");
  const path = c.sides[1]?.label ?? "";
  if (unifiedMode && unified && app) {
    app.hidden = true;
    unified.hidden = false;
    view = showUnified(unified, c.base, c.head, c.hunks, path, collapseMode);
  } else {
    if (unified) unified.hidden = true;
    if (app) app.hidden = false;
    view = showComparison(left, right, c.base, c.head, c.hunks, path);
  }
  const key = viewKey(
    c.sides.map((x) => x.label),
    unifiedMode,
  );
  const top = scrollToRestore(memory, key);
  memory = { key, scrollTop: 0 };
  if (top > 0) {
    const shown = view;
    // The editors need their first layout before they can scroll.
    requestAnimationFrame(() => shown.setScrollTop(top));
  }
}

function fail(e: unknown): void {
  if (status) status.textContent = `Diffy could not load the comparison: ${String(e)}`;
}

const encodingSelectors = document.querySelectorAll<HTMLSelectElement>("select[data-side]");
for (const select of encodingSelectors) {
  for (const { label, name } of LEGACY_ENCODINGS) {
    select.append(new Option(name, label));
  }
  select.addEventListener("change", () => {
    invoke<Comparison | null>("select_encoding", {
      side: select.dataset["side"],
      label: select.value,
    })
      .then((c) => c && show(c))
      .catch(fail);
  });
}

document.getElementById("swap")?.addEventListener("click", () => {
  invoke<Comparison | null>("swap_sides")
    .then((c) => c && show(c))
    .catch(fail);
});

load().then(show).catch(fail);

interface FileEntry {
  path: string;
  status: string;
  generated: boolean;
  reviewed: boolean;
}

interface Listing {
  files: FileEntry[];
  total: number;
  hidden: number;
  generated: number;
}

const explorer = document.getElementById("explorer");
const search = document.getElementById("search") as HTMLInputElement | null;
const generated = document.getElementById("generated") as HTMLInputElement | null;
const list = document.getElementById("files");
const counts = document.getElementById("counts");

function renderListing(l: Listing): void {
  if (!explorer || !list) return;
  explorer.hidden = false;
  if (counts) {
    counts.textContent = `${String(l.files.length)} shown, ${String(l.hidden)} hidden, ${String(l.total)} changed, ${String(l.generated)} generated`;
  }
  list.replaceChildren(
    ...l.files.map((f) => {
      const li = document.createElement("li");
      const button = document.createElement("button");
      button.type = "button";
      button.textContent = `${f.status[0]?.toUpperCase() ?? "?"}  ${f.path}`;
      // The mark has a check and a word for assistive technology, not only a style.
      if (f.reviewed) {
        button.classList.add("reviewed");
        button.setAttribute("aria-label", `${f.path}, reviewed`);
      }
      button.addEventListener("click", () => {
        selectFile(f.path);
        invoke<Comparison | null>("open_repository_file", { path: f.path })
          .then((c) => c && show(c))
          .catch(fail);
      });
      li.append(button);
      return li;
    }),
  );
}

function refreshListing(): void {
  invoke<Listing | null>("repository_files", {
    input: { search: search?.value ?? "", show_generated: generated?.checked ?? false },
  })
    .then((l) => l && renderListing(l))
    .catch(fail);
}

search?.addEventListener("input", refreshListing);
generated?.addEventListener("change", refreshListing);
refreshListing();

// The theme follows the system until the user cycles the control. The choice
// is kept in the browser storage when it works, and the app runs without it.
const THEME_KEY = "diffy.theme";
const themeButton = document.getElementById("theme");
const systemDark = window.matchMedia("(prefers-color-scheme: dark)");

function storedPreference(): ThemePreference {
  try {
    return parsePreference(window.localStorage.getItem(THEME_KEY));
  } catch {
    return "system";
  }
}

function applyTheme(pref: ThemePreference): void {
  const root = document.documentElement;
  if (pref === "system") root.removeAttribute("data-theme");
  else root.setAttribute("data-theme", pref);
  setEditorTheme(resolveTheme(pref, systemDark.matches) === "dark");
  if (themeButton) themeButton.textContent = `Theme: ${pref}`;
}

let preference = storedPreference();
applyTheme(preference);
systemDark.addEventListener("change", () => applyTheme(preference));
themeButton?.addEventListener("click", () => {
  preference = nextPreference(preference);
  try {
    window.localStorage.setItem(THEME_KEY, preference);
  } catch {
    // The choice then lasts for this run only.
  }
  applyTheme(preference);
});

// A second `diffy` forwards its input to this window. Rust reads it into a
// snapshot first, so the event only says "look again".
listen("diffy-changed", () => {
  load().then(show).catch(fail);
  refreshListing();
  refreshFindings();
}).catch(fail);

// The review: reviewed files and findings. Rust validates each input and saves
// it, so the frontend only sends the text of the form.
interface Finding {
  id: number;
  path: string;
  start_line: number;
  end_line: number;
  severity: string;
  category: string;
  notes: string;
  resolved: boolean;
}

const review = document.getElementById("review");
const selected = document.getElementById("selected");
const markButton = document.getElementById("mark-reviewed") as HTMLButtonElement | null;
const addButton = document.getElementById("f-add") as HTMLButtonElement | null;
const findingsList = document.getElementById("findings");
const reviewStatus = document.getElementById("review-status");
let selectedPath: string | undefined;

function say(message: string): void {
  if (reviewStatus) reviewStatus.textContent = message;
}

function selectFile(path: string): void {
  selectedPath = path;
  if (selected) selected.textContent = `Selected: ${path}`;
  if (markButton) markButton.disabled = false;
  if (addButton) addButton.disabled = false;
}

function renderFindings(findings: Finding[]): void {
  if (!findingsList) return;
  findingsList.replaceChildren(
    ...findings.map((f) => {
      const li = document.createElement("li");
      li.textContent = `${f.resolved ? "Resolved: " : ""}${f.severity} ${f.category} ${f.path}:${String(f.start_line)}-${String(f.end_line)} ${f.notes} `;
      const toggle = document.createElement("button");
      toggle.type = "button";
      toggle.textContent = f.resolved ? "Reopen" : "Resolve";
      toggle.addEventListener("click", () => {
        invoke("resolve_finding", { id: f.id, resolved: !f.resolved })
          .then(refreshFindings)
          .catch((e: unknown) => say(String(e)));
      });
      li.append(toggle);
      return li;
    }),
  );
}

function refreshFindings(): void {
  invoke<Finding[]>("review_findings")
    .then((f) => {
      if (review) review.hidden = false;
      renderFindings(f);
    })
    .catch((e: unknown) => {
      // No repository is open: the review panel stays hidden.
      void e;
    });
}

markButton?.addEventListener("click", () => {
  if (!selectedPath) return;
  invoke("mark_file_reviewed", { path: selectedPath })
    .then(() => {
      say("Marked as reviewed.");
      refreshListing();
    })
    .catch((e: unknown) => say(String(e)));
});

document.getElementById("finding-form")?.addEventListener("submit", (event) => {
  event.preventDefault();
  if (!selectedPath) return;
  const value = (id: string): string =>
    (document.getElementById(id) as HTMLInputElement | HTMLTextAreaElement).value;
  invoke("add_review_finding", {
    input: {
      path: selectedPath,
      start_line: Number(value("f-start")),
      end_line: Number(value("f-end")),
      severity: value("f-severity"),
      category: value("f-category"),
      notes: value("f-notes"),
    },
  })
    .then(() => {
      (document.getElementById("f-notes") as HTMLTextAreaElement).value = "";
      say("Finding added.");
      refreshFindings();
    })
    .catch((e: unknown) => say(String(e)));
});

document.getElementById("copy-findings")?.addEventListener("click", () => {
  invoke<string>("findings_markdown")
    .then(async (text) => {
      try {
        await navigator.clipboard.writeText(text);
        say(text === "" ? "No unresolved findings." : "Copied to the clipboard.");
      } catch {
        // The clipboard may be blocked: show the text so it can be copied by hand.
        say(text === "" ? "No unresolved findings." : text);
      }
    })
    .catch((e: unknown) => say(String(e)));
});

refreshFindings();

document.getElementById("view-mode")?.addEventListener("click", (event) => {
  unifiedMode = !unifiedMode;
  (event.currentTarget as HTMLElement).textContent =
    `View: ${unifiedMode ? "unified" : "side by side"}`;
  // The context control applies to the unified view only.
  const collapseButton = document.getElementById("collapse-mode");
  if (collapseButton) collapseButton.hidden = !unifiedMode;
  if (last) show(last);
});

document.getElementById("collapse-mode")?.addEventListener("click", (event) => {
  collapseMode = !collapseMode;
  (event.currentTarget as HTMLElement).textContent =
    `Context: ${collapseMode ? "collapsed" : "all lines"}`;
  if (last) show(last);
});

// Compare two pasted texts (C-02). Rust keeps them as the current comparison, so
// the swap works as for files. The texts are plain strings: no file is named.
const pasteDialog = document.getElementById("paste") as HTMLDialogElement | null;
const pasteBase = document.getElementById("paste-base") as HTMLTextAreaElement | null;
const pasteHead = document.getElementById("paste-head") as HTMLTextAreaElement | null;
const pasteStatus = document.getElementById("paste-status");

document.getElementById("paste-open")?.addEventListener("click", () => {
  if (pasteStatus) pasteStatus.textContent = "";
  pasteDialog?.showModal();
  pasteBase?.focus();
});
document.getElementById("paste-cancel")?.addEventListener("click", () => pasteDialog?.close());
document.getElementById("paste-form")?.addEventListener("submit", (event) => {
  event.preventDefault();
  invoke<Comparison>("open_text", {
    input: { base: pasteBase?.value ?? "", head: pasteHead?.value ?? "" },
  })
    .then((c) => {
      show(c);
      pasteDialog?.close();
    })
    .catch((e: unknown) => {
      if (pasteStatus) pasteStatus.textContent = String(e);
    });
});
