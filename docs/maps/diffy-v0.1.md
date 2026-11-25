# Map: Diffy v0.1

Status: open
Destination: Each v0.1 milestone (M0 to M4) has a plan and issues with acceptance criteria. Each spike has a verdict, and no open question blocks the first M0 issue.
Next: Q60

## Notes

- The specification is `Diffy_Master_Specification.md` (version 0.1 planning baseline, 2026-11-24). It is not tracked in Git yet. The owner decided to leave it untracked for now. Source: Q39 in this map.
- The specification makes DKSTACK the authoritative engineering guidance. Source: Diffy_Master_Specification.md section 19.
- DKSTACK requires one issue, one branch and one pull request, squash merge only, and a reviewer that is not the author. Source: https://raw.githubusercontent.com/daniel-kindl/dkstack/v1/workflow/review-and-merge.md
- DKSTACK requires that prototype branches are tagged and never merged. Source: https://raw.githubusercontent.com/daniel-kindl/dkstack/v1/skills/chart-route/SKILL.md
- DKSTACK allows a release only when the owner starts it, and a release workflow starts only by `workflow_dispatch`. Source: https://raw.githubusercontent.com/daniel-kindl/dkstack/v1/workflow/safety.md
- The repository has the 10 DKSTACK labels (six `type:` and four `priority:` labels) and no other labels. Source: `gh label list` on 2026-11-24.
- The specification links the logo as `diffy-assets/media/image1.png`, but the file is `image1.png` at the repository root. Source: Diffy_Master_Specification.md line 3.
- Some decisions change the specification. The changed sections are signing (Q6), M3 scope (Q16), OneDev versions (Q17) and the Windows minimum (Q18). Q24 adds requirement C-09 for the CLI. Q28 adds requirement R-08 for the Markdown copy. Source: the Decisions section of this map.
- The Q41 run used WebKitGTK 2.54.2 in the `rust-dev` distrobox container (Fedora 44) on the host, in a Wayland session. Source: https://github.com/daniel-kindl/diffy/blob/627236e4c92c2161e51aa06204d0705537b46f86/prototypes/s1-monaco/bench-results/q41-open-fedora.json
- Each frontend phase duration in the Q41 result file is a whole number of milliseconds. Q41 did not measure the clock resolution. Source: https://github.com/daniel-kindl/diffy/blob/627236e4c92c2161e51aa06204d0705537b46f86/prototypes/s1-monaco/bench-results/q41-open-fedora.json
- On the 1 MB fixture, 246 of the 250 option A presentation changes are equal to a canonical block. Q54 checks the coordinates. Source: https://github.com/daniel-kindl/diffy/blob/627236e4c92c2161e51aa06204d0705537b46f86/prototypes/s1-monaco/bench-results/q41-open-fedora.json
- The host has no X11 session. `/usr/share/xsessions` on the Bazzite 45 host is empty. The Q42 X11 results use XWayland with `GDK_BACKEND=x11`. Source: Q42 run, 2026-11-24.
- The Q42 Fedora runs used WebKitGTK 2.54.2 and GTK 3.24.52 in the `rust-dev` container. The Q42 Ubuntu runs used WebKitGTK 2.52.6 and GTK 3.24.41 in an Ubuntu 24.04.5 container. Each run used the DMABuf renderer with GPU threaded rendering, with no `WEBKIT_*` environment variable. Source: https://github.com/daniel-kindl/diffy/blob/37f9892/prototypes/s1-monaco/bench-results/q42-fedora-wayland-A.json and https://github.com/daniel-kindl/diffy/blob/37f9892/prototypes/s1-monaco/bench-results/q42-ubuntu-wayland-A.json
- wry 0.57.3 turns off the preedit of the WebKitGTK input method. The page gets the committed text, but no composition updates. A bare WebKitGTK 2.54.2 view gets the composition events. When the prototype turns the preedit on again, Monaco gets the composition events in each run. Source: wry 0.57.3 `src/webkitgtk/mod.rs:429` and https://github.com/daniel-kindl/diffy/blob/37f9892/prototypes/s1-monaco/bench-results/q42-fedora-x11-A.json
- The Q41 prototype imports the core Monaco editor. It has no find widget and no tab focus mode, so Tab does not leave an editable editor. The Q42 branch adds the two Monaco contributions. Source: https://github.com/daniel-kindl/diffy/blob/37f9892/prototypes/s1-monaco/src/q42.ts
- The Monaco default font, Droid Sans Mono, is not installed on Fedora or Ubuntu. WebKit uses the `monospace` fallback: Adwaita Mono on Fedora and DejaVu Sans Mono on Ubuntu. Source: https://github.com/daniel-kindl/diffy/blob/37f9892/prototypes/s1-monaco/bench-results/q42-fedora-x11-A.json and https://github.com/daniel-kindl/diffy/blob/37f9892/prototypes/s1-monaco/bench-results/q42-ubuntu-x11-A.json
- At 125% scale, the XWayland window has a device pixel ratio of 1.25. The Wayland window has a device pixel ratio of 2, because GTK 3 has no fractional scale. KWin scales the window down. Source: https://github.com/daniel-kindl/diffy/blob/37f9892/prototypes/s1-monaco/bench-results/q42-fedora125-x11-A.json and https://github.com/daniel-kindl/diffy/blob/37f9892/prototypes/s1-monaco/bench-results/q42-fedora125-wayland-A.json
- In option B, a horizontal scroll bar on one side only adds its height to that side. In a 1,120 px window, the content heights were 2,304 px and 2,316 px with the same 128 rows. Source: https://github.com/daniel-kindl/diffy/blob/37f9892/prototypes/s1-monaco/bench-results/q42-fedora125-x11-B.json
- On the 1 MB, 10 MB and 50 MB fixtures, the Monaco DiffEditor (option A) puts some deletions one line higher than the canonical block. An identical line is next to each such block. On the 1 MB fixture, 4 unchanged pairs are out of line and 8 left lines have the wrong mark. Option B has no fail on the five fixtures. Source: https://github.com/daniel-kindl/diffy/blob/f87a6fb/prototypes/s1-monaco/bench-results/q54-fedora-checks-A.json and https://github.com/daniel-kindl/diffy/blob/f87a6fb/prototypes/s1-monaco/bench-results/q54-fedora-checks-B.json
- The public `diffAlgorithm` option of Monaco 0.58 accepts only `legacy`, `advanced`, `advanced-external` and `advanced-wasm`. A custom diff provider is not possible through it. Source: monaco-editor 0.58.0 `esm/vs/editor/browser/widget/diffEditor/diffEditorOptions.js:167`
- The `10k-lines` fixture has 26 right lines of 83 characters, so only the right side shows a horizontal scroll bar. Option B then needs the scroll bar pad of Q54. Source: https://github.com/daniel-kindl/diffy/blob/f87a6fb/prototypes/s1-monaco/README.md
- Monaco keeps the scroll position in pixels when the model changes. A line inserted above the viewport moves the content, unless the refresh code anchors the first visible line. Source: https://github.com/daniel-kindl/diffy/blob/f87a6fb/prototypes/s1-monaco/src/q54.ts
- Git trims the common tail of the two inputs before the diff when the context is 0 and there is no function context. So `-U0` can place a change group at other lines than `-U3`. In Q43, this happened on 8 of 3,732 real pairs. Source: git 2.55.0 `xdiff-interface.c:127` and https://github.com/daniel-kindl/diffy/blob/061a6a3/prototypes/q43-parity/results/q43-real.json
- Git turns on the indent heuristic when no configuration sets `diff.indentHeuristic`. Source: git 2.55.0 `diff.c:57`
- In the histogram diff, Git falls back to the classic diff for a region where each common line has more than 64 copies. Source: git 2.55.0 `xdiff/xhistogram.c:284` and `xdiff/xhistogram.c:334`
- Git `xdiff/xhistogram.c` has the Eclipse Distribution License 1.0. The other Git `xdiff/` files, for example `xdiffi.c`, have the LGPL 2.1 or later. Source: git 2.55.0 `xdiff/xhistogram.c:6` and `xdiff/xdiffi.c:6`
- similar 3.2.1 has a histogram algorithm, but no indent heuristic. Source: similar 3.2.1 `src/algorithms/histogram.rs:1` and `src/algorithms/compact.rs:1`
- gix-imara-diff 0.3.4 is the gitoxide copy of imara-diff, with changes that follow Git. gix-diff 0.71.0 uses it. Source: gix-imara-diff 0.3.4 `src/postprocess.rs:22` and `src/myers.rs:47`, and https://index.crates.io/gi/x-/gix-diff
- With the Q43 oracle and `--indent-heuristic`, a crate passes a pair when the hunks and the change groups are identical. gix-imara-diff 0.3.4 with `postprocess_lines` passes 5 of 5 S1, 20 of 20 edge and 3,727 of 3,732 real pairs. imara-diff 0.2.0 with `postprocess_lines` passes 5, 19 and 3,726. The similar 3.2.1 histogram passes 1, 18 and 3,490. Source: https://github.com/daniel-kindl/diffy/blob/061a6a3/prototypes/q43-parity/README.md
- In each of the 5 gix-imara-diff fails, the crate and Git select different common lines, so the change groups have different sizes. Source: https://github.com/daniel-kindl/diffy/blob/061a6a3/prototypes/q43-parity/results/q43-real.json

