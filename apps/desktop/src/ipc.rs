//! The commands that the frontend can call. The frontend gets no credential
//! and no file system access: it never sends a path. Each input is validated
//! (spec section 11).

use diffy_comparison::Snapshot;
use diffy_comparison::canonical::canonical_hunks;
use diffy_comparison::text::decode;
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

/// The comparison that the command line asked for. Rust reads the files, so
/// the frontend never names a path. The state holds it until the UI asks.
#[derive(Default)]
pub struct Initial(pub Mutex<Option<Comparison>>);

/// Build the comparison of two snapshots. The text is decoded for the view and
/// the canonical diff runs on the decoded text, so the hunks match the lines
/// that the editors show. The snapshots keep the raw bytes.
pub fn compare_snapshots(left: &Snapshot, right: &Snapshot) -> Result<Comparison, String> {
    let (l, r) = (decode(left.raw(), None), decode(right.raw(), None));
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
            label: label(left),
            encoding: l.encoding.label().into(),
            first_error: l.first_error,
        },
        SideInfo {
            label: label(right),
            encoding: r.encoding.label().into(),
            first_error: r.first_error,
        },
    ];
    Ok(c)
}

#[tauri::command]
pub fn initial_comparison(state: tauri::State<'_, Initial>) -> Option<Comparison> {
    state.0.lock().ok()?.take()
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
        let c = compare_snapshots(&l, &r).unwrap();
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
        let c = compare_snapshots(&l, &r).unwrap();
        assert_eq!(c.sides[0].first_error, Some(2));
        assert_eq!(c.sides[1].first_error, None);
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
