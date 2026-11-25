//! The Q44 operations with gix. Prints one JSON answer.
//!
//! The repository opens with `gix::open::Options::isolated()`: no system, global or
//! environment configuration, and no `git` executable to find the installation config.
//! `Q44_GIX_DEFAULT_OPEN=1` opens with the default options instead.
//! `Q44_GIX_FILTERS` selects how the backend keeps filter drivers from running: `strip`
//! (default), `trust` (no change) or `section-filter`. See `open()`.

use gix::diff::tree_with_rewrites::Change;
use gix::objs::tree::EntryKind;
use gix::status::index_worktree::Item as WtItem;
use gix_status_types::*;
use model::{Answer, DiffEntry, Entry, StatusBuilder, Submodule, path_text, sha256_hex};

type R<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

mod gix_status_types {
    pub use gix::status::plumbing::index_as_worktree::{Change as WtChange, EntryStatus};
}

fn main() {
    let args = model::args();
    let answer = run(&args).unwrap_or_else(|e| Answer::Error {
        message: format!("{e:?}"),
    });
    model::print(model::sorted(answer));
}

fn open(path: &std::path::Path) -> R<gix::Repository> {
    let mut options = if std::env::var_os("Q44_GIX_DEFAULT_OPEN").is_some() {
        gix::open::Options::default()
    } else {
        gix::open::Options::isolated()
    };
    // gix runs the filter drivers (`filter.<name>.clean`) of each configuration section that the
    // section filter accepts. The default filter accepts the repository configuration of a
    // repository that the user owns, so the repository configuration can start a command.
    let mode = std::env::var("Q44_GIX_FILTERS").unwrap_or_else(|_| "strip".into());
    if mode == "section-filter" {
        // Refuses each repository-local section. gix then lacks `core.symlinks` and probes the
        // file system with a symbolic link in the worktree: a write. Kept to show it.
        options = options.filter_config_section(|meta| {
            meta.source.kind() != gix::config::source::Kind::Repository
        });
    }
    let mut repo = gix::open_opts(path, options)?;
    if mode == "strip" {
        // Removes each `filter` section from the configuration in memory, so no driver is left.
        // Nothing is written: `commit()` only rereads the values.
        let mut config = repo.config_snapshot_mut();
        let names: Vec<Option<gix::bstr::BString>> = config
            .sections_by_name("filter")
            .into_iter()
            .flatten()
            .map(|s| s.header().subsection_name().map(ToOwned::to_owned))
            .collect();
        for name in &names {
            while config
                .remove_section("filter", name.as_ref().map(AsRef::as_ref))
                .is_some()
            {}
        }
        config.commit()?;
    } else if mode != "trust" && mode != "section-filter" {
        return Err(
            format!("Q44_GIX_FILTERS must be strip, trust or section-filter, not {mode}").into(),
        );
    }
    Ok(repo)
}

fn run(args: &model::Args) -> R<Answer> {
    let repo = open(&args.repo)?;
    Ok(match args.op.as_str() {
        "head" => Answer::Head {
            oid: repo.head_id()?.to_string(),
            branch: repo.head_name()?.map(|n| path_text(n.as_bstr())),
        },
        "refs" => {
            let mut refs = Vec::new();
            for r in repo.references()?.all()? {
                let r = r?;
                if let Some(id) = r.try_id() {
                    refs.push((path_text(r.name().as_bstr()), id.to_string()));
                }
            }
            Answer::Refs { refs }
        }
        "status" => status(&repo)?,
        "index" => {
            let index = repo.index_or_empty()?;
            let mut entries = Vec::new();
            for e in index.entries() {
                let blob = e.mode != gix::index::entry::Mode::COMMIT;
                entries.push(Entry {
                    path: path_text(e.path(&index)),
                    mode: format!("{:06o}", e.mode.bits()),
                    oid: e.id.to_string(),
                    stage: e.stage_raw() as u8,
                    content_sha256: blob.then(|| content(&repo, e.id)).transpose()?,
                });
            }
            Answer::Index { entries }
        }
        "submodules" => {
            let mut entries = Vec::new();
            if let Some(modules) = repo.submodules()? {
                for sm in modules {
                    // `Submodule::head_id()` is the commit in the superproject `HEAD` tree.
                    // The checked-out commit is the `HEAD` of the submodule repository.
                    let head_oid = match sm.state()?.worktree_checkout {
                        true => sm
                            .open()?
                            .map(|r| r.head_id().map(|id| id.to_string()))
                            .transpose()?,
                        false => None,
                    };
                    entries.push(Submodule {
                        path: path_text(&sm.path()?),
                        index_oid: sm.index_id()?.map(|id| id.to_string()),
                        head_oid,
                    });
                }
            }
            Answer::Submodules { entries }
        }
        "tree" => {
            let rev = &args.revs[0];
            let tree = repo
                .rev_parse_single(rev.as_str())?
                .object()?
                .peel_to_tree()?;
            let mut recorder = gix::traverse::tree::Recorder::default();
            tree.traverse().breadthfirst(&mut recorder)?;
            let mut entries = Vec::new();
            for e in recorder.records {
                let kind = e.mode.kind();
                if kind == EntryKind::Tree {
                    continue;
                }
                entries.push(Entry {
                    path: path_text(&e.filepath),
                    mode: octal(kind).into(),
                    oid: e.oid.to_string(),
                    stage: 0,
                    content_sha256: (kind != EntryKind::Commit)
                        .then(|| content(&repo, e.oid))
                        .transpose()?,
                });
            }
            Answer::Tree {
                rev: rev.clone(),
                entries,
            }
        }
        "diff" => {
            let tree = |rev: &str| -> R<gix::Tree<'_>> {
                Ok(repo.rev_parse_single(rev)?.object()?.peel_to_tree()?)
            };
            let (old, new) = (tree(&args.revs[0])?, tree(&args.revs[1])?);
            let options =
                gix::diff::Options::default().with_rewrites(Some(gix::diff::Rewrites::default()));
            let mut entries = Vec::new();
            for change in repo.diff_tree_to_tree(&old, &new, options)? {
                if change.entry_mode().is_tree() {
                    continue;
                }
                entries.push(match change {
                    Change::Addition { location, .. } => DiffEntry {
                        status: 'A',
                        path: path_text(&location),
                        orig: None,
                    },
                    Change::Deletion { location, .. } => DiffEntry {
                        status: 'D',
                        path: path_text(&location),
                        orig: None,
                    },
                    Change::Modification {
                        location,
                        previous_entry_mode,
                        entry_mode,
                        ..
                    } => DiffEntry {
                        status: letter(previous_entry_mode.kind(), entry_mode.kind()),
                        path: path_text(&location),
                        orig: None,
                    },
                    Change::Rewrite {
                        source_location,
                        location,
                        copy,
                        ..
                    } => DiffEntry {
                        status: if copy { 'C' } else { 'R' },
                        path: path_text(&location),
                        orig: Some(path_text(&source_location)),
                    },
                });
            }
            Answer::Diff {
                from: args.revs[0].clone(),
                to: args.revs[1].clone(),
                entries,
            }
        }
        _ => unreachable!(),
    })
}

