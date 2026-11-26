//! Comparison contracts: immutable snapshots and normalized views.
//!
//! A [`Snapshot`] holds the raw bytes of one input at one moment. It has no
//! method that changes the bytes. A [`NormalizedView`] is a distinct type: it
//! is derived from a snapshot and never written back, so normalization (line
//! endings, whitespace) cannot change the original content.

pub mod anchor;
pub mod canonical;
pub mod profile;
pub mod rows;
pub mod text;

use sha2::{Digest, Sha256};
use std::sync::Arc;

/// Where a snapshot came from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source {
    File(String),
    Text(String),
    GitObject(String),
}

/// The content identity: the SHA-256 of the raw bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ContentId([u8; 32]);

impl ContentId {
    /// Read an identity back from its hex form. `None` when it is not 64 hex digits.
    pub fn from_hex(hex: &str) -> Option<Self> {
        if hex.len() != 64 || !hex.is_ascii() {
            return None;
        }
        let mut out = [0u8; 32];
        for (i, b) in out.iter_mut().enumerate() {
            *b = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).ok()?;
        }
        Some(Self(out))
    }

    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{b:02x}")).collect()
    }
}

/// A fixed copy of one input. Cloning is cheap and shares the same bytes.
#[derive(Clone, Debug)]
pub struct Snapshot {
    source: Source,
    raw: Arc<[u8]>,
    id: ContentId,
}

impl Snapshot {
    pub fn new(source: Source, raw: impl Into<Vec<u8>>) -> Self {
        let raw: Arc<[u8]> = raw.into().into();
        let id = ContentId(Sha256::digest(&raw).into());
        Self { source, raw, id }
    }

    pub fn source(&self) -> &Source {
        &self.source
    }

    /// The original bytes, exactly as read.
    pub fn raw(&self) -> &[u8] {
        &self.raw
    }

    pub fn id(&self) -> ContentId {
        self.id
    }

    /// Split into lines for a normalized view. The snapshot stays as it is.
    pub fn normalized(&self, options: NormalizeOptions) -> NormalizedView {
        NormalizedView::new(self, options)
    }
}

/// Options that decide how a view compares lines. They never change a snapshot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NormalizeOptions {
    pub ignore_line_ending: bool,
    pub ignore_trailing_whitespace: bool,
}

/// The lines of a snapshot as a comparison reads them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormalizedView {
    origin: ContentId,
    options: NormalizeOptions,
    lines: Vec<Vec<u8>>,
}

impl NormalizedView {
    fn new(snapshot: &Snapshot, options: NormalizeOptions) -> Self {
        let lines = snapshot
            .raw()
            .split_inclusive(|b| *b == b'\n')
            .map(|line| normalize_line(line, options))
            .collect();
        Self {
            origin: snapshot.id(),
            options,
            lines,
        }
    }

    /// The identity of the snapshot this view came from.
    pub fn origin(&self) -> ContentId {
        self.origin
    }

    pub fn options(&self) -> NormalizeOptions {
        self.options
    }

    pub fn lines(&self) -> &[Vec<u8>] {
        &self.lines
    }
}

fn normalize_line(line: &[u8], options: NormalizeOptions) -> Vec<u8> {
    let mut end = line.len();
    let had_lf = line.ends_with(b"\n");
    if had_lf {
        end -= 1;
    }
    let had_cr = had_lf && end > 0 && line[end - 1] == b'\r';
    if had_cr {
        end -= 1;
    }
    let mut body = &line[..end];
    if options.ignore_trailing_whitespace {
        while let [rest @ .., b' ' | b'\t'] = body {
            body = rest;
        }
    }
    let mut out = body.to_vec();
    if !options.ignore_line_ending {
        if had_cr {
            out.push(b'\r');
        }
        if had_lf {
            out.push(b'\n');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizing_never_changes_the_snapshot() {
        let s = Snapshot::new(Source::Text("t".into()), b"a \r\nb\n".to_vec());
        let before = s.id();
        let v = s.normalized(NormalizeOptions {
            ignore_line_ending: true,
            ignore_trailing_whitespace: true,
        });
        assert_eq!(v.lines(), &[b"a".to_vec(), b"b".to_vec()]);
        assert_eq!(s.raw(), b"a \r\nb\n");
        assert_eq!(s.id(), before);
        assert_eq!(v.origin(), before);
    }

    #[test]
    fn default_view_keeps_each_byte() {
        let s = Snapshot::new(Source::File("f".into()), b"a \r\nb".to_vec());
        let v = s.normalized(NormalizeOptions::default());
        assert_eq!(v.lines().concat(), s.raw());
    }

    #[test]
    fn an_identity_round_trips_through_hex() {
        let id = Snapshot::new(Source::Text("t".into()), b"x".to_vec()).id();
        assert_eq!(ContentId::from_hex(&id.to_hex()), Some(id));
        assert_eq!(ContentId::from_hex("zz"), None);
        assert_eq!(ContentId::from_hex(&"g".repeat(64)), None);
    }

    #[test]
    fn same_bytes_give_the_same_identity() {
        let a = Snapshot::new(Source::Text("a".into()), b"x".to_vec());
        let b = Snapshot::new(Source::File("b".into()), b"x".to_vec());
        assert_eq!(a.id(), b.id());
        assert_eq!(a.id().to_hex().len(), 64);
    }
}
