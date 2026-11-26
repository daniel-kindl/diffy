# Diffy

Diffy is a desktop app to compare files, folders and Git changes. A person reviews the changes, records findings and can publish them to a pull request.

## Language

### Comparison

**Snapshot**:
A fixed copy of one input at one moment, with its source and its content identity. A comparison reads snapshots, not live files.
_Avoid_: Capture, copy

**Comparison**:
The differences between a base snapshot and a head snapshot, under one comparison profile. It has the changed files and their canonical hunks.
_Avoid_: Compare, diff result

**Canonical diff**:
The diff that gives the canonical hunks. For the same input and the same options, its hunks match `git diff --histogram`. Anchors, checkpoints and publication use it. A view can show another diff.
_Avoid_: Real diff, Git diff

**Comparison profile**:
A named, reusable set of comparison options, for example the whitespace and line-ending options. A filter in a profile hides changes, but it never removes them.
_Avoid_: Preset, settings

### Review

**Review session**:
The local review of one repository or one pull request against one baseline. It holds the findings, the file review states and the checkpoints.
_Avoid_: Review job, session

**Baseline**:
The revision that a review session compares against. Examples are HEAD, a commit, a branch, the pull request base and the last reviewed snapshot.
_Avoid_: Reference point, target

**Checkpoint**:
A recorded point in a review session at which the person marks what is reviewed. A later re-review starts from the last checkpoint.
_Avoid_: Save point, milestone

**File review state**:
Whether a person has reviewed one file at its current content, as of a checkpoint. A change of the content makes the file need re-review.
_Avoid_: Viewed flag, seen

**Finding**:
A note of the reviewer on a range of lines in one file of one revision, with a severity, a category and text. It keeps its original position.
_Avoid_: Annotation, issue, remark

**Anchor**:
The range of lines that a finding points to, in one file of one revision, in canonical hunk coordinates. The anchor mapping moves it to a later revision or makes the finding outdated.
_Avoid_: Location, pin

**Outdated finding**:
A finding whose anchor Diffy cannot map to the current revision without ambiguity. It keeps its original anchor. The person must confirm a new one.
_Avoid_: Stale finding, orphaned finding

### Publication

**Publication**:
One attempt to send findings of a review session to a pull request on a provider, and the reconciled result of that attempt.
_Avoid_: Push, sync, upload

**Provider**:
A code hosting service with pull requests, where Diffy reads and publishes reviews. v0.1 has GitHub and OneDev. Capabilities differ by provider, server version and permission.
_Avoid_: Forge, host, remote
