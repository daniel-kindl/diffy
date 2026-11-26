//! Structured logging (spec section 13).
//!
//! Levels: trace and debug record state transitions. Info records lifecycle
//! and user actions. Warn records recoverable degradation. Error records
//! failed operations. A log never holds source content, a token or a
//! sensitive header. Two guards enforce this:
//!
//! - [`Sensitive`] wraps a value and prints `[redacted]` in `Debug` and
//!   `Display`. Wrap a token or a file content before it goes to a log call.
//! - The writer of [`init`] masks known token prefixes and sensitive header
//!   values on each line, as a second line of defence.

use std::io::{self, Write};
use std::sync::Mutex;
use tracing_subscriber::fmt::MakeWriter;

const MASK: &str = "[redacted]";

/// A value that never prints. Use it for tokens and for source content.
pub struct Sensitive<T>(pub T);

impl<T> std::fmt::Debug for Sensitive<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(MASK)
    }
}

impl<T> std::fmt::Display for Sensitive<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(MASK)
    }
}

const TOKEN_PREFIXES: &[&str] = &["ghp_", "gho_", "ghu_", "ghs_", "ghr_", "github_pat_"];
const HEADERS: &[&str] = &["authorization", "cookie", "set-cookie", "x-api-key"];

fn is_token_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '~' | '+' | '/' | '=')
}

/// Mask the secrets in one chunk of log text.
pub fn redact(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for line in input.split_inclusive('\n') {
        out.push_str(&redact_line(line));
    }
    out
}

fn redact_line(line: &str) -> String {
    let lower = line.to_ascii_lowercase();
    let mut line = line.to_owned();
    // Sensitive headers: mask everything after the name and the separator.
    for h in HEADERS {
        let mut from = 0;
        while let Some(pos) = lower[from..].find(h) {
            let start = from + pos;
            let after = start + h.len();
            let rest = &line[after..];
            let trimmed = rest.trim_start_matches(['"', '\'', ' ']);
            if trimmed.starts_with(':') || trimmed.starts_with('=') {
                let end_marker = line[after..]
                    .find(['\n', '\r'])
                    .map_or(line.len(), |p| after + p);
                let sep = line[after..end_marker]
                    .find([':', '='])
                    .map_or(after, |p| after + p + 1);
                line.replace_range(sep..end_marker, &format!(" {MASK}"));
                return redact_tokens(&line);
            }
            from = after;
        }
    }
    redact_tokens(&line)
}

fn redact_tokens(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    'outer: while !rest.is_empty() {
        for p in TOKEN_PREFIXES {
            if let Some(tail) = rest.strip_prefix(p) {
                let n: usize = tail
                    .chars()
                    .take_while(|c| is_token_char(*c))
                    .map(char::len_utf8)
                    .sum();
                out.push_str(MASK);
                rest = &tail[n..];
                continue 'outer;
            }
        }
        let c = rest.chars().next().unwrap();
        out.push(c);
        rest = &rest[c.len_utf8()..];
    }
    out
}

/// A writer that masks secrets before it forwards a line to the inner writer.
pub struct RedactingWriter<W: Write> {
    inner: W,
    pending: Vec<u8>,
}

impl<W: Write> Write for RedactingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.pending.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        let text = String::from_utf8_lossy(&self.pending).into_owned();
        self.pending.clear();
        self.inner.write_all(redact(&text).as_bytes())?;
        self.inner.flush()
    }
}

impl<W: Write> Drop for RedactingWriter<W> {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

/// Build a `MakeWriter` that redacts the output of `make`.
pub struct RedactingMakeWriter<F>(Mutex<F>);

impl<F, W> RedactingMakeWriter<F>
where
    F: FnMut() -> W,
    W: Write,
{
    pub fn new(make: F) -> Self {
        Self(Mutex::new(make))
    }
}

impl<'a, F, W> MakeWriter<'a> for RedactingMakeWriter<F>
where
    F: FnMut() -> W,
    W: Write,
{
    type Writer = RedactingWriter<W>;

    fn make_writer(&'a self) -> Self::Writer {
        let inner = (self.0.lock().unwrap())();
        RedactingWriter {
            inner,
            pending: Vec::new(),
        }
    }
}

/// Install the global subscriber. It logs to stderr. `RUST_LOG` sets the
/// level, and the default is `info`. Returns an error when one is installed.
pub fn init() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use tracing_subscriber::EnvFilter;
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(RedactingMakeWriter::new(io::stderr))
        .try_init()
}
