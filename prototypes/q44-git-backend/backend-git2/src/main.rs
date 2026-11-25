//! The Q44 operations with git2 (libgit2). Prints one JSON answer.
//!
//! libgit2 has no isolated open. The backend sets the system, global and XDG search paths
//! to an empty directory, so only the repository configuration counts.

use git2::{
    Delta, DiffFindOptions, ObjectType, Repository, Status, StatusOptions, TreeWalkMode,
    TreeWalkResult,
};
use model::{Answer, DiffEntry, Entry, StatusBuilder, Submodule, path_text, sha256_hex};

type R<T> = Result<T, Box<dyn std::error::Error>>;

fn main() {
    let args = model::args();
    let answer = run(&args).unwrap_or_else(|e| Answer::Error {
        message: e.to_string(),
    });
    model::print(model::sorted(answer));
}

fn isolate() -> R<()> {
    let empty = std::env::temp_dir().join("q44-git2-empty-config");
    std::fs::create_dir_all(&empty)?;
    for level in [
        git2::ConfigLevel::System,
        git2::ConfigLevel::Global,
        git2::ConfigLevel::XDG,
        git2::ConfigLevel::ProgramData,
    ] {
        // SAFETY: set once at start, before any repository is open and with one thread.
        unsafe { git2::opts::set_search_path(level, &empty)? };
    }
    Ok(())
}

