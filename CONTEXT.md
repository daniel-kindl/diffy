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

### Publication

**Publication**:
One attempt to send findings of a review session to a pull request on a provider, and the reconciled result of that attempt.
_Avoid_: Push, sync, upload
