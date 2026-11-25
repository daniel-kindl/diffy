# Map: Diffy v0.1

Status: open
Destination: Each v0.1 milestone (M0 to M4) has a plan and issues with acceptance criteria. Each spike has a verdict, and no open question blocks the first M0 issue.
Next: Q73

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
- After a match, gix-imara-diff 0.3.4 skips the next left occurrences of the line up to `end2`, the end on the right side. Git skips up to `ae`, the end on the left side. Source: gix-imara-diff 0.3.4 `src/histogram/lcs.rs:135` and git 2.55.0 `xdiff/xhistogram.c:212`
- gix-imara-diff 0.3.4 uses a common region only when each line in it has at most 63 copies. Git accepts 64 copies. Source: gix-imara-diff 0.3.4 `src/histogram.rs:14` and `src/histogram/lcs.rs:60`, and git 2.55.0 `xdiff/xhistogram.c:289` and `xdiff/xhistogram.c:294`
- gix-imara-diff 0.3.4 strips the common prefix and suffix before the histogram diff. Git does not trim the ends for histogram. Source: gix-imara-diff 0.3.4 `src/lib.rs:276` and git 2.55.0 `xdiff/xprepare.c:461`
- For histogram, Git re-diffs a group with Myers when the group changed while it slid and the group on the other side is not empty. gix-imara-diff 0.3.4 has no such step. Source: git 2.55.0 `xdiff/xdiffi.c:921` and gix-imara-diff 0.3.4 `src/postprocess.rs`
- A patch to gix-imara-diff 0.3.4 changes these four steps. With the patch and `postprocess_lines`, the crate passes 5 of 5 S1, 20 of 20 edge and 3,732 of 3,732 real Q43 pairs. Without any one of the four changes, 1 to 3 real pairs fail. Source: https://github.com/daniel-kindl/diffy/blob/3c0ae71/prototypes/q60-parity-cause/README.md
- With the patch, the histogram diff and the indent heuristic take 180.6 ms on the `50mb-stress` fixture, against 165.9 ms without the patch. Source: https://github.com/daniel-kindl/diffy/blob/3c0ae71/prototypes/q60-parity-cause/README.md
- The published gix-imara-diff 0.3.4 crate has no `src/tests.rs`, so its tests do not compile from the crate. Source: gix-imara-diff 0.3.4 `src/lib.rs:167`
- The Q44 run compared gix 0.91.0 and git2 0.21.0 (libgit2 1.9.7) with Git 2.55.0 as the oracle, on 7 fixtures with 58 operations. gix gave the same answer as Git for 58 operations, and git2 for 57. Source: https://github.com/daniel-kindl/diffy/blob/e8867f4/prototypes/q44-git-backend/README.md
- libgit2 reports an intent-to-add file (`git add -N`) as `AM`. Git reports `.A`. Source: https://github.com/daniel-kindl/diffy/blob/e8867f4/prototypes/q44-git-backend/results/git2.json
- With its default trust, gix status runs a filter driver command (`filter.<name>.clean`) from the repository configuration. Git also runs it. libgit2 has no external filter drivers. Source: https://github.com/daniel-kindl/diffy/blob/e8867f4/prototypes/q44-git-backend/results/gix-filters-trust.json and https://github.com/daniel-kindl/diffy/blob/e8867f4/prototypes/q44-git-backend/results/control-git-status.json
- When gix refuses the repository configuration sections, its status creates and removes a symbolic link in the worktree root, to probe the file system. Source: https://github.com/daniel-kindl/diffy/blob/e8867f4/prototypes/q44-git-backend/results/gix-filters-section-filter.json
- gix with the filter sections removed in memory wrote nothing and ran no command, also on a copy with no write permission. Source: https://github.com/daniel-kindl/diffy/blob/e8867f4/prototypes/q44-git-backend/results/gix.json and https://github.com/daniel-kindl/diffy/blob/e8867f4/prototypes/q44-git-backend/results/gix-no-write.json
- In a bubblewrap container with no `git` executable, the build of both backends and each check started no `git` process. Source: https://github.com/daniel-kindl/diffy/blob/e8867f4/prototypes/q44-git-backend/results/traces.json
- The gix build compiles no C. The git2 build compiles libgit2 and links the system zlib. Both build for `x86_64-pc-windows-gnu`, but neither ran on Windows. Source: https://github.com/daniel-kindl/diffy/blob/e8867f4/prototypes/q44-git-backend/results/build.json
- In Q45, a line is its bytes with its line terminator. A change of whitespace or of the line ending is an edit. Source: https://github.com/daniel-kindl/diffy/blob/4401b5c/prototypes/q45-anchors/README.md
- The Q45 expected result is the LCS rule. An old line maps to new line `j` only when each longest common subsequence (LCS) of the two files matches the line to `j`. Otherwise the anchor is outdated. Source: https://github.com/daniel-kindl/diffy/blob/4401b5c/prototypes/q45-anchors/src/core.rs
- Q45 compares three methods. M1 uses the canonical diff only. M2 adds a unique window of 3 context lines. M3 adds the LCS rule between bracket lines. On the 17 fixtures, M1 passes 14, M2 passes 5 and M3 passes 17. Source: https://github.com/daniel-kindl/diffy/blob/4401b5c/prototypes/q45-anchors/results/fixtures.json
- In the Q45 property test, M1 has 463,746 wrong mappings and M2 has 47,257, in 19,999,115 anchors of 1,000,000 histories. M3 has 0. In 100,000 larger histories, M1 has 120,012, M2 has 11,359 and M3 has 0. Source: https://github.com/daniel-kindl/diffy/blob/4401b5c/prototypes/q45-anchors/results/property-small.json and https://github.com/daniel-kindl/diffy/blob/4401b5c/prototypes/q45-anchors/results/property-medium.json
- M3 is outdated for 0.24 % and 0.12 % of the property test anchors where the LCS rule expects a line. Source: https://github.com/daniel-kindl/diffy/blob/4401b5c/prototypes/q45-anchors/README.md
- On 3,714 of the Q43 real pairs, M3 maps 185 anchors in 46 pairs where the LCS rule gives outdated. Each is at the line where the canonical diff keeps it. An example is `loose.c` in git/git commit `e8bf3190858e`, where part of a function moves into a new function above it. Source: https://github.com/daniel-kindl/diffy/blob/4401b5c/prototypes/q45-anchors/results/real.json and https://github.com/daniel-kindl/diffy/blob/4401b5c/prototypes/q45-anchors/README.md
- The LCS rule is quadratic in the file length. M3 uses it only in the segment between two bracket lines, and a segment with more than 1,000,000 cells is outdated. Q45 did not measure the time on large files. Source: https://github.com/daniel-kindl/diffy/blob/4401b5c/prototypes/q45-anchors/src/core.rs

