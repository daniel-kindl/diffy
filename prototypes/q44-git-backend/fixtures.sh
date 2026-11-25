#!/usr/bin/env bash
# Builds the Q44 Git fixtures, one repository for each fixture of specification section 19.
# Usage: ./fixtures.sh <out-dir>. The out-dir must not exist.
# Writes <out-dir>/repos/<name> (the fixtures), <out-dir>/origins (submodule sources),
# <out-dir>/scripts (commands that the untrusted-config fixture names) and <out-dir>/markers
# (where those commands leave a file when something runs them).
set -euo pipefail

out=$1
mkdir "$out"
out=$(cd "$out" && pwd)
mkdir "$out/repos" "$out/origins" "$out/home" "$out/scripts" "$out/markers"

# No system or global configuration, fixed identity and dates.
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null HOME="$out/home" XDG_CONFIG_HOME="$out/home"
export GIT_AUTHOR_NAME=Q44 GIT_AUTHOR_EMAIL=q44@example.invalid
export GIT_COMMITTER_NAME=Q44 GIT_COMMITTER_EMAIL=q44@example.invalid
tick=1764072000
commit() {
  tick=$((tick + 60))
  GIT_AUTHOR_DATE="@$tick +0000" GIT_COMMITTER_DATE="@$tick +0000" git commit -q --no-verify "$@"
}
new_repo() {
  git init -q -b main "$1"
  cd "$1"
}
lines() { # lines <prefix> <count>: numbered lines of text
  for i in $(seq 1 "$2"); do printf '%s line %02d of the fixture text\n' "$1" "$i"; done
}

# index-worktree: each staged and unstaged change kind, untracked and ignored files.
(
  new_repo "$out/repos/index-worktree"
  for f in a b c d g h; do lines "$f" 5 >"$f.txt"; done
  lines e 3 >e.sh
  ln -s a.txt link
  mkdir dir && lines f 4 >dir/f.txt
  printf '*.log\n' >.gitignore
  git add -A && commit -m base
  lines A 5 >a.txt && git add a.txt                    # M. staged modification
  lines n 2 >new.txt && git add new.txt                # A. staged addition
  git rm -q b.txt                                      # D. staged deletion
  lines C 5 >c.txt                                     # .M unstaged modification
  rm d.txt                                             # .D unstaged deletion
  chmod +x g.txt                                       # .M executable bit
  git update-index --chmod=+x e.sh                     # M. staged executable bit
  lines H 5 >h.txt && git add h.txt && lines HH 5 >h.txt # MM
  rm link && lines l 1 >link                           # .T symlink to file
  lines i 2 >ita.txt && git add -N ita.txt             # .A intent to add
  lines u 1 >u.txt                                     # ?? untracked file
  mkdir -p udir/x && lines y 1 >udir/x/y.txt           # ?? untracked file in a new directory
  lines log 1 >build.log                               # ignored
  lines f 5 >dir/f.txt                                 # .M in a directory
)

# unusual-names: spaces, control characters, quotes, Unicode, invalid UTF-8, long and deep paths.
(
  new_repo "$out/repos/unusual-names"
  names=(
    'space name.txt' $'tab\tname' $'new\nline' 'back\slash' 'quote"d' "single'quote" '-leading-dash'
    'ünïcödé.txt' 'emoji-🦀.txt' 'star*.txt' $'latin1-\xe9.txt' '#hash' '.hidden' ' leading-space'
    'trailing-space ' 'deep/a/b/c/d/e/f.txt' "$(printf 'long-%0200d' 0)" 'colon:name' 'dir with space/inner file'
  )
  for n in "${names[@]}"; do mkdir -p "$(dirname -- "$n")"; lines "$n" 3 >"./$n"; done
  git add -A && commit -m base
  for n in "${names[@]}"; do lines "changed $n" 3 >"./$n"; done
  git add -A && commit -m 'change each file'
  lines x 3 >'./space name.txt'                         # .M
  lines x 3 >$'./latin1-\xe9.txt'                       # .M invalid UTF-8
  git mv -- 'ünïcödé.txt' 'ünïcödé-renamed.txt'          # R. staged rename to Unicode
  lines u 1 >$'./untracked-\xff'                        # ?? invalid UTF-8
  lines u 1 >$'./untracked\nnewline'                    # ?? newline
)