fn status(repo: &gix::Repository) -> R<Answer> {
    let mut out = StatusBuilder::default();
    let iter = repo
        .status(gix::progress::Discard)?
        .untracked_files(gix::status::UntrackedFiles::Files)
        .index_worktree_submodules(gix::status::Submodule::AsConfigured { check_dirty: true })
        .into_iter(None)?;
    for item in iter {
        match item? {
            gix::status::Item::TreeIndex(change) => {
                use gix::diff::index::Change as Ix;
                match change {
                    Ix::Addition { location, .. } => out.index(path_text(&location), 'A', None),
                    Ix::Deletion { location, .. } => out.index(path_text(&location), 'D', None),
                    Ix::Modification {
                        location,
                        previous_entry_mode,
                        entry_mode,
                        ..
                    } => {
                        let l = letter(index_kind(previous_entry_mode), index_kind(entry_mode));
                        out.index(path_text(&location), l, None)
                    }
                    Ix::Rewrite {
                        source_location,
                        location,
                        copy,
                        ..
                    } => out.index(
                        path_text(&location),
                        if copy { 'C' } else { 'R' },
                        Some(path_text(&source_location)),
                    ),
                }
            }
            gix::status::Item::IndexWorktree(item) => match item {
                WtItem::Modification {
                    rela_path, status, ..
                } => {
                    let l = match status {
                        EntryStatus::Change(WtChange::Removed) => 'D',
                        EntryStatus::Change(WtChange::Type { .. }) => 'T',
                        EntryStatus::Change(WtChange::Modification { .. }) => 'M',
                        EntryStatus::Change(WtChange::SubmoduleModification(s)) => {
                            if s.is_dirty() == Some(true) {
                                'M'
                            } else {
                                continue;
                            }
                        }
                        EntryStatus::IntentToAdd => 'A',
                        EntryStatus::NeedsUpdate(_) => continue,
                        EntryStatus::Conflict { .. } => 'U',
                    };
                    out.worktree(path_text(&rela_path), l)
                }
                WtItem::DirectoryContents { entry, .. } => {
                    if entry.status == gix::dir::entry::Status::Untracked {
                        out.untracked(path_text(&entry.rela_path))
                    }
                }
                WtItem::Rewrite { .. } => return Err("unexpected index-to-worktree rewrite".into()),
            },
        }
    }
    Ok(out.finish())
}

fn content(repo: &gix::Repository, id: impl Into<gix::ObjectId>) -> R<String> {
    Ok(sha256_hex(&repo.find_object(id.into())?.data))
}

fn index_kind(mode: gix::index::entry::Mode) -> EntryKind {
    use gix::index::entry::Mode;
    match mode {
        m if m == Mode::FILE_EXECUTABLE => EntryKind::BlobExecutable,
        m if m == Mode::SYMLINK => EntryKind::Link,
        m if m == Mode::COMMIT => EntryKind::Commit,
        m if m == Mode::DIR => EntryKind::Tree,
        _ => EntryKind::Blob,
    }
}

/// `M` for a content or executable-bit change, `T` for a type change, as `git diff --name-status`.
fn letter(old: EntryKind, new: EntryKind) -> char {
    let class = |k| match k {
        EntryKind::Blob | EntryKind::BlobExecutable => 0,
        EntryKind::Link => 1,
        EntryKind::Commit => 2,
        EntryKind::Tree => 3,
    };
    if class(old) == class(new) { 'M' } else { 'T' }
}

fn octal(kind: EntryKind) -> &'static str {
    match kind {
        EntryKind::Blob => "100644",
        EntryKind::BlobExecutable => "100755",
        EntryKind::Link => "120000",
        EntryKind::Commit => "160000",
        EntryKind::Tree => "040000",
    }
}
