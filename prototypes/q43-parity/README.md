# Q43 histogram parity harness

This is a throwaway harness for map question Q43. See `docs/maps/diffy-v0.1.md` on `main`. The branch `prototype/diffy-v0.1-q43` is never merged.

The harness compares the output of Rust diff crates with the ADR 0002 oracle, `git diff --histogram`.

## Oracle

Git 2.55.0. Each run has no system or global Git configuration, and runs in the temporary folder, outside each repository:

```sh
GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null HOME=/nonexistent \
  git diff --no-index --histogram -U3 --inter-hunk-context=0 --indent-heuristic \
  --no-color --no-ext-diff --no-renames --no-textconv -- <left> <right>
```

The harness also runs the same command with `--no-indent-heuristic`.

The oracle result for each pair is the list of `-U3` hunk ranges, and the change groups that the `-` and `+` lines of the hunks give. A crate passes a pair only when both lists are identical. The harness groups the crate change groups into hunks with the Git rule of git 2.55.0 `xdiff/xemit.c:46-100` and `xdiff/xemit.c:186-243`.

Do not use `-U0` as the oracle. With no context, Git trims the common tail of the inputs before the diff (git 2.55.0 `xdiff-interface.c:127`). On 8 of the 3,732 real pairs, the `-U0` change groups are at other lines than the `-U3` change groups. The result files record this check as `git_u0_groups_equal_git_u3_groups`.

## Variants

- `ImaraRaw`, `ImaraNoHeuristic`, `ImaraIndentHeuristic`: imara-diff 0.2.0, `Algorithm::Histogram`, with no postprocessing, with `postprocess_no_heuristic`, and with `postprocess_lines` (the indent heuristic).
- `GixRaw`, `GixNoHeuristic`, `GixIndentHeuristic`: gix-imara-diff 0.3.4, the gitoxide copy of imara-diff, with the same three steps.
- `SimilarHistogram`: similar 3.2.1, `Algorithm::Histogram`. similar has no indent heuristic.

## Pair sets

- `s1`: the five S1 fixtures from the tag `prototype/diffy-v0.1-q54` (`fixtures/gen.mjs`, checked against `fixtures/checksums.txt`).
- `edge`: 20 hand-made pairs from `edge.sh`: empty files, no newline at the end, CRLF, invalid UTF-8, moved blocks, sliders and more than 64 copies of one line.
- `real`: 3,732 pairs from `corpus.sh`. Each modified file in the last 500 non-merge commits of each repository gives one pair: git/git at `77904e546de3`, tokio-rs/tokio at `854013a8cb7d` and BurntSushi/ripgrep at `3e85c6c148fd`. No pair is binary.

## Run

```sh
cargo build --release
./edge.sh <corpus dir>
./corpus.sh <corpus dir> 500 <bare clone>...
target/release/q43-parity <corpus dir>/pairs-real.tsv results/q43-real.json
```

## Result

The rows are the pairs with identical hunks and change groups. The oracle is `--indent-heuristic`.

| Variant | s1 (5) | edge (20) | real (3,732) |
|---------|--------|-----------|--------------|
| ImaraRaw | 2 | 20 | 3,176 |
| ImaraNoHeuristic | 5 | 20 | 3,596 |
| ImaraIndentHeuristic | 5 | 19 | 3,726 |
| GixRaw | 2 | 20 | 3,176 |
| GixNoHeuristic | 5 | 20 | 3,596 |
| GixIndentHeuristic | 5 | 20 | 3,727 |
| SimilarHistogram | 1 | 18 | 3,490 |

With the `--no-indent-heuristic` oracle, `GixNoHeuristic` has the same counts as `GixIndentHeuristic` with the `--indent-heuristic` oracle. The same is true for imara-diff.

No crate is identical to Git on each pair. The 5 `GixIndentHeuristic` fails are in tokio (`tests/macros_select.rs`, `src/process/unix/mod.rs`, `src/runtime/blocking/pool.rs`) and ripgrep (`crates/globset/src/lib.rs`, `crates/core/main.rs`). In each fail, the crate and Git select different common lines, so the change groups have different sizes. It is not only a different position of the same group. The cause is not known. `first_fails` in `results/q43-real.json` has the first different group of each fail.

imara-diff 0.2.0 has the same 5 fails, plus one slider fail on the `whitespace-lines` edge pair and one on `crates/printer/src/hyperlink/mod.rs`.
