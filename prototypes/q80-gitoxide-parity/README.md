# Q80: parity of the gitoxide 2bcb31d diff

Throwaway prototype for map question Q80. See `docs/maps/diffy-v0.1.md` on `main`. Never merge this branch. It starts from the tag `prototype/diffy-v0.1-q60` and uses the Q43 harness in `prototypes/q43-parity/` and `prototypes/q60-parity-cause/timing.rs`.

## Code under test

- gix-imara-diff from gitoxide `2bcb31df9f05fb1375e82f76521ac16ed5dbdf09`, the merge of pull request 3246. The folder `gix-imara-diff/` is the tree `675c5a1b693d3abb9dc1cbd93ef5f0cfa1c3f296`. The only change for the build is the removal of the line `lints.workspace = true` from its `Cargo.toml`, because the folder is built outside the gitoxide workspace.
- The control is the Q60 copy: gix-imara-diff 0.3.4 from crates.io with `prototypes/q60-parity-cause/gix-imara-diff-0.3.4-q60.patch`.

## Host

The Fedora 44 container on the Bazzite reference host. Git 2.55.0 is the oracle, with the Q43 oracle command. rustc 1.98.1, release build.

## Result

With `postprocess_lines` (`GixIndentHeuristic`) against the `--indent-heuristic` oracle:

| Set | Pairs | Pass | Result file |
|-----|-------|------|-------------|
| s1 | 5 | 5 | `results/q80-s1.json` |
| edge | 20 | 20 | `results/q80-edge.json` |
| real | 3,732 | 3,732 | `results/q80-real.json`, `results/q80-real.log` |
| q62 | 9 | 9 | `results/q80-q62.json` |

With `postprocess_no_heuristic` against the `--no-indent-heuristic` oracle, each real pair also passes.

The S1 fixtures match `fixtures/checksums.txt` of the tag `prototype/diffy-v0.1-q54`. The edge and real pairs are the Q43 corpus from `edge.sh` and `corpus.sh`.

`GixRaw`, with no postprocessing, passes fewer pairs than in Q43 (3,112 real and 18 edge, against 3,176 and 20). Diffy does not use the raw output.

### Q62 inputs

`q62-inputs/` has the input from the maintainer review of pull request 3246, `seed66`: each line is one token, `B` 4 times, `A A B A`, then `B` 61 times, against `B A B A`. The files `run<n>` have `n` copies of `B` at the end, so the seed `B` occurs `n + 5` times.

- 2bcb31d passes each of the 9 inputs. On `seed66`, Git and 2bcb31d remove 65 lines and add none.
- Control: the Q60 copy fails each input where `B` occurs more than 65 times (`run61` to `run100` and `seed66`), and passes `run59` and `run60` (`results/q80-q62-control-q60.json`). So the inputs detect the Q60 fail.

## Timing

`timing.rs` of Q60: `Diff::compute` with `Algorithm::Histogram` and `postprocess_lines`, best of 5 runs, with no interning. Three sessions, run one after the other in the order Q60, 2bcb31d:

| Fixture | Q60 copy | 2bcb31d |
|---------|----------|---------|
| `50mb-stress` | 163.6, 182.2, 165.2 ms | 122.4, 121.6, 117.0 ms |
| `10mb` | 24.0, 24.3, 24.8 ms | 16.3, 16.1, 16.2 ms |
| `1mb` | 1.4, 1.4, 1.4 ms | 0.9, 0.9, 0.9 ms |
| `10k-lines` | 5.5, 5.6, 5.5 ms | 3.9, 3.9, 3.9 ms |

The method is not the section 18 method.

## Not run

- The tests of the crate. `src/tests.rs` needs dev dependencies from the gitoxide workspace.
- Other Git versions than 2.55.0.
- The fuzz targets in `gix-imara-diff/fuzz/`.

## Run

```sh
git clone --filter=blob:none --no-checkout https://github.com/GitoxideLabs/gitoxide.git
cd gitoxide && git sparse-checkout set --no-cone /gix-imara-diff/ && git checkout 2bcb31d
cp -r gix-imara-diff <path to gix-2bcb31d> && sed -i '/^lints.workspace/d' <path to gix-2bcb31d>/Cargo.toml
```

Add this to `prototypes/q43-parity/Cargo.toml`, then run the Q43 harness as its README says, with each pair set and with a TSV of the `q62-inputs/` pairs:

```toml
[patch.crates-io]
gix-imara-diff = { path = "<path to gix-2bcb31d>" }
```

For the timing, build `prototypes/q60-parity-cause/timing.rs` once against each crate and run it with the fixture paths without `.left.txt`.
