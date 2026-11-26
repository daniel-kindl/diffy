import "./style.css";
import { invoke } from "@tauri-apps/api/core";
import { showComparison } from "./editor";
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

// The frontend never names a file: Rust reads the inputs from the command
// line and hands them over. With no input, a small example shows the renderer.
const sample = { base: "one\ntwo\nthree\n", head: "one\nTWO\nthree\nfour\n" };

async function load(): Promise<Comparison> {
  const initial = await invoke<Comparison | null>("initial_comparison");
  return initial ?? invoke<Comparison>("compare_text", { input: sample });
}

/** The line above an editor: the source label, the encoding and any error. */
export function describe(side: Side | undefined, fallback: string): string {
  if (!side) return fallback;
  const error =
    side.first_error === null ? "" : ` – invalid bytes from offset ${String(side.first_error)}`;
  return `${side.label} (${side.encoding})${error}`;
}

const left = document.getElementById("left");
const right = document.getElementById("right");
const status = document.getElementById("status");
if (left && right) {
  load()
    .then((c) => {
      if (status)
        status.textContent = `${describe(c.sides[0], "Base")} ↔ ${describe(c.sides[1], "Head")}`;
      showComparison(left, right, c.base, c.head, c.hunks, c.sides[1]?.label ?? "");
    })
    .catch((e: unknown) => {
      if (status) status.textContent = `Diffy could not load the comparison: ${String(e)}`;
    });
}