- The Q46 fixture repositories are commit to commit comparisons. Two have 1,000 changed files, of 6.2 MB and of 333 MB. Others have 2,000 changed files (668 MB) and 5,000 changed files. The last two have the 10,000 changed lines pair and the 50 MB pair. Source: https://github.com/daniel-kindl/diffy/blob/d569d85/prototypes/s1-monaco/bench-results/q46-fedora-repos.json
- In Q46, the change list p95 is 65 ms for 1,000 changed files and 241 ms for 5,000. With files of about 110 KB, it is 1,046 ms for 1,000 files and 2,169 ms for 2,000 files. Source: https://github.com/daniel-kindl/diffy/blob/d569d85/prototypes/s1-monaco/bench-results/q46-fedora-list-files-1000.json, https://github.com/daniel-kindl/diffy/blob/d569d85/prototypes/s1-monaco/bench-results/q46-fedora-list-files-5000.json and https://github.com/daniel-kindl/diffy/blob/d569d85/prototypes/s1-monaco/bench-results/q46-fedora-list-files-2000-large.json
- gix rename detection is most of the change list time on the large fixtures. With no rename detection, the p95 is 32 ms for 1,000 and 33 ms for 2,000 large files. Source: https://github.com/daniel-kindl/diffy/blob/d569d85/prototypes/s1-monaco/bench-results/q46-fedora-list-files-2000-large-no-renames.json
- In Q46, the line counts of each fixture are equal to `git diff --shortstat --histogram -M`. The counts run in 4 worker threads after the list shows. Source: https://github.com/daniel-kindl/diffy/blob/d569d85/prototypes/s1-monaco/README.md
- A cancel of the Q46 line counts on 5,000 files is acknowledged in 22 ms at p95. The workers stop within 52 ms. Source: https://github.com/daniel-kindl/diffy/blob/d569d85/prototypes/s1-monaco/bench-results/q46-fedora-cancel-stats.json
- A cancel of the 50 MB file open is acknowledged in 385 ms at p95 and 441 ms at most. A cancel during the Monaco model creation waits until the page is free, up to 364 ms. Source: https://github.com/daniel-kindl/diffy/blob/d569d85/prototypes/s1-monaco/bench-results/q46-fedora-cancel-open.json
- In Q46, the WebView RSS is 261 to 270 MB in the list part of each many-file fixture. The Rust working memory of the line counts does not grow from 1,000 to 2,000 large files. The Rust file-backed RSS grows with the pack data that gix reads, from 218 MB to 313 MB. Source: https://github.com/daniel-kindl/diffy/blob/d569d85/prototypes/s1-monaco/README.md
- While the Q46 list loads, counts or scrolls, no timer callback is more than 50 ms late. No frame gap is more than 50 ms. A file open stalls up to 145 ms for the first editor. It stalls up to 207 ms for 10,000 changed lines and 492 ms for 50 MB. Source: https://github.com/daniel-kindl/diffy/blob/d569d85/prototypes/s1-monaco/README.md
- The Q46 change list uses the gix feature `parallel`, so a repository can go to a worker thread. Q44 did not use it. Source: https://github.com/daniel-kindl/diffy/blob/d569d85/prototypes/s1-monaco/src-tauri/Cargo.toml
- gitoxide merged the Q62 pull request on 2026-11-25, at merge commit `2bcb31d`. Before the merge, the maintainer added three commits. The commits skip a seed that occurs more than 65 times and keep the Git tail trim for blame. They also count the lines after postprocessing and speed up long common tails. Source: https://github.com/GitoxideLabs/gitoxide/pull/3246
- On 2026-11-25 the newest gix-imara-diff on crates.io is 0.3.4, from 2026-11-22. No release has the fix yet. Source: https://crates.io/crates/gix-imara-diff
- The Q60 copy differs from Git on one input from the maintainer review. Each line is one token: `B` 4 times, `A A B A`, then `B` 61 times, against `B A B A`. The Q60 copy removes 66 lines and adds 1. Git 2.55.0 and gitoxide `2bcb31d` remove 65 lines and add none, with the same hunks. The Q43 pairs do not have this case. Source: Q62 run, 2026-11-25.
- The GitHub REST API has no endpoint that resolves a review thread. The endpoint that edits a review comment accepts only `body`. Source: https://github.com/github/rest-api-description/blob/c9c95fe34ced05fe076001f5c55704742bc8e7f8/descriptions/api.github.com/api.github.com.json#L90543 and https://docs.github.com/en/rest/pulls/comments
- A REST review comment has no thread ID and no resolved state. Its `node_id` is the GraphQL ID of the comment. Source: https://github.com/github/rest-api-description/blob/c9c95fe34ced05fe076001f5c55704742bc8e7f8/descriptions/api.github.com/api.github.com.json#L176114 and Q48 run on tokio-rs/tokio pull request 8781, 2026-11-25.
- The REST API can reply to a review comment, with `POST /repos/{owner}/{repo}/pulls/{pull_number}/comments/{comment_id}/replies`. Source: https://github.com/github/rest-api-description/blob/c9c95fe34ced05fe076001f5c55704742bc8e7f8/descriptions/api.github.com/api.github.com.json#L91466
- The GitHub GraphQL API resolves a review thread with `resolveReviewThread` and reopens it with `unresolveReviewThread`. Each takes the thread ID. Source: https://docs.github.com/en/graphql/reference/mutations#resolvereviewthread and https://docs.github.com/public/fpt/schema.docs.graphql lines 29511, 29848 and 60774, read on 2026-11-25.
- A GraphQL review thread has `isResolved` and `viewerCanResolve`. `pullRequest.reviewThreads` lists the threads with their comments, so a REST comment maps to its thread by its `node_id`. Source: https://docs.github.com/public/fpt/schema.docs.graphql lines 45960, 47759 and 47824, read on 2026-11-25, and Q48 run on tokio-rs/tokio pull request 8781.
- The Q14 homelab instance runs OneDev 17.2.4, the `1dev/server:17.2.4` image. On 2026-11-25 that image is also the `latest` tag. The agent upgraded the instance from 16.9.8 on 2026-11-25, with the owner's approval and after a backup of the data folder. Source: https://github.com/daniel-kindl/diffy/blob/4bcc767/prototypes/q49-onedev/README.md
- OneDev 17.2.4 has a REST endpoint for each review action: inline comment, reply, resolve and unresolve, pull request comment, approve and request changes. An access token authenticates with `Authorization: Bearer`. Each action passed on the homelab instance. Source: https://github.com/daniel-kindl/diffy/blob/4bcc767/prototypes/q49-onedev/results/homelab.json and https://github.com/theonedev/onedev/blob/v17.2.4/server-core/src/main/java/io/onedev/server/rest/resource/CodeCommentResource.java#L155
- OneDev does not show the Q34 marker. The visible text of the activity page, the changes page and the open code comment has no marker. Source: https://github.com/daniel-kindl/diffy/blob/4bcc767/prototypes/q49-onedev/results/ui-check.json
- Only a reviewer of a pull request can approve it or request changes. The submitter gets the error 406, and the submitter cannot be added as a reviewer. Source: https://github.com/theonedev/onedev/blob/v17.2.4/server-core/src/main/java/io/onedev/server/rest/resource/PullRequestResource.java#L382 and https://github.com/daniel-kindl/diffy/blob/4bcc767/prototypes/q49-onedev/results/homelab.json
- OneDev accepts an inline comment only inside a OneDev diff hunk, context rows included. Each comment has the whitespace option `IGNORE_TRAILING`, so a OneDev hunk can differ from a canonical hunk. Source: https://github.com/theonedev/onedev/blob/v17.2.4/server-core/src/main/java/io/onedev/server/rest/resource/CodeCommentResource.java#L237 and https://github.com/daniel-kindl/diffy/blob/4bcc767/prototypes/q49-onedev/results/homelab.json
- A repeated approve adds a second entry to the pull request activity. A repeated resolve changes nothing. Source: https://github.com/daniel-kindl/diffy/blob/4bcc767/prototypes/q49-onedev/results/homelab.json
- Each resolve or unresolve adds a reply. With an empty request body, the reply is empty. A note that ends with the marker adds that note, so the marker confirms it. Source: https://github.com/daniel-kindl/diffy/blob/4bcc767/prototypes/q49-onedev/README.md and https://github.com/theonedev/onedev/blob/v17.2.4/server-core/src/main/java/io/onedev/server/service/impl/DefaultCodeCommentStatusChangeService.java#L50
- On 2026-11-25 the newest stable releases are the updater plugin 2.14.1, Tauri 2.12.7, the Tauri CLI 2.12.6 and the Tauri bundler 2.10.6. The Q50 sources are these releases. Source: https://github.com/tauri-apps/plugins-workspace/releases/tag/updater-v2.14.1 and https://github.com/tauri-apps/tauri/releases/tag/tauri-v2.12.7
- Rust code sets the update endpoints at run time with `updater_builder().endpoints()`. The Tauri documentation uses this for separate release channels. The frontend `check` command takes headers, a timeout, a proxy and a target, but no endpoints. Source: https://github.com/tauri-apps/plugins-workspace/blob/128190689dbe59b59d344e01bf07b00862aa3791/plugins/updater/src/updater.rs#L277, https://github.com/tauri-apps/tauri-docs/blob/d930b9e2a538d6326de3d92d60604cba69218096/src/content/docs/plugin/updater.mdx#L615-L636 and https://github.com/tauri-apps/plugins-workspace/blob/128190689dbe59b59d344e01bf07b00862aa3791/plugins/updater/src/commands.rs#L42-L49
- The updater tries the endpoints in order. When each endpoint fails, `check` returns the last request error. With no network, the check gets an error and changes nothing. Source: https://github.com/tauri-apps/plugins-workspace/blob/128190689dbe59b59d344e01bf07b00862aa3791/plugins/updater/src/updater.rs#L644-L661
- The GitHub latest release is the newest release that is not a pre-release and not a draft. The `/releases/latest/download/<asset>` URL uses that release. So the pre-release channel needs its own manifest URL. Source: https://docs.github.com/en/rest/releases/releases#get-the-latest-release and https://docs.github.com/en/repositories/releasing-projects-on-github/linking-to-releases
- The updater checks the minisign signature on the downloaded bytes before it returns them. `install` checks the bytes again before it writes a file. An invalid signature or a changed artifact gives an error, and the installed app stays as it was. Source: https://github.com/tauri-apps/plugins-workspace/blob/128190689dbe59b59d344e01bf07b00862aa3791/plugins/updater/src/updater.rs#L891, https://github.com/tauri-apps/plugins-workspace/blob/128190689dbe59b59d344e01bf07b00862aa3791/plugins/updater/src/updater.rs#L921-L925 and https://github.com/tauri-apps/plugins-workspace/blob/128190689dbe59b59d344e01bf07b00862aa3791/plugins/updater/src/updater.rs#L1893-L1916
- The update manifest is not signed. The option `requireSignedVersion` rejects an update when the version in the signature differs from the version in the manifest. It also rejects a signature with no version. The default is off. `tauri build` writes the version into each signature. Source: https://github.com/tauri-apps/plugins-workspace/blob/128190689dbe59b59d344e01bf07b00862aa3791/plugins/updater/src/config.rs#L135, https://github.com/tauri-apps/plugins-workspace/blob/128190689dbe59b59d344e01bf07b00862aa3791/plugins/updater/src/updater.rs#L1925-L1960 and https://github.com/tauri-apps/tauri/blob/9319e539f34237035cb4693bed9246660bc0a926/crates/tauri-cli/src/bundle.rs#L312
- On an AppImage, the updater moves the running AppImage to a backup and writes the new file at the `APPIMAGE` path. If the write fails, it moves the backup back. If the AppImage folder is not writable, the first move fails and nothing changes. Source: https://github.com/tauri-apps/plugins-workspace/blob/128190689dbe59b59d344e01bf07b00862aa3791/plugins/updater/src/lib.rs#L115-L116 and https://github.com/tauri-apps/plugins-workspace/blob/128190689dbe59b59d344e01bf07b00862aa3791/plugins/updater/src/updater.rs#L1221-L1316
- On Linux, the app must restart itself after the install. `app.restart()` starts the `APPIMAGE` path with the same arguments and exits. Before it exits, the single instance plugin releases its lock. Source: https://github.com/tauri-apps/tauri-docs/blob/d930b9e2a538d6326de3d92d60604cba69218096/src/content/docs/plugin/updater.mdx#L572-L573, https://github.com/tauri-apps/tauri/blob/9319e539f34237035cb4693bed9246660bc0a926/crates/tauri/src/process.rs#L48-L88 and https://github.com/tauri-apps/plugins-workspace/blob/128190689dbe59b59d344e01bf07b00862aa3791/plugins/single-instance/src/lib.rs#L55-L65
- On Windows, the updater starts the NSIS installer with `/UPDATE`, and the app exits. In the default passive mode, it also passes `/R` and the app arguments. The installer starts the app again only after a successful install. Source: https://github.com/tauri-apps/plugins-workspace/blob/128190689dbe59b59d344e01bf07b00862aa3791/plugins/updater/src/updater.rs#L1009-L1080 and https://github.com/tauri-apps/tauri/blob/9319e539f34237035cb4693bed9246660bc0a926/crates/tauri-bundler/src/bundle/windows/nsis/installer.nsi#L788-L799
- The NSIS installer copies the new files over the installed files and keeps no backup. So a failed NSIS install can leave a partial installation. Q55 tests it. The default NSIS mode installs for the current user, in a folder that needs no administrator. Source: https://github.com/tauri-apps/tauri/blob/9319e539f34237035cb4693bed9246660bc0a926/crates/tauri-bundler/src/bundle/windows/nsis/installer.nsi#L690-L700 and https://github.com/tauri-apps/tauri/blob/9319e539f34237035cb4693bed9246660bc0a926/crates/tauri-utils/src/config.rs#L826-L832
- The updater keeps the download in memory. An interrupted download returns an error and writes no file. The download has no resume. The default timeout is none. A reqwest timeout covers the whole request, including the body. Source: https://github.com/tauri-apps/plugins-workspace/blob/128190689dbe59b59d344e01bf07b00862aa3791/plugins/updater/src/updater.rs#L881-L893, https://github.com/tauri-apps/plugins-workspace/blob/128190689dbe59b59d344e01bf07b00862aa3791/plugins/updater/src/updater.rs#L234 and https://docs.rs/reqwest/0.12.28/reqwest/struct.ClientBuilder.html#method.timeout
- SQLite `VACUUM INTO` writes a copy of a database to a new file. `PRAGMA user_version` stores a schema version in the database header. The SQLite procedure for a schema change runs in one transaction. A recovery path for a failed migration is: copy the database first, run the migration in a transaction and keep the copy after a failure. Source: https://sqlite.org/lang_vacuum.html#vacuuminto, https://sqlite.org/pragma.html#pragma_user_version and https://sqlite.org/lang_altertable.html#otheralter

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
- Q60 Cause of the histogram parity fails: gix-imara-diff 0.3.4 differs from Git 2.55.0 in four steps. They are the skip after a match, the limit of 63 copies, the strip of the common ends and no re-diff after a slide. A patch that changes the four steps passes each Q43 pair. The owner selected the option "Accept the Q60 result". Decided by: Daniel Kindl, 2026-11-25. Record: this line.
- Q61 Path to histogram parity: Diffy proposes the Q60 patch to gitoxide. Until a gitoxide release has the fix, Diffy keeps a copy of gix-imara-diff 0.3.4 with the Q60 patch and keeps its Apache-2.0 license notice. Diffy drops the copy when a gitoxide release has the fix and passes the parity tests. The proposal goes to Q62. The owner selected the option "Upstream + patched copy". Decided by: Daniel Kindl, 2026-11-25. Record: this line and ADR 0002.
- Q62 Upstream proposal: gitoxide merged the Q60 patch with three maintainer commits, in pull request https://github.com/GitoxideLabs/gitoxide/pull/3246. No gix-imara-diff release has it yet, so the Q61 copy stays. The Q60 copy differs from Git on a seed that occurs more than 65 times, and the merged code does not. What the copy holds until a release goes to Q68. The owner approved the pull request text before it was sent. Decided by: Daniel Kindl, 2026-11-25. Record: this line.
- Q44 S3 Git backend: gix. Diffy opens each repository isolated and removes the `filter` sections from the configuration in memory, so no command from the repository configuration runs. gix is the only backend that passes each fixture, the read-only proof and the build with no installed Git. The owner selected the option "gix". Decided by: Daniel Kindl, 2026-11-25. Record: [ADR 0005](../adr/0005-git-backend-gix.md).
- Q45 S5 anchor mapping: M3. A finding moves with the canonical diff only when the LCS rule gives the same new line. M3 applies the rule in the segment between the two nearest bracket lines. A bracket is a line that occurs once in each file and that the canonical diff keeps. The start and the end of the file are also brackets. Otherwise the finding is outdated. M3 is the only method with the expected result on each fixture and no wrong mapping in the property test. The owner selected the option "M3". Decided by: Daniel Kindl, 2026-11-25. Record: this line.
- Q46 S8 scale: each section 18 target has a measured result for option B on the Fedora reference host. The change list target is met on each fixture. The cancel target is met for the line counts and missed for the open of a 50 MB file. The follow-up questions are Q66 and Q67. The owner selected the option "Accept the Q46 result". Decided by: Daniel Kindl, 2026-11-25. Record: this line.
- Q48 GitHub thread resolution API: only the GraphQL API resolves a review thread, so M3a needs a GraphQL client. REST can post a reply, but it cannot resolve a thread or show the thread of a comment. Diffy finds the thread through `pullRequest.reviewThreads` and the comment `node_id`. How much of the GitHub client uses GraphQL goes to Q69. The owner selected the option "Accept the Q48 result". Decided by: Daniel Kindl, 2026-11-25. Record: this line.
- Q49 S4 OneDev capabilities: Diffy can do each review action through the OneDev 17.2.4 REST API. Each action has a read call that confirms it before a retry. Comments and replies are found by the marker. A resolve is confirmed by the `resolved` field, and an approval or a change request by the status of the user in the reviews. Diffy sends a note that ends with the marker on each resolve, because each status change adds a reply. OneDev does not show the marker, so ADR 0003 needs no amendment. The follow-up questions are Q70 and Q71. The owner selected the option "Accept the Q49 result". Decided by: Daniel Kindl, 2026-11-25. Record: this line.
- Q50 S6 updater pre-release channel: Diffy selects the channel in Rust, with a separate update endpoint for each channel. The updater checks the minisign signature before it writes a file. An AppImage update restores the old file on failure, and Diffy restarts the app itself. An NSIS update restarts the app only on success, and a failed NSIS install has no rollback. A failed migration keeps a copy of the database from before the migration. Q55 tests each item. The follow-up questions are Q72 and Q73. The owner selected the option "Accept the Q50 result". Decided by: Daniel Kindl, 2026-11-25. Record: this line.
- Q72 Manifest URL of the pre-release channel: the pre-release manifest is the `latest.json` asset of one GitHub pre-release with a fixed tag, for example `channel-prerelease`. The release workflow replaces the asset on each pre-release. The URL is `/releases/download/<fixed tag>/latest.json`. The fixed-tag release stays marked as a pre-release, so that it never becomes the latest release. The stable channel keeps `/releases/latest/download/latest.json`. The owner selected the option "Fixed-tag release". Decided by: Daniel Kindl, 2026-11-25. Record: this line.

