//! The commands that the frontend can call. The frontend gets no credential
//! and no file system access: it never sends a path. Each input is validated
//! (spec section 11).

use diffy_comparison::canonical::canonical_hunks;
use serde::{Deserialize, Serialize};

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
    })
}

#[tauri::command]
pub fn compare_text(input: CompareInput) -> Result<Comparison, String> {
    compare(input)
}

#[cfg(test)]
mod tests {
    use super::*;

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
