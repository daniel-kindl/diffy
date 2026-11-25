#!/usr/bin/env bash
# Q43 throwaway: write file pairs from the history of real repositories.
# Usage: corpus.sh <out dir> <commits per repo> <bare repo>...
# Each modified file of each non-merge commit gives one pair: the blob before
# and the blob after. The pairs go to <out dir>/<repo>/<n>.{left,right} and
# the list to <out dir>/pairs-real.tsv.
set -euo pipefail
out=$1; n=$2; shift 2
mkdir -p "$out"
list="$out/pairs-real.tsv"
: > "$list"
for repo in "$@"; do
  name=$(basename "$repo" .git)
  mkdir -p "$out/$name"
  i=0
  git -C "$repo" rev-list --no-merges -n "$n" HEAD | while read -r c; do
    git -C "$repo" diff-tree -r --no-renames --no-commit-id --raw "$c^" "$c" 2>/dev/null |
      awk '$5 == "M" { print $3, $4, $6 }' |
      while read -r a b path; do
        i=$((i + 1))
        git -C "$repo" cat-file blob "$a" > "$out/$name/$c-$i.left"
        git -C "$repo" cat-file blob "$b" > "$out/$name/$c-$i.right"
        printf '%s\t%s\t%s\n' "$name:${c:0:12}:$path" "$out/$name/$c-$i.left" "$out/$name/$c-$i.right" >> "$list"
      done
  done
done
wc -l "$list"