## Decisions

- Q1 Independent review: tiered by risk. Safety-critical areas (Git adapter, publication, anchors, updater, IPC and capabilities) need owner review. Other areas merge after agent review and green CI. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q2 Repository visibility: public and open source. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q3 Spike order: throwaway spikes before the M0 scaffold. Spike results decide the crate and renderer shape. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q4 Capacity: solo owner with coding agents, no target date. Quality gates decide the release. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q5 License: MIT OR Apache-2.0. Decided by: Daniel Kindl, 2026-11-24. Record: [ADR 0001](../adr/0001-license-mit-or-apache-2.md).
- Q6 Windows code signing: no Authenticode in v0.1. Owner words: "Lets not do signed, I dont want to apply or pay. This tool is mainly for me so its not a big deal." SmartScreen warnings are a known limitation. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q7 Test machines: the Fedora/Bazzite host is the main real test machine. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q8 Spike end: evidence-driven, with no time box. The agent proposed charters as a control, see Q40. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q9 Updater: keep the Tauri Updater with a self-generated minisign key. The private key is only in a GitHub Actions secret and an offline backup. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q10 Windows testing: CI builds and automated tests on Windows at all times. Manual QA on the Windows notebook only before a release. Owner words: "I have windows notebook. I can do some test there but prefer to do only testing on that before release." Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q11 Dogfood releases: Linux AppImage pre-releases (v0.0.x) after M1 and after M2, with no OS code signing. Update artifacts still get the Q9 minisign signature, see Q36. The full release gate applies only to v0.1.0. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q12 S1 on Windows: one S1 check on the Windows notebook is an exception to Q10. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q13 Canonical diff: hunks match `git diff --histogram`. Decided by: Daniel Kindl, 2026-11-24. Record: [ADR 0002](../adr/0002-canonical-diff-matches-git-histogram.md).
- Q14 OneDev server: the homelab instance. Owner words: "I have OneDev on my local homelab, we can test it there, but I dont use it often." Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q15 GitHub tests: recorded fixtures in CI, plus a manual live smoke test against a sandbox repository. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q16 Provider order: M3a is GitHub and M3b is OneDev. Both are necessary for v0.1.0. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q17 OneDev versions: latest stable only, tested in a CI container. The homelab is upgraded to match. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q18 Windows minimum: Windows 11 only. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q19 Linux baseline: build the AppImage on Ubuntu 22.04. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q20 Planning output: this map in `docs/maps/` through chart-route, with ADRs. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q21 Coverage gates: tiered by crate. Domain crates (diffy-comparison, diffy-review) need at least 90% lines and 85% branches, with nightly cargo-mutants. Adapters and UI need at least 75% lines. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q22 Dependency updates: Renovate (GitHub app, free) for update pull requests. GitHub Dependabot security alerts stay on. Owner words: "is renovate free to use and setup as Dependadbot? if yes the Renovate". Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q23 Agents: any client (Claude, Codex or Grok) can author a change. The reviewer must use a different client than the author. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q24 CLI: `diffy <left> <right>`, `diffy <repo-path>` and `git difftool` integration in v0.1. All are read-only and open in the running instance. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q25 Window model: single instance, one tab for each comparison or review session. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q26 Encodings: detect UTF-8 and UTF-16 by BOM. The user can select a legacy encoding for each side. There is no heuristic detection. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q27 Watch scope: live mode respects `.gitignore` and always watches the Git index and refs. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q28 Agent handoff: one action copies the selected or unresolved findings to the clipboard as Markdown. Full report export stays in v0.2. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q29 Reviewed state: one reviewed state for each file, as in the specification. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q30 Checkpoints: the user creates a checkpoint with an explicit action. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q31 Finding taxonomy: a fixed set. Severity is blocker, major, minor or nit. Category is correctness, security, performance, design, tests, docs or style. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q32 GitHub authentication: OAuth device flow with a registered OAuth App. The token is in the OS credential store. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q33 Publication unit: on GitHub, one batched review with an explicit review event. The question named GitHub pull requests. Q49 covers OneDev. Decided by: Daniel Kindl, 2026-11-24. Record: [ADR 0003](../adr/0003-remote-publication-format.md).
- Q34 Publication marker: a hidden HTML comment marker with the finding ID and the attempt ID. Decided by: Daniel Kindl, 2026-11-24. Record: [ADR 0003](../adr/0003-remote-publication-format.md).
- Q35 Remote threads: Diffy can reply to and resolve threads where the provider allows it. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q36 Dogfood updates: the v0.0.x AppImages use the updater from v0.0.2, on a pre-release channel. The user installs v0.0.1 manually, and it contains the updater public key. Each release from v0.0.2 has update artifacts with a minisign signature. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q37 Screen readers: Orca on Fedora during development, axe-core checks in CI, and NVDA on Windows at release candidate. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q38 Documentation and performance reference: README and Markdown in `docs/`. The Fedora host is the reference hardware for the performance targets. CI benchmarks show relative regressions only. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q39 Specification files: leave the specification and the logo untracked for now. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q40 Spike charters: each spike S1 to S8 has one charter, with the question, the evidence and the stop conditions. The S1 charter is in Q54 and the S6 charter is in Q55. The other charters are in Q42, Q44 to Q47 and Q49. Owner words, after the PR #24 review: "Approve the revised charters". The owner record is on issue #4. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q41 S1 Monaco prototype build: both options open the small and the 1 MB fixture on Fedora. The prototype, the fixture set and the benchmark harness are on the tag `prototype/diffy-v0.1-q41`. Owner words: "Accept the build phase". The owner record is on issue #5. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q56 Container as the reference host: Q54 measurements in the `rust-dev` distrobox container count as the Fedora reference host of Q38. Owner words: "yes they do count". The owner record is on issue #27. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q42 S2 Linux WebKitGTK behavior: both options work on WebKitGTK under Wayland and XWayland, on Fedora and on Ubuntu 24.04. No check has a blocker. The input method preedit goes to Q57, and the Monaco features and font go to Q58. Owner words: "Accept the S2 results". Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q54 S1 renderer verdict: option B, two Monaco editors that show the canonical hunks from Rust. Only option B passes the coordinate check. Owner words: "B". The Windows run of Q12 comes later, in Q59. For the order, the owner selected the option "Record B, Windows later". Decided by: Daniel Kindl, 2026-11-24. Record: [ADR 0004](../adr/0004-renderer-two-monaco-editors-with-canonical-hunks.md).
- Q47 S7 CodeMirror fallback: skipped. Monaco passes in Q54, and the Q47 charter (Q40) skips the benchmark in that case. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q43 Diff crate with histogram parity: no crate is identical to Git on each pair. The closest is gix-imara-diff 0.3.4 with `postprocess_lines`, with 5 fails in 3,732 real pairs. The oracle is Git 2.55.0, `git diff --no-index --histogram -U3 --inter-hunk-context=0 --indent-heuristic --no-color --no-ext-diff --no-renames --no-textconv`, with `GIT_CONFIG_NOSYSTEM=1` and `GIT_CONFIG_GLOBAL=/dev/null`, in a folder outside each repository. The cause of the fails goes to Q60 and the fix to Q61. The owner selected the option "Accept the Q43 result". Decided by: Daniel Kindl, 2026-11-25. Record: this line.

