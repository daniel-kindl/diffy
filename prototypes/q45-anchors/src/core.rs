//! Lines, the canonical diff, the three mapping methods and the LCS rule.
//!
//! A line is its bytes with its line terminator, as Git and gix-imara-diff split it.
//! Positions are 0-based here. The result files print 1-based line numbers.

use gix_imara_diff::{Algorithm, Diff, InternedInput};
use std::collections::HashMap;

/// The context lines above and below the anchor line in the window method.
pub const K: usize = 3;

/// Splits after each `\n`. The last line has no terminator when the file does not end with `\n`.
pub fn split(bytes: &[u8]) -> Vec<&[u8]> {
    let mut lines = Vec::new();
    let mut start = 0;
    for (i, b) in bytes.iter().enumerate() {
        if *b == b'\n' {
            lines.push(&bytes[start..=i]);
            start = i + 1;
        }
    }
    if start < bytes.len() {
        lines.push(&bytes[start..]);
    }
    lines
}

/// The two files as token IDs. Equal lines have equal IDs.
pub struct Pair {
    pub old: Vec<u32>,
    pub new: Vec<u32>,
    pub tokens: u32,
}

impl Pair {
    pub fn new<'a>(old: &[&'a [u8]], new: &[&'a [u8]]) -> Pair {
        let mut ids: HashMap<&'a [u8], u32> = HashMap::new();
        let mut intern = |l: &'a [u8]| -> u32 {
            let n = ids.len() as u32;
            *ids.entry(l).or_insert(n)
        };
        let o = old.iter().map(|l| intern(l)).collect();
        let n = new.iter().map(|l| intern(l)).collect();
        let tokens = ids.len() as u32;
        Pair { old: o, new: n, tokens }
    }
}

/// The canonical diff of Q43 and Q61: histogram, gix-imara-diff 0.3.4 with the Q60 patch,
/// then `postprocess_lines` (the indent heuristic). The result maps each old line to its new
/// line, or to None when the line is in a change group.
pub fn canonical_map(old: &[u8], new: &[u8]) -> Vec<Option<usize>> {
    let input = InternedInput::new(old, new);
    let mut diff = Diff::compute(Algorithm::Histogram, &input);
    diff.postprocess_lines(&input);
    let n = input.before.len();
    let mut map = vec![None; n];
    let (mut i, mut j) = (0usize, 0usize);
    for h in diff.hunks() {
        let (bs, be, a_s) = (h.before.start as usize, h.before.end as usize, h.after.start as usize);
        while i < bs {
            map[i] = Some(j);
            i += 1;
            j += 1;
        }
        debug_assert_eq!(j, a_s);
        i = be;
        j = h.after.end as usize;
    }
    while i < n {
        map[i] = Some(j);
        i += 1;
        j += 1;
    }
    map
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize)]
pub enum Method {
    /// M1: the line moves with the canonical diff. A line in a change group is outdated.
    HunkMap,
    /// M2: M1, then the line and K lines above and below must be equal in both files, and this
    /// window must be unique in each file.
    HunkWindow,
    /// M3: M1, then the LCS rule inside the segment between two brackets must give the same line.
    /// A bracket is a line that is unique in each file and that the diff keeps, or the start or
    /// the end of the file. A bracket line maps as the diff maps it.
    HunkBrackets,
}

pub const METHODS: [Method; 3] = [Method::HunkMap, Method::HunkWindow, Method::HunkBrackets];

/// Maps each old line with one method. None is outdated.
pub fn map_all(method: Method, pair: &Pair, cmap: &[Option<usize>]) -> Vec<Option<usize>> {
    match method {
        Method::HunkMap => cmap.to_vec(),
        Method::HunkWindow => window(pair, cmap),
        Method::HunkBrackets => brackets(pair, cmap),
    }
}

const BOF: u32 = u32::MAX - 1;
const EOF: u32 = u32::MAX;

fn tok(seq: &[u32], p: isize) -> u32 {
    if p < 0 {
        BOF
    } else if p as usize >= seq.len() {
        EOF
    } else {
        seq[p as usize]
    }
}

fn win(seq: &[u32], p: usize) -> [u32; 2 * K + 1] {
    let mut w = [0; 2 * K + 1];
    for (d, slot) in w.iter_mut().enumerate() {
        *slot = tok(seq, p as isize + d as isize - K as isize);
    }
    w
}

fn window(pair: &Pair, cmap: &[Option<usize>]) -> Vec<Option<usize>> {
    let mut count_old: HashMap<[u32; 2 * K + 1], u32> = HashMap::new();
    let mut count_new: HashMap<[u32; 2 * K + 1], u32> = HashMap::new();
    for p in 0..pair.old.len() {
        *count_old.entry(win(&pair.old, p)).or_default() += 1;
    }
    for p in 0..pair.new.len() {
        *count_new.entry(win(&pair.new, p)).or_default() += 1;
    }
    (0..pair.old.len())
        .map(|i| {
            let n = cmap[i]?;
            let w = win(&pair.old, i);
            let ok = w == win(&pair.new, n)
                && count_old.get(&w) == Some(&1)
                && count_new.get(&w) == Some(&1);
            ok.then_some(n)
        })
        .collect()
}

