//! The commands that the frontend can call. The frontend gets no credential
//! and no file system access: it never sends a path. Each input is validated
//! (spec section 11).

use diffy_comparison::Snapshot;
use diffy_comparison::canonical::canonical_hunks;
use diffy_comparison::text::{decode, legacy_encoding};
use encoding_rs::Encoding;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// The most bytes of one side that a command accepts.
pub const MAX_SIDE_BYTES: usize = 64 * 1024 * 1024;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompareInput {
    pub base: String,
    pub head: String,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Range {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct HunkDto {
    pub base: Range,
    pub head: Range,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Comparison {
    pub base: String,
    pub head: String,
    pub hunks: Vec<HunkDto>,
    /// The source labels, the encodings and the decode errors for the UI.
    pub sides: Vec<SideInfo>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct SideInfo {
    pub label: String,
    pub encoding: String,
    /// The byte offset of the first invalid sequence, if any.
    pub first_error: Option<usize>,
}

pub fn compare(input: CompareInput) -> Result<Comparison, String> {
    if input.base.len() > MAX_SIDE_BYTES || input.head.len() > MAX_SIDE_BYTES {
        return Err(format!("a side is larger than {MAX_SIDE_BYTES} bytes"));
    }
    let hunks = canonical_hunks(input.base.as_bytes(), input.head.as_bytes())
        .into_iter()
        .map(|h| HunkDto {
            base: Range {
                start: h.base.start,
                end: h.base.end,
            },
            head: Range {
                start: h.head.start,
                end: h.head.end,
            },
        })
        .collect();
    Ok(Comparison {
        base: input.base,
        head: input.head,
        hunks,
        sides: Vec::new(),
    })
}

/// The two inputs of the open comparison. Rust keeps the raw snapshots, so a
/// change of the encoding or of the order never rereads a file and never
/// changes a source. Each snapshot keeps its own source label.
pub struct Session {
    left: Snapshot,
    right: Snapshot,
    left_encoding: Option<&'static Encoding>,
    right_encoding: Option<&'static Encoding>,
}

impl Session {
    pub fn new(left: Snapshot, right: Snapshot) -> Self {
        Self {
            left,
            right,
            left_encoding: None,
            right_encoding: None,
        }
    }

    /// Select the legacy encoding of one side by its label. There is no
    /// heuristic detection. An empty label goes back to the default.
    pub fn set_encoding(&mut self, side: Side, label: &str) -> Result<(), String> {
        let enc = if label.is_empty() {
            None
        } else {
            Some(legacy_encoding(label).ok_or_else(|| format!("unknown encoding {label}"))?)
        };
        match side {
            Side::Left => self.left_encoding = enc,
            Side::Right => self.right_encoding = enc,
        }
        Ok(())
    }

    /// Swap the sides. The sources and the encodings go with their inputs.
    pub fn swap(&mut self) {
        std::mem::swap(&mut self.left, &mut self.right);
        std::mem::swap(&mut self.left_encoding, &mut self.right_encoding);
    }

    /// The comparison. The text is decoded for the view and the canonical
    /// diff runs on the decoded text, so the hunks match the shown lines.
    pub fn comparison(&self) -> Result<Comparison, String> {
        let (l, r) = (
            decode(self.left.raw(), self.left_encoding),
            decode(self.right.raw(), self.right_encoding),
        );
        let label = |s: &Snapshot| match s.source() {
            diffy_comparison::Source::File(p) | diffy_comparison::Source::Text(p) => p.clone(),
            diffy_comparison::Source::GitObject(o) => o.clone(),
        };
        let mut c = compare(CompareInput {
            base: l.text.clone(),
            head: r.text.clone(),
        })?;
        c.sides = vec![
            SideInfo {
                label: label(&self.left),
                encoding: l.encoding.label().into(),
                first_error: l.first_error,
            },
            SideInfo {
                label: label(&self.right),
                encoding: r.encoding.label().into(),
                first_error: r.first_error,
            },
        ];
        Ok(c)
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Left,
    Right,
}

/// The open comparison, owned by Rust.
#[derive(Default)]
pub struct Open(pub Mutex<Option<Session>>);

fn with_session<T>(
    state: &Open,
    f: impl FnOnce(&mut Session) -> Result<T, String>,
) -> Result<Option<T>, String> {
    let mut guard = state
        .0
        .lock()
        .map_err(|_| "the session is locked".to_owned())?;
    guard.as_mut().map(f).transpose()
}

#[tauri::command]
pub fn current_comparison(state: tauri::State<'_, Open>) -> Result<Option<Comparison>, String> {
    with_session(&state, |s| s.comparison())
}

#[tauri::command]
pub fn select_encoding(
    state: tauri::State<'_, Open>,
    side: Side,
    label: String,
) -> Result<Option<Comparison>, String> {
    with_session(&state, |s| {
        s.set_encoding(side, &label)?;
        s.comparison()
    })
}

#[tauri::command]
pub fn swap_sides(state: tauri::State<'_, Open>) -> Result<Option<Comparison>, String> {
    with_session(&state, |s| {
        s.swap();
        s.comparison()
    })
}

#[tauri::command]
pub fn compare_text(input: CompareInput) -> Result<Comparison, String> {
    compare(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use diffy_comparison::Source;

    #[test]
    fn two_snapshots_give_hunks_labels_and_encodings() {
        let l = Snapshot::new(Source::File("a.txt".into()), b"one\ntwo\n".to_vec());
        let r = Snapshot::new(
            Source::File("b.txt".into()),
            b"\xEF\xBB\xBFone\nTWO\n".to_vec(),
        );
        let c = Session::new(l, r).comparison().unwrap();
        assert_eq!(c.hunks.len(), 1);
        assert_eq!(
            (c.sides[0].label.as_str(), c.sides[0].encoding.as_str()),
            ("a.txt", "UTF-8")
        );
        assert_eq!(c.sides[1].encoding, "UTF-8 (BOM)");
        assert_eq!(c.head, "one\nTWO\n", "the BOM is not part of the text");
    }

    #[test]
    fn an_invalid_side_reports_its_error_and_still_compares() {
        let l = Snapshot::new(Source::File("a".into()), b"ab\xFF\n".to_vec());
        let r = Snapshot::new(Source::File("b".into()), b"ab\n".to_vec());
        let c = Session::new(l, r).comparison().unwrap();
        assert_eq!(c.sides[0].first_error, Some(2));
        assert_eq!(c.sides[1].first_error, None);
    }

    fn snaps(l: &[u8], r: &[u8]) -> Session {
        Session::new(
            Snapshot::new(Source::File("l".into()), l.to_vec()),
            Snapshot::new(Source::File("r".into()), r.to_vec()),
        )
    }

    #[test]
    fn a_legacy_encoding_is_selected_per_side_and_not_guessed() {
        let mut s = snaps(b"\xE8esky\n", b"\xE8esky\n");
        assert!(
            s.comparison().unwrap().sides[0].first_error.is_some(),
            "no guessing"
        );
        s.set_encoding(Side::Left, "windows-1250").unwrap();
        let c = s.comparison().unwrap();
        assert_eq!(c.base, "česky\n");
        assert_eq!(c.sides[0].encoding, "windows-1250");
        assert!(
            c.sides[1].first_error.is_some(),
            "the other side is unchanged"
        );
        assert!(s.set_encoding(Side::Left, "nope").is_err());
        s.set_encoding(Side::Left, "").unwrap();
        assert!(s.comparison().unwrap().sides[0].first_error.is_some());
    }

    #[test]
    fn swapping_keeps_each_source_with_its_input() {
        let mut s = snaps(b"a\n", b"b\n");
        s.swap();
        let c = s.comparison().unwrap();
        assert_eq!((c.base.as_str(), c.head.as_str()), ("b\n", "a\n"));
        assert_eq!(
            (c.sides[0].label.as_str(), c.sides[1].label.as_str()),
            ("r", "l")
        );
    }

    #[test]
    fn swapping_twice_gives_the_first_comparison() {
        let mut s = snaps(b"a\nb\n", b"a\nB\n");
        let first = s.comparison().unwrap();
        s.swap();
        s.swap();
        assert_eq!(s.comparison().unwrap(), first);
    }

    #[test]
    fn returns_the_canonical_hunks() {
        let c = compare(CompareInput {
            base: "a\nb\n".into(),
            head: "a\nB\n".into(),
        })
        .unwrap();
        assert_eq!(c.hunks.len(), 1);
        assert_eq!(c.hunks[0].base, Range { start: 1, end: 2 });
        assert_eq!(c.hunks[0].head, Range { start: 1, end: 2 });
    }

    #[test]
    fn rejects_an_input_that_is_too_large() {
        let big = "x".repeat(MAX_SIDE_BYTES + 1);
        assert!(
            compare(CompareInput {
                base: big,
                head: String::new()
            })
            .is_err()
        );
    }

    #[test]
    fn the_input_has_no_path_and_rejects_unknown_fields() {
        let ok: Result<CompareInput, _> = serde_json::from_str(r#"{"base":"a","head":"b"}"#);
        assert!(ok.is_ok());
        let path: Result<CompareInput, _> =
            serde_json::from_str(r#"{"base":"a","head":"b","path":"/etc/passwd"}"#);
        assert!(path.is_err());
    }
}
