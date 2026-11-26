//! Parity with `git diff --histogram --indent-heuristic` (ADR 0002, Q43).
use diffy_comparison::canonical::{Hunk, canonical_hunks};
use std::{fs, path::Path, process::Command};

fn range(start: u32, count: u32) -> std::ops::Range<u32> {
    // Git counts from 1, and a zero count names the line before the change.
    let s = if count == 0 { start } else { start - 1 };
    s..s + count
}

fn git_hunks(dir: &Path, a: &[u8], b: &[u8]) -> Vec<Hunk> {
    fs::write(dir.join("a"), a).unwrap();
    fs::write(dir.join("b"), b).unwrap();
    let out = Command::new("git")
        .args([
            "-c",
            "core.autocrlf=false",
            "diff",
            "--no-index",
            "--histogram",
            "--indent-heuristic",
            "-U0",
            "--no-color",
            "--no-ext-diff",
            "a",
            "b",
        ])
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .current_dir(dir)
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    text.lines()
        .filter(|l| l.starts_with("@@ "))
        .map(|l| {
            let mut it = l.split_whitespace().skip(1);
            let parse = |s: &str| {
                let s = &s[1..];
                let (a, b) = s.split_once(',').unwrap_or((s, "1"));
                (a.parse::<u32>().unwrap(), b.parse::<u32>().unwrap())
            };
            let (a0, a1) = parse(it.next().unwrap());
            let (b0, b1) = parse(it.next().unwrap());
            Hunk {
                base: range(a0, a1),
                head: range(b0, b1),
            }
        })
        .collect()
}

fn lines(n: usize, f: impl Fn(usize) -> String) -> Vec<u8> {
    (0..n).map(|i| f(i) + "\n").collect::<String>().into_bytes()
}

#[test]
fn matches_git_on_the_fixtures() {
    let big: Vec<u8> = lines(3000, |i| format!("line {}", i % 7));
    let mut big2 = big.clone();
    big2.splice(100..100, b"inserted\n".iter().copied());
    let cases: Vec<(&str, Vec<u8>, Vec<u8>)> = vec![
        ("empty", b"".to_vec(), b"".to_vec()),
        ("empty to text", b"".to_vec(), b"a\nb\n".to_vec()),
        ("text to empty", b"a\nb\n".to_vec(), b"".to_vec()),
        ("identical", b"a\nb\n".to_vec(), b"a\nb\n".to_vec()),
        (
            "repeated",
            b"a\na\na\nb\na\na\n".to_vec(),
            b"a\na\nb\na\na\na\n".to_vec(),
        ),
        (
            "unicode",
            "žluťoučký\nkůň\n".as_bytes().to_vec(),
            "žluťoučký\nkoň\n".as_bytes().to_vec(),
        ),
        (
            "invalid utf8",
            b"a\xff\nb\n".to_vec(),
            b"a\xfe\nb\n".to_vec(),
        ),
        ("crlf vs lf", b"a\r\nb\r\n".to_vec(), b"a\nb\n".to_vec()),
        ("no final newline", b"a\nb".to_vec(), b"a\nb\n".to_vec()),
        ("no final newline both", b"a\nb".to_vec(), b"a\nc".to_vec()),
        (
            "moved block",
            b"x\n{\n a\n}\ny\n".to_vec(),
            b"y\nx\n{\n a\n}\n".to_vec(),
        ),
        ("large repetitive", big, big2),
    ];
    let dir = std::env::temp_dir().join(format!("diffy-parity-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    for (name, a, b) in cases {
        assert_eq!(
            canonical_hunks(&a, &b),
            git_hunks(&dir, &a, &b),
            "case: {name}"
        );
    }
    fs::remove_dir_all(&dir).ok();
}
