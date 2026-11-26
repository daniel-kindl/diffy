//! The review domain: sessions, baselines and file review state.
//!
//! A session keeps the source state (what the code is) apart from the review
//! state (what the person did). A change of the source never edits the review
//! state, and a review action never edits the source.

use diffy_comparison::ContentId;
use std::collections::BTreeMap;

/// The identity of one repository: its canonical root path or its remote URL.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RepositoryId(pub String);

/// The revision that a session compares against.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Baseline {
    Head,
    Commit(String),
    Branch(String),
    PullRequestBase(String),
    LastReviewedSnapshot(String),
}

/// What the code is: the revision under review. Only the source changes it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceState {
    pub head_revision: String,
}

/// Whether the person reviewed a file at its current content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileReview {
    /// The person marked the file reviewed at this content.
    Reviewed(ContentId),
    NotReviewed,
}

/// What the person did. Only a review action changes it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReviewState {
    files: BTreeMap<String, ContentId>,
    findings: Vec<Finding>,
    next_id: u64,
    checkpoints: Vec<Checkpoint>,
}

impl ReviewState {
    pub fn file(&self, path: &str) -> FileReview {
        self.files
            .get(path)
            .map_or(FileReview::NotReviewed, |id| FileReview::Reviewed(*id))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewSession {
    repository: RepositoryId,
    baseline: Baseline,
    source: SourceState,
    review: ReviewState,
}

impl ReviewSession {
    pub fn new(repository: RepositoryId, baseline: Baseline, source: SourceState) -> Self {
        Self {
            repository,
            baseline,
            source,
            review: ReviewState::default(),
        }
    }

    pub fn repository(&self) -> &RepositoryId {
        &self.repository
    }

    pub fn baseline(&self) -> &Baseline {
        &self.baseline
    }

    pub fn source(&self) -> &SourceState {
        &self.source
    }

    pub fn review(&self) -> &ReviewState {
        &self.review
    }

    /// The source moved to a new revision. The review state stays as it is.
    pub fn update_source(&mut self, source: SourceState) {
        self.source = source;
    }

    /// The person marks a file reviewed at its current content.
    pub fn mark_reviewed(&mut self, path: &str, content: ContentId) {
        self.review.files.insert(path.to_owned(), content);
    }

    /// A file needs review when it was never reviewed, or when its content
    /// differs from the content that was reviewed.
    pub fn needs_review(&self, path: &str, current: ContentId) -> bool {
        self.review.file(path) != FileReview::Reviewed(current)
    }
}

/// How serious a finding is (Q31).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Blocker,
    Major,
    Minor,
    Nit,
}

/// What kind of problem a finding is (Q31).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Correctness,
    Security,
    Performance,
    Design,
    Tests,
    Docs,
    Style,
}

/// A stable local ID. It never changes after the finding is created.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FindingId(pub u64);

/// The lines a finding points to: one file, one revision, canonical
/// coordinates. Lines are 0-based, and the range end is exclusive.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Anchor {
    pub path: String,
    pub revision: String,
    pub lines: std::ops::Range<u32>,
}

/// The note of the reviewer. The original anchor never changes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    pub id: FindingId,
    pub anchor: Anchor,
    pub severity: Severity,
    pub category: Category,
    pub notes: String,
    pub resolved: bool,
}

/// An edit to the reviewer's own fields. It cannot name the source or the anchor.
#[derive(Clone, Debug, Default)]
pub struct FindingEdit {
    pub severity: Option<Severity>,
    pub category: Option<Category>,
    pub notes: Option<String>,
}

impl Anchor {
    /// Move the anchor to a later revision with the M0 mapping (Q45). The
    /// result is `None` (outdated) when any line of the range is outdated or
    /// the mapped lines are not one consecutive range. The anchor is not
    /// changed: a finding keeps its original position.
    pub fn map_to(&self, old: &[u8], new: &[u8]) -> Option<std::ops::Range<u32>> {
        if self.lines.is_empty() {
            return None;
        }
        let map = diffy_comparison::anchor::map_anchors(old, new);
        let mut mapped = Vec::new();
        for l in self.lines.clone() {
            mapped.push((*map.get(l as usize)?)? as u32);
        }
        mapped
            .windows(2)
            .all(|w| w[1] == w[0] + 1)
            .then(|| mapped[0]..mapped[mapped.len() - 1] + 1)
    }
}

/// A recorded point: the content of each file the person had reviewed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Checkpoint {
    pub number: u32,
    pub files: BTreeMap<String, ContentId>,
}

