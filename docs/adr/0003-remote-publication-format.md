# ADR 0003: Publish findings as one review with hidden markers

Status: Proposed
Affects: docs/maps/diffy-v0.1.md

The owner decided this on 2026-11-24. The Status stays Proposed until M3a creates the provider crate. That pull request changes the Affects line to the provider paths and sets the Status to Accepted.

## Context

The publication lifecycle (specification section 07) requires reconciliation before a retry. A timeout is not proof of failure, and a retry must not make duplicate comments. Diffy must find its own published comments in the remote data. Comments that are already published stay on the provider after a format change, so the format is hard to reverse.

## Options considered

- Publish as one batched review, or publish each finding as a separate comment.
- Find own comments by a hidden HTML comment marker, by content match (path, line, body and author), or by a visible footer.

## Decision

- Diffy publishes the selected findings as the inline comments of one review. The user selects the review event explicitly: COMMENT, APPROVE or REQUEST_CHANGES.
- Each published comment body ends with a hidden marker in this format: `<!-- diffy:f=<finding-id> a=<attempt-id> -->`.
- Reconciliation finds the comments by the marker before a retry.

See [the v0.1 map](../maps/diffy-v0.1.md), decisions Q33 and Q34.

## Consequences

- One review gives followers of the pull request one notification. A partial failure of a review must be reconciled as one unit.
- The marker contains only local IDs. It must not contain paths outside the repository, tokens or personal data.
- OneDev must render the marker invisibly. Map question Q47 verifies this. If OneDev shows the marker, the OneDev adapter needs a different marker, and this ADR gets an amendment.
- A user can edit or delete a marker on the provider. Reconciliation must handle a missing marker as an unknown result, not as a failure.
