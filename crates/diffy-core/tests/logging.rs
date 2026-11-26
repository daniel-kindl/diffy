use diffy_core::logging::{RedactingMakeWriter, Sensitive};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct Buf(Arc<Mutex<Vec<u8>>>);
impl std::io::Write for Buf {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(b);
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn capture(f: impl FnOnce()) -> String {
    let buf = Buf(Arc::new(Mutex::new(Vec::new())));
    let b = buf.clone();
    let sub = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_ansi(false)
        .with_writer(RedactingMakeWriter::new(move || b.clone()))
        .finish();
    tracing::subscriber::with_default(sub, f);
    String::from_utf8(buf.0.lock().unwrap().clone()).unwrap()
}

fn token() -> String {
    format!("{}{}", "ghp", "_ABCDEF0123456789abcdef0123456789abcd")
}
const MARKER: &str = "FILE_CONTENT_MARKER_9f3a";

#[test]
fn sensitive_wrapper_hides_token_and_content() {
    let out = capture(|| {
        tracing::info!(token = ?Sensitive(token()), "signed in");
        tracing::debug!(content = %Sensitive(MARKER), "read file");
    });
    assert!(out.contains("signed in"));
    assert!(!out.contains(&token()), "{out}");
    assert!(!out.contains(MARKER), "{out}");
}

#[test]
fn writer_masks_a_token_that_was_logged_by_mistake() {
    let out = capture(|| tracing::warn!("request failed for {} twice", token()));
    assert!(out.contains("request failed"));
    assert!(out.contains("twice"));
    assert!(!out.contains(&token()), "{out}");
}

#[test]
fn writer_masks_sensitive_header_values() {
    let out = capture(|| tracing::error!("headers: Authorization: Bearer abc.def.ghi"));
    assert!(!out.contains("abc.def.ghi"), "{out}");
}