impl ReviewSession {
    /// The explicit action that creates a checkpoint (Q30). It records the
    /// files that are reviewed now. Nothing creates one by itself.
    pub fn create_checkpoint(&mut self) -> &Checkpoint {
        let number = self.review.checkpoints.len() as u32 + 1;
        let files = self.review.files.clone();
        self.review.checkpoints.push(Checkpoint { number, files });
        &self.review.checkpoints[number as usize - 1]
    }

    pub fn last_checkpoint(&self) -> Option<&Checkpoint> {
        self.review.checkpoints.last()
    }

    /// The files that need a re-review since the last checkpoint: each file
    /// whose current content differs from the checkpoint, or that the
    /// checkpoint does not have. A file with the same content keeps its status.
    /// With no checkpoint, each file needs review.
    pub fn files_to_re_review(&self, current: &BTreeMap<String, ContentId>) -> Vec<String> {
        let reviewed = self.last_checkpoint().map(|c| &c.files);
        current
            .iter()
            .filter(|(path, id)| reviewed.and_then(|r| r.get(*path)) != Some(*id))
            .map(|(path, _)| path.clone())
            .collect()
    }
}

impl ReviewSession {
    pub fn set_resolved(&mut self, id: FindingId, resolved: bool) -> bool {
        match self.review.findings.iter_mut().find(|f| f.id == id) {
            Some(f) => {
                f.resolved = resolved;
                true
            }
            None => false,
        }
    }

    /// The findings as Markdown for an agent (Q28). `selected` picks findings
    /// by ID. With `None`, it takes each unresolved finding. Each finding
    /// shows its file, its 1-based lines, severity, category and notes.
    pub fn findings_markdown(&self, selected: Option<&[FindingId]>) -> String {
        let mut out = String::new();
        for f in self.review.findings.iter().filter(|f| match selected {
            Some(ids) => ids.contains(&f.id),
            None => !f.resolved,
        }) {
            let (a, b) = (f.anchor.lines.start + 1, f.anchor.lines.end);
            let lines = if a >= b {
                format!("{a}")
            } else {
                format!("{a}-{b}")
            };
            out.push_str(&format!(
                "- **{:?}** ({:?}) `{}:{}` at `{}`\n",
                f.severity, f.category, f.anchor.path, lines, f.anchor.revision
            ));
            for line in f.notes.lines() {
                out.push_str(&format!("  {line}\n"));
            }
        }
        out
    }
}

impl ReviewSession {
    /// Add a finding. The session gives it the next stable ID.
    pub fn add_finding(
        &mut self,
        anchor: Anchor,
        severity: Severity,
        category: Category,
        notes: String,
    ) -> FindingId {
        let id = FindingId(self.review.next_id);
        self.review.next_id += 1;
        self.review.findings.push(Finding {
            id,
            anchor,
            severity,
            category,
            notes,
            resolved: false,
        });
        id
    }

    pub fn edit_finding(&mut self, id: FindingId, edit: FindingEdit) -> bool {
        let Some(f) = self.review.findings.iter_mut().find(|f| f.id == id) else {
            return false;
        };
        if let Some(v) = edit.severity {
            f.severity = v;
        }
        if let Some(v) = edit.category {
            f.category = v;
        }
        if let Some(v) = edit.notes {
            f.notes = v;
        }
        true
    }

