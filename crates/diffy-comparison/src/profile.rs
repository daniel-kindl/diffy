//! Comparison profiles and the intraline refinement (Q82, C-04).
//!
//! A profile changes only the view. It never changes the canonical diff, the
//! anchors or a publication. A filter in a profile hides changes, and the
//! counts still include them.

use gix_imara_diff::{Algorithm, Diff, InternedInput, TokenSource, sources};
use std::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Whitespace {
    Exact,
    Trailing,
    Amount,
    All,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Intraline {
    Off,
    Word,
    Character,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profile {
    pub name: String,
    pub whitespace: Whitespace,
    pub ignore_cr_at_eol: bool,
    pub ignore_blank_lines: bool,
    pub intraline: Intraline,
    pub context_lines: u32,
    pub collapse_unchanged: bool,
    /// Generated files are collapsed in the view and still counted.
    pub collapse_generated: bool,
    /// The glob patterns of the user that mark a file as generated.
    pub generated_globs: Vec<String>,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            name: "Default".into(),
            whitespace: Whitespace::Exact,
            ignore_cr_at_eol: false,
            ignore_blank_lines: false,
            intraline: Intraline::Word,
            context_lines: 3,
            collapse_unchanged: true,
            collapse_generated: true,
            generated_globs: Vec::new(),
        }
    }
}

impl Profile {
    /// The key that two lines compare by in the view. It is not the canonical
    /// comparison: the canonical diff always uses the exact bytes.
    pub fn view_key(&self, line: &str) -> String {
        let mut s = line.trim_end_matches('\n');
        if self.ignore_cr_at_eol {
            s = s.strip_suffix('\r').unwrap_or(s);
        }
        match self.whitespace {
            Whitespace::Exact => s.to_owned(),
            Whitespace::Trailing => s.trim_end().to_owned(),
            Whitespace::Amount => {
                // Runs of whitespace are one space. Trailing space is dropped.
                s.split_whitespace().collect::<Vec<_>>().join(" ")
                    + if s.starts_with(char::is_whitespace) {
                        " "
                    } else {
                        ""
                    }
            }
            Whitespace::All => s.chars().filter(|c| !c.is_whitespace()).collect(),
        }
    }

    /// Is a line hidden because it is blank and the profile ignores blanks?
    pub fn hides_blank(&self, line: &str) -> bool {
        self.ignore_blank_lines && line.trim().is_empty()
    }
}

/// The changed ranges inside one changed line pair, as byte ranges of
/// `old` and of `new`. `Off` gives none.
pub fn intraline(old: &str, new: &str, mode: Intraline) -> (Vec<Range<usize>>, Vec<Range<usize>>) {
    match mode {
        Intraline::Off => (Vec::new(), Vec::new()),
        Intraline::Word => refine(sources::words(old), sources::words(new), old, new),
        Intraline::Character => {
            let chars = |s: &str| -> Vec<(usize, usize)> {
                s.char_indices()
                    .map(|(i, c)| (i, i + c.len_utf8()))
                    .collect()
            };
            let (a, b) = (chars(old), chars(new));
            let ta: Vec<&str> = a.iter().map(|(s, e)| &old[*s..*e]).collect();
            let tb: Vec<&str> = b.iter().map(|(s, e)| &new[*s..*e]).collect();
            diff_tokens(&ta, &tb, &a, &b)
        }
    }
}

fn refine<'a>(
    a: impl Iterator<Item = &'a str>,
    b: impl Iterator<Item = &'a str>,
    old: &str,
    new: &str,
) -> (Vec<Range<usize>>, Vec<Range<usize>>) {
    let (ta, tb): (Vec<&str>, Vec<&str>) = (a.collect(), b.collect());
    let offsets = |toks: &[&str], whole: &str| -> Vec<(usize, usize)> {
        let mut pos = 0;
        toks.iter()
            .map(|t| {
                let start = whole[pos..].find(t).map_or(pos, |i| pos + i);
                pos = start + t.len();
                (start, pos)
            })
            .collect()
    };
    let (oa, ob) = (offsets(&ta, old), offsets(&tb, new));
    diff_tokens(&ta, &tb, &oa, &ob)
}

