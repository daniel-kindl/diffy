import "./style.css";
import { invoke } from "@tauri-apps/api/core";
import { showComparison, type ComparisonView } from "./editor";
import { LEGACY_ENCODINGS } from "./encodings";
import type { Hunk } from "./hunks";

interface Side {
  label: string;
  encoding: string;
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
let view: ComparisonView | undefined;

function show(c: Comparison): void {
  if (!left || !right) return;
  view?.dispose();
  if (status) {
    status.textContent = `${describe(c.sides[0], "Base")} ↔ ${describe(c.sides[1], "Head")}`;
  }
  view = showComparison(left, right, c.base, c.head, c.hunks, c.sides[1]?.label ?? "");
}

function fail(e: unknown): void {
  if (status) status.textContent = `Diffy could not load the comparison: ${String(e)}`;
}

for (const select of document.querySelectorAll<HTMLSelectElement>("select[data-side]")) {
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
  // The encodings go with their inputs, so the selectors swap too.
  const l = document.querySelector<HTMLSelectElement>("#enc-left");
  const r = document.querySelector<HTMLSelectElement>("#enc-right");
  if (l && r) [l.value, r.value] = [r.value, l.value];
  invoke<Comparison | null>("swap_sides")
    .then((c) => c && show(c))
    .catch(fail);
});

load().then(show).catch(fail);
