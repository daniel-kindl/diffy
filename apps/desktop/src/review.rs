//! The review state of a repository in the app: reviewed files and findings,
//! saved in the SQLite store (R-03 to R-05, R-08). The frontend names a file
//! only by a path that Rust checks, and sends text that Rust validates.

use crate::ipc::{RepoState, Repository};
use diffy_comparison::{ContentId, Snapshot, Source};
use diffy_git::Repo;
use diffy_review::{
    Anchor, Baseline, Category, FindingId, RepositoryId, ReviewSession, Severity, SourceState,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// The most bytes of the notes of one finding.
pub const MAX_NOTES_BYTES: usize = 20_000;
/// The most line number of an anchor. A larger value is a mistake.
pub const MAX_LINE: u32 = 10_000_000;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AddFindingInput {
    pub path: String,
    /// 1-based, inclusive.
    pub start_line: u32,
    pub end_line: u32,
    pub severity: String,
    pub category: String,
    pub notes: String,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct FindingDto {
    pub id: u64,
    pub path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub severity: String,
    pub category: String,
    pub notes: String,
    pub resolved: bool,
}

pub fn parse_severity(s: &str) -> Result<Severity, String> {
    Ok(match s {
        "blocker" => Severity::Blocker,
        "major" => Severity::Major,
        "minor" => Severity::Minor,
        "nit" => Severity::Nit,
        _ => return Err(format!("unknown severity {s}")),
    })
}

pub fn parse_category(s: &str) -> Result<Category, String> {
    Ok(match s {
        "correctness" => Category::Correctness,
        "security" => Category::Security,
        "performance" => Category::Performance,
        "design" => Category::Design,
        "tests" => Category::Tests,
        "docs" => Category::Docs,
        "style" => Category::Style,
        _ => return Err(format!("unknown category {s}")),
    })
}

fn names(f: &diffy_review::Finding) -> (&'static str, &'static str) {
    let sev = match f.severity {
        Severity::Blocker => "blocker",
        Severity::Major => "major",
        Severity::Minor => "minor",
        Severity::Nit => "nit",
    };
    let cat = match f.category {
        Category::Correctness => "correctness",
        Category::Security => "security",
        Category::Performance => "performance",
        Category::Design => "design",
        Category::Tests => "tests",
        Category::Docs => "docs",
        Category::Style => "style",
    };
    (sev, cat)
}

fn db_path(r: &RepoState) -> Result<&std::path::Path, String> {
    r.db.as_deref()
        .ok_or_else(|| "the review database is not set".to_owned())
}

fn head_commit(r: &RepoState) -> Result<String, String> {
    let repo = Repo::discover(&r.root).map_err(|e| e.to_string())?;
    // An unborn branch has no commit: the revision is empty then.
    Ok(repo
        .head_commit()
        .map_err(|e| e.to_string())?
        .unwrap_or_default())
}

fn repository_id(r: &RepoState) -> RepositoryId {
    RepositoryId(
        r.root
            .canonicalize()
            .unwrap_or_else(|_| r.root.clone())
            .display()
            .to_string(),
    )
}

/// The content identity of a worktree file now. A missing file has the identity
/// of empty content, so a deleted file can be reviewed.
fn content_id(r: &RepoState, path: &str) -> Result<ContentId, String> {
    let repo = Repo::discover(&r.root).map_err(|e| e.to_string())?;
    let bytes = repo
        .read_worktree_file(path)
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    Ok(Snapshot::new(Source::File(path.to_owned()), bytes).id())
}

fn changed(r: &RepoState, path: &str) -> Result<(), String> {
    if r.files.iter().any(|f| f.path == path) {
        Ok(())
    } else {
        Err(format!("{path} is not a changed file"))
    }
}

fn load(r: &RepoState) -> Result<(rusqlite::Connection, ReviewSession), String> {
    let conn = diffy_storage::open(db_path(r)?).map_err(|e| e.to_string())?;
    let id = repository_id(r);
    let session = diffy_storage::review_store::load(&conn, &id, &Baseline::Head)
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| {
            ReviewSession::new(
                id.clone(),
                Baseline::Head,
                SourceState {
                    head_revision: String::new(),
                },
            )
        });
    Ok((conn, session))
}

fn save(conn: &mut rusqlite::Connection, session: &ReviewSession) -> Result<(), String> {
    diffy_storage::review_store::save(conn, session).map_err(|e| e.to_string())
}

/// The changed files that are reviewed at their current content. It reads only
/// the files that have a saved reviewed mark, and opens the repository once, so
/// a repository with thousands of changed files stays fast.
pub fn reviewed_paths(r: &RepoState) -> Result<HashSet<String>, String> {
    let (_, session) = load(r)?;
    let marked: Vec<(&str, ContentId)> = session
        .reviewed_files()
        .filter(|(p, _)| r.files.iter().any(|f| f.path == *p))
        .collect();
    if marked.is_empty() {
        return Ok(HashSet::new());
    }
    let repo = Repo::discover(&r.root).map_err(|e| e.to_string())?;
    let mut out = HashSet::new();
    for (path, saved) in marked {
        let bytes = repo
            .read_worktree_file(path)
            .map_err(|e| e.to_string())?
            .unwrap_or_default();
        if Snapshot::new(Source::File(path.to_owned()), bytes).id() == saved {
            out.insert(path.to_owned());
        }
    }
    Ok(out)
}

pub fn mark_reviewed(r: &RepoState, path: &str) -> Result<(), String> {
    changed(r, path)?;
    let (mut conn, mut session) = load(r)?;
    session.update_source(SourceState {
        head_revision: head_commit(r)?,
    });
    session.mark_reviewed(path, content_id(r, path)?);
    save(&mut conn, &session)
}

pub fn add_finding(r: &RepoState, input: &AddFindingInput) -> Result<u64, String> {
    changed(r, &input.path)?;
    if input.start_line == 0 || input.end_line < input.start_line || input.end_line > MAX_LINE {
        return Err("the lines must start at 1 and end at or after the start".into());
    }
    if input.notes.trim().is_empty() || input.notes.len() > MAX_NOTES_BYTES {
        return Err(format!("the notes must have 1 to {MAX_NOTES_BYTES} bytes"));
    }
    let severity = parse_severity(&input.severity)?;
    let category = parse_category(&input.category)?;
    let (mut conn, mut session) = load(r)?;
    let revision = head_commit(r)?;
    session.update_source(SourceState {
        head_revision: revision.clone(),
    });
    let id = session.add_finding(
        Anchor {
            path: input.path.clone(),
            revision,
            lines: (input.start_line - 1)..input.end_line,
        },
        severity,
        category,
        input.notes.clone(),
    );
    save(&mut conn, &session)?;
    Ok(id.0)
}

pub fn findings(r: &RepoState) -> Result<Vec<FindingDto>, String> {
    let (_, session) = load(r)?;
    Ok(session
        .findings()
        .iter()
        .map(|f| {
            let (sev, cat) = names(f);
            FindingDto {
                id: f.id.0,
                path: f.anchor.path.clone(),
                start_line: f.anchor.lines.start + 1,
                end_line: f.anchor.lines.end,
                severity: sev.into(),
                category: cat.into(),
                notes: f.notes.clone(),
                resolved: f.resolved,
            }
        })
        .collect())
}

pub fn set_resolved(r: &RepoState, id: u64, resolved: bool) -> Result<(), String> {
    let (mut conn, mut session) = load(r)?;
    if !session.set_resolved(FindingId(id), resolved) {
        return Err(format!("no finding {id}"));
    }
    save(&mut conn, &session)
}

/// The unresolved findings, or the selected ones, as Markdown for an agent (Q28).
pub fn markdown(r: &RepoState, selected: Option<&[u64]>) -> Result<String, String> {
    let (_, session) = load(r)?;
    let ids: Option<Vec<FindingId>> = selected.map(|s| s.iter().map(|i| FindingId(*i)).collect());
    Ok(session.findings_markdown(ids.as_deref()))
}

fn with_repo<T>(
    state: &Repository,
    f: impl FnOnce(&RepoState) -> Result<T, String>,
) -> Result<T, String> {
    let guard = state
        .0
        .lock()
        .map_err(|_| "the repository is locked".to_owned())?;
    f(guard
        .as_ref()
        .ok_or_else(|| "no repository is open".to_owned())?)
}

#[tauri::command]
pub fn mark_file_reviewed(state: tauri::State<'_, Repository>, path: String) -> Result<(), String> {
    with_repo(&state, |r| mark_reviewed(r, &path))
}

#[tauri::command]
pub fn add_review_finding(
    state: tauri::State<'_, Repository>,
    input: AddFindingInput,
) -> Result<u64, String> {
    with_repo(&state, |r| add_finding(r, &input))
}

#[tauri::command]
pub fn review_findings(state: tauri::State<'_, Repository>) -> Result<Vec<FindingDto>, String> {
    with_repo(&state, findings)
}

#[tauri::command]
pub fn resolve_finding(
    state: tauri::State<'_, Repository>,
    id: u64,
    resolved: bool,
) -> Result<(), String> {
    with_repo(&state, |r| set_resolved(r, id, resolved))
}

#[tauri::command]
pub fn findings_markdown(state: tauri::State<'_, Repository>) -> Result<String, String> {
    with_repo(&state, |r| markdown(r, None))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn git(dir: &std::path::Path, args: &[&str]) {
        let ok = std::process::Command::new("git")
            .args(args)
            .current_dir(dir)
            .env(
                "GIT_CONFIG_GLOBAL",
                if cfg!(windows) { "NUL" } else { "/dev/null" },
            )
            .env("GIT_AUTHOR_NAME", "t")
            .env("GIT_AUTHOR_EMAIL", "t@example.com")
            .env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@example.com")
            .status()
            .unwrap()
            .success();
        assert!(ok, "git {args:?}");
    }

    /// A repository with `a.txt` changed in the worktree, and a database.
    fn fixture() -> (tempfile::TempDir, tempfile::TempDir, RepoState) {
        let d = tempfile::tempdir().unwrap();
        git(d.path(), &["init", "-q", "-b", "main"]);
        fs::write(d.path().join("a.txt"), "one\ntwo\n").unwrap();
        git(d.path(), &["add", "."]);
        git(d.path(), &["commit", "-q", "-m", "first"]);
        fs::write(d.path().join("a.txt"), "one\nTWO\n").unwrap();
        let db = tempfile::tempdir().unwrap();
        let mut state = RepoState::open(d.path()).unwrap();
        state.db = Some(db.path().join("diffy.db"));
        (d, db, state)
    }

    fn input(path: &str) -> AddFindingInput {
        AddFindingInput {
            path: path.into(),
            start_line: 2,
            end_line: 3,
            severity: "major".into(),
            category: "security".into(),
            notes: "check this".into(),
        }
    }

    #[test]
    fn a_reviewed_file_needs_review_again_when_its_content_changes() {
        let (d, _db, state) = fixture();
        assert!(reviewed_paths(&state).unwrap().is_empty());
        mark_reviewed(&state, "a.txt").unwrap();
        assert!(reviewed_paths(&state).unwrap().contains("a.txt"));
        fs::write(d.path().join("a.txt"), "one\nTHREE\n").unwrap();
        assert!(reviewed_paths(&state).unwrap().is_empty());
    }

    #[test]
    fn findings_are_saved_and_survive_a_restart() {
        let (d, db, state) = fixture();
        let id = add_finding(&state, &input("a.txt")).unwrap();
        // A new state over the same database stands for a restart.
        let mut again = RepoState::open(d.path()).unwrap();
        again.db = Some(db.path().join("diffy.db"));
        let f = findings(&again).unwrap();
        assert_eq!(f.len(), 1);
        assert_eq!((f[0].id, f[0].start_line, f[0].end_line), (id, 2, 3));
        assert_eq!(
            (f[0].severity.as_str(), f[0].category.as_str()),
            ("major", "security")
        );
        let second = add_finding(&again, &input("a.txt")).unwrap();
        assert_ne!(second, id, "the IDs stay unique after a restart");
    }

    #[test]
    fn markdown_lists_unresolved_findings_for_an_agent() {
        let (_d, _db, state) = fixture();
        let a = add_finding(&state, &input("a.txt")).unwrap();
        let mut second = input("a.txt");
        second.notes = "second".into();
        let b = add_finding(&state, &second).unwrap();
        set_resolved(&state, b, true).unwrap();
        let md = markdown(&state, None).unwrap();
        assert!(md.contains("check this") && md.contains("`a.txt:2-3`") && !md.contains("second"));
        assert!(markdown(&state, Some(&[b])).unwrap().contains("second"));
        assert!(set_resolved(&state, 999, true).is_err());
        let _ = a;
    }

    #[test]
    fn invalid_input_is_rejected() {
        let (_d, _db, state) = fixture();
        let bad = |f: &dyn Fn(&mut AddFindingInput)| {
            let mut i = input("a.txt");
            f(&mut i);
            assert!(add_finding(&state, &i).is_err());
        };
        bad(&|i| i.path = "unknown.txt".into());
        bad(&|i| i.path = "../outside".into());
        bad(&|i| i.start_line = 0);
        bad(&|i| i.end_line = 1);
        bad(&|i| i.end_line = MAX_LINE + 1);
        bad(&|i| i.severity = "huge".into());
        bad(&|i| i.category = "other".into());
        bad(&|i| i.notes = "  ".into());
        bad(&|i| i.notes = "x".repeat(MAX_NOTES_BYTES + 1));
        assert!(mark_reviewed(&state, "unknown.txt").is_err());
        assert!(findings(&state).unwrap().is_empty());
    }

    #[test]
    fn the_input_rejects_unknown_fields() {
        let ok = r#"{"path":"a","start_line":1,"end_line":1,"severity":"nit","category":"docs","notes":"x"}"#;
        assert!(serde_json::from_str::<AddFindingInput>(ok).is_ok());
        let extra = ok.replace("}", r#","db":"/etc/x"}"#);
        assert!(serde_json::from_str::<AddFindingInput>(&extra).is_err());
    }

    #[test]
    fn without_a_database_the_commands_fail_clearly() {
        let (_d, _db, mut state) = fixture();
        state.db = None;
        assert!(
            add_finding(&state, &input("a.txt"))
                .unwrap_err()
                .contains("database")
        );
    }
}
