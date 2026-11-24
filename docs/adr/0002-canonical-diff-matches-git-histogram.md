# ADR 0002: Canonical hunks match git diff --histogram

Status: Proposed
Affects: docs/maps/diffy-v0.1.md

The owner decided this on 2026-11-24. The Status stays Proposed until M0 creates the comparison crate. That pull request changes the Affects line to the crate paths and sets the Status to Accepted.

## Context

The specification lists the canonical diff algorithm as an open decision (section 21). Findings, review checkpoints and remote publication use the canonical hunk coordinates (sections 07 and 12). A change to the algorithm after release changes the hunks of stored comparisons. Then stored anchors can map differently, so the decision is hard to reverse.

## Options considered

- Match `git diff --histogram`: Git is a test oracle for each fixture. Histogram output is usually readable for moved and repeated code.
- Match the default `git diff` (Myers): output is like plain `git diff`. Myers output is often less readable for repeated code.
- Best readability with no parity: select the algorithm by benchmark. There is no oracle, so the tests need more hand-written expected results.

## Decision

The canonical hunks for the same input and the same options are identical to the hunks of `git diff --histogram`. A comparison profile can offer other algorithms for display. Anchors always use the canonical hunks. See [the v0.1 map](../maps/diffy-v0.1.md), decision Q13.

## Consequences

- The comparison test suite compares Diffy hunks with `git diff --histogram` output for each fixture. Git is necessary only in the test environment, not at run time.
- The oracle uses a pinned Git version and explicit hunk options. It runs with no system or global Git configuration, because a setting such as `diff.context` changes the hunks. A repository setting such as `diff.interHunkContext` also changes the hunks. So the oracle runs outside each repository, or in a fixture repository with known local settings. Each hunk option is given on the command line. Map question Q43 records the version and the options.
- The selection of a diff crate (map question Q43) must show histogram parity on the fixtures. If no crate has parity, Diffy needs its own histogram implementation.
- A profile that selects a different display algorithm must not change stored anchors.
