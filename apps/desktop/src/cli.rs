//! Command line parsing (C-09, Q24, Q74).
//!
//! `diffy <left> <right>`, `diffy <repo-path>` and `git difftool` (which runs
//! `diffy <local> <remote>`). All are read-only. A second instance forwards
//! its arguments and its working folder, so relative paths resolve against the
//! folder it sends. Inputs are read into snapshots before the callback returns.

use diffy_comparison::{Snapshot, Source};
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq)]
pub enum Launch {
    /// No argument: just open the app.
    Open,
    Files {
        left: PathBuf,
        right: PathBuf,
    },
    Folders {
        left: PathBuf,
        right: PathBuf,
    },
    Repository(PathBuf),
    Help,
    Version,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CliError {
    TooManyArguments,
    UnknownOption(String),
    NotFound(PathBuf),
    /// One side is a file and the other a folder.
    MixedKinds,
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooManyArguments => f.write_str("usage: diffy [<left> <right> | <repo-path>]"),
            Self::UnknownOption(o) => write!(f, "unknown option {o}"),
            Self::NotFound(p) => write!(f, "{} does not exist", p.display()),
            Self::MixedKinds => f.write_str("compare two files or two folders, not one of each"),
        }
    }
}

impl std::error::Error for CliError {}

/// Parse the arguments after the program name. `cwd` is the working folder
/// of the process that was started: the `OWD` of an AppImage, or the folder
/// that a second instance sends.
pub fn parse(args: &[String], cwd: &Path) -> Result<Launch, CliError> {
    let mut paths = Vec::new();
    for a in args {
        match a.as_str() {
            "-h" | "--help" => return Ok(Launch::Help),
            "-V" | "--version" => return Ok(Launch::Version),
            o if o.starts_with('-') && o.len() > 1 => {
                return Err(CliError::UnknownOption(o.into()));
            }
            p => paths.push(cwd.join(p)),
        }
    }
    for p in &paths {
        if !p.exists() {
            return Err(CliError::NotFound(p.clone()));
        }
    }
    match paths.len() {
        0 => Ok(Launch::Open),
        1 => {
            let p = paths.remove(0);
            // A file argument alone has nothing to compare it with.
            if p.is_dir() {
                Ok(Launch::Repository(p))
            } else {
                Err(CliError::TooManyArguments)
            }
        }
        2 => {
            let right = paths.remove(1);
            let left = paths.remove(0);
            match (left.is_dir(), right.is_dir()) {
                (false, false) => Ok(Launch::Files { left, right }),
                (true, true) => Ok(Launch::Folders { left, right }),
                _ => Err(CliError::MixedKinds),
            }
        }
        _ => Err(CliError::TooManyArguments),
    }
}

/// Read both files into snapshots, with the default Rust share mode. Call it
/// in the single instance callback, before it returns (Q74).
pub fn read_snapshots(left: &Path, right: &Path) -> std::io::Result<(Snapshot, Snapshot)> {
    let read = |p: &Path| -> std::io::Result<Snapshot> {
        Ok(Snapshot::new(
            Source::File(p.display().to_string()),
            std::fs::read(p)?,
        ))
    };
    Ok((read(left)?, read(right)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn args(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| (*s).to_owned()).collect()
    }

    fn tree() -> tempfile::TempDir {
        let d = tempfile::tempdir().unwrap();
        fs::write(d.path().join("a.txt"), "a\n").unwrap();
        fs::write(d.path().join("b.txt"), "b\n").unwrap();
        fs::create_dir(d.path().join("repo")).unwrap();
        fs::create_dir(d.path().join("repo2")).unwrap();
        d
    }

    #[test]
    fn no_argument_opens_the_app() {
        assert_eq!(parse(&[], Path::new("/")), Ok(Launch::Open));
    }

    #[test]
    fn two_files_compare_and_paths_resolve_against_the_sent_folder() {
        let d = tree();
        let got = parse(&args(&["a.txt", "b.txt"]), d.path()).unwrap();
        assert_eq!(
            got,
            Launch::Files {
                left: d.path().join("a.txt"),
                right: d.path().join("b.txt")
            }
        );
    }

    #[test]
    fn git_difftool_style_absolute_paths_work() {
        let d = tree();
        let a = d.path().join("a.txt").display().to_string();
        let b = d.path().join("b.txt").display().to_string();
        assert!(matches!(
            parse(&[a, b], Path::new("/elsewhere")),
            Ok(Launch::Files { .. })
        ));
    }

    #[test]
    fn one_folder_is_a_repository_and_two_folders_compare() {
        let d = tree();
        assert_eq!(
            parse(&args(&["repo"]), d.path()),
            Ok(Launch::Repository(d.path().join("repo")))
        );
        assert!(matches!(
            parse(&args(&["repo", "repo2"]), d.path()),
            Ok(Launch::Folders { .. })
        ));
    }

    #[test]
    fn errors_are_reported_not_guessed() {
        let d = tree();
        assert_eq!(
            parse(&args(&["a.txt", "repo"]), d.path()),
            Err(CliError::MixedKinds)
        );
        assert_eq!(
            parse(&args(&["a.txt"]), d.path()),
            Err(CliError::TooManyArguments)
        );
        assert_eq!(
            parse(&args(&["a", "b", "c"]), d.path()),
            Err(CliError::NotFound(d.path().join("a")))
        );
        assert_eq!(
            parse(&args(&["--nope"]), d.path()),
            Err(CliError::UnknownOption("--nope".into()))
        );
        assert_eq!(parse(&args(&["--help"]), d.path()), Ok(Launch::Help));
    }

    #[test]
    fn inputs_are_read_into_snapshots_and_the_files_stay_as_they_are() {
        let d = tree();
        let (l, r) = read_snapshots(&d.path().join("a.txt"), &d.path().join("b.txt")).unwrap();
        assert_eq!((l.raw(), r.raw()), (&b"a\n"[..], &b"b\n"[..]));
        assert_eq!(fs::read(d.path().join("a.txt")).unwrap(), b"a\n");
    }
}
