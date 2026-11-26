//! Anchor mapping, method M3 (Q45, Q64, Q65).
//!
//! An anchor is one line of the old revision. The mapping returns the new
//! line, or `None` when the anchor is outdated. A line maps only when the
//! canonical diff keeps it and, inside the segment between its two nearest
//! bracket lines, each longest common subsequence (LCS) matches it to the
//! same new line.
//!
//! The mapping compares each line without its line terminator (Q64). The
//! canonical diff and the stored coordinates keep each byte.

use crate::canonical::canonical_hunks;
use std::collections::HashMap;

/// The most cells (old lines times new lines) of one segment. A larger
/// segment is outdated.
pub const SEGMENT_CELLS: usize = 1_000_000;

/// The most lines on one side of a segment. The LCS tables use `u16`, so a
/// segment with 65,535 lines or more on a side is outdated (Q65).
pub const SEGMENT_LINES: usize = u16::MAX as usize - 1;

fn split(bytes: &[u8]) -> Vec<&[u8]> {
    bytes.split_inclusive(|b| *b == b'\n').collect()
}

fn strip_terminator(line: &[u8]) -> &[u8] {
    let line = line.strip_suffix(b"\n").unwrap_or(line);
    line.strip_suffix(b"\r").unwrap_or(line)
}

/// Maps each old line to its new line under the canonical diff, or `None`
/// for a line in a change group.
fn canonical_map(old: &[u8], new: &[u8], n_old: usize) -> Vec<Option<usize>> {
    let mut map = vec![None; n_old];
    let (mut i, mut j) = (0usize, 0usize);
    for h in canonical_hunks(old, new) {
        while i < h.base.start as usize {
            map[i] = Some(j);
            i += 1;
            j += 1;
        }
        i = h.base.end as usize;
        j = h.head.end as usize;
    }
    while i < n_old {
        map[i] = Some(j);
        i += 1;
        j += 1;
    }
    map
}

/// Map each old line of `old` to a line of `new`. `None` means outdated.
pub fn map_anchors<'a>(old: &'a [u8], new: &'a [u8]) -> Vec<Option<usize>> {
    let old_lines = split(old);
    let new_lines = split(new);
    let cmap = canonical_map(old, new, old_lines.len());

    let mut ids: HashMap<&[u8], u32> = HashMap::new();
    let mut intern = |l: &'a [u8]| {
        let l = strip_terminator(l);
        let n = ids.len() as u32;
        *ids.entry(l).or_insert(n)
    };
    let a: Vec<u32> = old_lines.iter().map(|l| intern(l)).collect();
    let b: Vec<u32> = new_lines.iter().map(|l| intern(l)).collect();
    let tokens = ids.len();

    let mut count = vec![(0u32, 0u32); tokens];
    for t in &a {
        count[*t as usize].0 += 1;
    }
    for t in &b {
        count[*t as usize].1 += 1;
    }
    // Brackets: lines that are unique on each side and that the diff keeps.
    let mut marks: Vec<(usize, usize)> = (0..a.len())
        .filter(|p| count[a[*p] as usize] == (1, 1))
        .filter_map(|p| cmap[p].map(|q| (p, q)))
        .collect();
    marks.push((a.len(), b.len()));

    let mut out = vec![None; a.len()];
    let (mut os, mut ns) = (0usize, 0usize);
    for (p1, q1) in marks {
        // Brackets are in order because the diff keeps the line order.
        if p1 >= os && q1 >= ns {
            let (oa, ob) = (&a[os..p1], &b[ns..q1]);
            if oa.len() <= SEGMENT_LINES
                && ob.len() <= SEGMENT_LINES
                && oa.len() * ob.len() <= SEGMENT_CELLS
            {
                for (k, j) in lcs_rule(oa, ob).into_iter().enumerate() {
                    if let Some(j) = j
                        && cmap[os + k] == Some(ns + j)
                    {
                        out[os + k] = Some(ns + j);
                    }
                }
            }
        }
        if p1 < a.len() {
            out[p1] = Some(q1);
        }
        os = p1 + 1;
        ns = q1 + 1;
    }
    out
}