## Open questions

### Q44: S3 Git backend

Type: prototype
Blocked by: Q40

Compare gix and git2 on the Git fixtures: index and worktree, renames, submodules, unusual paths and detached HEAD. Prove read-only behavior and a build that needs no installed Git.

Charter question: Which Git backend, gix or git2, reads the fixtures correctly, never writes, and builds with no installed Git?

Evidence, for each backend on the branch `prototype/diffy-v0.1-q44`:

- A result for each Git fixture in specification section 19: index and worktree, unusual filenames, renames, detached HEAD, submodules and binary files.
- The oracle is the `git` CLI with no system or global Git configuration, as in Q43.
- Read-only proof: the checksum of each file in the fixture repository, `.git` included, is the same before and after each operation. No lock file appears.
- The same operations pass on a fixture copy that has no write permission.
- A build and a test run in a container with no `git` executable. A process trace shows that no `git` process starts.
- Build facts: the native dependencies, the binary size and a Windows x64 build.

Stop when:

- A backend qualifies only when it passes each fixture, the read-only proof and the build with no installed Git.
- Each backend has a result for each item, and at least one backend qualifies. The owner picks from the qualified backends.
- No backend qualifies. List the gaps of each backend and ask the owner.

### Q45: S5 anchor mapping

Type: prototype
Blocked by: Q43

