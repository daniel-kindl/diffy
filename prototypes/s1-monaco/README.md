# S1 Monaco prototype (Q41)

This is a throwaway prototype for map question Q41, the build phase of the S1 charter in Q54. See `docs/maps/diffy-v0.1.md` on `main`. The branch `prototype/diffy-v0.1-q41` is never merged. Q54 gives the verdict, not this prototype.

## Options

- Option A: the Monaco DiffEditor. Monaco calculates its own presentation diff in a web worker.
- Option B: two Monaco editors that show the canonical hunks from Rust. Whole-line decorations mark the removed and added lines. View zones pad the shorter side of each change block, so the rows align. The two editors scroll together.

Until Q43 selects a diff crate, the canonical hunks come from the ADR 0002 oracle command. The Rust command `canonical_hunks` runs it outside each repository, with no system or global Git configuration:

```sh
GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null \
  git diff --no-index --histogram -U3 --inter-hunk-context=0 --no-color --no-ext-diff --no-renames -- <left> <right>
```

## Fixtures

`fixtures/gen.mjs` writes the fixture set to `fixtures/out/`. Git ignores that folder. The output is deterministic, and `fixtures/checksums.txt` has the SHA-256 of each file. Q54, Q47 and Q46 use the same set.

| Fixture | Each side | Canonical diff (git --shortstat) |
|---------|-----------|----------------------------------|
| `small` | about 4 KB, 120 lines | 10 insertions, 8 deletions |
| `1mb` | about 1.0 MB, 28,000 lines | 529 insertions, 688 deletions |
| `10mb` | about 10 MB, 282,000 lines | 5,852 insertions, 5,662 deletions |
| `50mb-stress` | about 50 MB, 1.4 million lines | 23,414 insertions, 23,309 deletions |
| `10k-lines` | about 2 MB, 60,000 lines | 10,000 insertions, 10,000 deletions |

## Benchmark harness

The harness records the hardware, the OS, the WebKitGTK version, the session type, the workload and the method with each result, as specification section 18 says. The method is the `METHOD` list in `src/main.ts`. Each result file goes to `bench-results/`.

## Run

You need Node.js 24, Rust stable and the Tauri Linux libraries. On Fedora:

```sh
sudo dnf install webkit2gtk4.1-devel openssl-devel libappindicator-gtk3-devel librsvg2-devel libxdo-devel
npm ci
npm run fixtures
npx tauri build --no-bundle
src-tauri/target/release/diffy-s1-monaco
```

Select an option and a fixture, then click Open or Run benchmark.

For an automatic run with no input, set these variables. The app writes `bench-results/<label>.json` and exits with 0 when each open passes its checks:

```sh
DIFFY_S1_AUTORUN="A:small,A:1mb,B:small,B:1mb" DIFFY_S1_RUNS=5 DIFFY_S1_LABEL=q41-open-fedora \
  src-tauri/target/release/diffy-s1-monaco
```

## Q41 result

`bench-results/q41-open-fedora.json` is the Q41 run on the Fedora host. Both options opened the `small` and the `1mb` fixture in each of 5 runs, and each check passed. The numbers are build-phase measurements only. Q54 measures on the reference host with its own method and gives the verdict.

## Q54 verdict checks

Q54 adds `src/q54.ts` and `src-tauri/src/q54.rs`. The checks run when `DIFFY_S1_Q54` is `A` or `B`. Each result file records the method (the `METHOD` list in `src/q54.ts`), the hardware, the OS and the WebKitGTK version.

```sh
DIFFY_S1_Q54=B DIFFY_S1_Q54_PARTS="coord+findings+scroll+live+hscroll" DIFFY_S1_Q54_LABEL=q54-fedora-checks-B \
  src-tauri/target/release/diffy-s1-monaco
DIFFY_S1_Q54=B DIFFY_S1_Q54_PARTS=stress DIFFY_S1_Q54_FIXTURES=50mb-stress DIFFY_S1_Q54_LABEL=q54-fedora-stress-B \
  src-tauri/target/release/diffy-s1-monaco
DIFFY_S1_AUTORUN="B:1mb,B:10mb" DIFFY_S1_RUNS=21 DIFFY_S1_LABEL=q54-fedora-open-B src-tauri/target/release/diffy-s1-monaco
```

`DIFFY_S1_NO_CACHE=1` runs the oracle on each open. `q54-windows.ps1` is the run for the Windows notebook (map decision Q12).

Changes to the Q41 open path:

- Option B adds a view zone of the scroll bar height below the last line of the side with no horizontal scroll bar. Without it, the rows at the bottom are 12 px out of line (Q42). The `10k-lines` fixture has this case: 26 right lines have 83 characters.
- The Q42 checks build on Linux only, so that the prototype builds on Windows.

## Q54 Fedora result

The Fedora reference host (Q38, Q56): AMD Ryzen 5 3600, 16 GB, amdgpu, the `rust-dev` container (Fedora 44), WebKitGTK 2.54.2, Wayland, a 1,400 by 900 window at scale 1. The oracle is Git 2.55.0.

| Check | Option A | Option B |
|-------|----------|----------|
| Coordinates, `small` and `10k-lines` | pass | pass |
| Coordinates, `1mb` | fail: 4 pairs out of line, 8 left lines marked wrong | pass |
| Coordinates, `10mb` | fail: 43 pairs, 86 left lines | pass |
| Coordinates, `50mb-stress` | fail: 121 pairs, 238 left lines | pass |
| Findings (5 widgets), `small` and `10k-lines` | pass | pass |
| Findings, `1mb` and `10mb` | fail (the coordinate fail) | pass |
| Scrolling, `1mb` and `10mb` | 60 FPS, sides aligned | 60 FPS, sides aligned |
| Live refresh p95, `small` | 236 ms | 38 ms |
| Live refresh p95, `1mb` | 350 ms | 58 ms |
| Live refresh p95, `10mb` | 5,016 ms; selection kept in 3 of 10 runs | 401 ms |
| Open p95, `1mb` (target < 250 ms) | 257 ms | 102 ms (118 ms with no oracle cache) |
| Open p95, `10mb` (target < 3 s) | 2,687 ms | 422 ms (539 ms with no oracle cache) |
| `50mb-stress` open | 25.7 s | 1.8 s |
| `50mb-stress` peak memory of the process tree | 1.63 GB | 1.11 GB |
| `50mb-stress` CPU time to open | 33.0 s | 2.6 s |
| `50mb-stress` longest stall while it opens | 1,254 ms | 390 ms |
| `50mb-stress` scrolling | 58.4 FPS, 6 long frames | 60 FPS, 0 long frames |
| One side with a horizontal scroll bar | pass | pass with the pad; fail without it |

The option A coordinate fail: next to some deleted blocks there is an identical line. Monaco puts the deletion one line higher than Git does. For example, on `1mb` Monaco marks left line 6 as removed, but the canonical block is lines 7 to 11. The rows then show a line that the canonical diff does not change. Monaco 0.58 has no public option for a custom diff: `diffAlgorithm` accepts only `legacy`, `advanced`, `advanced-external` and `advanced-wasm` (`esm/vs/editor/browser/widget/diffEditor/diffEditorOptions.js:167`).

Each live refresh anchors the first visible line with a tracked decoration. Monaco alone keeps the scroll position in pixels, so a line inserted above the viewport moves the content.

The scrolling is programmatic (`setScrollTop` in each animation frame), not wheel input. The display refresh limits the frame rate to 60 FPS.
