//! GitHub review comments as neutral discussions (G-03).
//!
//! REST lists the review comments of a pull request. A reply has an
//! `in_reply_to_id` that points to the first comment of its thread.
//! GraphQL `pullRequest.reviewThreads` gives whether a thread is resolved, and
//! each comment has a `node_id` that links the two (Q48, Q69). This module
//! only maps JSON that was already read. It does no network access.

use crate::{Comment, Discussion, Revision, Side};
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Deserialize)]
struct User {
    login: String,
}

/// One review comment of the REST API (the fields that Diffy uses).
#[derive(Debug, Deserialize)]
pub struct RestComment {
    pub id: u64,
    pub node_id: String,
    pub path: String,
    pub commit_id: String,
    #[serde(default)]
    pub original_commit_id: Option<String>,
    /// The line at the current head. `null` when the comment is outdated.
    #[serde(default)]
    pub line: Option<u32>,
    #[serde(default)]
    pub original_line: Option<u32>,
    #[serde(default)]
    pub start_line: Option<u32>,
    #[serde(default)]
    pub original_start_line: Option<u32>,
    #[serde(default)]
    pub side: Option<String>,
    #[serde(default)]
    pub in_reply_to_id: Option<u64>,
    user: Option<User>,
    pub body: String,
}

pub fn parse_review_comments(json: &str) -> Result<Vec<RestComment>, serde_json::Error> {
    serde_json::from_str(json)
}

/// Group the comments into threads. `resolved` maps the `node_id` of a
/// thread's first comment to the `isResolved` value from GraphQL. A thread
/// that GraphQL does not list is open. Threads come in the order of their
/// first comment.
pub fn discussions(comments: &[RestComment], resolved: &HashMap<String, bool>) -> Vec<Discussion> {
    let by_id: HashMap<u64, &RestComment> = comments.iter().map(|c| (c.id, c)).collect();
    let root_of = |c: &RestComment| -> u64 {
        let mut cur = c;
        // A thread is flat on GitHub, and the guard stops a cycle in bad data.
        for _ in 0..comments.len() {
            match cur.in_reply_to_id.and_then(|id| by_id.get(&id)) {
                Some(parent) => cur = parent,
                None => break,
            }
        }
        cur.id
    };
    let mut threads: BTreeMap<u64, Vec<&RestComment>> = BTreeMap::new();
    for c in comments {
        threads.entry(root_of(c)).or_default().push(c);
    }
    let mut out = Vec::new();
    for (root_id, mut members) in threads {
        members.sort_by_key(|c| c.id);
        let Some(root) = by_id
            .get(&root_id)
            .copied()
            .or_else(|| members.first().copied())
        else {
            continue;
        };
        let outdated = root.line.is_none();
        let (end, start, revision) = if outdated {
            (
                root.original_line,
                root.original_start_line,
                root.original_commit_id.as_deref(),
            )
        } else {
            (root.line, root.start_line, None)
        };
        let end = end.unwrap_or(1).max(1);
        let start = start.unwrap_or(end).clamp(1, end);
        out.push(Discussion {
            remote_id: root.node_id.clone(),
            path: root.path.clone(),
            revision: Revision {
                // The REST comment names one commit. The base is not known here.
                base: String::new(),
                head: revision.unwrap_or(&root.commit_id).to_owned(),
            },
            lines: (start - 1)..end,
            side: if root.side.as_deref() == Some("LEFT") {
                Side::Base
            } else {
                Side::Head
            },
            resolved: resolved.get(&root.node_id).copied().unwrap_or(false),
            outdated,
            comments: members
                .iter()
                .map(|c| Comment {
                    remote_id: c.node_id.clone(),
                    author: c
                        .user
                        .as_ref()
                        .map_or_else(|| "ghost".to_owned(), |u| u.login.clone()),
                    body: c.body.clone(),
                })
                .collect(),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const JSON: &str = r#"[
      {"id": 10, "node_id": "PRRC_a", "path": "src/a.rs", "commit_id": "head1",
       "original_commit_id": "orig1", "line": 12, "original_line": 12,
       "start_line": 10, "original_start_line": 10, "side": "RIGHT",
       "in_reply_to_id": null, "user": {"login": "alice"}, "body": "Why?"},
      {"id": 11, "node_id": "PRRC_b", "path": "src/a.rs", "commit_id": "head1",
       "original_commit_id": "orig1", "line": 12, "original_line": 12,
       "side": "RIGHT", "in_reply_to_id": 10, "user": {"login": "bob"}, "body": "Because."},
      {"id": 20, "node_id": "PRRC_c", "path": "old.rs", "commit_id": "head1",
       "original_commit_id": "orig0", "line": null, "original_line": 3,
       "side": "LEFT", "in_reply_to_id": null, "user": null, "body": "Gone"}
    ]"#;

    #[test]
    fn replies_join_their_thread_and_lines_are_zero_based() {
        let c = parse_review_comments(JSON).unwrap();
        let d = discussions(&c, &HashMap::new());
        assert_eq!(d.len(), 2);
        assert_eq!(d[0].remote_id, "PRRC_a");
        assert_eq!(d[0].lines, 9..12);
        assert_eq!(d[0].side, Side::Head);
        assert_eq!(d[0].revision.head, "head1");
        assert_eq!(
            d[0].comments
                .iter()
                .map(|c| c.author.as_str())
                .collect::<Vec<_>>(),
            ["alice", "bob"]
        );
        assert!(!d[0].outdated && !d[0].resolved);
    }

    #[test]
    fn an_outdated_thread_uses_its_original_position_and_commit() {
        let c = parse_review_comments(JSON).unwrap();
        let d = discussions(&c, &HashMap::new());
        assert!(d[1].outdated);
        assert_eq!(d[1].lines, 2..3);
        assert_eq!(d[1].revision.head, "orig0");
        assert_eq!(d[1].side, Side::Base);
        assert_eq!(
            d[1].comments[0].author, "ghost",
            "a deleted user has no login"
        );
    }

    #[test]
    fn graphql_marks_a_thread_resolved_by_its_first_comment() {
        let c = parse_review_comments(JSON).unwrap();
        let resolved = HashMap::from([("PRRC_a".to_owned(), true)]);
        let d = discussions(&c, &resolved);
        assert!(d[0].resolved);
        assert!(!d[1].resolved);
    }

    #[test]
    fn a_reply_to_an_unknown_comment_starts_its_own_thread_and_a_cycle_ends() {
        let json = r#"[
          {"id": 1, "node_id": "n1", "path": "p", "commit_id": "h", "line": 1, "in_reply_to_id": 99, "body": "x"},
          {"id": 2, "node_id": "n2", "path": "p", "commit_id": "h", "line": 1, "in_reply_to_id": 3, "body": "y"},
          {"id": 3, "node_id": "n3", "path": "p", "commit_id": "h", "line": 1, "in_reply_to_id": 2, "body": "z"}
        ]"#;
        let d = discussions(&parse_review_comments(json).unwrap(), &HashMap::new());
        assert!(!d.is_empty());
        assert_eq!(d.iter().map(|t| t.comments.len()).sum::<usize>(), 3);
    }

    #[test]
    fn bad_json_is_an_error() {
        assert!(parse_review_comments("{").is_err());
    }
}
