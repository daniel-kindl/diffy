import "./style.css";
import { invoke } from "@tauri-apps/api/core";
import { showComparison } from "./editor";
import type { Hunk } from "./hunks";

interface Comparison {
  base: string;
  head: string;
  hunks: Hunk[];
}

// The frontend sends text only. It never names a file: Rust owns the inputs.
const input = { base: "one\ntwo\nthree\n", head: "one\nTWO\nthree\nfour\n" };

async function load(): Promise<Comparison> {
  return invoke<Comparison>("compare_text", { input });
}

const left = document.getElementById("left");
const right = document.getElementById("right");
if (left && right) {
  load()
    .then((c) => showComparison(left, right, c.base, c.head, c.hunks))
    .catch((e: unknown) => {
      right.textContent = `Diffy could not load the comparison: ${String(e)}`;
    });
}
