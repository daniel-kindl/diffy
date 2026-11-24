#!/usr/bin/env bash
# Throwaway Q42 run: each option under XWayland (GDK_BACKEND=x11) and Wayland.
# Usage: ./q42-run.sh <host-tag> [session...] [option...]   e.g. ./q42-run.sh fedora x11 A
# Do not use the keyboard or the mouse during the run.
set -u
cd "$(dirname "$0")"
host="${1:?host tag, for example fedora or ubuntu}"; shift
sessions=(); options=()
for a in "$@"; do case "$a" in x11|wayland) sessions+=("$a");; A|B) options+=("$a");; esac; done
[ ${#sessions[@]} -eq 0 ] && sessions=(x11 wayland)
[ ${#options[@]} -eq 0 ] && options=(A B)
bin="${CARGO_TARGET_DIR:-src-tauri/target}/release/diffy-s1-monaco"

# The input method: ibus with the m17n latn-post engine, in this container.
pgrep -x ibus-daemon >/dev/null || ibus-daemon -drx -p disable
sleep 3
ibus engine xkb:us::eng || true
echo "ibus engine: $(ibus engine 2>&1)"

# ydotool needs its daemon for the Wayland runs.
if printf '%s\n' "${sessions[@]}" | grep -qx wayland; then
  pgrep -x ydotoold >/dev/null || { ydotoold >/dev/null 2>&1 & sleep 1; }
fi

rc=0
for s in "${sessions[@]}"; do
  for o in "${options[@]}"; do
    label="q42-${host}-${s}-${o}"
    echo "== $label"
    if [ "$s" = x11 ]; then backend=x11; else backend=wayland; fi
    env -u WEBKIT_DISABLE_DMABUF_RENDERER -u WEBKIT_DISABLE_COMPOSITING_MODE \
      GDK_BACKEND="$backend" GTK_IM_MODULE=ibus DIFFY_S2_CHECK="$o" DIFFY_S2_LABEL="$label" \
      timeout 600 "$bin" || rc=1
    node -e 'const r=require("./bench-results/'"$label"'.json"); console.log(`passed ${r.passed} of ${r.checks.length}`); for (const f of r.failed) console.log("FAIL", f)' || rc=1
  done
done
exit $rc
