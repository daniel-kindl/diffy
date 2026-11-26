//! The review overview (R-06): counts, unresolved findings, outdated
//! anchors and progress.

use crate::{Finding, ReviewSession, Severity};
use diffy_comparison::ContentId;
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Eq)]
pub struct Overview {
    pub files_total: usize,
    pub files_reviewed: usize,
    pub findings_total: usize,
    pub findings_unresolved: usize,
    pub blockers_unresolved: usize,
    /// Unresolved findings whose anchor cannot map to the current revision.
    pub outdated: usize,
    /// Reviewed files as a whole percent of the changed files. 100 for none.
    pub progress_percent: u32,
}

/// Summarize a session. `current` maps each changed file to its current
/// content identity. `is_outdated` tells whether a finding's anchor no longer
/// maps (the caller uses `Anchor::map_to`).
pub fn overview(
    session: &ReviewSession,
    current: &BTreeMap<String, ContentId>,
    is_outdated: impl Fn(&Finding) -> bool,
) -> Overview {
    let files_total = current.len();
    let files_reviewed = current
        .iter()
        .filter(|(p, id)| !session.needs_review(p, **id))
        .count();
    let findings = session.findings();
    let unresolved: Vec<&Finding> = findings.iter().filter(|f| !f.resolved).collect();
    Overview {
        files_total,
        files_reviewed,
        findings_total: findings.len(),
        findings_unresolved: unresolved.len(),
        blockers_unresolved: unresolved
            .iter()
            .filter(|f| f.severity == Severity::Blocker)
            .count(),
        outdated: unresolved.iter().filter(|f| is_outdated(f)).count(),
        progress_percent: if files_total == 0 {
            100
        } else {
            (files_reviewed * 100 / files_total) as u32
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Anchor, Baseline, Category, RepositoryId, SourceState};
    use diffy_comparison::{Snapshot, Source};

    fn id(s: &str) -> ContentId {
        Snapshot::new(Source::Text("t".into()), s.as_bytes().to_vec()).id()
    }

    fn anchor(path: &str) -> Anchor {
        Anchor {
            path: path.into(),
            revision: "r".into(),
            lines: 0..1,
        }
    }

    #[test]
    fn counts_progress_and_unresolved_findings() {
        let mut s = ReviewSession::new(
            RepositoryId("/r".into()),
            Baseline::Head,
            SourceState {
                head_revision: "r".into(),
            },
        );
        let current: BTreeMap<String, ContentId> =
            [("a".to_owned(), id("a1")), ("b".to_owned(), id("b1"))].into();
        s.mark_reviewed("a", id("a1"));
        s.mark_reviewed("b", id("old"));
        let f1 = s.add_finding(
            anchor("a"),
            Severity::Blocker,
            Category::Security,
            "x".into(),
        );
        let f2 = s.add_finding(anchor("b"), Severity::Nit, Category::Style, "y".into());
        s.add_finding(anchor("b"), Severity::Minor, Category::Docs, "z".into());
        s.set_resolved(f2, true);
        let o = overview(&s, &current, |f| f.id == f1);
        assert_eq!(
            o,
            Overview {
                files_total: 2,
                files_reviewed: 1,
                findings_total: 3,
                findings_unresolved: 2,
                blockers_unresolved: 1,
                outdated: 1,
                progress_percent: 50,
            }
        );
    }

    #[test]
    fn no_changed_files_is_complete() {
        let s = ReviewSession::new(
            RepositoryId("/r".into()),
            Baseline::Head,
            SourceState {
                head_revision: "r".into(),
            },
        );
        assert_eq!(
            overview(&s, &BTreeMap::new(), |_| false).progress_percent,
            100
        );
    }
}
