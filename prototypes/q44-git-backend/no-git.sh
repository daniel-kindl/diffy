#!/usr/bin/env bash
# Runs a command in a bubblewrap container with no `git` executable: /usr/bin holds each
# program of the host except git*, and /usr/libexec/git-core is empty. The rest of the file
# system is the host's. Usage: ./no-git.sh <command> [args...]
set -euo pipefail
binds=()
for f in /usr/bin/*; do
  case "${f##*/}" in git*) continue ;; esac
  binds+=(--ro-bind "$f" "$f")
done
exec bwrap --dev-bind / / --tmpfs /usr/bin "${binds[@]}" --tmpfs /usr/libexec/git-core \
  --setenv Q44_NO_GIT 1 --unshare-pid --die-with-parent -- "$@"
