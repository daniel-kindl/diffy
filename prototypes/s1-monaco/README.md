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
