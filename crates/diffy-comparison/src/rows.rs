//! Aligned rows for the side-by-side and unified views, with context collapse
//! (C-05). They come from the canonical hunks, so the views never disagree
//! with the anchors.

use crate::canonical::Hunk;

/// One side-by-side row. A line number is 0-based. `None` is a gap.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub base: Option<u32>,
    pub head: Option<u32>,
    pub changed: bool,
}

/// The side-by-side rows. Unchanged lines pair up. In a hunk, the removed and
/// the added lines pair up in order, and the longer side gets gaps opposite.
pub fn side_by_side(base_len: u32, head_len: u32, hunks: &[Hunk]) -> Vec<Row> {
    let mut rows = Vec::new();
    let (mut b, mut h) = (0u32, 0u32);
    let same = |rows: &mut Vec<Row>, b: &mut u32, h: &mut u32, to_b: u32| {
        while *b < to_b {
            rows.push(Row {
                base: Some(*b),
                head: Some(*h),
                changed: false,
            });
            *b += 1;
            *h += 1;
        }
    };
    for hunk in hunks {
        same(&mut rows, &mut b, &mut h, hunk.base.start);
        let (nb, nh) = (
            hunk.base.end - hunk.base.start,
            hunk.head.end - hunk.head.start,
        );
        for i in 0..nb.max(nh) {
            rows.push(Row {
                base: (i < nb).then_some(hunk.base.start + i),
                head: (i < nh).then_some(hunk.head.start + i),
                changed: true,
            });
        }
        b = hunk.base.end;
        h = hunk.head.end;
    }
    same(&mut rows, &mut b, &mut h, base_len);
    debug_assert_eq!(h, head_len, "the hunks must account for each head line");
    rows
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnifiedLine {
    Context { base: u32, head: u32 },
    Removed { base: u32 },
    Added { head: u32 },
}

/// The unified lines: in a hunk, the removed lines come before the added lines.
pub fn unified(base_len: u32, head_len: u32, hunks: &[Hunk]) -> Vec<UnifiedLine> {
    let mut out = Vec::new();
    let (mut b, mut h) = (0u32, 0u32);
    for hunk in hunks {
        while b < hunk.base.start {
            out.push(UnifiedLine::Context { base: b, head: h });
            b += 1;
            h += 1;
        }
        out.extend(hunk.base.clone().map(|base| UnifiedLine::Removed { base }));
        out.extend(hunk.head.clone().map(|head| UnifiedLine::Added { head }));
        b = hunk.base.end;
        h = hunk.head.end;
    }
    while b < base_len {
        out.push(UnifiedLine::Context { base: b, head: h });
        b += 1;
        h += 1;
    }
    debug_assert_eq!(h, head_len);
    out
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Segment {
    /// A range of row indexes to show.
    Shown(std::ops::Range<usize>),
    /// A run of unchanged rows that the view collapses. It stays expandable.
    Collapsed(std::ops::Range<usize>),
}

/// Collapse the unchanged runs, and keep `context` rows around each change.
/// With no change at all, one collapsed run holds the whole file.
pub fn collapse(changed: &[bool], context: usize) -> Vec<Segment> {
    let n = changed.len();
    let mut keep = vec![false; n];
    for (i, c) in changed.iter().enumerate() {
        if *c {
            let (lo, hi) = (i.saturating_sub(context), (i + context + 1).min(n));
            keep[lo..hi].fill(true);
        }
    }
    let mut out = Vec::new();
    let mut i = 0;
    while i < n {
        let start = i;
        while i < n && keep[i] == keep[start] {
            i += 1;
        }
        out.push(if keep[start] {
            Segment::Shown(start..i)
        } else {
            Segment::Collapsed(start..i)
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hunk(b: (u32, u32), h: (u32, u32)) -> Hunk {
        Hunk {
            base: b.0..b.1,
            head: h.0..h.1,
        }
    }

    #[test]
    fn rows_pair_unchanged_lines_and_gap_the_shorter_side() {
        // base: a b c d   head: a X Y c d  (b -> X Y)
        let rows = side_by_side(4, 5, &[hunk((1, 2), (1, 3))]);
        let got: Vec<_> = rows.iter().map(|r| (r.base, r.head, r.changed)).collect();
        assert_eq!(
            got,
            vec![
                (Some(0), Some(0), false),
                (Some(1), Some(1), true),
                (None, Some(2), true),
                (Some(2), Some(3), false),
                (Some(3), Some(4), false),
            ]
        );
    }

    #[test]
    fn pure_insertion_and_deletion_have_gaps() {
        let rows = side_by_side(2, 3, &[hunk((1, 1), (1, 2))]);
        assert_eq!(
            rows[1],
            Row {
                base: None,
                head: Some(1),
                changed: true
            }
        );
        let rows = side_by_side(3, 2, &[hunk((1, 2), (1, 1))]);
        assert_eq!(
            rows[1],
            Row {
                base: Some(1),
                head: None,
                changed: true
            }
        );
    }

    #[test]
    fn unified_puts_removed_before_added() {
        let u = unified(3, 3, &[hunk((1, 2), (1, 2))]);
        assert_eq!(
            u,
            vec![
                UnifiedLine::Context { base: 0, head: 0 },
                UnifiedLine::Removed { base: 1 },
                UnifiedLine::Added { head: 1 },
                UnifiedLine::Context { base: 2, head: 2 },
            ]
        );
    }

    #[test]
    fn rows_agree_with_the_canonical_diff() {
        let (a, b) = (
            b"a\nb\nc\nd\ne\n".as_slice(),
            b"a\nB\nc\nd\nd2\ne\n".as_slice(),
        );
        let hunks = crate::canonical::canonical_hunks(a, b);
        let rows = side_by_side(5, 6, &hunks);
        assert_eq!(rows.iter().filter_map(|r| r.base).count(), 5);
        assert_eq!(rows.iter().filter_map(|r| r.head).count(), 6);
        let u = unified(5, 6, &hunks);
        assert_eq!(
            u.iter()
                .filter(|l| !matches!(l, UnifiedLine::Added { .. }))
                .count(),
            5
        );
    }

    #[test]
    fn collapse_keeps_context_around_changes() {
        let mut changed = vec![false; 20];
        changed[10] = true;
        assert_eq!(
            collapse(&changed, 3),
            vec![
                Segment::Collapsed(0..7),
                Segment::Shown(7..14),
                Segment::Collapsed(14..20)
            ]
        );
        assert_eq!(collapse(&[false; 5], 3), vec![Segment::Collapsed(0..5)]);
        assert_eq!(collapse(&[], 3), vec![]);
        assert_eq!(collapse(&[true, false], 3), vec![Segment::Shown(0..2)]);
    }
}
