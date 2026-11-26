//! Parity with `git diff --histogram --indent-heuristic` on real file pairs.
//!
//! The pairs come from the history of the repository that holds this crate:
//! each file that a recent commit changed, before and after. A checkout with
//! no history (a shallow CI clone) has no pair, and the test says so and
//! passes.
//!
//! To use other pairs, set `DIFFY_PARITY_REPO` to a Git directory (a bare
//! clone works) and `DIFFY_PARITY_COMMITS` to the number of commits. The default
//! is this repository and 60 commits. A clone of git/git gives the kind of pairs
//! that Q43 used.

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

/// The Q43 oracle: `-U3 --inter-hunk-context=0`, with the change groups read
/// from the body of the diff. `-U0` is not used, because Git trims the common
/// tail when the context is 0, which can place a group at other lines.
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
            "-U3",
            "--inter-hunk-context=0",
            "--indent-heuristic",
            "--no-color",
            "--no-ext-diff",
            "--no-renames",
            "--no-textconv",
            "a",
            "b",
        ],
    )
    .map(|o| o.stdout)
    .unwrap_or_default();
    parse_groups(&String::from_utf8_lossy(&out))
}

fn parse_groups(diff: &str) -> Vec<Hunk> {
    let (mut groups, mut open): (Vec<Hunk>, Option<(u32, u32)>) = (Vec::new(), None);
    let (mut old, mut new) = (0u32, 0u32);
    let mut in_hunk = false;
    let flush = |groups: &mut Vec<Hunk>, open: &mut Option<(u32, u32)>, old: u32, new: u32| {
        if let Some((bs, hs)) = open.take() {
            groups.push(Hunk {
                base: bs..old,
                head: hs..new,
            });
        }
    };
    for line in diff.lines() {
        if let Some(rest) = line.strip_prefix("@@ ") {
            flush(&mut groups, &mut open, old, new);
            in_hunk = true;
            let mut it = rest.split_whitespace();
            let parse = |s: &str| {
                let s = &s[1..];
                let (a, c) = s.split_once(',').unwrap_or((s, "1"));
                let (a, c): (u32, u32) = (a.parse().unwrap(), c.parse().unwrap());
                if c == 0 { a } else { a - 1 }
            };
            old = parse(it.next().unwrap());
            new = parse(it.next().unwrap());
        } else if let (true, Some(c)) = (in_hunk, line.chars().next()) {
            match c {
                ' ' => {
                    flush(&mut groups, &mut open, old, new);
                    old += 1;
                    new += 1;
                }
                '-' => {
                    open.get_or_insert((old, new));
                    old += 1;
                }
                '+' => {
                    open.get_or_insert((old, new));
                    new += 1;
                }
                _ => {}
            }
        }
    }
    flush(&mut groups, &mut open, old, new);
    groups
}

#[test]
fn matches_git_on_the_files_of_recent_commits() {
    let repo_var = std::env::var_os("DIFFY_PARITY_REPO");
    let repo = repo_var
        .as_deref()
        .map_or(Path::new(env!("CARGO_MANIFEST_DIR")), Path::new);
    let count = std::env::var("DIFFY_PARITY_COMMITS").unwrap_or_else(|_| "60".into());
    let Some(log) = git(repo, &["log", "-n", &count, "--format=%H"]) else {
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
    let mut mismatches: Vec<String> = Vec::new();
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
            if canonical_hunks(&a, &b) != oracle(&tmp, &a, &b) {
                mismatches.push(format!("{name} at {c}"));
            }
            pairs += 1;
        }
    }
    fs::remove_dir_all(&tmp).ok();
    eprintln!(
        "compared {pairs} real pairs, {} differ from Git",
        mismatches.len()
    );
    for m in &mismatches {
        eprintln!("  differs: {m}");
    }
    // DIFFY_PARITY_REPORT=1 reports the differences and does not fail, to count them.
    if std::env::var_os("DIFFY_PARITY_REPORT").is_none() {
        assert!(
            mismatches.is_empty(),
            "{} pairs differ from Git",
            mismatches.len()
        );
    }
    if commits.len() > 5 {
        assert!(pairs > 0, "the history has commits but no comparable pair");
    }
}