Prove conservative re-anchoring across revisions. An ambiguous mapping must become outdated and never publish to a wrong line.

Charter question: Which method moves a finding to the correct line of a new revision, and marks it outdated when the mapping is not certain?

Evidence, on the branch `prototype/diffy-v0.1-q45`:

- Two or three methods. Examples are a map through the Q43 hunks only, and that map plus an exact match of the line and its context.
- Fixture revision pairs. Each pair has one expected result for each anchor: an exact new line, or outdated.
- The cases are: line unchanged, lines added above, line edited, line deleted and duplicate blocks. They also include file renamed, file deleted, whitespace changes and line-ending changes.
- An anchor on a unique unchanged line must map to its exact new line. An anchor in identical duplicate blocks must be outdated.
- A property test that generates an edit history. The generator records the expected new line of each anchor, or outdated when the anchor is edited, deleted or ambiguous.
- In the property test, the method must return the expected line or outdated. Each other line is a wrong mapping.
- For each method: the number of wrong mappings, and the number of outdated results where the generator expected a line.

Stop when:

- A method qualifies when each fixture gives its expected result and the property test has zero wrong mappings. The owner picks from the qualified methods.
- No method qualifies. Ask the owner before more work.

### Q46: S8 scale

Type: prototype
Blocked by: Q44

