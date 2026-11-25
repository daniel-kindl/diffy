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

## Q46 S8 scale checks

Q46 adds `src/q46.ts`, `src-tauri/src/q46.rs`, `fixtures/gen-repos.mjs` and `q46-run.sh`. It measures option B only (ADR 0004). The checks run when `DIFFY_Q46` names the parts. Each result file has the method (the `METHOD` list in `src/q46.ts`), the hardware, the OS and the WebKitGTK version.

```sh
./q46-run.sh <work-dir>
```

`q46-run.sh` checks the S1 fixtures, writes the fixture repositories, builds the app and runs each part. Each list fixture runs in a new app process.

Changes for Q46:

- The change list uses gix 0.91.0 as ADR 0005 says, with the features `status`, `revision`, `sha1` and `parallel`. Q44 had no `parallel`. Without it, a gix repository cannot go to another thread.
- The line counts and the open path use gix-imara-diff 0.3.4 with the Q60 patch. `src-tauri/q46-setup.sh` downloads the crate, checks its SHA-256 and applies `src-tauri/gix-imara-diff-0.3.4-q60.patch`.
- The Q54 memory sampler also records the anonymous (`RssAnon`) and the file-backed (`RssFile`) part of the RSS.

### Fixture repositories

`fixtures/gen-repos.mjs` writes each repository with `git fast-import`, with fixed names and dates. The comparison is branch `base` to branch `head`, tree to tree. `bench-results/q46-fedora-repos.json` has the object IDs.

| Fixture | Files in `base` | Content of `base` | Changed files | Changed lines (`git diff --shortstat --histogram -M`) |
|---------|-----------------|-------------------|---------------|-------------------------------------------------------|
| `files-1000` | 2,000 | 6.2 MB | 1,000 | 6,957 insertions, 5,117 deletions |
| `files-1000-large` | 2,000 | 333 MB | 1,000 | 124,600 insertions, 122,760 deletions |
| `files-2000-large` | 4,000 | 668 MB | 2,000 | 249,199 insertions, 245,519 deletions |
| `files-5000` | 10,000 | 31 MB | 5,000 | 34,777 insertions, 25,577 deletions |
| `lines-10k` | 11 | 2.1 MB | 1 (the S1 `10k-lines` pair) | 10,000 insertions, 10,000 deletions |
| `stress-50mb` | 11 | 51 MB | 1 (the S1 `50mb-stress` pair) | 23,414 insertions, 23,309 deletions |

Each many-file fixture has modified, added, deleted and renamed files, in the ratio 90:4:4:2.

### Q46 Fedora result

The Fedora reference host of Q54: AMD Ryzen 5 3600, 16 GB, the `rust-dev` container (Fedora 44), WebKitGTK 2.54.2, Wayland. 21 runs of each change list.

| Fixture | Files in the list | Change list p95 (target < 3 s) | Line counts | Line counts equal Git |
|---------|-------------------|--------------------------------|-------------|-----------------------|
| `files-1000` | 1,000 | 65 ms | 150 ms | yes |
| `files-1000-large` | 1,000 | 1,046 ms | 3,507 ms | yes |
| `files-2000-large` | 2,000 | 2,169 ms | 6,863 ms | yes |
| `files-5000` | 5,000 | 241 ms | 651 ms | yes |
| `lines-10k` | 1 | 33 ms | 50 ms | yes |
| `stress-50mb` | 1 | 33 ms | 751 ms | yes |

- Rename detection is most of the change list time on the large fixtures. With `DIFFY_Q46_RENAMES=0`, the p95 is 32 ms for `files-1000-large` and 33 ms for `files-2000-large` (`*-no-renames.json`). Those lists have 1,020 and 2,040 files, because each rename is a deletion and an addition.
- No phase of the list part had a stall. No timer callback ran more than 50 ms late, and no frame gap was more than 50 ms. The list part is the change list runs, the line counts and the list scroll. The list scroll had no long frame on any fixture.

Cancellation (target: acknowledgement < 250 ms):

| Case | Runs | Acknowledgement p50 | p95 | Max |
|------|------|---------------------|-----|-----|
| Line counts of `files-5000`, cancelled at 300 ms | 21 | 21 ms | 22 ms | 28 ms |
| Open of the 50 MB file, cancelled at 5 to 90 % of the open | 20 | 80 ms | 385 ms | 441 ms |

- The line count workers stopped 52 ms or less after the cancel action, before their next file. No batch after the cancel was applied.
- The 50 MB open takes 1,382 ms in full. A cancel action at 60 % of the open comes during the Monaco model creation. The timer callback then runs 261 to 364 ms late, and the acknowledgement takes 343 to 441 ms. The other cancel points take 18 to 190 ms. No cancelled open left an editor.

Memory of the list part, the peak of each sampler (Rust process `diffy-s1-monaco`):

| Fixture | Rust anonymous: change list runs | Rust anonymous: line counts | Rust anonymous: after the counts | Rust file-backed | WebView RSS |
|---------|------|------|------|------|------|
| `files-1000` | 48 MB | 48 MB | 48 MB | 125 MB | 270 MB |
| `files-1000-large` | 58 MB | 153 MB | 79 MB | 218 MB | 263 MB |
| `files-2000-large` | 64 MB | 131 MB | 114 MB | 313 MB | 261 MB |
| `files-5000` | 49 MB | 52 MB | 52 MB | 135 MB | 270 MB |

The MB values are kB / 1,000, rounded.

- The WebView memory does not grow with the files or their content: 261 to 270 MB for each list fixture.
- The Rust anonymous peak of the line counts does not grow from `files-1000-large` to `files-2000-large`. It is working memory of the 4 workers. It goes down after the counts. With 1 worker, the peak was about 25 MB over the list runs, but the counts took 4 times longer (probe runs, not in `bench-results/`).
- After the counts, the Rust anonymous memory is 79 MB for `files-1000-large` and 114 MB for `files-2000-large`. With no rename detection, it is 84 MB and 69 MB. So the growth comes with rename detection.
- The Rust file-backed memory grows with the pack data that gix reads: the pack is memory-mapped. The page cache holds these pages, and the kernel can reclaim them. The packs are 142 MB and 283 MB.
- The gix delta cache settings (`core.deltaBaseCacheLimit` and `gitoxide.core.defaultPackCacheMemoryLimit`, `DIFFY_Q46_PACK_CACHE`) did not change the peaks (probe runs).

Open of one file in option B, after the list:

| Fixture | First file | Longest stall in the open phase | Rust RSS | WebView RSS |
|---------|------------|---------------------------------|----------|-------------|
| `files-1000` | 209 ms | 145 ms | 173 MB | 312 MB |
| `files-5000` | 172 ms | 102 ms | 187 MB | 325 MB |
| `lines-10k` | 399 ms | 207 ms | 186 MB | 473 MB |
| `stress-50mb` | 1,376 ms | 492 ms | 426 MB | 1,107 MB |

The first open in each process creates the first Monaco editor, so it is slower. The next opens on the many-file fixtures take 36 to 68 ms.

### Not measured in Q46

- A worktree status (the index and the files on disk). Each comparison is commit to commit.
- A cold start with an empty page cache. Each run reads files that are in the page cache.
- Windows. Q59 has the Windows notebook.
- A change list with rename detection on more than 2,000 large changed files, and the rename limit of gix.
