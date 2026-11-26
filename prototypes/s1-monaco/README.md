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

## Q78 check: fcitx5 with the preedit on

Map question Q78 checks fcitx5 with an engine that has a preedit, with the preedit on (Q57). It uses option B (ADR 0004).

- `src/q78.ts` runs when `DIFFY_Q78_LABEL` is set. It turns the preedit on, focuses the target, switches fcitx5 to Mozc and types `nihon`, Space, Space and Return. The targets are the modified editor and a plain textarea.
- `q78-kwin-geo.py` loads a KWin script that returns the screen geometry of each window. The check takes the candidate window as the popup that is new after the typing. It passes when the top-left corner of the candidate window is at most 60 px from the caret on the x axis and 0 to 60 px below the caret bottom.
- Keys go to the focused window, so the check sends no key unless the window has the focus.

Run, with `fcitx5 -d` and `ydotoold` running and fcitx5 set to `keyboard-us` and `mozc`:

    env GDK_BACKEND=wayland GTK_IM_MODULE=fcitx Q78_GEO=$PWD/q78-kwin-geo.py DIFFY_Q78_LABEL=q78-fedora-wayland src-tauri/target/release/diffy-s1-monaco
    env GDK_BACKEND=x11 GTK_IM_MODULE=fcitx Q78_GEO=$PWD/q78-kwin-geo.py DIFFY_Q78_LABEL=q78-fedora-x11 src-tauri/target/release/diffy-s1-monaco

### Q78 result

Host: the Bazzite 45 reference host, KWin 6.8.3 on Wayland, two screens at scale 1. The app ran in the Fedora 44 `rust-dev` container with WebKitGTK 2.54.2, fcitx5 5.1.23, fcitx5-gtk3 5.1.7 and fcitx5-mozc 2.30.5618.102. fcitx5 ran in the container with the GTK input method module (`GTK_IM_MODULE=fcitx`), not as the KWin virtual keyboard.

- Wayland (`bench-results/q78-fedora-wayland.json`) and XWayland (`bench-results/q78-fedora-x11.json`): 7 of 7 checks pass.
- In the editor, the page gets `compositionstart` and a `compositionupdate` for each step of the preedit, `ｎ` to `にほｎ`. Mozc converts to `日本`, and Return commits it. The textarea gets the same events and commits `にほん`.
- Wayland: fcitx5-gtk draws the candidate window in the app, as a popup. In the editor, its top-left corner is 27 px left of the caret, at the start of the conversion, and 0 px below the caret bottom.
- XWayland: fcitx5 draws the candidate window as an unmanaged X11 window. In the editor, it is 27 px left of the caret and 1 px below the caret bottom.
- Not tested: fcitx5 as the KWin virtual keyboard (text-input-v3), Rime, a screen scale other than 1, and GNOME.
