//! Parity with `git diff --histogram --indent-heuristic` on real file pairs.
//!
//! The pairs come from the history of the repository that holds this crate:
//! each file that a recent commit changed, before and after. A checkout with
//! no history (a shallow CI clone) has no pair, and the test says so and
//! passes. The Q43 pairs (git/git) are not used here.

use diffy_comparison::canonical::{Hunk, canonical_hunks};
use std::{fs, path::Path, process::Command};

fn run(dir: &Path, args: &[&str]) -> Option<std::process::Output> {
    Command::new("git")
        .args(args)
        .current_dir(dir)
        .env(
            "GIT_CONFIG_GLOBAL",
            if cfg!(windows) { "NUL" } else { "/dev/null" },
        )
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .ok()
}

fn git(dir: &Path, args: &[&str]) -> Option<Vec<u8>> {
    let out = run(dir, args)?;
    out.status.success().then_some(out.stdout)
}

fn range(start: u32, count: u32) -> std::ops::Range<u32> {
    let s = if count == 0 { start } else { start - 1 };
    s..s + count
}

fn oracle(tmp: &Path, a: &[u8], b: &[u8]) -> Vec<Hunk> {
    fs::write(tmp.join("a"), a).unwrap();
    fs::write(tmp.join("b"), b).unwrap();
    // `git diff --no-index` exits with 1 when the files differ, so the exit
    // code is not checked here.
    let out = run(
        tmp,
        &[
            "diff",
            "--no-index",
            "--histogram",
            "--indent-heuristic",
            "-U0",
            "--no-color",
            "--no-ext-diff",
            "a",
            "b",
        ],
    )
    .map(|o| o.stdout)
    .unwrap_or_default();
    String::from_utf8_lossy(&out)
        .lines()
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

#[test]
fn matches_git_on_the_files_of_recent_commits() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"));
    let Some(log) = git(repo, &["log", "-n", "60", "--format=%H"]) else {
        eprintln!("no Git history: the real pair test is skipped");
        return;
    };
    let commits: Vec<String> = String::from_utf8_lossy(&log)
        .lines()
        .map(str::to_owned)
        .collect();
    let tmp = std::env::temp_dir().join(format!("diffy-real-{}", std::process::id()));
    fs::create_dir_all(&tmp).unwrap();
    let mut pairs = 0;
    for c in &commits {
        let Some(names) = git(
            repo,
            &["diff", "--name-only", "--no-renames", &format!("{c}~1"), c],
        ) else {
            continue; // the root commit, or a shallow boundary
        };
        for name in String::from_utf8_lossy(&names).lines().take(8) {
            let (Some(a), Some(b)) = (
                git(repo, &["show", &format!("{c}~1:{name}")]),
                git(repo, &["show", &format!("{c}:{name}")]),
            ) else {
                continue; // added or deleted
            };
            if a.len() > 1 << 20 || b.len() > 1 << 20 || a.contains(&0) || b.contains(&0) {
                continue; // large or binary
            }
            assert_eq!(
                canonical_hunks(&a, &b),
                oracle(&tmp, &a, &b),
                "{name} at {c}"
            );
            pairs += 1;
        }
    }
    fs::remove_dir_all(&tmp).ok();
    eprintln!("compared {pairs} real pairs");
    if commits.len() > 5 {
        assert!(pairs > 0, "the history has commits but no comparable pair");
    }
}
