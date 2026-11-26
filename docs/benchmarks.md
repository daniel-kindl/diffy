# Benchmarks

This page documents the benchmark method of requirement F-07 (issue 116).

## Method

- Run `cargo bench -p diffy-comparison --bench canonical`. Set `DIFFY_BENCH_RUNS` to change the number of runs. The default is 5.
- The inputs are synthetic and deterministic. Each line has 80 bytes of hexadecimal digits. The changed lines are spread evenly.
- Each case prints one JSON line.
- The line has the median and the maximum time of the canonical diff.
- It also has the time of the anchor mapping (one run), the number of hunks and mapped anchors, and the peak resident memory (`VmHWM`, Linux only).
- The reference hardware is the Fedora host (Q38, Q56). A run on other hardware shows relative changes only.

## Result on the reference host

The host has an AMD Ryzen 5 3600 and runs in the `rust-dev` container. The build is the release profile.

| Case | Hunks | Canonical diff (median) | Anchor mapping | Peak memory |
|---|---|---|---|---|
| 1 MB, 100 changes | 100 | 2.3 ms | 6.7 ms | 9.6 MB |
| 100,000 lines, 10,000 changed lines | 10,000 | 1,282 ms | 1,320 ms | 60 MB |
| 50 MB, 1,000 changes | 1,000 | 791 ms | 1,162 ms | 400 MB |

The 10,000 changed lines case is slower than the 36 ms that Q65 measured for its own pair. The shapes differ. This input is 100,000 unique lines with a changed line every 10 lines.

The time is in the histogram algorithm, not in the postprocessing: the histogram step takes 1.33 s and `postprocess_lines` takes 0.3 ms.

Git 2.55.0 with `git diff --no-index --histogram -U0` takes 4.0 s on the same two files, and `--patience` takes 0.08 s. So the canonical diff is about three times faster than Git on this shape. The cost comes from the histogram algorithm that ADR 0002 requires, and it is not a defect of the copy.

A target of spec section 18 for this shape needs an owner decision on whether the canonical diff may change the algorithm for such inputs. A change would break the parity with Git.

## Not measured yet

- Memory beyond the peak, CPU use and stalls of the renderer.
- The live refresh of option B on the Windows notebook, the read IPC time, and the settled time against 500 ms (Q79, Q59). Only the owner can run these.
- CI runs of the benchmark. They need a workflow change, so they wait for the owner.
