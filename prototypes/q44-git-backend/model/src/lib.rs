//! The answer that each backend and the Git oracle give for one operation.
//!
//! Each list is sorted, so two answers compare with `==`.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// The operations of the Q44 charter. Each backend binary takes one of them.
pub const OPERATIONS: &[&str] = &[
    "head",
    "refs",
    "status",
    "index",
    "submodules",
    "tree",
    "diff",
];

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "snake_case", tag = "op")]
pub enum Answer {
    /// `HEAD`: the commit and the branch, or no branch for a detached `HEAD`.
    Head { oid: String, branch: Option<String> },
    /// Each reference under `refs/` and the object that it points to, unpeeled.
    Refs { refs: Vec<(String, String)> },
    /// The two-letter status of `git status --porcelain=v2`, with index renames.
    Status { entries: Vec<StatusEntry> },
    /// Each index entry, with the SHA-256 of the blob content.
    Index { entries: Vec<Entry> },
    /// Each submodule: path, commit in the index and checked-out commit.
    Submodules { entries: Vec<Submodule> },
    /// Each entry of a commit tree, recursive, with the SHA-256 of each blob.
    Tree { rev: String, entries: Vec<Entry> },
    /// The changes between two commits, with rename detection at 50 %.
    Diff {
        from: String,
        to: String,
        entries: Vec<DiffEntry>,
    },
    /// The backend gave an error.
    Error { message: String },
}

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct StatusEntry {
    pub path: String,
    /// `.`, `M`, `T`, `A`, `D`, `R` or `?`.
    pub index: char,
    pub worktree: char,
    pub orig: Option<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct Entry {
    pub path: String,
    /// The octal mode, for example `100644`.
    pub mode: String,
    pub oid: String,
    pub stage: u8,
    /// The SHA-256 of the blob content. None for a gitlink.
    pub content_sha256: Option<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct Submodule {
    pub path: String,
    pub index_oid: Option<String>,
    pub head_oid: Option<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct DiffEntry {
    /// `A`, `D`, `M`, `T` or `R`.
    pub status: char,
    pub path: String,
    pub orig: Option<String>,
}

/// A lossless text form of a path: UTF-8 stays, a backslash becomes `\\`,
/// and each control byte or invalid UTF-8 byte becomes `\xNN`.
pub fn path_text(bytes: &[u8]) -> String {
    let mut text = String::new();
    for chunk in bytes.utf8_chunks() {
        for c in chunk.valid().chars() {
            match c {
                '\\' => text.push_str("\\\\"),
                c if (c as u32) < 0x20 || c == '\x7f' => {
                    text.push_str(&format!("\\x{:02x}", c as u32))
                }
                c => text.push(c),
            }
        }
        for b in chunk.invalid() {
            text.push_str(&format!("\\x{b:02x}"));
        }
    }
    text
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// The arguments of a backend binary: `<repo> <operation> [<rev> [<rev>]]`.
pub struct Args {
    pub repo: std::path::PathBuf,
    pub op: String,
    pub revs: Vec<String>,
}

pub fn args() -> Args {
    let mut args = std::env::args_os().skip(1);
    let usage = "usage: <backend> <repo> <head|refs|status|index|submodules|tree REV|diff FROM TO>";
    let repo = args.next().expect(usage).into();
    let op = args.next().expect(usage).into_string().expect(usage);
    let revs: Vec<String> = args.map(|a| a.into_string().expect(usage)).collect();
    let wanted = match op.as_str() {
        "tree" => 1,
        "diff" => 2,
        _ => 0,
    };
    assert!(
        OPERATIONS.contains(&op.as_str()) && revs.len() == wanted,
        "{usage}"
    );
    Args { repo, op, revs }
}

/// Prints the answer as JSON. An error answer exits with code 2.
pub fn print(answer: Answer) {
    let failed = matches!(answer, Answer::Error { .. });
    println!("{}", serde_json::to_string(&answer).expect("serialize"));
    if failed {
        std::process::exit(2);
    }
}

/// Sorts each list of the answer.
pub fn sorted(mut answer: Answer) -> Answer {
    match &mut answer {
        Answer::Refs { refs } => refs.sort(),
        Answer::Status { entries } => entries.sort(),
        Answer::Index { entries } | Answer::Tree { entries, .. } => entries.sort(),
        Answer::Submodules { entries } => entries.sort(),
        Answer::Diff { entries, .. } => entries.sort(),
        Answer::Head { .. } | Answer::Error { .. } => {}
    }
    answer
}

/// Merges the head-to-index and the index-to-worktree letters of each path into status entries.
#[derive(Default)]
pub struct StatusBuilder {
    map: std::collections::BTreeMap<String, StatusEntry>,
}

impl StatusBuilder {
    fn entry(&mut self, path: String) -> &mut StatusEntry {
        self.map.entry(path.clone()).or_insert(StatusEntry {
            path,
            index: '.',
            worktree: '.',
            orig: None,
        })
    }
    pub fn index(&mut self, path: String, letter: char, orig: Option<String>) {
        let e = self.entry(path);
        e.index = letter;
        e.orig = orig;
    }
    pub fn worktree(&mut self, path: String, letter: char) {
        self.entry(path).worktree = letter;
    }
    pub fn untracked(&mut self, path: String) {
        let e = self.entry(path);
        e.index = '?';
        e.worktree = '?';
    }
    pub fn finish(self) -> Answer {
        Answer::Status {
            entries: self.map.into_values().collect(),
        }
    }
}