Measure 1,000 or more changed files with bounded memory and cancellation against the section 18 targets. Measure only option B, the renderer that Q54 selects (ADR 0004).

Charter question: Does Diffy stay responsive with 1,000 or more changed files, with bounded memory and fast cancellation?

Evidence, on the Fedora reference host (Q38) with the selected renderer:

- Generated fixture repositories with 1,000 and with 5,000 changed files, and one with 10,000 changed lines.
- The p95 time to show the change list of 1,000 files. The target is less than 3 s.
- The p95 time from a cancel action to its acknowledgement. The target is less than 250 ms.
- The peak memory of the Rust process and of the WebView process for each fixture. Memory is bounded when it does not grow with the content of files that are not open.
- The CPU use and the stalls, with the method that specification section 18 asks for.

Stop when:

- Each target has a measured result. A missed target is a finding for the owner. It is not an automatic fail, because the section 18 targets are proposals.

### Q48: GitHub thread resolution API

Type: research
Blocked by: none

Find out if GitHub can resolve a review thread through REST, or only through GraphQL (for example `resolveReviewThread`). The answer decides if M3a needs a GraphQL client.

### Q49: S4 OneDev capabilities

Type: prototype
Blocked by: Q40

Find out which review actions the latest stable OneDev REST API supports. Check inline comments, replies, resolution and approval. Check that the Q34 marker is invisible in OneDev. Define how Diffy confirms each OneDev operation (comment, approval, change request) before a retry. See ADR 0003. Run a smoke test against the Q14 homelab instance on the latest stable version (Q17). The test authenticates, retrieves a pull request and does each supported review action.

Charter question: Which review actions can Diffy do through the latest stable OneDev REST API, and how does Diffy confirm each one?

Evidence:

