#!/usr/bin/env bash
# Runs the whole Q44 comparison and writes results/. Usage: ./run.sh <work-dir>
# Needs: git (only for the fixtures and the oracle), cargo, bwrap, strace, mingw64-gcc and jq.
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
work=$1
mkdir -p "$work"
work=$(cd "$work" && pwd)
res=$here/results
mkdir -p "$res" "$work/tmp" "$work/traces"
export TMPDIR=$work/tmp Q44_SCRATCH=$work/tmp

# 1. Fixtures and oracle, with git.
rm -rf "$work/fx"
"$here/fixtures.sh" "$work/fx" >/dev/null
CARGO_TARGET_DIR=$work/target cargo build -q --release --locked --manifest-path "$here/Cargo.toml"
"$work/target/release/harness" oracle "$work/fx" "$res/expected.json"

# 2. Build in a container with no git executable, traced.
rm -rf "$work/target-nogit"
"$here/no-git.sh" env CARGO_TARGET_DIR="$work/target-nogit" \
  strace -f -qq -e trace=execve,execveat -o "$work/traces/build.trace" \
  cargo build -q --release --offline --locked --manifest-path "$here/Cargo.toml"
bin=$work/target-nogit/release

# 3. Checks in the same container, traced: a fresh copy, and a copy with no write permission.
check() { # check <name> <backend> [--no-write-permission], with the environment of the caller
  "$here/no-git.sh" strace -f -qq -e trace=execve,execveat -o "$work/traces/$1.trace" \
    "$bin/harness" check "$work/fx" "$res/expected.json" "$2" "$res/$1.json" ${3:-} || true
}
check gix "$bin/backend-gix"
check gix-no-write "$bin/backend-gix" --no-write-permission
check git2 "$bin/backend-git2"
check git2-no-write "$bin/backend-git2" --no-write-permission
Q44_GIX_FILTERS=trust check gix-filters-trust "$bin/backend-gix"
Q44_GIX_FILTERS=section-filter check gix-filters-section-filter "$bin/backend-gix"
Q44_GIX_FILTERS=section-filter check gix-filters-section-filter-no-write "$bin/backend-gix" --no-write-permission
Q44_GIX_DEFAULT_OPEN=1 check gix-default-open "$bin/backend-gix"

# 4. Negative control for the read-only check, with git.
"$bin/harness" check "$work/fx" "$res/expected.json" "$here/controls/git-status-writes.sh" \
  "$res/control-git-status.json" || true

# 5. Trace summary: each program that a traced run started, and each attempt to start git.
{
  echo '{'
  first=1
  for t in "$work"/traces/*.trace; do
    name=$(basename "$t" .trace)
    progs=$(grep -oE 'execve(at)?\("[^"]*"' "$t" | sed -E 's/.*\///; s/"$//' | sort | uniq -c |
      awk '{printf "%s\"%s\": %s", (NR > 1 ? ", " : ""), $2, $1}')
    attempts=$(grep -cE 'execve(at)?\("([^"]*/)?git(-[^"/]*)?"' "$t" || true)
    [ $first = 1 ] || echo ','
    first=0
    printf '  "%s": { "git_start_attempts": %s, "programs": { %s } }' "$name" "$attempts" "$progs"
  done
  echo
  echo '}'
} >"$res/traces.json"

# 6. Build facts: native crates, Linux and Windows x64 binaries.
CARGO_TARGET_DIR=$work/target-win cargo build -q --release --offline --locked \
  --manifest-path "$here/Cargo.toml" -p backend-gix -p backend-git2 --target x86_64-pc-windows-gnu
win=$work/target-win/x86_64-pc-windows-gnu/release
{
  echo '{'
  echo "  \"rustc\": \"$(rustc --version)\","
  for b in gix git2; do
    natives=$(cargo tree -q --manifest-path "$here/Cargo.toml" -p "backend-$b" -e normal,build \
      --prefix none --locked --offline | sort -u | grep -E -- '-sys |^cc |^zlib-rs ' | sed 's/ (\*)//' |
      sort -u | awk '{printf "%s\"%s %s\"", (NR > 1 ? ", " : ""), $1, $2}')
    linux_libs=$(ldd "$bin/backend-$b" | awk '{print $1}' | grep -v vdso | sort |
      awk '{printf "%s\"%s\"", (NR > 1 ? ", " : ""), $1}')
    win_dlls=$(x86_64-w64-mingw32-objdump -p "$win/backend-$b.exe" | awk '/DLL Name/ {print tolower($3)}' | sort -u |
      awk '{printf "%s\"%s\"", (NR > 1 ? ", " : ""), $1}')
    cat <<EOF
  "$b": {
    "native_crates": [ $natives ],
    "linux_x86_64_bytes": $(stat -c %s "$bin/backend-$b"),
    "linux_x86_64_shared_libraries": [ $linux_libs ],
    "windows_x86_64_gnu_bytes": $(stat -c %s "$win/backend-$b.exe"),
    "windows_x86_64_gnu_dlls": [ $win_dlls ]
  }$( [ $b = gix ] && echo ',')
EOF
  done
  echo '}'
} >"$res/build.json"

jq -c '.summary' "$res"/*.json 2>/dev/null | grep -v '^null$' || true
jq . "$res/traces.json" >/dev/null && jq . "$res/build.json" >/dev/null
echo "results in $res"
