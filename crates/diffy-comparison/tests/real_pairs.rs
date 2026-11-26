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

mod common;

use common::{git, oracle};
use diffy_comparison::canonical::canonical_hunks;
use std::{fs, path::Path};

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