- The OneDev version in the CI container and on the Q14 homelab instance.
- A table with one row for each action: authenticate, get a pull request, inline comment, reply, resolve, approve and request changes. Each row has the endpoint and a source, or "not supported".
- A comment with the Q34 marker, examined in the OneDev web UI. Screenshots stay on the prototype branch or its pull request.
- For each supported comment and reply: the read call that finds it by its marker before a retry (ADR 0003).
- For each supported review action, for example approve, request changes and resolve: the read call that returns the state before a retry. The state is of the pull request or of the thread. ADR 0003 says that a marker does not confirm a review action.
- A smoke test against the homelab instance that does each supported action.

Stop when:

- Each row has a source and a live result. An action with no API is recorded as not supported (Q35).
- The homelab instance is not on the latest stable version. Ask the owner to upgrade it (Q17).

### Q50: S6 updater pre-release channel

Type: research
Blocked by: Q40

Find out how the Tauri Updater can serve a pre-release channel next to the stable channel (for example a separate update endpoint). Include signature checks, restart and migration recovery.

Charter: the S6 charter in Q55. Q50 is its research phase.

### Q51: Single instance and CLI forwarding

Type: research
Blocked by: none

Find out how a second `diffy` process forwards its arguments to the running instance on Windows and Linux. Find out how `git difftool` gives temporary files to Diffy. Git can delete a temporary file when the helper process exits. Prove that Diffy has read each input before the helper exits.

### Q52: Owner setup actions

Type: task
Blocked by: none

List the actions that only the owner can do, with the latest date for each. Examples are the GitHub OAuth App, the minisign key, the Renovate app and the branch protection. The minisign key must exist before the v0.0.1 build, because v0.0.1 contains the public key (Q36).

### Q53: Domain glossary

Type: owner
Blocked by: none

Approve the `CONTEXT.md` terms from specification section 07: snapshot, comparison, review session, finding, file review state, publication, baseline and checkpoint.

### Q55: S6 signed update prototype

Type: prototype
Blocked by: Q50

Run signed updates on Linux and Windows with the design from Q50. Check the restart, the database migration and the recovery from each failure.

S6 charter. Q50 is the research phase and Q55 is the prototype phase.

Charter question: How does the Tauri Updater serve a pre-release channel next to the stable channel? Does a signed update install, restart and recover safely?

Research phase evidence (Q50): a source for each item, as a documentation link or a plugin source file and line:

- How the app selects the update endpoint of a channel at run time.
- How the updater checks the minisign signature, and what it does with an invalid or changed artifact.
- How an AppImage and an NSIS installation restart after an update.
- What happens after an interrupted download, a failed installation, a location that is not writable, and no network.
- A recovery path for a failed database migration. The specification does not promise automatic rollback.

Research phase stop when:

- Each item has an answer with a source, or is listed as unknown with the test that can answer it. Q55 runs these tests.

Prototype phase evidence (Q55), on the branch `prototype/diffy-v0.1-q55`:

- Two signed test releases of a minimal Tauri app, and an update from the older release to the newer release.
- Sign them with a throwaway test key, not with the Q9 release key.
- Linux: the AppImage on the Fedora host. Windows: the NSIS installer on a Windows CI runner. A manual run on the notebook happens only before a release (Q10).
- The updater rejects an artifact with an invalid signature, and an artifact that was changed after signing.
- The app restarts on the new version, and a test database migration runs at startup.
- Recovery from an interrupted download, a failed installation, a location that is not writable, no network and a failed migration. The user data stays.
- The pre-release channel and the stable channel each get only their own updates.

Prototype phase stop when:

- Each item has a pass or fail result on both platforms. Each fail has a recovery path, or goes to the owner.

### Q57: WebKitGTK input method preedit

Type: owner
Blocked by: none

wry 0.57.3 turns off the WebKitGTK preedit, so the input method shows the text that is not committed in its own window. Decide if Diffy turns the preedit on again, so that the text shows in the editor. Q42 showed that this works.

### Q58: Monaco features and font for M0

Type: task
Blocked by: none

List the Monaco contributions that the M0 scaffold imports, at least the find widget and the tab focus mode. Select a font list for the editor, because the Monaco default font is not installed on Fedora or Ubuntu.

### Q59: S1 checks on the Windows notebook

Type: task
Blocked by: none

Run the Q54 checks for option B once on the Windows notebook (Q12), with `prototypes/s1-monaco/q54-windows.ps1` from the tag `prototype/diffy-v0.1-q54`. A fail for option B goes to the owner. Issue #30.

### Q60: Cause of the histogram parity fails

