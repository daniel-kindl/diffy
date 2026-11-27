//! The commands that the frontend can call. The frontend gets no credential
//! and no file system access: it never sends a path. Each input is validated
//! (spec section 11).

use diffy_comparison::Snapshot;
use diffy_comparison::canonical::canonical_hunks;
use diffy_comparison::text::{decode, legacy_encoding};
use diffy_git::{Repo, WorktreeKind};
use diffy_review::explorer::{self, ChangedFile, Filter, GeneratedRules, Status};
use encoding_rs::Encoding;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// The most bytes of one side that a command accepts.
pub const MAX_SIDE_BYTES: usize = 64 * 1024 * 1024;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompareInput {
    pub base: String,
    pub head: String,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Range {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct HunkDto {
    pub base: Range,
    pub head: Range,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Comparison {
    pub base: String,
    pub head: String,
    pub hunks: Vec<HunkDto>,
    /// The source labels, the encodings and the decode errors for the UI.
    pub sides: Vec<SideInfo>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct SideInfo {
    pub label: String,
    pub encoding: String,
    /// The byte offset of the first invalid sequence, if any.
    pub first_error: Option<usize>,
}

pub fn compare(input: CompareInput) -> Result<Comparison, String> {
    if input.base.len() > MAX_SIDE_BYTES || input.head.len() > MAX_SIDE_BYTES {
        return Err(format!("a side is larger than {MAX_SIDE_BYTES} bytes"));
    }
    let hunks = canonical_hunks(input.base.as_bytes(), input.head.as_bytes())
        .into_iter()
        .map(|h| HunkDto {
            base: Range {
                start: h.base.start,
                end: h.base.end,
            },
            head: Range {
                start: h.head.start,
                end: h.head.end,
            },
        })
        .collect();
    Ok(Comparison {
        base: input.base,
        head: input.head,
        hunks,
        sides: Vec::new(),
    })
}

/// The two inputs of the open comparison. Rust keeps the raw snapshots, so a
/// change of the encoding or of the order never rereads a file and never
/// changes a source. Each snapshot keeps its own source label.
pub struct Session {
    left: Snapshot,
    right: Snapshot,
    left_encoding: Option<&'static Encoding>,
    right_encoding: Option<&'static Encoding>,
}

impl Session {
    pub fn new(left: Snapshot, right: Snapshot) -> Self {
        Self {
            left,
            right,
            left_encoding: None,
            right_encoding: None,
        }
    }

    /// Select the legacy encoding of one side by its label. There is no
    /// heuristic detection. An empty label goes back to the default.
    pub fn set_encoding(&mut self, side: Side, label: &str) -> Result<(), String> {
        let enc = if label.is_empty() {
            None
        } else {
            Some(legacy_encoding(label).ok_or_else(|| format!("unknown encoding {label}"))?)
        };
        match side {
            Side::Left => self.left_encoding = enc,
            Side::Right => self.right_encoding = enc,
        }
        Ok(())
    }

    /// Swap the sides. The sources and the encodings go with their inputs.
    pub fn swap(&mut self) {
        std::mem::swap(&mut self.left, &mut self.right);
        std::mem::swap(&mut self.left_encoding, &mut self.right_encoding);
    }

    /// The comparison. The text is decoded for the view and the canonical
    /// diff runs on the decoded text, so the hunks match the shown lines.
    pub fn comparison(&self) -> Result<Comparison, String> {
        let (l, r) = (
            decode(self.left.raw(), self.left_encoding),
            decode(self.right.raw(), self.right_encoding),
        );
        let label = |s: &Snapshot| match s.source() {
            diffy_comparison::Source::File(p) | diffy_comparison::Source::Text(p) => p.clone(),
            diffy_comparison::Source::GitObject(o) => o.clone(),
        };
        let mut c = compare(CompareInput {
            base: l.text.clone(),
            head: r.text.clone(),
        })?;
        c.sides = vec![
            SideInfo {
                label: label(&self.left),
                encoding: l.encoding.label().into(),
                first_error: l.first_error,
            },
            SideInfo {
                label: label(&self.right),
                encoding: r.encoding.label().into(),
                first_error: r.first_error,
            },
        ];
        Ok(c)
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Left,
    Right,
}

/// The open comparison, owned by Rust.
#[derive(Default)]
pub struct Open(pub Mutex<Option<Session>>);

fn with_session<T>(
    state: &Open,
    f: impl FnOnce(&mut Session) -> Result<T, String>,
) -> Result<Option<T>, String> {
    let mut guard = state
        .0
        .lock()
        .map_err(|_| "the session is locked".to_owned())?;
    guard.as_mut().map(f).transpose()
}

#[tauri::command]
pub fn current_comparison(state: tauri::State<'_, Open>) -> Result<Option<Comparison>, String> {
    with_session(&state, |s| s.comparison())
}

#[tauri::command]
pub fn select_encoding(
    state: tauri::State<'_, Open>,
    side: Side,
    label: String,
) -> Result<Option<Comparison>, String> {
    with_session(&state, |s| {
        s.set_encoding(side, &label)?;
        s.comparison()
    })
}

#[tauri::command]
pub fn swap_sides(state: tauri::State<'_, Open>) -> Result<Option<Comparison>, String> {
    with_session(&state, |s| {
        s.swap();
        s.comparison()
    })
}

/// The repository of a `diffy <repo-path>` start. Rust owns it. The frontend
/// names a file only by its path inside the repository, and Rust checks it.
pub struct RepoState {
    /// The path to reopen. A gix repository is not `Send`, so each command
    /// opens it again, isolated.
    pub root: std::path::PathBuf,
    pub files: Vec<ChangedFile>,
    /// The database of the review state. `None` until the app sets it.
    pub db: Option<std::path::PathBuf>,
}

impl RepoState {
    pub fn open(path: &std::path::Path) -> Result<Self, String> {
        let repo = Repo::discover(path).map_err(|e| e.to_string())?;
        let files = repo
            .worktree_status()
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(|e| {
                let kind = e.unstaged.or(e.staged);
                ChangedFile {
                    path: e.path,
                    status: if e.untracked {
                        Status::Added
                    } else {
                        match kind {
                            Some(WorktreeKind::Added) => Status::Added,
                            Some(WorktreeKind::Deleted) => Status::Deleted,
                            Some(WorktreeKind::Renamed) => Status::Renamed,
                            _ => Status::Modified,
                        }
                    },
                    generated_attribute: None,
                    reviewed: false,
                }
            })
            .collect();
        Ok(Self {
            root: path.to_owned(),
            files,
            db: None,
        })
    }

    /// The comparison of `HEAD` against the worktree for one changed file.
    pub fn session_for(&self, path: &str) -> Result<Session, String> {
        if !self.files.iter().any(|f| f.path == path) {
            return Err(format!("{path} is not a changed file"));
        }
        let repo = Repo::discover(&self.root).map_err(|e| e.to_string())?;
        let base = repo
            .read_file("HEAD", path)
            .map_err(|e| e.to_string())?
            .unwrap_or_default();
        let head = repo
            .read_worktree_file(path)
            .map_err(|e| e.to_string())?
            .unwrap_or_default();
        Ok(Session::new(
            Snapshot::new(
                diffy_comparison::Source::GitObject(format!("HEAD:{path}")),
                base,
            ),
            Snapshot::new(diffy_comparison::Source::File(path.to_owned()), head),
        ))
    }
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct FileDto {
    pub path: String,
    pub status: String,
    pub generated: bool,
    pub reviewed: bool,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct ListingDto {
    pub files: Vec<FileDto>,
    pub total: usize,
    pub hidden: usize,
    pub generated: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListInput {
    pub search: String,
    pub show_generated: bool,
}

pub fn list_files(
    files: &[ChangedFile],
    reviewed: &std::collections::HashSet<String>,
    input: &ListInput,
) -> Result<ListingDto, String> {
    let rules = GeneratedRules::new(&[]).map_err(|e| e.to_string())?;
    let filter = Filter {
        search: input.search.clone(),
        show_generated: input.show_generated,
        ..Filter::default()
    };
    let l = explorer::list(files, &rules, &filter);
    Ok(ListingDto {
        files: l
            .visible
            .iter()
            .map(|f| FileDto {
                path: f.path.clone(),
                status: format!("{:?}", f.status).to_lowercase(),
                generated: rules.is_generated(&f.path, f.generated_attribute),
                reviewed: reviewed.contains(&f.path),
            })
            .collect(),
        total: l.counts.total,
        hidden: l.counts.hidden,
        generated: l.counts.generated,
    })
}

#[derive(Default)]
pub struct Repository(pub Mutex<Option<RepoState>>);

#[tauri::command]
pub fn repository_files(
    state: tauri::State<'_, Repository>,
    input: ListInput,
) -> Result<Option<ListingDto>, String> {
    let guard = state
        .0
        .lock()
        .map_err(|_| "the repository is locked".to_owned())?;
    guard
        .as_ref()
        .map(|r| {
            // A review state that cannot load shows every file as not reviewed.
            let reviewed = crate::review::reviewed_paths(r).unwrap_or_default();
            list_files(&r.files, &reviewed, &input)
        })
        .transpose()
}

#[tauri::command]
pub fn open_repository_file(
    repo: tauri::State<'_, Repository>,
    open: tauri::State<'_, Open>,
    path: String,
) -> Result<Option<Comparison>, String> {
    let session = {
        let guard = repo
            .0
            .lock()
            .map_err(|_| "the repository is locked".to_owned())?;
        match guard.as_ref() {
            Some(r) => r.session_for(&path)?,
            None => return Ok(None),
        }
    };
    let comparison = session.comparison()?;
    *open
        .0
        .lock()
        .map_err(|_| "the session is locked".to_owned())? = Some(session);
    Ok(Some(comparison))
}

/// An error that the page reports. It goes to the standard error output, cut to
/// a short length, so a page that fails to run leaves a trace.
#[tauri::command]
pub fn report_ui_error(message: String) {
    eprintln!(
        "diffy: ui error: {}",
        message.chars().take(500).collect::<String>()
    );
}

/// Open two pasted texts as the current comparison. The texts become snapshots
/// with the sources "Pasted text (base)" and "Pasted text (head)", so the swap
/// works as for files. The size limit of the IPC applies.
pub fn open_text_session(input: CompareInput) -> Result<Session, String> {
    if input.base.len() > MAX_SIDE_BYTES || input.head.len() > MAX_SIDE_BYTES {
        return Err(format!("a side is larger than {MAX_SIDE_BYTES} bytes"));
    }
    Ok(Session::new(
        Snapshot::new(
            diffy_comparison::Source::Text("Pasted text (base)".into()),
            input.base.into_bytes(),
        ),
        Snapshot::new(
            diffy_comparison::Source::Text("Pasted text (head)".into()),
            input.head.into_bytes(),
        ),
    ))
}

#[tauri::command]
pub fn open_text(state: tauri::State<'_, Open>, input: CompareInput) -> Result<Comparison, String> {
    let session = open_text_session(input)?;
    let comparison = session.comparison()?;
    *state
        .0
        .lock()
        .map_err(|_| "the session is locked".to_owned())? = Some(session);
    Ok(comparison)
}

#[tauri::command]
pub fn compare_text(input: CompareInput) -> Result<Comparison, String> {
    compare(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use diffy_comparison::Source;

    #[test]
    fn two_snapshots_give_hunks_labels_and_encodings() {
        let l = Snapshot::new(Source::File("a.txt".into()), b"one\ntwo\n".to_vec());
        let r = Snapshot::new(
            Source::File("b.txt".into()),
            b"\xEF\xBB\xBFone\nTWO\n".to_vec(),
        );
        let c = Session::new(l, r).comparison().unwrap();
        assert_eq!(c.hunks.len(), 1);
        assert_eq!(
            (c.sides[0].label.as_str(), c.sides[0].encoding.as_str()),
            ("a.txt", "UTF-8")
        );
        assert_eq!(c.sides[1].encoding, "UTF-8 (BOM)");
        assert_eq!(c.head, "one\nTWO\n", "the BOM is not part of the text");
    }

    #[test]
    fn an_invalid_side_reports_its_error_and_still_compares() {
        let l = Snapshot::new(Source::File("a".into()), b"ab\xFF\n".to_vec());
        let r = Snapshot::new(Source::File("b".into()), b"ab\n".to_vec());
        let c = Session::new(l, r).comparison().unwrap();
        assert_eq!(c.sides[0].first_error, Some(2));
        assert_eq!(c.sides[1].first_error, None);
    }

    fn snaps(l: &[u8], r: &[u8]) -> Session {
        Session::new(
            Snapshot::new(Source::File("l".into()), l.to_vec()),
            Snapshot::new(Source::File("r".into()), r.to_vec()),
        )
    }

    #[test]
    fn a_legacy_encoding_is_selected_per_side_and_not_guessed() {
        let mut s = snaps(b"\xE8esky\n", b"\xE8esky\n");
        assert!(
            s.comparison().unwrap().sides[0].first_error.is_some(),
            "no guessing"
        );
        s.set_encoding(Side::Left, "windows-1250").unwrap();
        let c = s.comparison().unwrap();
        assert_eq!(c.base, "česky\n");
        assert_eq!(c.sides[0].encoding, "windows-1250");
        assert!(
            c.sides[1].first_error.is_some(),
            "the other side is unchanged"
        );
        assert!(s.set_encoding(Side::Left, "nope").is_err());
        s.set_encoding(Side::Left, "").unwrap();
        assert!(s.comparison().unwrap().sides[0].first_error.is_some());
    }

    #[test]
    fn swapping_keeps_each_source_with_its_input() {
        let mut s = snaps(b"a\n", b"b\n");
        s.swap();
        let c = s.comparison().unwrap();
        assert_eq!((c.base.as_str(), c.head.as_str()), ("b\n", "a\n"));
        assert_eq!(
            (c.sides[0].label.as_str(), c.sides[1].label.as_str()),
            ("r", "l")
        );
    }

    #[test]
    fn swapping_twice_gives_the_first_comparison() {
        let mut s = snaps(b"a\nb\n", b"a\nB\n");
        let first = s.comparison().unwrap();
        s.swap();
        s.swap();
        assert_eq!(s.comparison().unwrap(), first);
    }

    fn file(path: &str) -> ChangedFile {
        ChangedFile {
            path: path.into(),
            status: Status::Modified,
            generated_attribute: None,
            reviewed: false,
        }
    }

    fn git(dir: &std::path::Path, args: &[&str]) {
        let ok = std::process::Command::new("git")
            .args(args)
            .current_dir(dir)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_AUTHOR_NAME", "t")
            .env("GIT_AUTHOR_EMAIL", "t@example.com")
            .env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@example.com")
            .status()
            .unwrap()
            .success();
        assert!(ok, "git {args:?}");
    }

    #[test]
    fn a_repository_lists_changes_and_compares_head_with_the_worktree() {
        let d = tempfile::tempdir().unwrap();
        git(d.path(), &["init", "-q", "-b", "main"]);
        std::fs::write(d.path().join("a.txt"), "one\ntwo\n").unwrap();
        git(d.path(), &["add", "."]);
        git(d.path(), &["commit", "-q", "-m", "first"]);
        std::fs::write(d.path().join("a.txt"), "one\nTWO\n").unwrap();
        std::fs::write(d.path().join("new.txt"), "n\n").unwrap();

        let state = RepoState::open(d.path()).unwrap();
        let paths: Vec<_> = state.files.iter().map(|f| f.path.as_str()).collect();
        assert_eq!(paths, vec!["a.txt", "new.txt"]);
        let c = state.session_for("a.txt").unwrap().comparison().unwrap();
        assert_eq!(
            (c.base.as_str(), c.head.as_str()),
            ("one\ntwo\n", "one\nTWO\n")
        );
        assert_eq!(c.sides[0].label, "HEAD:a.txt");
        let c = state.session_for("new.txt").unwrap().comparison().unwrap();
        assert_eq!((c.base.as_str(), c.head.as_str()), ("", "n\n"));
        // The frontend can only open a changed file, and never a path outside.
        assert!(state.session_for("../secret").is_err());
        assert!(state.session_for("unchanged-or-unknown.txt").is_err());
    }

    #[test]
    fn the_listing_hides_generated_files_but_counts_them() {
        let files = vec![file("src/a.rs"), file("Cargo.lock")];
        let none = std::collections::HashSet::new();
        let l = list_files(
            &files,
            &none,
            &ListInput {
                search: String::new(),
                show_generated: false,
            },
        )
        .unwrap();
        assert_eq!(
            (l.files.len(), l.total, l.hidden, l.generated),
            (1, 2, 1, 1)
        );
        let l = list_files(
            &files,
            &none,
            &ListInput {
                search: "cargo".into(),
                show_generated: true,
            },
        )
        .unwrap();
        assert_eq!(l.files[0].path, "Cargo.lock");
        assert!(l.files[0].generated);
    }

    #[test]
    fn the_list_input_rejects_unknown_fields() {
        let ok: Result<ListInput, _> =
            serde_json::from_str(r#"{"search":"","show_generated":true}"#);
        assert!(ok.is_ok());
        assert!(
            serde_json::from_str::<ListInput>(r#"{"search":"","show_generated":true,"root":"/"}"#)
                .is_err()
        );
    }

    #[test]
    fn returns_the_canonical_hunks() {
        let c = compare(CompareInput {
            base: "a\nb\n".into(),
            head: "a\nB\n".into(),
        })
        .unwrap();
        assert_eq!(c.hunks.len(), 1);
        assert_eq!(c.hunks[0].base, Range { start: 1, end: 2 });
        assert_eq!(c.hunks[0].head, Range { start: 1, end: 2 });
    }

    #[test]
    fn rejects_an_input_that_is_too_large() {
        let big = "x".repeat(MAX_SIDE_BYTES + 1);
        assert!(
            compare(CompareInput {
                base: big,
                head: String::new()
            })
            .is_err()
        );
    }

    #[test]
    fn the_input_has_no_path_and_rejects_unknown_fields() {
        let ok: Result<CompareInput, _> = serde_json::from_str(r#"{"base":"a","head":"b"}"#);
        assert!(ok.is_ok());
        let path: Result<CompareInput, _> =
            serde_json::from_str(r#"{"base":"a","head":"b","path":"/etc/passwd"}"#);
        assert!(path.is_err());
    }

    #[test]
    fn pasted_text_opens_as_a_session_with_text_sources_and_swaps() {
        let mut s = open_text_session(CompareInput {
            base: "one\ntwo\n".into(),
            head: "one\nTWO\n".into(),
        })
        .unwrap();
        let c = s.comparison().unwrap();
        assert_eq!(c.hunks.len(), 1);
        assert_eq!(c.sides[0].label, "Pasted text (base)");
        assert_eq!(c.sides[1].label, "Pasted text (head)");
        s.swap();
        let c = s.comparison().unwrap();
        assert_eq!(
            (c.base.as_str(), c.head.as_str()),
            ("one\nTWO\n", "one\ntwo\n")
        );
        assert_eq!(c.sides[0].label, "Pasted text (head)");
    }

    #[test]
    fn pasted_text_over_the_limit_is_rejected_and_empty_text_is_fine() {
        let big = "x".repeat(MAX_SIDE_BYTES + 1);
        assert!(
            open_text_session(CompareInput {
                base: big,
                head: String::new()
            })
            .is_err()
        );
        let c = open_text_session(CompareInput {
            base: String::new(),
            head: String::new(),
        })
        .unwrap()
        .comparison()
        .unwrap();
        assert!(c.hunks.is_empty());
    }
}
