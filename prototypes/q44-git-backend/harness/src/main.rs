//! Q44 harness.
//!
//! `harness oracle <fixtures> <expected.json>` asks the `git` CLI each operation, with no
//! system or global configuration, on a copy of each fixture repository.
//!
//! `harness check <fixtures> <expected.json> <backend-binary> <results.json> [--no-write-permission]`
//! runs the backend on a fresh copy of each fixture and compares each answer with the oracle.
//! It never runs `git`. Before and after each operation it takes a snapshot of each file and
//! directory in the copy, `.git` included: type, permission bits, size, modification time and
//! the SHA-256 of the content or the link target. With `--no-write-permission`, the copy has no
//! write permission for anyone. After each operation it also lists `<fixtures>/markers`, where
//! each command that the untrusted-config fixture names leaves a file when it runs.

use model::{Answer, DiffEntry, Entry, StatusEntry, Submodule, path_text, sha256_hex};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::Command;

type R<T> = Result<T, Box<dyn std::error::Error>>;

fn main() -> R<()> {
    let args: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let usage = "usage: harness oracle <fixtures> <expected.json> | harness check <fixtures> <expected.json> <backend> <results.json> [--no-write-permission]";
    match args.first().and_then(|a| a.to_str()) {
        Some("oracle") if args.len() == 3 => oracle(&args[1], &args[2]),
        Some("check") if args.len() == 5 || args.len() == 6 => {
            let no_write = match args.get(5) {
                None => false,
                Some(a) if a == OsStr::new("--no-write-permission") => true,
                _ => return Err(usage.into()),
            };
            let failed = check(&args[1], &args[2], &args[3], &args[4], no_write)?;
            std::process::exit(if failed { 1 } else { 0 });
        }
        _ => Err(usage.into()),
    }
}

fn repos(fixtures: &Path) -> R<Vec<(String, PathBuf)>> {
    let mut repos = Vec::new();
    for e in std::fs::read_dir(fixtures.join("repos"))? {
        let e = e?;
        repos.push((
            e.file_name().into_string().map_err(|_| "fixture name")?,
            e.path(),
        ));
    }
    repos.sort();
    Ok(repos)
}