Type: research
Blocked by: Q43

Find the cause of the 5 Q43 fails of gix-imara-diff 0.3.4. Name the step where the crate and Git first select different lines. Find out if one change to the crate removes each fail and keeps the pairs that pass. Issue #32.

### Q61: Path to histogram parity

Type: owner
Blocked by: Q60

Decide how Diffy gets canonical hunks that are identical to Git (ADR 0002). Examples are a fix to gix-imara-diff, upstream or in a fork, a Diffy histogram implementation, and a documented gap. Issue #33.

## Not yet clear

- The rules that mark a file as generated (R-02).
- The fields of a comparison profile (C-04).
- Whether L-06 and L-07 (P1) are in v0.1.0 or later.
- The SQLite migration tool and the schema version rules.
- How Monaco works with Orca and NVDA.
- OneDev authentication details and multi-instance setup.

## Out of scope

- Windows Authenticode signing. Reason: Q6.
- Windows 10. Reason: Q18.
- OneDev versions older than the latest stable. Reason: Q17.
- Heuristic encoding detection. Reason: Q26.
- Reviewed state for each hunk. Reason: Q29.
- User-defined severity and category lists. Reason: Q31.
- A documentation website. Reason: Q38.
- A webhook receiver. Reason: specification section 06.

## Prototypes

### Q41: S1 Monaco prototype build

- Options: A: the Monaco DiffEditor, with its own presentation diff. B: two Monaco editors that show the canonical hunks from the ADR 0002 oracle command.
- Picked: none yet. Q41 is the build phase. The owner picks an eligible option in Q54.
- Branch: `prototype/diffy-v0.1-q41` at `627236e`. Tag: `prototype/diffy-v0.1-q41`. Not merged.

### Q42: S2 Linux WebKitGTK behavior

- Options: A and B from Q41, each under Wayland and XWayland, on Fedora and on Ubuntu 24.04.
- Result: each clipboard, keyboard, GPU and rendering check passes for both options. The only fail is the in-page preedit, which wry turns off. Turning it on again is the workaround.
- Picked: none. The owner picks the option in Q54.
- Branch: `prototype/diffy-v0.1-q42` at `37f9892`. Tag: `prototype/diffy-v0.1-q42`. Not merged.

### Q54: S1 renderer verdict

- Options: A and B from Q41, on the Fedora reference host (Q38, Q56) with WebKitGTK 2.54.2 under Wayland.
- Coordinates: option A fails on the 1 MB, 10 MB and 50 MB fixtures. Option B passes on each fixture, and with five finding widgets on the 1 MB and 10 MB fixtures.
- Open p95: option A 257 ms for 1 MB and 2,687 ms for 10 MB. Option B 102 ms and 422 ms, or 118 ms and 539 ms with the oracle on each open.
- Live refresh p95: option A 350 ms for 1 MB and 5,016 ms for 10 MB. Option B 58 ms and 401 ms. With the anchor, both options keep the scroll position and the selection. One exception: option A on 10 MB, with a line inserted above the viewport. Only 3 of 10 runs keep the selection.
- Scrolling: 60 FPS for both options on 1 MB and 10 MB, programmatic, and the sides stay aligned.
- 50 MB stress: option A opens in 25.7 s, with a peak of 1.63 GB and a longest stall of 1,254 ms. Option B opens in 1.8 s, with a peak of 1.11 GB and a longest stall of 390 ms.
- Picked: option B, because only option B passes the coordinate check (ADR 0004).
- Branch: `prototype/diffy-v0.1-q54` at `f87a6fb`. Tag: `prototype/diffy-v0.1-q54`. Not merged.

### Q43: Diff crate with histogram parity

- Options: imara-diff 0.2.0, gix-imara-diff 0.3.4 and similar 3.2.1, each with the histogram algorithm. The imara-diff crates run with no postprocessing, with no slider heuristic and with the indent heuristic.
- Result: the closest is gix-imara-diff 0.3.4 with the indent heuristic. It passes each S1 and edge pair. It passes 3,727 of 3,732 real pairs from git, tokio and ripgrep.
- Picked: none. Q61 decides the path to parity.
- Branch: `prototype/diffy-v0.1-q43` at `061a6a3`. Tag: `prototype/diffy-v0.1-q43`. Not merged.

## Hand-off

None yet.
