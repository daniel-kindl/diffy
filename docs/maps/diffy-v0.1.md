# Map: Diffy v0.1

Status: open
Destination: Each v0.1 milestone (M0 to M4) has a plan and issues with acceptance criteria. Each spike has a verdict, and no open question blocks the first M0 issue.
Next: Q41

## Notes

- The specification is `Diffy_Master_Specification.md` (version 0.1 planning baseline, 2026-11-24). It is not tracked in Git yet. The owner decided to leave it untracked for now. Source: Q39 in this map.
- The specification makes DKSTACK the authoritative engineering guidance. Source: Diffy_Master_Specification.md section 19.
- DKSTACK requires one issue, one branch and one pull request, squash merge only, and a reviewer that is not the author. Source: https://raw.githubusercontent.com/daniel-kindl/dkstack/v1/workflow/review-and-merge.md
- DKSTACK requires that prototype branches are tagged and never merged. Source: https://raw.githubusercontent.com/daniel-kindl/dkstack/v1/skills/chart-route/SKILL.md
- DKSTACK allows a release only when the owner starts it, and a release workflow starts only by `workflow_dispatch`. Source: https://raw.githubusercontent.com/daniel-kindl/dkstack/v1/workflow/safety.md
- The repository has the 10 DKSTACK labels (six `type:` and four `priority:` labels) and no other labels. Source: `gh label list` on 2026-11-24.
- The specification links the logo as `diffy-assets/media/image1.png`, but the file is `image1.png` at the repository root. Source: Diffy_Master_Specification.md line 3.
- Some decisions change the specification. The changed sections are signing (Q6), M3 scope (Q16), OneDev versions (Q17) and the Windows minimum (Q18). Q24 adds requirement C-09 for the CLI. Q28 adds requirement R-08 for the Markdown copy. Source: the Decisions section of this map.

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
- Q40 Spike charters: each spike question has a charter in its entry, with the question, the evidence and the stop conditions. The spikes are Q41, Q42, Q44 to Q47, Q49, Q50 and Q54. Owner words: "Approve all as written". Decided by: Daniel Kindl, 2026-11-24. Record: this line.

## Open questions

### Q41: S1 Monaco prototype build

Type: task
Blocked by: Q40

Build a throwaway prototype with two renderer options: the Monaco DiffEditor, and two Monaco editors that show the canonical hunks from Rust. The task is done when both options run on Fedora. It gives no verdict. Q54 gives the verdict.

Charter question: Do both renderer options run on Fedora with the same fixtures?

Evidence:

- A Tauri prototype on the branch `prototype/diffy-v0.1-q41`. Option A is the Monaco DiffEditor. Option B is two Monaco editors that show the canonical hunks from Rust.
- Until Q43 selects a crate, option B can take its hunks from the ADR 0002 oracle command.
- A fixture set that Q54, Q47 and Q46 use again. It has a small change, a 1 MB diff and a 10 MB diff. It also has a 50 MB stress input and 10,000 changed lines.
- A benchmark harness that records the hardware, the OS, the workload and the method, as specification section 18 says.

Stop when:

- Both options open the small fixture and the 1 MB fixture on Fedora. Tag the commit and continue with Q42 and Q54.
- An option cannot open these fixtures. Record the cause and ask the owner before more work on that option.

### Q42: S2 Linux WebKitGTK behavior

Type: prototype
Blocked by: Q41

Check the Q41 prototype on WebKitGTK under Wayland and X11: clipboard, keyboard, GPU and rendering. The Q54 verdict uses these results.

Charter question: Does the Q41 prototype work on WebKitGTK under Wayland and under X11?

Evidence, for each option and each session type:

- The WebKitGTK version and the session type. For X11, record if it is an X11 session or XWayland with `GDK_BACKEND=x11`.
- Clipboard: copy and paste text in both directions between the prototype and a different application.
- Keyboard: the editor shortcuts, the Tab focus order and an input method.
- GPU: the compositing mode, and each environment variable that the prototype needs, for example `WEBKIT_DISABLE_DMABUF_RENDERER`.
- Rendering: fonts, fractional scaling, and no blank or flickering area when you scroll the 10 MB fixture.
- The same checks on Ubuntu LTS, for example in a distrobox container on the host. Specification section 17 names Ubuntu.

Stop when:

- Each check has a pass or fail result with the versions. Q54 uses the results.
- A check fails and has no workaround. Record it as a blocker for that option and tell the owner.

### Q43: Diff crate with histogram parity

Type: research
Blocked by: Q40

Find a Rust diff crate (for example similar or imara-diff) whose output is identical to `git diff --histogram` on the fixtures. Pin the reference Git version and the hunk options, for example the context lines. Run the oracle command with no system or global Git configuration, for example with `GIT_CONFIG_NOSYSTEM=1` and `GIT_CONFIG_GLOBAL=/dev/null`. Repository settings such as `diff.interHunkContext` also change the hunks. So run `git diff --no-index` in a temporary folder outside each repository, or in a fixture repository with known local settings. Give each hunk option on the command line, for example `-U3` and `--inter-hunk-context=0`. See ADR 0002.

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

- Each backend has a result for each fixture. The owner picks one backend.
- No backend passes all fixtures. List the gaps of each backend and ask the owner.

