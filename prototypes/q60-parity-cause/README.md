# Q60 cause of the histogram parity fails

This is a throwaway prototype for map question Q60. See `docs/maps/diffy-v0.1.md` on `main`. The branch `prototype/diffy-v0.1-q60` is never merged. It starts from the tag `prototype/diffy-v0.1-q43` and uses the Q43 harness in `prototypes/q43-parity/`.

## Result

gix-imara-diff 0.3.4 differs from Git 2.55.0 `xdiff/` in four steps. With all four changes, the crate with `postprocess_lines` matches the Q43 oracle (`--indent-heuristic`) on each pair: 5 of 5 S1, 20 of 20 edge and 3,732 of 3,732 real. With `postprocess_no_heuristic`, it matches the `--no-indent-heuristic` oracle on each real pair.

| Step | gix-imara-diff 0.3.4 | Git 2.55.0 | Change in the patch |
|------|----------------------|------------|---------------------|
| 1. Skip of the next occurrences after a match | skips while the next left position is not greater than `end2`, the right side end (`src/histogram/lcs.rs:135`) | skips while the next left position is not greater than `ae`, the left side end (`xdiff/xhistogram.c:212`) | compare with `end1` |
| 2. Occurrence limit | accepts a common region only when each line in it has at most 63 copies (`src/histogram.rs:14`, `src/histogram/lcs.rs:60`) | accepts at most 64 copies (`xdiff/xhistogram.c:284`, `xdiff/xhistogram.c:294`) | `MAX_CHAIN_LEN` 64, and one more list size class (`src/histogram/list_pool.rs:78`) |
| 3. Common ends | strips the common prefix and suffix before the histogram (`src/lib.rs:276-277`) | does not trim the ends for histogram (`xdiff/xprepare.c:461`) | no strip for histogram |
| 4. Re-diff after a slide | none (`src/postprocess.rs`) | a group that changed while it slid gets a Myers diff against the group on the other side, when that group is not empty (`xdiff/xdiffi.c:921-955`) | a `rediff` step after each group |

The list pool keeps at most `MAX_CHAIN_LEN + 1` positions for each line (`src/histogram/list_pool.rs:237` and `:253`). So the crate cannot tell 64 copies from more than 64. Step 2 needs the larger size class so that the pool keeps 65 positions.

## Fails for each build

The real pairs are the 3,732 Q43 pairs. The counts are the fails of `GixIndentHeuristic` against the `--indent-heuristic` oracle.

| Build | Real fails | Result file |
|-------|------------|-------------|
| Q43 crate, no change | 5 | `prototypes/q43-parity/results/q43-real.json` |
| Step 1 | 2 | `results/q60-skip-real.json` |
| Step 2 | 4 | `results/q60-chain-real.json` |
| Steps 1 and 2 | 1 | `results/q60-both-real.json` |
| Steps 1, 2 and 3 | 1 (a different pair) | `results/q60-all-real.json` |
| Steps 1, 2 and 4 | 1 | `results/q60-r-real.json` |
| Steps 1 to 4 | 0 | `results/q60-full-real.json` |
| Steps 2, 3 and 4 | 3 | `results/q60-noskip-real.json` |
| Steps 1, 3 and 4 | 1 | `results/q60-nochain-real.json` |
| Steps 1, 2 and 4, with the strip | 1 | `results/q60-trim-real.json` |

So each of the four steps is necessary on these pairs.

- Step 1 repairs three Q43 fails: tokio `src/runtime/blocking/pool.rs` and `src/process/unix/mod.rs`, and ripgrep `crates/globset/src/lib.rs`.
- Step 2 repairs tokio `tests/macros_select.rs`.
- Step 3 repairs ripgrep `crates/core/main.rs`. `ddmin.py` shrank that pair to `min.left` and `min.right` (3 and 4 lines). With the strip, the crate removes the suffix line `}` before the histogram and anchors the first of the two `    }` lines. Git anchors the second.
- Step 4 repairs ripgrep `crates/ignore/src/pathutil.rs`. That pair fails only after step 3.

## Run

```sh
cp -r ~/.cargo/registry/src/index.crates.io-*/gix-imara-diff-0.3.4 gix-q60
(cd gix-q60 && git apply -p1 <repository>/prototypes/q60-parity-cause/gix-imara-diff-0.3.4-q60.patch)
```

Then add this to `prototypes/q43-parity/Cargo.toml` and run the Q43 harness as its README says:

```toml
[patch.crates-io]
gix-imara-diff = { path = "<path to gix-q60>" }
```

## Timing

`timing.rs` times `Diff::compute` with `Algorithm::Histogram` and `postprocess_lines`, best of 5 runs, with no interning. On the Fedora host container, release build:

| Fixture | Crate, no change | Steps 1 to 4 |
|---------|------------------|--------------|
| `1mb` | 1.4 ms | 1.5 ms |
| `10mb` | 23.2 ms | 24.5 ms |
| `50mb-stress` | 165.9 ms | 180.6 ms |
| `10k-lines` | 5.5 ms | 5.6 ms |

Each value is one measurement session. The method is not the section 18 method.

## Not run

- The tests of the crate. The published crate has no `src/tests.rs`, so `cargo test` does not compile.
- Other Git versions than 2.55.0.
