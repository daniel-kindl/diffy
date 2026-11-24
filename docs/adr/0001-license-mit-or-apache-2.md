# ADR 0001: License Diffy as MIT OR Apache-2.0

Status: Accepted
Affects: **

## Context

Diffy is a public, open-source repository. The license sets the dependency allowlist from the first commit. The dependency license check (for example cargo-deny and a pnpm license check) reads this allowlist. A license change after outside contributions needs the consent of each contributor, so it is hard to reverse.

Key dependencies are Monaco (MIT), Tauri (MIT OR Apache-2.0), gix (MIT OR Apache-2.0) and git2 with libgit2 (GPL-2.0 with a linking exception).

## Options considered

- MIT OR Apache-2.0: the Rust ecosystem norm. Permissive. Apache-2.0 adds a patent grant.
- GPL-3.0-or-later: copyleft. Forks stay open. Each dependency must be compatible, and some organizations do not use GPL tools.
- MPL-2.0: file-level copyleft. Changed Diffy files stay open, and other code can be combined with them.

## Decision

Diffy uses the dual license MIT OR Apache-2.0. The owner selected this option on 2026-11-24. See [the v0.1 map](../maps/diffy-v0.1.md), decision Q5.

## Consequences

- The repository gets `LICENSE-MIT` and `LICENSE-APACHE` files in M0.
- The dependency license check allows only licenses that are compatible with this dual license. A new license in the allowlist needs review.
- A dependency with a strong copyleft license (GPL without a linking exception, AGPL) is not allowed.