# renames: commit-to-commit renames above and below the 50 % threshold, a moved directory,
# a staged rename, and a worktree rename. The objects are repacked into a pack with deltas.
(
  new_repo "$out/repos/renames"
  for f in r1 r2 r3 r4 keep; do lines "$f" 20 >"$f.txt"; done
  mkdir dir1 && lines x 20 >dir1/x.txt && lines y 20 >dir1/y.txt
  git add -A && commit -m base
  git mv r1.txt renamed1.txt                                        # R100
  git mv r2.txt renamed2.txt && sed -i '1s/.*/edited/' renamed2.txt # R95
  mkdir moved && git mv r3.txt moved/r3.txt && sed -i '1,6s/.*/edited/' moved/r3.txt # R70
  git mv r4.txt renamed4.txt && sed -i '1,14s/.*/edited/' renamed4.txt # 30 % similar: D and A
  git mv dir1 dir2                                                  # directory rename
  git add -A && commit -m renames
  lines z 20 >z.txt && git add z.txt && commit -m 'add z'
  git mv keep.txt kept.txt                                          # R. staged rename
  mv z.txt z-moved.txt                                              # .D and ?? (no worktree rename)
  git repack -q -a -d -f --depth=50 --window=50
  git prune-packed
)

# detached-head: HEAD on a commit, tags, packed and loose refs.
(
  new_repo "$out/repos/detached-head"
  for i in 1 2 3; do lines "v$i" 3 >file.txt && git add file.txt && commit -m "commit $i"; done
  git tag light HEAD~2
  GIT_COMMITTER_DATE="@$tick +0000" git tag -a -m annotated annotated HEAD~1
  git branch side HEAD~2
  git pack-refs --all
  git branch loose-branch HEAD
  git checkout -q --detach HEAD~1
  lines detached 3 >file.txt                         # .M on a detached HEAD
)

# submodules: new commits, modified content, untracked content, an uninitialized submodule,
# and a commit that moves a submodule pointer.
(
  for s in s1 s2 s3 s4; do
    (new_repo "$out/origins/$s" && lines "$s" 3 >file.txt && git add file.txt && commit -m "$s base"
     lines "$s two" 3 >file.txt && git add file.txt && commit -m "$s second")
  done
  new_repo "$out/repos/submodules"
  lines top 3 >top.txt && git add top.txt && commit -m base
  for s in s1 s2 s3 s4; do
    git -c protocol.file.allow=always submodule add -q "../../origins/$s" "sub-$s"
    git -C "sub-$s" checkout -q HEAD~1
  done
  git add -A && commit -m 'add submodules'
  git -C sub-s1 checkout -q main && git add sub-s1 && commit -m 'move sub-s1' # commit-to-commit gitlink change
  git -C sub-s1 checkout -q HEAD~1                    # .M new commits (checked-out HEAD differs)
  lines dirty 3 >sub-s2/file.txt                      # .M modified content
  lines untracked 1 >sub-s3/untracked.txt             # untracked content only
  git submodule deinit -q sub-s4                      # uninitialized
)

# binary: binary blobs added, changed, text to binary, and an unstaged binary change.
(
  new_repo "$out/repos/binary"
  bytes() { # bytes <seed> <count>: deterministic bytes with NUL
    LC_ALL=C awk -v s="$1" -v n="$2" 'BEGIN { srand(s); for (i = 0; i < n; i++) printf "%c", int(rand() * 256) }'
  }
  bytes 1 4096 >image.png
  lines t 5 >text-to-binary.txt
  bytes 2 70000 >big.bin
  git add -A && commit -m base
  bytes 3 4096 >image.png
  bytes 4 100 >text-to-binary.txt
  bytes 5 512 >added.dat
  git add -A && commit -m 'binary changes'
  bytes 6 70000 >big.bin                             # .M unstaged binary change
  bytes 7 64 >untracked.bin                          # ?? untracked binary
)

# untrusted-config (beyond section 19, for the section 04 safety rules): the repository
# configuration names a clean filter, a smudge filter and an fsmonitor command. Each one leaves a
# marker file. The copy in each check has new file times, so Git runs the clean filter in status.
(
  for s in clean smudge fsmonitor; do
    printf '#!/bin/sh\ntouch "%s/%s-$$"\n%s\n' "$out/markers" "$s" "$( [ $s = fsmonitor ] && echo 'exit 1' || echo 'exec cat')" \
      >"$out/scripts/$s.sh"
    chmod +x "$out/scripts/$s.sh"
  done
  new_repo "$out/repos/untrusted-config"
  printf '*.txt filter=q44\n' >.gitattributes
  lines a 3 >a.txt && lines b 3 >b.txt
  git add -A && commit -m base
  git config filter.q44.clean "$out/scripts/clean.sh"
  git config filter.q44.smudge "$out/scripts/smudge.sh"
  git config core.fsmonitor "$out/scripts/fsmonitor.sh"
  lines A 3 >a.txt                                   # .M
  rm -f "$out"/markers/*
)

echo "fixtures in $out/repos:"
ls "$out/repos"
