//! Decoding of input bytes to text (Q26, C-02).
//!
//! The BOM picks UTF-8 or UTF-16. With no BOM the text is UTF-8, and invalid
//! bytes are reported, not guessed. The user can pick a legacy encoding for
//! each side. There is no heuristic detection. The raw bytes are never
//! changed: the decoded text is a view.

use encoding_rs::Encoding;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Detected {
    Utf8Bom,
    Utf16Le,
    Utf16Be,
    /// No BOM: UTF-8 by default.
    Utf8,
    /// The user selected this legacy encoding.
    Legacy(&'static Encoding),
}

impl Detected {
    /// A label for the source line of the UI.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Utf8Bom => "UTF-8 (BOM)",
            Self::Utf16Le => "UTF-16 LE (BOM)",
            Self::Utf16Be => "UTF-16 BE (BOM)",
            Self::Utf8 => "UTF-8",
            Self::Legacy(e) => e.name(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decoded {
    pub text: String,
    pub encoding: Detected,
    /// The byte offset of the first invalid sequence, if any. The text then
    /// holds U+FFFD for each invalid sequence.
    pub first_error: Option<usize>,
}

/// Look up a legacy encoding by its WHATWG label, for example `windows-1250`.
pub fn legacy_encoding(label: &str) -> Option<&'static Encoding> {
    Encoding::for_label(label.as_bytes())
}

/// Decode `raw`. A BOM wins over `legacy`, because it is explicit in the file.
pub fn decode(raw: &[u8], legacy: Option<&'static Encoding>) -> Decoded {
    let (encoding, body) = if let Some(rest) = raw.strip_prefix(&[0xEF, 0xBB, 0xBF]) {
        (Detected::Utf8Bom, rest)
    } else if let Some(rest) = raw.strip_prefix(&[0xFF, 0xFE]) {
        (Detected::Utf16Le, rest)
    } else if let Some(rest) = raw.strip_prefix(&[0xFE, 0xFF]) {
        (Detected::Utf16Be, rest)
    } else if let Some(e) = legacy {
        (Detected::Legacy(e), raw)
    } else {
        (Detected::Utf8, raw)
    };
    let enc = match encoding {
        Detected::Utf8Bom | Detected::Utf8 => encoding_rs::UTF_8,
        Detected::Utf16Le => encoding_rs::UTF_16LE,
        Detected::Utf16Be => encoding_rs::UTF_16BE,
        Detected::Legacy(e) => e,
    };
    let (text, had_errors) = enc.decode_without_bom_handling(body);
    let first_error = had_errors.then(|| first_invalid(enc, body));
    Decoded {
        text: text.into_owned(),
        encoding,
        first_error,
    }
}

fn first_invalid(enc: &'static Encoding, body: &[u8]) -> usize {
    if enc == encoding_rs::UTF_8 {
        return std::str::from_utf8(body)
            .err()
            .map_or(0, |e| e.valid_up_to());
    }
    // For the others, find the shortest prefix that does not decode cleanly.
    (0..=body.len())
        .find(|n| {
            enc.decode_without_bom_handling_and_without_replacement(&body[..*n])
                .is_none()
        })
        .map_or(0, |n| n.saturating_sub(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf8_with_and_without_bom() {
        let d = decode(b"\xEF\xBB\xBFhi", None);
        assert_eq!((d.text.as_str(), d.encoding), ("hi", Detected::Utf8Bom));
        let d = decode("žluť".as_bytes(), None);
        assert_eq!(
            (d.text.as_str(), d.encoding, d.first_error),
            ("žluť", Detected::Utf8, None)
        );
    }

    #[test]
    fn utf16_is_detected_by_the_bom() {
        let d = decode(b"\xFF\xFEh\0i\0", None);
        assert_eq!((d.text.as_str(), d.encoding), ("hi", Detected::Utf16Le));
        let d = decode(b"\xFE\xFF\0h\0i", None);
        assert_eq!((d.text.as_str(), d.encoding), ("hi", Detected::Utf16Be));
    }

    #[test]
    fn invalid_utf8_is_reported_and_never_guessed() {
        let d = decode(b"ab\xFFcd", None);
        assert_eq!(d.encoding, Detected::Utf8);
        assert_eq!(d.first_error, Some(2));
        assert!(d.text.contains('\u{FFFD}'));
    }

    #[test]
    fn the_user_can_select_a_legacy_encoding() {
        let e = legacy_encoding("windows-1250").unwrap();
        let d = decode(b"\xE8esky", Some(e));
        assert_eq!(d.text, "česky");
        assert_eq!(d.encoding.label(), "windows-1250");
        assert_eq!(d.first_error, None);
    }

    #[test]
    fn a_bom_wins_over_the_selected_encoding() {
        let e = legacy_encoding("windows-1250").unwrap();
        assert_eq!(
            decode(b"\xEF\xBB\xBFx", Some(e)).encoding,
            Detected::Utf8Bom
        );
    }

    #[test]
    fn an_unknown_label_gives_none() {
        assert!(legacy_encoding("no-such-encoding").is_none());
    }
}