/// A list of tokens as a token source.
#[derive(Clone, Copy)]
struct Toks<'a>(&'a [&'a str]);

impl<'a> TokenSource for Toks<'a> {
    type Token = &'a str;
    type Tokenizer = std::iter::Copied<std::slice::Iter<'a, &'a str>>;

    fn tokenize(&self) -> Self::Tokenizer {
        self.0.iter().copied()
    }

    fn estimate_tokens(&self) -> u32 {
        self.0.len() as u32
    }
}

fn diff_tokens(
    ta: &[&str],
    tb: &[&str],
    oa: &[(usize, usize)],
    ob: &[(usize, usize)],
) -> (Vec<Range<usize>>, Vec<Range<usize>>) {
    let input = InternedInput::new(Toks(ta), Toks(tb));
    let diff = Diff::compute(Algorithm::Myers, &input);
    let (mut left, mut right) = (Vec::new(), Vec::new());
    let span = |offs: &[(usize, usize)], r: Range<u32>| -> Option<Range<usize>> {
        (!r.is_empty()).then(|| offs[r.start as usize].0..offs[r.end as usize - 1].1)
    };
    for h in diff.hunks() {
        left.extend(span(oa, h.before));
        right.extend(span(ob, h.after));
    }
    (left, right)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_profile_matches_q82() {
        let p = Profile::default();
        assert_eq!(p.whitespace, Whitespace::Exact);
        assert_eq!(p.intraline, Intraline::Word);
        assert_eq!(p.context_lines, 3);
        assert!(p.collapse_generated);
        assert!(!p.ignore_cr_at_eol && !p.ignore_blank_lines);
    }

    #[test]
    fn whitespace_modes_change_only_the_view_key() {
        let mut p = Profile::default();
        assert_ne!(p.view_key("a  b \n"), p.view_key("a b\n"));
        p.whitespace = Whitespace::Trailing;
        assert_eq!(p.view_key("a  b \n"), "a  b");
        p.whitespace = Whitespace::Amount;
        assert_eq!(p.view_key("a   b  \n"), p.view_key("a b\n"));
        p.whitespace = Whitespace::All;
        assert_eq!(p.view_key("a   b  \n"), p.view_key("ab\n"));
        p.ignore_cr_at_eol = true;
        assert_eq!(p.view_key("ab\r\n"), "ab");
    }

    #[test]
    fn a_profile_does_not_change_the_canonical_diff() {
        let (a, b) = (b"a  b\nc\n".as_slice(), b"a b\nc\n".as_slice());
        let before = crate::canonical::canonical_hunks(a, b);
        let _p = Profile {
            whitespace: Whitespace::All,
            ..Profile::default()
        };
        assert_eq!(crate::canonical::canonical_hunks(a, b), before);
        assert_eq!(before.len(), 1);
    }

    #[test]
    fn word_refinement_marks_the_changed_word() {
        let (l, r) = intraline("let x = foo(1);", "let x = bar(1);", Intraline::Word);
        assert_eq!(
            l.iter()
                .map(|r| &"let x = foo(1);"[r.clone()])
                .collect::<Vec<_>>(),
            ["foo"]
        );
        assert_eq!(
            r.iter()
                .map(|r| &"let x = bar(1);"[r.clone()])
                .collect::<Vec<_>>(),
            ["bar"]
        );
    }

    #[test]
    fn character_refinement_handles_unicode() {
        let (l, r) = intraline("kůň", "koň", Intraline::Character);
        assert_eq!(&"kůň"[l[0].clone()], "ů");
        assert_eq!(&"koň"[r[0].clone()], "o");
    }

    #[test]
    fn off_marks_nothing() {
        assert_eq!(intraline("a", "b", Intraline::Off), (vec![], vec![]));
    }
}
