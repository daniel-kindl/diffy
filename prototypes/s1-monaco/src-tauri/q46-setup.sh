#!/usr/bin/env bash
# Q46 throwaway. Downloads gix-imara-diff 0.3.4, checks its SHA-256 and applies the Q60 patch
# (from the tag prototype/diffy-v0.1-q60) into vendor/gix-imara-diff-q60.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
out=$here/vendor/gix-imara-diff-q60
[ -d "$out" ] && exit 0
mkdir -p "$here/vendor"
crate=$here/vendor/gix-imara-diff-0.3.4.crate
curl -fsSL -o "$crate" https://static.crates.io/crates/gix-imara-diff/gix-imara-diff-0.3.4.crate
echo "23347fa4a29a440eebf045d0bc21da465c0dc70434c2a4bcdfcc658f16c9e27e  $crate" | sha256sum -c --quiet
tar -xzf "$crate" -C "$here/vendor"
mv "$here/vendor/gix-imara-diff-0.3.4" "$out"
(cd "$out" && GIT_CEILING_DIRECTORIES="$here/vendor" git apply -p1 "$here/gix-imara-diff-0.3.4-q60.patch")
rm "$crate"