/// The most cells (old lines times new lines) of one segment for the LCS rule in M3.
/// A larger segment is outdated.
pub const SEGMENT_CELLS: usize = 1_000_000;

fn brackets(pair: &Pair, cmap: &[Option<usize>]) -> Vec<Option<usize>> {
    let mut count = vec![(0u32, 0u32); pair.tokens as usize];
    for t in &pair.old {
        count[*t as usize].0 += 1;
    }
    for t in &pair.new {
        count[*t as usize].1 += 1;
    }
    // The brackets: each old line that is unique in each file and that the diff keeps, then
    // the end of the file. The start of the file is the bracket (-1, -1).
    let mut marks: Vec<(isize, isize)> = (0..pair.old.len())
        .filter(|p| count[pair.old[*p] as usize] == (1, 1))
        .filter_map(|p| cmap[p].map(|q| (p as isize, q as isize)))
        .collect();
    marks.push((pair.old.len() as isize, pair.new.len() as isize));
    let mut out = vec![None; pair.old.len()];
    let mut prev = (-1isize, -1isize);
    for (p1, q1) in marks {
        let (p0, q0) = prev;
        let (os, oe) = ((p0 + 1) as usize, p1 as usize);
        let (ns, ne) = ((q0 + 1) as usize, q1 as usize);
        if (oe - os) * (ne - ns) <= SEGMENT_CELLS {
            let local = lcs_rule(&pair.old[os..oe], &pair.new[ns..ne]).expected;
            for (k, j) in local.into_iter().enumerate() {
                // The line maps when the segment rule and the canonical diff agree.
                if let Some(j) = j {
                    if cmap[os + k] == Some(ns + j) {
                        out[os + k] = Some(ns + j);
                    }
                }
            }
        }
        if (p1 as usize) < pair.old.len() {
            out[p1 as usize] = Some(q1 as usize);
        }
        prev = (p1, q1);
    }
    out
}

/// The LCS rule, the expected result: an old line maps to new line j only when each longest
/// common subsequence of the two files matches the line, and matches it to j. Otherwise the
/// anchor is outdated: the line was edited or deleted, or another equally short edit explains
/// the change in another way.
pub struct Lcs {
    pub len: u32,
    pub expected: Vec<Option<usize>>,
}

pub fn lcs_rule(a: &[u32], b: &[u32]) -> Lcs {
    let (n, m) = (a.len(), b.len());
    assert!(n < u16::MAX as usize && m < u16::MAX as usize, "the LCS tables use u16");
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
    let len = f[n * w + m] as u32;
    let expected = (0..n)
        .map(|i| {
            let skip = (0..=m).map(|j| (f[i * w + j] + g[(i + 1) * w + j]) as u32).max().unwrap();
            if skip == len {
                return None;
            }
            let mut found = None;
            for j in 0..m {
                if a[i] == b[j] && (f[i * w + j] + 1 + g[(i + 1) * w + j + 1]) as u32 == len {
                    if found.is_some() {
                        return None;
                    }
                    found = Some(j);
                }
            }
            found
        })
        .collect();
    Lcs { len, expected }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each longest common subsequence of a and b, as the list of matched pairs.
    fn all_lcs(a: &[u32], b: &[u32]) -> Vec<Vec<(usize, usize)>> {
        fn walk(a: &[u32], b: &[u32], i: usize, j: usize, cur: &mut Vec<(usize, usize)>, out: &mut Vec<Vec<(usize, usize)>>) {
            out.push(cur.clone());
            for i2 in i..a.len() {
                for j2 in j..b.len() {
                    if a[i2] == b[j2] {
                        cur.push((i2, j2));
                        walk(a, b, i2 + 1, j2 + 1, cur, out);
                        cur.pop();
                    }
                }
            }
        }
        let mut out = Vec::new();
        walk(a, b, 0, 0, &mut Vec::new(), &mut out);
        let best = out.iter().map(Vec::len).max().unwrap();
        out.retain(|s| s.len() == best);
        out
    }

    #[test]
    fn lcs_rule_matches_brute_force() {
        let mut r = crate::history::Rng::new(7);
        for _ in 0..20_000 {
            let n = r.range(0, 7);
            let m = r.range(0, 7);
            let a: Vec<u32> = (0..n).map(|_| r.range(0, 2) as u32).collect();
            let b: Vec<u32> = (0..m).map(|_| r.range(0, 2) as u32).collect();
            let all = all_lcs(&a, &b);
            let rule = lcs_rule(&a, &b);
            assert_eq!(rule.len as usize, all[0].len());
            for i in 0..n {
                let targets: std::collections::BTreeSet<Option<usize>> = all
                    .iter()
                    .map(|s| s.iter().find(|(x, _)| *x == i).map(|(_, y)| *y))
                    .collect();
                let forced = if targets.len() == 1 { *targets.iter().next().unwrap() } else { None };
                assert_eq!(rule.expected[i], forced, "a={a:?} b={b:?} i={i}");
            }
        }
    }
}