### Q45: S5 anchor mapping

Type: prototype
Blocked by: Q43

Prove conservative re-anchoring across revisions. An ambiguous mapping must become outdated and never publish to a wrong line.

Charter question: Which method moves a finding to the correct line of a new revision, and marks it outdated when the mapping is not certain?

Evidence, on the branch `prototype/diffy-v0.1-q45`:

- Two or three methods. Examples are a map through the Q43 hunks only, and that map plus an exact match of the line and its context.
- Fixture revision pairs, each with an expected result. The cases are: line unchanged, lines added above, line edited, line deleted and duplicate lines. They also include file renamed, file deleted, whitespace changes and line-ending changes.
- A property test with random edits. A finding that maps to a line must point to the same line text and context. If not, the finding must be outdated.
- For each method: the number of wrong mappings and the number of outdated findings.

Stop when:

- A method has zero wrong mappings on all fixtures and property cases. The owner picks from the methods that have zero.
- No method has zero wrong mappings. Ask the owner before more work.

### Q46: S8 scale

Type: prototype
Blocked by: Q54, Q44, Q47

Measure 1,000 or more changed files with bounded memory and cancellation against the section 18 targets. Measure only the renderer that Q54, or Q47 when Monaco fails, selects.

Charter question: Does Diffy stay responsive with 1,000 or more changed files, with bounded memory and fast cancellation?

Evidence, on the Fedora reference host (Q38) with the selected renderer:

- Generated fixture repositories with 1,000 and with 5,000 changed files, and one with 10,000 changed lines.
- The p95 time to show the change list of 1,000 files. The target is less than 3 s.
- The p95 time from a cancel action to its acknowledgement. The target is less than 250 ms.
- The peak memory of the Rust process and of the WebView process for each fixture. Memory is bounded when it does not grow with the content of files that are not open.
- The CPU use and the stalls, with the method that specification section 18 asks for.

Stop when:

- Each target has a measured result. A missed target is a finding for the owner. It is not an automatic fail, because the section 18 targets are proposals.

### Q47: S7 CodeMirror fallback

Type: prototype
Blocked by: Q54

Run the CodeMirror 6 benchmark only if the Q54 verdict says that Monaco fails. If Monaco passes, record Q47 in Decisions as skipped, with the Q54 verdict as the reason.

Charter question: If Monaco fails, does CodeMirror 6 meet the targets that Monaco did not meet?

Evidence, only if the Q54 verdict says that Monaco fails:

- A CodeMirror 6 merge view that shows the canonical hunks, on the branch `prototype/diffy-v0.1-q47`.
- The Q41 fixtures and harness, and the same measurements as Q54.

Stop when:

- Q54 says that Monaco passes. Record Q47 as skipped.
- Each Q54 measurement has a CodeMirror result. The owner picks the renderer.

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
- For each supported write: the read call that finds the write by its marker before a retry (ADR 0003).
- A smoke test against the homelab instance that does each supported action.

Stop when:

- Each row has a source and a live result. An action with no API is recorded as not supported (Q35).
- The homelab instance is not on the latest stable version. Ask the owner to upgrade it (Q17).

### Q50: S6 updater pre-release channel

Type: research
Blocked by: Q40

Find out how the Tauri Updater can serve a pre-release channel next to the stable channel (for example a separate update endpoint). Include signature checks, restart and migration recovery.

Charter question: How does the Tauri Updater serve a pre-release channel next to the stable channel, and how does it fail safely?

Evidence: a source for each item, as a documentation link or a plugin source file and line:

- How the app selects the update endpoint of a channel at run time.
- How the updater checks the minisign signature, and what it does with an invalid or changed artifact.
- How an AppImage and an NSIS installation restart after an update.
- What happens after an interrupted download, a failed installation, a location that is not writable, and no network.
- A recovery path for a failed database migration. The specification does not promise automatic rollback.

Stop when:

- Each item has an answer with a source, or is listed as unknown with the test that can answer it. Each unknown becomes a new question in this map.

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

### Q54: S1 renderer verdict

Type: prototype
Blocked by: Q41, Q42

Select the renderer option from the Q41 prototype. Measure large diffs, aligned rows, inline findings, scrolling and live refresh. Use the Q42 results and the Windows check from Q12. The owner picks the option.

Charter question: Which renderer option does Diffy use?

Evidence, for each Q41 option on the Fedora reference host (Q38):

- Aligned rows: the rows on screen agree with the canonical hunks on each fixture. Option A calculates its own diff, so record each difference.
- Inline findings: a finding widget in a row does not break the alignment.
- Scrolling: 60 FPS normal scrolling, and the two sides stay aligned.
- Live refresh: a file change keeps the scroll position and the selection. The typical settled refresh takes less than 500 ms.
- Large diffs: the p95 time to show the 1 MB diff and the 10 MB diff. The targets are less than 250 ms and less than 3 s.
- The memory, the CPU use and the stalls for the 50 MB stress input.
- The Q42 results, and one run on the Windows notebook (Q12).

Stop when:

- Each item has a result for each option. The owner picks an option, or says that Monaco fails. A Monaco fail starts Q47.

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

None yet.

## Hand-off

None yet.
