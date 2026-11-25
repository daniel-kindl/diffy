# ADR 0004: The diff view is two Monaco editors that show the canonical hunks

Status: Proposed
Affects: docs/maps/diffy-v0.1.md

The owner decided this on 2026-11-24. The Status stays Proposed until M0 creates the diff view. That pull request changes the Affects line to the frontend paths and sets the Status to Accepted.

## Context

Findings, checkpoints and remote publication use the canonical hunk coordinates (ADR 0002, specification section 12). The diff view must show each row and each finding at those coordinates. A presentation difference is allowed only if the canonical coordinates stay correct. The renderer is the base of the frontend, so a change after M0 is expensive.

Spike S1 (map questions Q41, Q42 and Q54) compared two options in a Tauri prototype on the Fedora reference host. The results are on the tag `prototype/diffy-v0.1-q54`.

## Options considered

- A: the Monaco DiffEditor. Monaco calculates its own presentation diff in a web worker and aligns the two sides. The alignment of the finding widgets comes from Monaco. On the 1 MB, 10 MB and 50 MB fixtures, Monaco put some deletions one line away from the canonical block. So the coordinate check failed. Monaco 0.58 has no public option to use a different diff. Live refresh of the 10 MB fixture took about 5 s.
- B: two Monaco editors. The canonical hunks come from Rust. Line decorations mark the changed lines. View zones pad the shorter side of each block and the finding widgets. Diffy code keeps the two sides aligned. The coordinate check passed on each fixture. The open, live refresh and stress times were lower than for option A.
- C: CodeMirror 6 (map question Q47). The S1 charter runs it only if Monaco fails. Option B passes, so it did not run.

## Decision

The diff view uses option B: two Monaco editors that show the canonical hunks from the Rust side. Monaco does not calculate a diff for the view. See [the v0.1 map](../maps/diffy-v0.1.md), decision Q54.

## Consequences

- Diffy code owns the row alignment: block padding, finding widgets, scroll sync and the horizontal scroll bar pad. Tests must prove the alignment for each of these. The Q54 coordinate check and row hit test are a model for these tests.
- When only one side shows a horizontal scroll bar, the other side needs a pad of the scroll bar height below its last line. The pad goes after each other view zone on that line.
- A live refresh must keep the first visible line in place. Monaco alone keeps the scroll position in pixels, so an insert above the viewport moves the content.
- The frontend needs no diff worker for the view. The canonical diff crate from map question Q43 gives the hunks.
- Features of the Monaco DiffEditor, for example the inline view and the moved-code display, are not available. Diffy must build each one that it needs.
- The one Windows run of map decision Q12 comes after this decision, in map question Q59. A fail for option B on Windows goes to the owner.
