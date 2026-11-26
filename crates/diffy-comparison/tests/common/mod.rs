//! Shared by the parity tests: the Q43 oracle (`git diff --histogram`).
#![allow(dead_code)]

use diffy_comparison::canonical::Hunk;
use std::{fs, path::Path, process::Command};

pub fn run(dir: &Path, args: &[&str]) -> Option<std::process::Output> {
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

pub fn git(dir: &Path, args: &[&str]) -> Option<Vec<u8>> {
    let out = run(dir, args)?;
    out.status.success().then_some(out.stdout)
}

/// The Q43 oracle: `-U3 --inter-hunk-context=0`, with the change groups read
/// from the body of the diff. `-U0` is not used, because Git trims the common
/// tail when the context is 0, which can place a group at other lines.
pub fn oracle(tmp: &Path, a: &[u8], b: &[u8]) -> Vec<Hunk> {
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

pub fn parse_groups(diff: &str) -> Vec<Hunk> {
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