    pub fn findings(&self) -> &[Finding] {
        &self.review.findings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use diffy_comparison::{Snapshot, Source};

    fn id(s: &str) -> ContentId {
        Snapshot::new(Source::Text("t".into()), s.as_bytes().to_vec()).id()
    }

    fn session() -> ReviewSession {
        ReviewSession::new(
            RepositoryId("/repo".into()),
            Baseline::Branch("main".into()),
            SourceState {
                head_revision: "aaa".into(),
            },
        )
    }

    #[test]
    fn a_session_has_explicit_repository_and_baseline() {
        let s = session();
        assert_eq!(s.repository(), &RepositoryId("/repo".into()));
        assert_eq!(s.baseline(), &Baseline::Branch("main".into()));
    }

    #[test]
    fn a_new_source_does_not_change_the_review_state() {
        let mut s = session();
        s.mark_reviewed("a.rs", id("v1"));
        let review = s.review().clone();
        s.update_source(SourceState {
            head_revision: "bbb".into(),
        });
        assert_eq!(s.source().head_revision, "bbb");
        assert_eq!(s.review(), &review);
    }

    #[test]
    fn a_review_action_does_not_change_the_source() {
        let mut s = session();
        let source = s.source().clone();
        s.mark_reviewed("a.rs", id("v1"));
        assert_eq!(s.source(), &source);
    }

    #[test]
    fn changed_content_needs_review_again() {
        let mut s = session();
        assert!(s.needs_review("a.rs", id("v1")));
        s.mark_reviewed("a.rs", id("v1"));
        assert!(!s.needs_review("a.rs", id("v1")));
        assert!(s.needs_review("a.rs", id("v2")));
    }

    fn anchor() -> Anchor {
        Anchor {
            path: "a.rs".into(),
            revision: "aaa".into(),
            lines: 3..5,
        }
    }

    #[test]
    fn findings_get_stable_ids_that_survive_edits_and_deletes_of_others() {
        let mut s = session();
        let a = s.add_finding(anchor(), Severity::Major, Category::Security, "x".into());
        let b = s.add_finding(anchor(), Severity::Nit, Category::Style, "y".into());
        assert_ne!(a, b);
        assert!(s.edit_finding(
            a,
            FindingEdit {
                notes: Some("z".into()),
                ..Default::default()
            }
        ));
        assert_eq!(s.findings()[0].id, a);
        assert_eq!(s.findings()[0].notes, "z");
        assert_eq!(s.findings()[0].severity, Severity::Major);
        assert!(!s.edit_finding(FindingId(99), FindingEdit::default()));
    }

    #[test]
    fn editing_a_finding_keeps_its_anchor_and_the_source() {
        let mut s = session();
        let source = s.source().clone();
        let id = s.add_finding(
            anchor(),
            Severity::Blocker,
            Category::Correctness,
            "n".into(),
        );
        s.edit_finding(
            id,
            FindingEdit {
                severity: Some(Severity::Minor),
                category: Some(Category::Docs),
                notes: None,
            },
        );
        assert_eq!(s.findings()[0].anchor, anchor());
        assert_eq!(s.source(), &source);
    }

    #[test]
    fn an_anchor_maps_to_a_later_revision_or_is_outdated() {
        let old = b"a\nb\nc\nd\n";
        let a = Anchor {
            path: "f".into(),
            revision: "r1".into(),
            lines: 1..3,
        };
        assert_eq!(a.map_to(old, b"x\na\nb\nc\nd\n"), Some(2..4));
        assert_eq!(a.map_to(old, b"a\nB\nc\nd\n"), None);
        assert_eq!(a.lines, 1..3, "the original anchor does not change");
    }

    #[test]
    fn a_checkpoint_keeps_unchanged_files_and_flags_changed_ones() {
        let mut s = session();
        let current: BTreeMap<String, ContentId> =
            [("a.rs".to_owned(), id("a1")), ("b.rs".to_owned(), id("b1"))].into();
        assert_eq!(s.files_to_re_review(&current), vec!["a.rs", "b.rs"]);
        s.mark_reviewed("a.rs", id("a1"));
        s.mark_reviewed("b.rs", id("b1"));
        assert_eq!(s.create_checkpoint().number, 1);
        assert!(s.files_to_re_review(&current).is_empty());
        let later: BTreeMap<String, ContentId> = [
            ("a.rs".to_owned(), id("a1")),
            ("b.rs".to_owned(), id("b2")),
            ("c.rs".to_owned(), id("c1")),
        ]
        .into();
        assert_eq!(s.files_to_re_review(&later), vec!["b.rs", "c.rs"]);
    }

    #[test]
    fn no_checkpoint_exists_until_the_user_creates_one() {
        let mut s = session();
        s.mark_reviewed("a.rs", id("a1"));
        assert!(s.last_checkpoint().is_none());
    }

    #[test]
    fn markdown_copies_unresolved_or_selected_findings() {
        let mut s = session();
        let a = s.add_finding(
            anchor(),
            Severity::Major,
            Category::Security,
            "line one\nline two".into(),
        );
        let b = s.add_finding(anchor(), Severity::Nit, Category::Style, "tidy".into());
        s.set_resolved(b, true);
        let md = s.findings_markdown(None);
        assert_eq!(
            md,
            "- **Major** (Security) `a.rs:4-5` at `aaa`\n  line one\n  line two\n"
        );
        let md = s.findings_markdown(Some(&[b]));
        assert!(md.contains("tidy") && !md.contains("line one"));
        let _ = a;
    }
}