## Open questions

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
Blocked by: Q73

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

### Q63: Files with a filter driver

Type: owner
Blocked by: none

Diffy runs no filter driver (ADR 0005), so a file with a `filter=` attribute compares with no filter. For Git LFS, that is the raw file against the pointer blob. Decide how Diffy shows such files, for example as a file with a limitation, or with a built-in LFS pointer check.

### Q64: Anchors after whitespace and line-ending changes

Type: owner
Blocked by: none

In Q45 each byte of a line counts, so a whitespace or line-ending change of the anchor line makes the finding outdated. A file that changes from LF to CRLF makes each finding in it outdated. Decide whether such a change keeps the anchor.

### Q65: Time of the anchor mapping on large files

Type: task
Blocked by: Q46

Measure the Q45 M3 mapping on the Q46 fixtures. Record the time, and the findings that are outdated because a segment has more than 1,000,000 cells.

### Q66: Cancel during the open of a large file

Type: owner
Blocked by: none

In Q46, a cancel during the Monaco model creation of the 50 MB file takes up to 441 ms. The target is 250 ms. Decide whether v0.1 accepts this for large files. The other way is a prototype that creates the models without a long block of the page.

### Q67: Rename detection on large change lists

Type: owner
Blocked by: none

In Q46, rename detection takes most of the change list time when the changed files are large. For 2,000 files of about 110 KB, the list takes 2,169 ms, against 33 ms without it. Decide how Diffy keeps the change list fast. Examples are a list first and the renames later, or a limit for rename detection.

