// Keyboard controls (C-06). The mapping is a pure function, so it is tested
// without a browser. Tab is never bound: Monaco's tab focus mode (Q58) and the
// browser keep Tab for moving focus, so there is no keyboard trap.

export type Action = "nextChange" | "previousChange" | "find" | "goToLine" | "switchSide";

export interface KeyInfo {
  key: string;
  ctrlKey: boolean;
  metaKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
}

export function actionFor(e: KeyInfo): Action | undefined {
  const primary = e.ctrlKey || e.metaKey;
  if (e.key === "F3" && !primary && !e.altKey) return e.shiftKey ? "previousChange" : "nextChange";
  if (e.altKey && !primary && !e.shiftKey) {
    if (e.key === "ArrowDown") return "nextChange";
    if (e.key === "ArrowUp") return "previousChange";
  }
  if (primary && !e.altKey && !e.shiftKey) {
    if (e.key.toLowerCase() === "f") return "find";
    if (e.key.toLowerCase() === "g") return "goToLine";
  }
  if (e.key === "F6" && !primary && !e.altKey && !e.shiftKey) return "switchSide";
  return undefined;
}