/// The LCS rule: an old line maps to new line `j` only when each longest
/// common subsequence of the two sides matches it to `j`.
fn lcs_rule(a: &[u32], b: &[u32]) -> Vec<Option<usize>> {
    let (n, m) = (a.len(), b.len());
    let w = m + 1;
    // f[i][j]: LCS of a[..i] and b[..j]. g[i][j]: LCS of a[i..] and b[j..].
    let mut f = vec![0u16; (n + 1) * w];
    for i in 1..=n {
        for j in 1..=m {
            f[i * w + j] = if a[i - 1] == b[j - 1] {
                f[(i - 1) * w + j - 1] + 1
            } else {
                f[(i - 1) * w + j].max(f[i * w + j - 1])
            };
        }
    }
    let mut g = vec![0u16; (n + 1) * w];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            g[i * w + j] = if a[i] == b[j] {
                g[(i + 1) * w + j + 1] + 1
            } else {
                g[(i + 1) * w + j].max(g[i * w + j + 1])
            };
        }
    }
    let len = u32::from(f[n * w + m]);
    (0..n)
        .map(|i| {
            let skip = (0..=m)
                .map(|j| u32::from(f[i * w + j]) + u32::from(g[(i + 1) * w + j]))
                .max()
                .unwrap_or(0);
            if skip == len {
                return None;
            }
            let mut found = None;
            for j in 0..m {
                if a[i] == b[j]
                    && u32::from(f[i * w + j]) + 1 + u32::from(g[(i + 1) * w + j + 1]) == len
                {
                    if found.is_some() {
                        return None;
                    }
                    found = Some(j);
                }
            }
            found
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(lines: &[&str]) -> Vec<u8> {
        lines
            .iter()
            .map(|l| format!("{l}\n"))
            .collect::<String>()
            .into_bytes()
    }

    #[test]
    fn unchanged_and_shifted_lines_map() {
        let old = text(&["a", "b", "c"]);
        let new = text(&["x", "a", "b", "c"]);
        assert_eq!(map_anchors(&old, &new), vec![Some(1), Some(2), Some(3)]);
    }

    #[test]
    fn edited_and_deleted_lines_are_outdated() {
        let old = text(&["a", "b", "c"]);
        let new = text(&["a", "B", "c"]);
        assert_eq!(map_anchors(&old, &new), vec![Some(0), None, Some(2)]);
        let new = text(&["a", "c"]);
        assert_eq!(map_anchors(&old, &new), vec![Some(0), None, Some(1)]);
    }

    #[test]
    fn deleting_one_of_two_identical_blocks_is_outdated() {
        let old = text(&["p", "}", "}", "q"]);
        let new = text(&["p", "}", "q"]);
        let m = map_anchors(&old, &new);
        assert_eq!(m[1], None);
        assert_eq!(m[2], None);
        assert_eq!((m[0], m[3]), (Some(0), Some(2)));
    }

    #[test]
    fn a_changed_terminator_is_a_change_of_the_canonical_diff() {
        // The canonical diff keeps each byte, so LF to CRLF changes each line.
        // The terminator-free comparison only applies inside the LCS segments.
        let old = b"a\nb\nc\n".to_vec();
        let new = b"a\r\nb\r\nc\r\n".to_vec();
        assert_eq!(map_anchors(&old, &new), vec![None, None, None]);
    }

    #[test]
    fn strips_one_terminator_only() {
        assert_eq!(strip_terminator(b"a\r\n"), b"a");
        assert_eq!(strip_terminator(b"a\n"), b"a");
        assert_eq!(strip_terminator(b"a"), b"a");
        assert_eq!(strip_terminator(b"a\r"), b"a");
    }

    #[test]
    fn a_segment_of_70000_inserted_lines_does_not_panic() {
        let mut old = text(&["start", "end"]);
        old.splice(0..0, b"".iter().copied());
        let mut new = b"start\n".to_vec();
        for i in 0..70_000 {
            new.extend_from_slice(format!("ins {}\n", i % 3).as_bytes());
        }
        new.extend_from_slice(b"end\n");
        let m = map_anchors(&old, &new);
        assert_eq!(m, vec![Some(0), Some(70_001)]);
    }

    #[test]
    fn a_large_one_sided_segment_is_outdated_not_a_panic() {
        let mut old = b"start\n".to_vec();
        for i in 0..70_000 {
            old.extend_from_slice(format!("l {}\n", i % 3).as_bytes());
        }
        old.extend_from_slice(b"end\n");
        let new = text(&["start", "end"]);
        let m = map_anchors(&old, &new);
        assert_eq!(m[0], Some(0));
        assert_eq!(m[1], None);
        assert_eq!(*m.last().unwrap(), Some(1));
    }
}
