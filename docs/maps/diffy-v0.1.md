# Map: Diffy v0.1

Status: open
Destination: Each v0.1 milestone (M0 to M4) has a plan and issues with acceptance criteria. Each spike has a verdict, and no open question blocks the first M0 issue.
Next: Q40

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
- Q11 Dogfood releases: unsigned Linux AppImage pre-releases (v0.0.x) after M1 and after M2. The full release gate applies only to v0.1.0. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
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
- Q33 Publication unit: one batched review with an explicit review event. Decided by: Daniel Kindl, 2026-11-24. Record: [ADR 0003](../adr/0003-remote-publication-format.md).
- Q34 Publication marker: a hidden HTML comment marker with the finding ID and the attempt ID. Decided by: Daniel Kindl, 2026-11-24. Record: [ADR 0003](../adr/0003-remote-publication-format.md).
- Q35 Remote threads: Diffy can reply to and resolve threads where the provider allows it. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q36 Dogfood updates: the v0.0.x AppImages use the updater from v0.0.2, on a pre-release channel. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q37 Screen readers: Orca on Fedora during development, axe-core checks in CI, and NVDA on Windows at release candidate. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q38 Documentation and performance reference: README and Markdown in `docs/`. The Fedora host is the reference hardware for the performance targets. CI benchmarks show relative regressions only. Decided by: Daniel Kindl, 2026-11-24. Record: this line.
- Q39 Specification files: leave the specification and the logo untracked for now. Decided by: Daniel Kindl, 2026-11-24. Record: this line.

## Open questions

### Q40: Spike charters

Type: task
Blocked by: none

Write one charter for each spike (S1 to S8): the question, the evidence that answers it, and the stop conditions. The owner approves each charter before the spike starts.

### Q41: S1 Monaco renderer shape

Type: prototype
Blocked by: Q40

Compare the Monaco DiffEditor with two Monaco editors that show the canonical hunks from Rust. Measure large diffs, aligned rows, inline findings, scrolling and live refresh. Include the Windows check from Q12.

### Q42: S2 Linux WebKitGTK behavior

Type: prototype
Blocked by: Q40

Check the S1 prototype on WebKitGTK under Wayland and X11: clipboard, keyboard, GPU and rendering.

### Q43: Diff crate with histogram parity

Type: research
Blocked by: Q40

Find a Rust diff crate (for example similar or imara-diff) whose output is identical to `git diff --histogram` on the fixtures. See ADR 0002.

### Q44: S3 Git backend

Type: prototype
Blocked by: Q40

Compare gix and git2 on the Git fixtures: index and worktree, renames, submodules, unusual paths and detached HEAD. Prove read-only behavior and a build that needs no installed Git.

### Q45: S5 anchor mapping

Type: prototype
Blocked by: Q43

Prove conservative re-anchoring across revisions. An ambiguous mapping must become outdated and never publish to a wrong line.

### Q46: S8 scale

Type: prototype
Blocked by: Q41, Q44

Measure 1,000 or more changed files with bounded memory and cancellation against the section 18 targets.

### Q47: S7 CodeMirror fallback

Type: prototype
Blocked by: Q41

Run the CodeMirror 6 benchmark only if the Q41 verdict says that Monaco fails.

### Q48: GitHub thread resolution API

Type: research
Blocked by: none

Find out if GitHub can resolve a review thread through REST, or only through GraphQL (for example `resolveReviewThread`). The answer decides if M3a needs a GraphQL client.

### Q49: S4 OneDev capabilities

Type: research
Blocked by: Q40

Find out which review actions the latest stable OneDev REST API supports. Check inline comments, replies, resolution and approval. Check that the Q34 marker is invisible in OneDev.

### Q50: S6 updater pre-release channel

Type: research
Blocked by: Q40

Find out how the Tauri Updater can serve a pre-release channel next to the stable channel (for example a separate update endpoint). Include signature checks, restart and migration recovery.

### Q51: Single instance and CLI forwarding

Type: research
Blocked by: none

Find out how a second `diffy` process forwards its arguments to the running instance on Windows and Linux. Find out how `git difftool` gives temporary files to Diffy.

### Q52: Owner setup actions

Type: task
Blocked by: none

List the actions that only the owner can do, with the latest date for each. Examples are the GitHub OAuth App, the minisign key, the Renovate app, the labels and the branch protection.

### Q53: Domain glossary

Type: owner
Blocked by: none

Approve the `CONTEXT.md` terms from specification section 07: snapshot, comparison, review session, finding, file review state, publication, baseline and checkpoint.

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
