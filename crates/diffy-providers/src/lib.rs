//! Provider-neutral contracts for pull requests, revisions, discussions and
//! publications. A provider adapter implements [`Provider`]. Nothing here
//! names GitHub or OneDev, and nothing here holds a credential.

use diffy_review::Finding;

/// The capabilities differ by provider, server version and permission.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Capabilities {
    pub inline_comments: bool,
    pub multi_line_comments: bool,
    pub publish_review: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PullRequestRef {
    /// The provider instance, for example a host name.
    pub instance: String,
    /// The repository path on that instance.
    pub repository: String,
    pub number: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PullRequest {
    pub reference: PullRequestRef,
    pub title: String,
    pub open: bool,
}

/// One revision of a pull request: the base and the head commit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Revision {
    pub base: String,
    pub head: String,
}

/// A remote comment thread, anchored to a file and a line range of a revision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Discussion {
    pub remote_id: String,
    pub path: String,
    pub revision: Revision,
    pub lines: std::ops::Range<u32>,
    pub resolved: bool,
    pub comments: Vec<Comment>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Comment {
    pub remote_id: String,
    pub author: String,
    pub body: String,
}

/// What happened to one finding in a publication attempt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Published {
        remote_id: String,
    },
    /// The provider cannot anchor it. It is not lost: it stays local.
    Unsupported {
        reason: String,
    },
    Failed {
        reason: String,
    },
}

/// One attempt to send findings to a pull request, and its reconciled result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Publication {
    pub pull_request: PullRequestRef,
    pub results: Vec<(u64, Outcome)>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ProviderError {
    Unauthorized,
    NotFound,
    Unavailable(String),
}

/// The contract that each provider adapter implements.
pub trait Provider {
    fn capabilities(&self) -> Capabilities;
    fn list_pull_requests(&self, repository: &str) -> Result<Vec<PullRequest>, ProviderError>;
    fn revisions(&self, pr: &PullRequestRef) -> Result<Vec<Revision>, ProviderError>;
    fn discussions(&self, pr: &PullRequestRef) -> Result<Vec<Discussion>, ProviderError>;
    fn publish(
        &self,
        pr: &PullRequestRef,
        revision: &Revision,
        findings: &[Finding],
    ) -> Result<Publication, ProviderError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use diffy_review::{Anchor, Category, Severity};

    /// A fake provider proves that the contract needs no provider detail.
    struct Fake;

    impl Provider for Fake {
        fn capabilities(&self) -> Capabilities {
            Capabilities {
                inline_comments: true,
                ..Default::default()
            }
        }
        fn list_pull_requests(&self, repository: &str) -> Result<Vec<PullRequest>, ProviderError> {
            Ok(vec![PullRequest {
                reference: PullRequestRef {
                    instance: "h".into(),
                    repository: repository.into(),
                    number: 1,
                },
                title: "t".into(),
                open: true,
            }])
        }
        fn revisions(&self, _: &PullRequestRef) -> Result<Vec<Revision>, ProviderError> {
            Ok(vec![Revision {
                base: "b".into(),
                head: "h".into(),
            }])
        }
        fn discussions(&self, _: &PullRequestRef) -> Result<Vec<Discussion>, ProviderError> {
            Ok(vec![])
        }
        fn publish(
            &self,
            pr: &PullRequestRef,
            _: &Revision,
            findings: &[Finding],
        ) -> Result<Publication, ProviderError> {
            Ok(Publication {
                pull_request: pr.clone(),
                results: findings
                    .iter()
                    .map(|f| {
                        (
                            f.id.0,
                            Outcome::Published {
                                remote_id: "r".into(),
                            },
                        )
                    })
                    .collect(),
            })
        }
    }

    #[test]
    fn a_provider_works_through_the_neutral_contract() {
        fn run(p: &dyn Provider) -> Publication {
            let pr = p.list_pull_requests("o/r").unwrap().remove(0).reference;
            let rev = p.revisions(&pr).unwrap().remove(0);
            let f = Finding {
                id: diffy_review::FindingId(7),
                anchor: Anchor {
                    path: "a".into(),
                    revision: "h".into(),
                    lines: 0..1,
                },
                severity: Severity::Nit,
                category: Category::Style,
                notes: "n".into(),
                resolved: false,
            };
            p.publish(&pr, &rev, &[f]).unwrap()
        }
        let out = run(&Fake);
        assert_eq!(out.results[0].0, 7);
        assert!(Fake.capabilities().inline_comments);
    }
}
