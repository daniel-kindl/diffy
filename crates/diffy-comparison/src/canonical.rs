//! The canonical diff. Its hunks match `git diff --histogram` with the indent
//! heuristic (ADR 0002). It uses the Diffy copy of gix-imara-diff.

use gix_imara_diff::{Algorithm, Diff, InternedInput};
use std::ops::Range;

/// One canonical hunk. The ranges are 0-based line indexes. A pure insertion
/// has an empty `base` range, and a pure deletion has an empty `head` range.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hunk {
    pub base: Range<u32>,
    pub head: Range<u32>,
}

/// Compute the canonical hunks of two raw inputs. Lines end at `\n`, and a
/// line keeps its terminator, so a missing final newline is a change.
pub fn canonical_hunks(base: &[u8], head: &[u8]) -> Vec<Hunk> {
    let input = InternedInput::new(base, head);
    let mut diff = Diff::compute(Algorithm::Histogram, &input);
    diff.postprocess_lines(&input);
    diff.hunks()
        .map(|h| Hunk {
            base: h.before,
            head: h.after,
        })
        .collect()
}
