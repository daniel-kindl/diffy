//! Read-only Git access with gix (ADR 0005).
//!
//! Each repository opens isolated, so no system, global or environment
//! configuration applies. The `filter` sections are removed in memory, so no
//! filter driver command runs. Nothing here writes to the repository.

pub mod watch;

use gix::bstr::{BString, ByteSlice};
use gix::object::tree::diff::Change as TreeChange;
use std::path::Path;

#[derive(Debug)]
pub struct GitError(pub String);

impl std::fmt::Display for GitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for GitError {}

fn err(e: impl std::fmt::Display) -> GitError {
    GitError(e.to_string())
}

/// Where HEAD points.
#[derive(Debug, PartialEq, Eq)]
pub enum Head {
    Branch(String),
    Detached(String),
    /// A branch with no commit yet.
    Unborn(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ChangeKind {
    Added,
    Deleted,
    Modified,
    /// An exact rename: the content is identical.
    Renamed {
        from: String,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub struct Change {
    pub path: String,
    pub kind: ChangeKind,
}

pub struct Repo {
    inner: gix::Repository,
}

impl Repo {
    /// Discover the repository at or above `path` and open it isolated.
    pub fn discover(path: &Path) -> Result<Self, GitError> {
        let opts = gix::open::Options::isolated();
        let mut inner = gix::ThreadSafeRepository::discover_with_environment_overrides_opts(
            path,
            gix::discover::upwards::Options::default(),
            move |_, _| opts,
        )
        .map_err(err)?
        .to_thread_local();
        {
            let mut config = inner.config_snapshot_mut();
            // Remove each `filter` section, with or without a name.
            let names: Vec<_> = config
                .sections_by_name("filter")
                .into_iter()
                .flatten()
                .map(|s| s.header().subsection_name().map(ToOwned::to_owned))
                .collect();
            for n in names {
                config.remove_section("filter", n.as_ref().map(|n| n.as_bstr()));
            }
        }
        Ok(Self { inner })
    }

    pub fn is_bare(&self) -> bool {
        self.inner.is_bare()
    }

    pub fn head(&self) -> Result<Head, GitError> {
        let head = self.inner.head().map_err(err)?;
        let name = head
            .referent_name()
            .map(|n| n.shorten().to_str_lossy().into_owned());
        Ok(match (name, head.id()) {
            (Some(n), Some(_)) => Head::Branch(n),
            (Some(n), None) => Head::Unborn(n),
            (None, Some(id)) => Head::Detached(id.to_string()),
            (None, None) => Head::Unborn(String::new()),
        })
    }

    fn tree(&self, rev: &str) -> Result<gix::Tree<'_>, GitError> {
        self.inner
            .rev_parse_single(rev)
            .map_err(err)?
            .object()
            .map_err(err)?
            .peel_to_tree()
            .map_err(err)
    }

    /// The changed files between two revisions, with exact renames only.
    pub fn changes(&self, base: &str, head: &str) -> Result<Vec<Change>, GitError> {
        let (a, b) = (self.tree(base)?, self.tree(head)?);
        let mut platform = a.changes().map_err(err)?;
        platform.options(|o| {
            o.track_rewrites(Some(gix::diff::Rewrites {
                copies: None,
                percentage: None,
                limit: 0,
                track_empty: false,
            }));
        });
        let mut out = Vec::new();
        platform
            .for_each_to_obtain_tree(&b, |change| -> gix::Result<_> {
                let entry = |m: gix::object::tree::EntryMode| m.is_blob() || m.is_link();
                match change {
                    TreeChange::Addition {
                        location,
                        entry_mode,
                        ..
                    } if entry(entry_mode) => {
                        out.push(Change {
                            path: location.to_string(),
                            kind: ChangeKind::Added,
                        });
                    }
                    TreeChange::Deletion {
                        location,
                        entry_mode,
                        ..
                    } if entry(entry_mode) => {
                        out.push(Change {
                            path: location.to_string(),
                            kind: ChangeKind::Deleted,
                        });
                    }
                    TreeChange::Modification {
                        location,
                        entry_mode,
                        ..
                    } if entry(entry_mode) => {
                        out.push(Change {
                            path: location.to_string(),
                            kind: ChangeKind::Modified,
                        });
                    }
                    TreeChange::Rewrite {
                        source_location,
                        location,
                        ..
                    } => out.push(Change {
                        path: location.to_string(),
                        kind: ChangeKind::Renamed {
                            from: source_location.to_string(),
                        },
                    }),
                    _ => {}
                }
                Ok(std::ops::ControlFlow::Continue(()))
            })
            .map_err(err)?;
        out.sort_by(|x, y| x.path.cmp(&y.path));
        Ok(out)
    }

    /// The raw bytes of a file at a revision, or `None` when it is absent.
    pub fn read_file(&self, rev: &str, path: &str) -> Result<Option<Vec<u8>>, GitError> {
        let tree = self.tree(rev)?;
        let Some(entry) = tree.lookup_entry_by_path(path).map_err(err)? else {
            return Ok(None);
        };
        let obj = entry.object().map_err(err)?;
        Ok(Some(obj.data.clone()))
    }
}

/// A path as the bytes Git stores.
pub fn path_bytes(p: &str) -> BString {
    BString::from(p)
}