### Q68: Code of the patched gix-imara-diff copy

Type: owner
Blocked by: none

Until a gitoxide release has the fix, Diffy keeps a copy of gix-imara-diff (Q61). The Q60 copy differs from Git on a seed that occurs more than 65 times. The code that gitoxide merged in `2bcb31d` matches Git on that input. Decide what the copy holds until the release. Examples are the Q60 patch with a fix for the seed limit, or the gix-imara-diff source of `2bcb31d`, which then needs the Q43 parity run.

### Q69: GraphQL in the GitHub client

Type: owner
Blocked by: none

Q48 shows that M3a needs GraphQL to resolve a thread. REST covers the batched review (ADR 0003) and the replies. Decide whether the GitHub client uses GraphQL only for the threads and REST for the rest, or GraphQL for each call.

### Q70: Findings outside a OneDev hunk

Type: owner
Blocked by: none

OneDev accepts an inline comment only inside a OneDev diff hunk, and OneDev ignores trailing whitespace by default (Q49). A comment on another line gets an error. So does a comment on a line that only the canonical diff changes. Decide what Diffy does with such a finding. Examples are a pull request comment with the path and the line, or a block before publication.

### Q71: Review actions when the user is not a reviewer

Type: owner
Blocked by: none

On OneDev, only a reviewer of the pull request can approve or request changes (Q49). The submitter never can. Decide what Diffy does when the user is not a reviewer. Examples are no review action in the dialog, or a pull request comment. Another example: Diffy adds the user as a reviewer first, if the user has the permission.

