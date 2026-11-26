#!/usr/bin/env bash
# Q85 run: python3 -I is used for the reader. Throwaway. Never merge.
# usage: q85-run.sh <label> <x11|wayland> <off|on> <left|right>
set -u
cd "$(dirname "$0")"
label=$1
env GDK_BACKEND=$2 DIFFY_Q85_LABEL=$label DIFFY_Q85_SR=$3 DIFFY_Q85_TARGET=$4 ${Q85_FIX:+DIFFY_Q85_FIX=1} \
  DIFFY_S1_FIXTURES=${DIFFY_S1_FIXTURES:-$PWD/fixtures/out} \
  src-tauri/target/release/diffy-s1-monaco >/dev/null 2>&1 &
app=$!
python3 -I q85-atspi.py "$label" "$app" "bench-results/$label-atspi.json"
kill $app 2>/dev/null; wait $app 2>/dev/null
