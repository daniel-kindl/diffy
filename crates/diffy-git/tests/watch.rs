use diffy_git::watch::{CoalescingQueue, RepoWatcher};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{Duration, Instant},
};

#[test]
fn the_queue_coalesces_a_burst() {
    let q = CoalescingQueue::new(10);
    for _ in 0..1000 {
        q.push(PathBuf::from("a"));
    }
    let b = q.take(Duration::from_millis(10));
    assert_eq!(b.paths, vec![PathBuf::from("a")]);
    assert!(!b.overflow);
}

#[test]
fn a_full_queue_asks_for_a_rescan() {
    let q = CoalescingQueue::new(2);
    for i in 0..5 {
        q.push(PathBuf::from(format!("f{i}")));
    }
    let b = q.take(Duration::from_millis(10));
    assert!(b.overflow);
    assert!(b.paths.is_empty());
    // The flag resets after it is taken.
    assert!(!q.take(Duration::from_millis(10)).overflow);
}

fn wait_for(w: &RepoWatcher, name: &str) -> bool {
    let end = Instant::now() + Duration::from_secs(5);
    while Instant::now() < end {
        let b = w.queue.take(Duration::from_millis(200));
        if b.overflow || b.paths.iter().any(|p| p.ends_with(name)) {
            return true;
        }
    }
    false
}

#[test]
fn sees_a_change_atomic_replace_and_rename_and_skips_ignored_files() {
    let d = tempfile::tempdir().unwrap();
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(d.path())
            .status()
            .unwrap()
            .success()
    );
    fs::write(d.path().join(".gitignore"), "ignored.txt\n").unwrap();
    let w = RepoWatcher::start(d.path(), 1000).unwrap();
    fs::write(d.path().join("a.txt"), "1").unwrap();
    assert!(wait_for(&w, "a.txt"), "a new file");
    fs::write(d.path().join("tmp"), "2").unwrap();
    fs::rename(d.path().join("tmp"), d.path().join("a.txt")).unwrap();
    assert!(wait_for(&w, "a.txt"), "an atomic replace");
    fs::rename(d.path().join("a.txt"), d.path().join("b.txt")).unwrap();
    assert!(wait_for(&w, "b.txt"), "a rename");
    fs::remove_file(d.path().join("b.txt")).unwrap();
    assert!(wait_for(&w, "b.txt"), "a delete");
    // Drain, then an ignored file must not show, and the index must.
    while !w.queue.take(Duration::from_millis(300)).paths.is_empty() {}
    fs::write(d.path().join("ignored.txt"), "x").unwrap();
    std::thread::sleep(Duration::from_millis(500));
    let b = w.queue.take(Duration::from_millis(100));
    assert!(!b.paths.iter().any(|p| p.ends_with("ignored.txt")), "{b:?}");
    fs::write(d.path().join("c.txt"), "c").unwrap();
    assert!(
        Command::new("git")
            .args(["add", "c.txt"])
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .current_dir(d.path())
            .status()
            .unwrap()
            .success()
    );
    assert!(wait_for(&w, "index"), "the Git index");
}
