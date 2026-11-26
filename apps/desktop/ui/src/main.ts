import "./style.css";
import { showComparison } from "./editor";

// The shell (F-03, Tauri) passes the real comparison from Rust. Until then the
// page shows a small fixed example, so the renderer can be checked on its own.
const left = document.getElementById("left");
const right = document.getElementById("right");
if (left && right) {
  showComparison(left, right, "one\ntwo\nthree\n", "one\nTWO\nthree\nfour\n", [
    { base: { start: 1, end: 2 }, head: { start: 1, end: 2 } },
    { base: { start: 3, end: 3 }, head: { start: 3, end: 4 } },
  ]);
}
