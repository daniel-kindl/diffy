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
}