### Q73: Signed version check

Type: owner
Blocked by: none

The update manifest is not signed (Q50). With `requireSignedVersion`, the updater rejects an update when the signed version differs from the manifest version. Without it, a changed manifest can install an older signed release. `tauri build` writes the version into each signature, so each Diffy release can pass the check. Decide whether Diffy turns it on from v0.0.1.

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
- Picked: gix-imara-diff 0.3.4 with the Q60 patch (Q61).
- Branch: `prototype/diffy-v0.1-q43` at `061a6a3`. Tag: `prototype/diffy-v0.1-q43`. Not merged.

### Q60: Cause of the histogram parity fails

- Options: gix-imara-diff 0.3.4 with combinations of four changes. The changes are the skip after a match, the limit of 64 copies, no strip of the common ends and the re-diff after a slide.
- Result: only the build with all four changes passes each S1, edge and real pair.
- Picked: the build with all four changes (Q61).
- Branch: `prototype/diffy-v0.1-q60` at `3c0ae71`. Tag: `prototype/diffy-v0.1-q60`. Not merged.

### Q44: S3 Git backend

- Options: gix 0.91.0 and git2 0.21.0 (libgit2 1.9.7), on the section 19 fixtures and an untrusted configuration fixture. For gix, also three ways to keep filter drivers from running.
- Result: only gix with the filter sections removed in memory passes each fixture, the read-only proof and the build with no installed Git.
- Picked: gix, because it is the only backend that qualifies.
- Branch: `prototype/diffy-v0.1-q44` at `e8867f4`. Tag: `prototype/diffy-v0.1-q44`. Not merged.

