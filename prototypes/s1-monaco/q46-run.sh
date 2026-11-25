#!/usr/bin/env bash
# Q46 throwaway: the S8 scale run. Usage: ./q46-run.sh <work-dir>
# Needs the S1 setup of README.md (npm ci, npm run fixtures, the Tauri Linux libraries) and git.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
work=$1
mkdir -p "$work/tmp"
work=$(cd "$work" && pwd)
export TMPDIR=$work/tmp
export DIFFY_S1_FIXTURES=${DIFFY_S1_FIXTURES:-$here/fixtures/out}
export DIFFY_Q46_REPOS=$work/repos
(cd "$DIFFY_S1_FIXTURES" && sha256sum -c --quiet "$here/fixtures/checksums.txt")
node "$here/fixtures/gen-repos.mjs" "$DIFFY_Q46_REPOS"
(cd "$here/src-tauri" && ./q46-setup.sh)
(cd "$here" && npx tauri build --no-bundle)
bin=${CARGO_TARGET_DIR:-$here/src-tauri/target}/release/diffy-s1-monaco
label=${Q46_LABEL:-q46-fedora}
for f in files-1000 files-1000-large files-2000-large files-5000 lines-10k stress-50mb; do
  DIFFY_Q46=list DIFFY_Q46_FIXTURES=$f DIFFY_Q46_RUNS=21 DIFFY_Q46_LABEL="$label-list-$f" "$bin"
done
# The same two large fixtures with no rename detection.
for f in files-1000-large files-2000-large; do
  DIFFY_Q46_RENAMES=0 DIFFY_Q46=list DIFFY_Q46_FIXTURES=$f DIFFY_Q46_RUNS=21 DIFFY_Q46_LABEL="$label-list-$f-no-renames" "$bin"
done
DIFFY_Q46=cancel-stats DIFFY_Q46_FIXTURES=files-5000 DIFFY_Q46_RUNS=21 DIFFY_Q46_LABEL="$label-cancel-stats" "$bin"
DIFFY_Q46=cancel-open DIFFY_Q46_FIXTURES=stress-50mb DIFFY_Q46_RUNS=20 DIFFY_Q46_LABEL="$label-cancel-open" "$bin"
cp "$DIFFY_Q46_REPOS/repos.json" "$here/bench-results/$label-repos.json"
