use diffy_git::{ChangeKind, Head, Repo};
use std::{fs, path::Path, process::Command};

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@example.com")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@example.com")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap().trim().to_owned()
}

fn fixture() -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    git(d.path(), &["init", "-q", "-b", "main"]);
    fs::write(d.path().join("a.txt"), "one\ntwo\nthree\n").unwrap();
    fs::write(d.path().join("old.txt"), "same content\nfor a rename\n").unwrap();
    fs::write(d.path().join("gone.txt"), "bye\n").unwrap();
    fs::write(d.path().join("bin.dat"), [0u8, 159, 146, 150]).unwrap();
    git(d.path(), &["add", "."]);
    git(d.path(), &["commit", "-q", "-m", "first"]);
    fs::write(d.path().join("a.txt"), "one\nTWO\nthree\n").unwrap();
    git(d.path(), &["mv", "old.txt", "new.txt"]);
    git(d.path(), &["rm", "-q", "gone.txt"]);
    fs::write(d.path().join("added.txt"), "new\n").unwrap();
    fs::write(d.path().join("bin.dat"), [0u8, 1, 2, 3]).unwrap();
    git(d.path(), &["add", "."]);
    git(d.path(), &["commit", "-q", "-m", "second"]);
    d
}

#[test]
fn lists_changes_with_exact_renames() {
    let d = fixture();
    let repo = Repo::discover(d.path()).unwrap();
    let c = repo.changes("HEAD~1", "HEAD").unwrap();
    let got: Vec<(String, &ChangeKind)> = c.iter().map(|c| (c.path.clone(), &c.kind)).collect();
    assert_eq!(
        got,
        vec![
            ("a.txt".into(), &ChangeKind::Modified),
            ("added.txt".into(), &ChangeKind::Added),
            ("bin.dat".into(), &ChangeKind::Modified),
            ("gone.txt".into(), &ChangeKind::Deleted),
            (
                "new.txt".into(),
                &ChangeKind::Renamed {
                    from: "old.txt".into()
                }
            ),
        ]
    );
    // The same answer as the git CLI.
    let names = git(
        d.path(),
        &["diff", "--name-only", "--no-renames", "HEAD~1", "HEAD"],
    );
    assert_eq!(
        names.lines().count(),
        6,
        "git lists old.txt and new.txt apart"
    );
}

#[test]
fn reads_files_at_a_revision() {
    let d = fixture();
    let repo = Repo::discover(d.path()).unwrap();
    assert_eq!(
        repo.read_file("HEAD~1", "a.txt").unwrap().unwrap(),
        b"one\ntwo\nthree\n"
    );
    assert_eq!(
        repo.read_file("HEAD", "bin.dat").unwrap().unwrap(),
        [0, 1, 2, 3]
    );
    assert!(repo.read_file("HEAD", "gone.txt").unwrap().is_none());
}

#[test]
fn reports_branch_and_detached_head() {
    let d = fixture();
    let repo = Repo::discover(d.path()).unwrap();
    assert_eq!(repo.head().unwrap(), Head::Branch("main".into()));
    let id = git(d.path(), &["rev-parse", "HEAD"]);
    git(d.path(), &["checkout", "-q", "--detach"]);
    let repo = Repo::discover(d.path()).unwrap();
    assert_eq!(repo.head().unwrap(), Head::Detached(id));
}

#[test]
fn reports_an_unborn_branch() {
    let d = tempfile::tempdir().unwrap();
    git(d.path(), &["init", "-q", "-b", "main"]);
    let repo = Repo::discover(d.path()).unwrap();
    assert_eq!(repo.head().unwrap(), Head::Unborn("main".into()));
}

#[test]
fn reading_does_not_change_the_repository_or_run_a_filter() {
    let d = fixture();
    let marker = d.path().join("filter-ran");
    let cfg = d.path().join(".git/config");
    let mut text = fs::read_to_string(&cfg).unwrap();
    text.push_str(&format!(
        "[filter \"evil\"]\n\tclean = touch {}\n\tsmudge = touch {}\n",
        marker.display(),
        marker.display()
    ));
    fs::write(&cfg, text).unwrap();
    fs::write(d.path().join(".gitattributes"), "*.txt filter=evil\n").unwrap();
    let before = git(d.path(), &["status", "--porcelain=v1", "--ignored"]);
    // The git CLI runs the filter itself, so clear its marker first.
    fs::remove_file(&marker).ok();
    let snapshot = fs::read(d.path().join(".git/index")).unwrap();
    let repo = Repo::discover(d.path()).unwrap();
    repo.changes("HEAD~1", "HEAD").unwrap();
    repo.read_file("HEAD", "a.txt").unwrap();
    assert!(!marker.exists(), "a filter command ran");
    assert_eq!(fs::read(d.path().join(".git/index")).unwrap(), snapshot);
    assert_eq!(
        git(d.path(), &["status", "--porcelain=v1", "--ignored"]),
        before
    );
}
