#!/usr/bin/env bash
# Q65: exports the old and new blob of each modified or renamed file of the Q46 fixture
# repositories, and writes pairs-q65.tsv (fixture/path, old file, new file) and deleted-q65.tsv.
# Usage: ./q65-pairs.sh <q46 repos dir> <out dir>
set -euo pipefail
repos=$(cd "$1" && pwd)
mkdir -p "$2"
out=$(cd "$2" && pwd)
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null LC_ALL=C
: > "$out/pairs-q65.tsv"
: > "$out/deleted-q65.tsv"
for repo in "$repos"/*.git; do
  name=$(basename "$repo" .git)
  mkdir -p "$out/$name"
  n=0
  git -C "$repo" diff --name-status -z --histogram -M base head | tr '\0' '\n' | {
    while read -r st; do
      case $st in
        M) read -r p; old=$p; new=$p ;;
        R*) read -r old; read -r new ;;
        D) read -r p; printf '%s/%s\t%s\n' "$name" "$p" "$(git -C "$repo" show "base:$p" | wc -l)" >> "$out/deleted-q65.tsv"; continue ;;
        A) read -r p; continue ;;
        *) echo "unexpected status $st" >&2; exit 1 ;;
      esac
      n=$((n + 1))
      git -C "$repo" show "base:$old" > "$out/$name/$n.old"
      git -C "$repo" show "head:$new" > "$out/$name/$n.new"
      printf '%s/%s\t%s\t%s\n' "$name" "$new" "$out/$name/$n.old" "$out/$name/$n.new" >> "$out/pairs-q65.tsv"
    done
  }
done
wc -l "$out/pairs-q65.tsv" "$out/deleted-q65.tsv"