### Q45: S5 anchor mapping

- Options: M1, the canonical diff only. M2, the canonical diff and an exact match of the line and 3 lines of context that occur once in each file. M3, the canonical diff and the LCS rule in the segment between bracket lines.
- Result: only M3 passes each fixture and has no wrong mapping in the property test.
- Picked: M3, because it is the only method that qualifies.
- Branch: `prototype/diffy-v0.1-q45` at `4401b5c`. Tag: `prototype/diffy-v0.1-q45`. Not merged.

### Q46: S8 scale

- Options: none. Q46 measures option B (ADR 0004) with the gix change list of ADR 0005 and the canonical diff of Q61.
- Result: the change list target is met on each fixture. The cancel target is met for the line counts and missed for the open of a 50 MB file.
- Picked: the owner accepted the result.
- Branch: `prototype/diffy-v0.1-q46` at `d569d85`. Tag: `prototype/diffy-v0.1-q46`. Not merged.

### Q49: S4 OneDev capabilities

- Options: none. Q49 checks each review action through the OneDev 17.2.4 REST API on the Q14 homelab instance.
- Result: each action passes and has a read call that confirms it. The marker is not visible.
- Picked: the owner accepted the result.
- Branch: `prototype/diffy-v0.1-q49` at `4bcc767`. Tag: `prototype/diffy-v0.1-q49`. Not merged.

## Hand-off

None yet.
