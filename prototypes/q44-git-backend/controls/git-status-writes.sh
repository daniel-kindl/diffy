#!/usr/bin/env bash
# Negative control for the read-only check: plain `git status` may refresh and write the
# index of a fresh copy. `harness check` must report the changed paths for this "backend".
# It prints no valid answer, so each operation also fails the comparison.
git -C "$1" status >/dev/null 2>&1 || true
echo '{}'