fn scratch(name: &str) -> R<PathBuf> {
    let base = std::env::var_os("Q44_SCRATCH")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let dir = base.join(format!("q44-{}-{name}", std::process::id()));
    if dir.exists() {
        remove(&dir)?;
    }
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

// ---------- oracle ----------

fn git(repo: &Path, args: &[&str]) -> R<Vec<u8>> {
    let out = git_status(repo, args)?;
    if !out.status.success() {
        return Err(format!(
            "git {args:?} in {}: {}",
            repo.display(),
            String::from_utf8_lossy(&out.stderr)
        )
        .into());
    }
    Ok(out.stdout)
}

fn git_status(repo: &Path, args: &[&str]) -> R<std::process::Output> {
    let empty = std::env::temp_dir().join("q44-oracle-home");
    std::fs::create_dir_all(&empty)?;
    Ok(Command::new("git")
        .arg("-C")
        .arg(repo)
        .arg("--no-optional-locks")
        .args(args)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("HOME", &empty)
        .env("XDG_CONFIG_HOME", &empty)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .output()?)
}

fn text(bytes: Vec<u8>) -> R<String> {
    Ok(String::from_utf8(bytes)?.trim_end().to_owned())
}

fn fields(bytes: &[u8]) -> Vec<&[u8]> {
    let mut v: Vec<&[u8]> = bytes.split(|b| *b == 0).collect();
    if v.last().is_some_and(|l| l.is_empty()) {
        v.pop();
    }
    v
}

fn blob_sha(repo: &Path, oid: &str) -> R<String> {
    Ok(sha256_hex(&git(repo, &["cat-file", "blob", oid])?))
}

fn oracle(fixtures: &Path, expected: &Path) -> R<()> {
    let version = text(git(fixtures, &["version"])?)?;
    let mut all = BTreeMap::new();
    for (name, src) in repos(fixtures)? {
        let dir = scratch("oracle")?;
        let repo = dir.join(&name);
        copy(&src, &repo)?;
        let mut answers = Vec::new();

        let oid = text(git(&repo, &["rev-parse", "HEAD"])?)?;
        let sym = git_status(&repo, &["symbolic-ref", "-q", "HEAD"])?;
        let branch = sym
            .status
            .success()
            .then(|| path_text(sym.stdout.trim_ascii_end()));
        answers.push(Answer::Head { oid, branch });

        let mut refs = Vec::new();
        let raw = git(
            &repo,
            &["for-each-ref", "--format=%(objectname) %(refname)"],
        )?;
        for line in raw.split(|b| *b == b'\n').filter(|l| !l.is_empty()) {
            let (oid, name) = line.split_at(40);
            refs.push((path_text(&name[1..]), String::from_utf8(oid.to_vec())?));
        }
        answers.push(Answer::Refs { refs });

        answers.push(status(&repo)?);

        let mut entries = Vec::new();
        for rec in fields(&git(&repo, &["ls-files", "-s", "-z"])?) {
            let tab = rec.iter().position(|b| *b == b'\t').ok_or("ls-files tab")?;
            let meta: Vec<&str> = std::str::from_utf8(&rec[..tab])?.split(' ').collect();
            entries.push(Entry {
                path: path_text(&rec[tab + 1..]),
                mode: meta[0].into(),
                oid: meta[1].into(),
                stage: meta[2].parse()?,
                content_sha256: (meta[0] != "160000")
                    .then(|| blob_sha(&repo, meta[1]))
                    .transpose()?,
            });
        }
        answers.push(Answer::Index { entries });

        answers.push(submodules(&repo)?);

        let revs = text(git(&repo, &["rev-list", "--all", "--parents"])?)?;
        for line in revs.lines() {
            let ids: Vec<&str> = line.split(' ').collect();
            answers.push(tree(&repo, ids[0])?);
            if let Some(parent) = ids.get(1) {
                answers.push(diff(&repo, parent, ids[0])?);
            }
        }
        all.insert(
            name,
            answers.into_iter().map(model::sorted).collect::<Vec<_>>(),
        );
        remove(&dir)?;
    }
    // The oracle is Git, which runs the configured commands. Clear their markers.
    markers(fixtures)?;
    let doc = json!({ "oracle": version, "fixtures": all });
    std::fs::write(expected, serde_json::to_string_pretty(&doc)? + "\n")?;
    eprintln!(
        "oracle: {version}, {} fixtures",
        doc["fixtures"].as_object().map_or(0, |o| o.len())
    );
    Ok(())
}

fn status(repo: &Path) -> R<Answer> {
    let raw = git(
        repo,
        &[
            "status",
            "--porcelain=v2",
            "-z",
            "--untracked-files=all",
            "--ignored=no",
            "--ignore-submodules=none",
        ],
    )?;
    let recs = fields(&raw);
    let mut entries = Vec::new();
    let mut i = 0;
    while i < recs.len() {
        let rec = recs[i];
        let letters = |r: &[u8]| (r[2] as char, r[3] as char);
        match rec[0] {
            b'1' => {
                let path = rec
                    .splitn(9, |b| *b == b' ')
                    .nth(8)
                    .ok_or("status 1 path")?;
                let (x, y) = letters(rec);
                entries.push(StatusEntry {
                    path: path_text(path),
                    index: x,
                    worktree: y,
                    orig: None,
                });
            }
            b'2' => {
                let path = rec
                    .splitn(10, |b| *b == b' ')
                    .nth(9)
                    .ok_or("status 2 path")?;
                let (x, y) = letters(rec);
                i += 1;
                entries.push(StatusEntry {
                    path: path_text(path),
                    index: x,
                    worktree: y,
                    orig: Some(path_text(recs[i])),
                });
            }
            b'?' => entries.push(StatusEntry {
                path: path_text(&rec[2..]),
                index: '?',
                worktree: '?',
                orig: None,
            }),
            _ => return Err(format!("status record {:?}", String::from_utf8_lossy(rec)).into()),
        }
        i += 1;
    }
    Ok(Answer::Status { entries })
}

fn submodules(repo: &Path) -> R<Answer> {
    let mut entries = Vec::new();
    if !repo.join(".gitmodules").exists() {
        return Ok(Answer::Submodules { entries });
    }
    let out = git(
        repo,
        &[
            "config",
            "-f",
            ".gitmodules",
            "-z",
            "--get-regexp",
            r"^submodule\..*\.path$",
        ],
    )?;
    for rec in fields(&out) {
        let nl = rec
            .iter()
            .position(|b| *b == b'\n')
            .ok_or("config record")?;
        let path = &rec[nl + 1..];
        let path_str = std::str::from_utf8(path)?;
        let staged = git(repo, &["ls-files", "-s", "--", path_str])?;
        let index_oid = std::str::from_utf8(&staged)?
            .split(' ')
            .nth(1)
            .map(str::to_owned);
        let head = repo.join(path_str);
        let head_oid = if head.join(".git").exists() {
            Some(text(git(&head, &["rev-parse", "HEAD"])?)?)
        } else {
            None
        };
        entries.push(Submodule {
            path: path_text(path),
            index_oid,
            head_oid,
        });
    }
    Ok(Answer::Submodules { entries })
}

fn tree(repo: &Path, rev: &str) -> R<Answer> {
    let mut entries = Vec::new();
    for rec in fields(&git(repo, &["ls-tree", "-r", "-z", rev])?) {
        let tab = rec.iter().position(|b| *b == b'\t').ok_or("ls-tree tab")?;
        let meta: Vec<&str> = std::str::from_utf8(&rec[..tab])?.split(' ').collect();
        entries.push(Entry {
            path: path_text(&rec[tab + 1..]),
            mode: meta[0].into(),
            oid: meta[2].into(),
            stage: 0,
            content_sha256: (meta[1] == "blob")
                .then(|| blob_sha(repo, meta[2]))
                .transpose()?,
        });
    }
    Ok(Answer::Tree {
        rev: rev.into(),
        entries,
    })
}

fn diff(repo: &Path, from: &str, to: &str) -> R<Answer> {
    let raw = git(
        repo,
        &[
            "diff-tree",
            "-r",
            "-z",
            "-M",
            "--name-status",
            "--no-commit-id",
            from,
            to,
        ],
    )?;
    let recs = fields(&raw);
    let mut entries = Vec::new();
    let mut i = 0;
    while i < recs.len() {
        let status = recs[i][0] as char;
        if status == 'R' || status == 'C' {
            entries.push(DiffEntry {
                status,
                path: path_text(recs[i + 2]),
                orig: Some(path_text(recs[i + 1])),
            });
            i += 3;
        } else {
            entries.push(DiffEntry {
                status,
                path: path_text(recs[i + 1]),
                orig: None,
            });
            i += 2;
        }
    }
    Ok(Answer::Diff {
        from: from.into(),
        to: to.into(),
        entries,
    })
}

// ---------- check ----------

fn op_args(a: &Answer) -> Vec<String> {
    match a {
        Answer::Head { .. } => vec!["head".into()],
        Answer::Refs { .. } => vec!["refs".into()],
        Answer::Status { .. } => vec!["status".into()],
        Answer::Index { .. } => vec!["index".into()],
        Answer::Submodules { .. } => vec!["submodules".into()],
        Answer::Tree { rev, .. } => vec!["tree".into(), rev.clone()],
        Answer::Diff { from, to, .. } => vec!["diff".into(), from.clone(), to.clone()],
        Answer::Error { .. } => unreachable!(),
    }
}

fn check(
    fixtures: &Path,
    expected: &Path,
    backend: &Path,
    results: &Path,
    no_write: bool,
) -> R<bool> {
    let doc: Value = serde_json::from_str(&std::fs::read_to_string(expected)?)?;
    let backend = std::fs::canonicalize(backend)?;
    let mut out = BTreeMap::new();
    let (mut passed, mut total, mut unchanged, mut no_command) = (0, 0, 0, 0);
    if !markers(fixtures)?.is_empty() {
        return Err("markers exist before the check".into());
    }
    for (name, src) in repos(fixtures)? {
        let wanted: Vec<Answer> = serde_json::from_value(doc["fixtures"][&name].clone())?;
        let dir = scratch("check")?;
        let repo = dir.join(&name);
        copy(&src, &repo)?;
        if no_write {
            set_writable(&repo, false)?;
        }
        let mut ops = Vec::new();
        for want in &wanted {
            let args = op_args(want);
            let before = snapshot(&repo)?;
            let run = Command::new(&backend).arg(&repo).args(&args).output()?;
            let after = snapshot(&repo)?;
            let commands = markers(fixtures)?;
            let got: Answer =
                serde_json::from_slice(&run.stdout).unwrap_or_else(|e| Answer::Error {
                    message: format!("bad output ({e}): {}", String::from_utf8_lossy(&run.stderr)),
                });
            let same = &got == want;
            let changed = changes(&before, &after);
            total += 1;
            passed += same as usize;
            unchanged += changed.is_empty() as usize;
            no_command += commands.is_empty() as usize;
            let mut rec = json!({
                "op": args.join(" "),
                "pass": same,
                "repository_unchanged": changed.is_empty(),
                "no_configured_command": commands.is_empty(),
            });
            if !commands.is_empty() {
                rec["commands_run"] = json!(commands);
            }
            if !same {
                rec["expected"] = serde_json::to_value(want)?;
                rec["actual"] = serde_json::to_value(&got)?;
            }
            if !changed.is_empty() {
                rec["changed_paths"] = json!(changed);
            }
            ops.push(rec);
        }
        if no_write {
            set_writable(&repo, true)?;
        }
        remove(&dir)?;
        out.insert(name, ops);
    }
    let summary = json!({
        "backend": backend.file_name().map(|n| n.to_string_lossy().into_owned()),
        "oracle": doc["oracle"],
        "no_write_permission": no_write,
        "operations": total,
        "pass": passed,
        "repository_unchanged": unchanged,
        "no_configured_command": no_command,
    });
    eprintln!("{summary}");
    std::fs::write(
        results,
        serde_json::to_string_pretty(&json!({ "summary": summary, "fixtures": out }))? + "\n",
    )?;
    Ok(passed != total || unchanged != total || no_command != total)
}

/// Lists and removes the marker files. Each name is `<command>-<pid>`.
fn markers(fixtures: &Path) -> R<Vec<String>> {
    let dir = fixtures.join("markers");
    let mut names = Vec::new();
    for e in std::fs::read_dir(&dir)? {
        let e = e?;
        let name = e.file_name().to_string_lossy().into_owned();
        names.push(name.split('-').next().unwrap_or_default().to_owned());
        std::fs::remove_file(e.path())?;
    }
    names.sort();
    Ok(names)
}

// ---------- files ----------

type Snapshot = BTreeMap<Vec<u8>, String>;

fn snapshot(root: &Path) -> R<Snapshot> {
    let mut map = Snapshot::new();
    walk(root, root, &mut map)?;
    Ok(map)
}

fn walk(root: &Path, path: &Path, map: &mut Snapshot) -> R<()> {
    let meta = std::fs::symlink_metadata(path)?;
    let rel = path.strip_prefix(root)?.as_os_str().as_bytes().to_vec();
    let base = format!(
        "mode={:o} size={} mtime={}.{:09}",
        meta.mode(),
        meta.size(),
        meta.mtime(),
        meta.mtime_nsec()
    );
    let ft = meta.file_type();
    let desc = if ft.is_symlink() {
        format!(
            "link {base} target={}",
            sha256_hex(std::fs::read_link(path)?.as_os_str().as_bytes())
        )
    } else if ft.is_dir() {
        let mut children: Vec<_> = std::fs::read_dir(path)?.collect::<Result<_, _>>()?;
        children.sort_by_key(|c| c.file_name());
        let names: Vec<u8> = children
            .iter()
            .flat_map(|c| [c.file_name().as_bytes(), b"\0"].concat())
            .collect();
        for c in &children {
            walk(root, &c.path(), map)?;
        }
        format!("dir {base} entries={}", sha256_hex(&names))
    } else {
        // Reading needs the read permission, which the no-write copy keeps.
        format!("file {base} sha256={}", sha256_hex(&std::fs::read(path)?))
    };
    map.insert(rel, desc);
    Ok(())
}

fn changes(before: &Snapshot, after: &Snapshot) -> Vec<String> {
    let mut out = Vec::new();
    for (k, v) in before {
        match after.get(k) {
            None => out.push(format!("removed {}", path_text(k))),
            Some(w) if w != v => out.push(format!("changed {}: {v} -> {w}", path_text(k))),
            _ => {}
        }
    }
    for k in after.keys().filter(|k| !before.contains_key(*k)) {
        out.push(format!("added {}", path_text(k)));
    }
    out
}

fn copy(src: &Path, dst: &Path) -> R<()> {
    let meta = std::fs::symlink_metadata(src)?;
    if meta.file_type().is_symlink() {
        std::os::unix::fs::symlink(std::fs::read_link(src)?, dst)?;
    } else if meta.is_dir() {
        std::fs::create_dir(dst)?;
        for e in std::fs::read_dir(src)? {
            let e = e?;
            copy(&e.path(), &dst.join(e.file_name()))?;
        }
        std::fs::set_permissions(dst, meta.permissions())?;
    } else {
        std::fs::copy(src, dst)?;
    }
    Ok(())
}

fn set_writable(path: &Path, writable: bool) -> R<()> {
    let meta = std::fs::symlink_metadata(path)?;
    if meta.file_type().is_symlink() {
        return Ok(());
    }
    let mut mode = meta.permissions().mode();
    if writable {
        mode |= 0o200;
    } else {
        mode &= !0o222;
    }
    if meta.is_dir() && writable {
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))?;
    }
    if meta.is_dir() {
        for e in std::fs::read_dir(path)? {
            set_writable(&e?.path(), writable)?;
        }
    }
    if !(meta.is_dir() && writable) {
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))?;
    }
    Ok(())
}

fn remove(dir: &Path) -> R<()> {
    set_writable(dir, true)?;
    Ok(std::fs::remove_dir_all(dir)?)
}
