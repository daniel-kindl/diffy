#!/usr/bin/env bash
# Q43 throwaway: write hand-made edge case pairs.
# Usage: edge.sh <out dir>. The list goes to <out dir>/pairs-edge.tsv.
set -euo pipefail
out=$1/edge
mkdir -p "$out"
list="$1/pairs-edge.tsv"
: > "$list"
pair() { # name, left content, right content (printf formats)
  printf "$2" > "$out/$1.left"
  printf "$3" > "$out/$1.right"
  printf 'edge:%s\t%s\t%s\n' "$1" "$out/$1.left" "$out/$1.right" >> "$list"
}
pair empty-to-text '' 'a\nb\n'
pair text-to-empty 'a\nb\n' ''
pair no-eol-left 'a\nb' 'a\nb\n'
pair no-eol-right 'a\nb\n' 'a\nb'
pair no-eol-both-change 'a\nb' 'a\nc'
pair crlf-to-lf 'a\r\nb\r\nc\r\n' 'a\nb\nc\n'
pair mixed-eol 'a\nb\r\nc\n' 'a\r\nb\nc\n'
pair trailing-space 'a\nb\nc\n' 'a\nb \nc\n'
pair one-line 'a\n' 'b\n'
pair all-changed 'a\nb\nc\nd\n' 'w\nx\ny\nz\n'
pair moved-block 'a\nb\nc\nd\ne\nf\ng\nh\n' 'e\nf\ng\nh\na\nb\nc\nd\n'
pair swap-two 'x\ny\n' 'y\nx\n'
pair blank-slider 'a\n\nb\n\nc\n' 'a\n\nb\n\nnew\n\nc\n'
pair brace-slider 'fn a() {\n    x();\n}\n\nfn c() {\n    z();\n}\n' 'fn a() {\n    x();\n}\n\nfn b() {\n    y();\n}\n\nfn c() {\n    z();\n}\n'
pair indent-slider 'if a {\n    one();\n}\nif b {\n    two();\n}\n' 'if a {\n    one();\n}\nif n {\n    new();\n}\nif b {\n    two();\n}\n'
pair utf8 'grüße\nwelt\n' 'grüsse\nwelt\n'
pair invalid-utf8 'a\n\xff\xfe\nb\n' 'a\n\xfe\xff\nb\n'
pair whitespace-lines '\n\n\n\n' '\n\n\n'
# More than 64 copies of one line: Git falls back to Myers in that region.
rep=$(for i in $(seq 1 200); do printf 'same\\n'; done)
pair many-repeats "${rep}x\\n${rep}" "${rep}y\\n${rep}z\\n"
rep2=$(for i in $(seq 1 100); do printf 'r\\nq\\n'; done)
pair repeats-interleaved "a\\n${rep2}b\\n" "a\\n${rep2}r\\nb\\n"
wc -l "$list"
