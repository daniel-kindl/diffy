// The theme follows the system by default, with a manual override (spec
// section 15). The CSS tokens do the work: `data-theme` on the root element
// forces light or dark, and no attribute follows `prefers-color-scheme`.

export type ThemePreference = "system" | "light" | "dark";

export function resolveTheme(pref: ThemePreference, systemDark: boolean): "light" | "dark" {
  if (pref === "system") return systemDark ? "dark" : "light";
  return pref;
}

export function parsePreference(value: string | null): ThemePreference {
  return value === "light" || value === "dark" ? value : "system";
}

/** The next preference when the user cycles the control. */
export function nextPreference(pref: ThemePreference): ThemePreference {
  return pref === "system" ? "light" : pref === "light" ? "dark" : "system";
}
