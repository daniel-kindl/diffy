#!/usr/bin/env bash
# Runs the whole Q45 comparison and writes results/. Usage: ./run.sh <work-dir>
# Needs: git, curl, cargo and jq. The real corpus needs a network connection for the clones.
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
work=$1
mkdir -p "$work/tmp" "$work/repos"
work=$(cd "$work" && pwd)
export TMPDIR=$work/tmp CARGO_TARGET_DIR=$work/target
res=$here/results
mkdir -p "$res"

"$here/setup.sh"
cargo test -q --release --locked --manifest-path "$here/Cargo.toml"
cargo build -q --release --locked --manifest-path "$here/Cargo.toml"
bin=$CARGO_TARGET_DIR/release/q45-anchors

"$bin" fixtures "$res/fixtures.json"
"$bin" property small 1000000 "$res/property-small.json"
"$bin" property medium 100000 "$res/property-medium.json"

# The Q43 real corpus: the last 500 non-merge commits of each repository, at the Q43 commits.
for spec in "git/git 77904e546de3" "tokio-rs/tokio 854013a8cb7d" "BurntSushi/ripgrep 3e85c6c148fd"; do
  set -- $spec
  repo=$work/repos/$(basename "$1").git
  [ -d "$repo" ] || git clone -q --bare "https://github.com/$1.git" "$repo"
  git -C "$repo" update-ref --no-deref HEAD "$(git -C "$repo" rev-parse --verify "$2^{commit}")"
done
rm -rf "$work/corpus"
"$here/corpus.sh" "$work/corpus" 500 "$work/repos/git.git" "$work/repos/tokio.git" "$work/repos/ripgrep.git"
"$bin" real "$work/corpus/pairs-real.tsv" "$res/real.json"
echo "results in $res"