fn run(args: &model::Args) -> R<Answer> {
    isolate()?;
    let repo = Repository::open(&args.repo)?;
    Ok(match args.op.as_str() {
        "head" => {
            let head = repo.head()?;
            let branch = if repo.head_detached()? {
                None
            } else {
                Some(path_text(head.name_bytes()))
            };
            Answer::Head {
                oid: head.target().ok_or("symbolic HEAD target")?.to_string(),
                branch,
            }
        }
        "refs" => {
            let mut refs = Vec::new();
            for r in repo.references()? {
                let r = r?;
                if let Some(id) = r.target() {
                    refs.push((path_text(r.name_bytes()), id.to_string()));
                }
            }
            Answer::Refs { refs }
        }
        "status" => status(&repo)?,
        "index" => {
            let index = repo.index()?;
            let mut entries = Vec::new();
            for e in index.iter() {
                let blob = e.mode != 0o160000;
                entries.push(Entry {
                    path: path_text(&e.path),
                    mode: format!("{:06o}", e.mode),
                    oid: e.id.to_string(),
                    stage: ((e.flags >> 12) & 3) as u8,
                    content_sha256: blob.then(|| content(&repo, e.id)).transpose()?,
                });
            }
            Answer::Index { entries }
        }
        "submodules" => {
            let mut entries = Vec::new();
            for sm in repo.submodules()? {
                entries.push(Submodule {
                    path: path_text(sm.path().as_os_str().as_encoded_bytes()),
                    index_oid: sm.index_id().map(|id| id.to_string()),
                    head_oid: sm.workdir_id().map(|id| id.to_string()),
                });
            }
            Answer::Submodules { entries }
        }
        "tree" => {
            let rev = &args.revs[0];
            let tree = repo.revparse_single(rev)?.peel_to_tree()?;
            let mut entries = Vec::new();
            let mut failure = None;
            tree.walk(TreeWalkMode::PreOrder, |dir, e| {
                if e.kind() == Some(ObjectType::Tree) {
                    return TreeWalkResult::Ok;
                }
                let mut path = dir.as_bytes().to_vec();
                path.extend_from_slice(e.name_bytes());
                let mode = e.filemode() as u32;
                let sha = if mode == 0o160000 {
                    Ok(None)
                } else {
                    content(&repo, e.id()).map(Some)
                };
                match sha {
                    Ok(content_sha256) => entries.push(Entry {
                        path: path_text(&path),
                        mode: format!("{mode:06o}"),
                        oid: e.id().to_string(),
                        stage: 0,
                        content_sha256,
                    }),
                    Err(err) => {
                        failure = Some(err.to_string());
                        return TreeWalkResult::Abort;
                    }
                }
                TreeWalkResult::Ok
            })?;
            if let Some(f) = failure {
                return Err(f.into());
            }
            Answer::Tree {
                rev: rev.clone(),
                entries,
            }
        }
        "diff" => {
            let old = repo.revparse_single(&args.revs[0])?.peel_to_tree()?;
            let new = repo.revparse_single(&args.revs[1])?.peel_to_tree()?;
            let mut diff = repo.diff_tree_to_tree(Some(&old), Some(&new), None)?;
            diff.find_similar(Some(DiffFindOptions::new().renames(true)))?;
            let mut entries = Vec::new();
            for d in diff.deltas() {
                let new_path = path_text(d.new_file().path_bytes().unwrap_or_default());
                let old_path = path_text(d.old_file().path_bytes().unwrap_or_default());
                entries.push(match d.status() {
                    Delta::Added => DiffEntry {
                        status: 'A',
                        path: new_path,
                        orig: None,
                    },
                    Delta::Deleted => DiffEntry {
                        status: 'D',
                        path: old_path,
                        orig: None,
                    },
                    Delta::Modified => DiffEntry {
                        status: 'M',
                        path: new_path,
                        orig: None,
                    },
                    Delta::Typechange => DiffEntry {
                        status: 'T',
                        path: new_path,
                        orig: None,
                    },
                    Delta::Renamed => DiffEntry {
                        status: 'R',
                        path: new_path,
                        orig: Some(old_path),
                    },
                    Delta::Copied => DiffEntry {
                        status: 'C',
                        path: new_path,
                        orig: Some(old_path),
                    },
                    other => return Err(format!("unexpected delta {other:?}").into()),
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

fn status(repo: &Repository) -> R<Answer> {
    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(true)
        .include_ignored(false)
        .exclude_submodules(false)
        .renames_head_to_index(true);
    let mut out = StatusBuilder::default();
    for s in repo.statuses(Some(&mut opts))?.iter() {
        let st = s.status();
        let path = path_text(s.path_bytes());
        if st.contains(Status::WT_NEW) && !st.intersects(Status::INDEX_NEW | Status::INDEX_MODIFIED)
        {
            out.untracked(path);
            continue;
        }
        if st.contains(Status::INDEX_RENAMED) {
            let d = s.head_to_index().ok_or("rename without delta")?;
            let new = path_text(d.new_file().path_bytes().unwrap_or_default());
            let old = path_text(d.old_file().path_bytes().unwrap_or_default());
            out.index(new.clone(), 'R', Some(old));
            if let Some(l) = wt_letter(st) {
                out.worktree(new, l);
            }
            continue;
        }
        let index = [
            (Status::INDEX_NEW, 'A'),
            (Status::INDEX_MODIFIED, 'M'),
            (Status::INDEX_DELETED, 'D'),
            (Status::INDEX_TYPECHANGE, 'T'),
        ]
        .into_iter()
        .find(|(f, _)| st.contains(*f));
        if let Some((_, l)) = index {
            out.index(path.clone(), l, None);
        }
        if let Some(l) = wt_letter(st) {
            out.worktree(path, l);
        }
    }
    Ok(out.finish())
}

fn wt_letter(st: Status) -> Option<char> {
    [
        (Status::WT_MODIFIED, 'M'),
        (Status::WT_DELETED, 'D'),
        (Status::WT_TYPECHANGE, 'T'),
        (Status::WT_RENAMED, 'R'),
    ]
    .into_iter()
    .find(|(f, _)| st.contains(*f))
    .map(|(_, l)| l)
}

fn content(repo: &Repository, id: git2::Oid) -> R<String> {
    Ok(sha256_hex(repo.find_blob(id)?.content()))
}
