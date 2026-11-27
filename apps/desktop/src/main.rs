mod cli;
mod ipc;
mod review;

/// Turn a launch request into the state to show. It reads the inputs into
/// snapshots, so the caller can return at once (Q74).
fn startup_for(launch: cli::Launch) -> Result<Startup, String> {
    match launch {
        cli::Launch::Files { left, right } => {
            let (l, r) = cli::read_snapshots(&left, &right).map_err(|e| e.to_string())?;
            Ok(Startup {
                session: Some(ipc::Session::new(l, r)),
                repository: None,
            })
        }
        cli::Launch::Repository(path) => Ok(Startup {
            session: None,
            repository: Some(ipc::RepoState::open(&path)?),
        }),
        _ => Ok(Startup::default()),
    }
}

/// Make each path argument absolute against `cwd`. An option (starting with `-`)
/// and an absolute path stay as they are.
#[cfg_attr(not(unix), allow(dead_code))]
fn absolute_args(args: &[String], cwd: &std::path::Path) -> Vec<String> {
    args.iter()
        .map(|a| {
            if a.starts_with('-') || std::path::Path::new(a).is_absolute() {
                a.clone()
            } else {
                cwd.join(a).to_string_lossy().into_owned()
            }
        })
        .collect()
}

/// Inside an AppImage the process starts in the AppImage folder, and the single
/// instance plugin forwards that folder with the arguments. So a relative path
/// would resolve against the wrong folder in the running instance. Run again
/// with absolute paths, so the forwarded arguments are right.
#[cfg(unix)]
fn reexec_with_absolute_paths(args: &[String], cwd: &std::path::Path) {
    use std::os::unix::process::CommandExt;
    let absolute = absolute_args(args, cwd);
    if absolute == args {
        return;
    }
    if let Ok(exe) = std::env::current_exe() {
        let err = std::process::Command::new(exe).args(&absolute).exec();
        eprintln!("diffy: could not run again with absolute paths: {err}");
    }
}

fn main() {
    // An AppImage sets OWD to the folder it was started in. Relative paths
    // resolve against it. The process does not change its working folder: the
    // AppImage starts in its own `usr` folder, and WebKit finds its helper
    // process by a path relative to that folder (Q75).
    let cwd = std::env::var_os("OWD")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_default();
    let args: Vec<String> = std::env::args().skip(1).collect();
    #[cfg(unix)]
    if std::env::var_os("OWD").is_some() {
        reexec_with_absolute_paths(&args, &cwd);
    }
    match cli::parse(&args, &cwd) {
        Ok(cli::Launch::Help) => println!("usage: diffy [<left> <right> | <repo-path>]"),
        Ok(cli::Launch::Version) => println!("diffy {}", env!("CARGO_PKG_VERSION")),
        Ok(launch) => run_app(launch),
        Err(e) => {
            eprintln!("diffy: {e}");
            std::process::exit(2);
        }
    }
}

#[derive(Default)]
struct Startup {
    session: Option<ipc::Session>,
    repository: Option<ipc::RepoState>,
}

/// Give a repository the review database in the app data folder.
fn attach_db(app: &tauri::AppHandle, repo: &mut ipc::RepoState) {
    use tauri::Manager;
    match app.path().app_data_dir() {
        Ok(dir) => {
            if let Err(e) = std::fs::create_dir_all(&dir) {
                eprintln!("diffy: cannot create {}: {e}", dir.display());
                return;
            }
            repo.db = Some(dir.join("diffy.db"));
        }
        Err(e) => eprintln!("diffy: no app data folder: {e}"),
    }
}

/// A second `diffy` forwards its arguments and its working folder here. The
/// inputs are read into snapshots before this returns, because on Windows a
/// second instance loses its input after 10 s (Q74, Q75). The helper cannot
/// report an error, so a failure goes to the log of this instance.
fn forward(app: &tauri::AppHandle, argv: Vec<String>, cwd: String) {
    use tauri::{Emitter, Manager};
    let args: Vec<String> = argv.into_iter().skip(1).collect();
    let launch = match cli::parse(&args, std::path::Path::new(&cwd)) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("diffy: {e}");
            return;
        }
    };
    match startup_for(launch) {
        Ok(start) => {
            if let (Some(s), Ok(mut open)) = (start.session, app.state::<ipc::Open>().0.lock()) {
                *open = Some(s);
            }
            if let (Some(mut r), Ok(mut repo)) =
                (start.repository, app.state::<ipc::Repository>().0.lock())
            {
                attach_db(app, &mut r);
                *repo = Some(r);
            }
            let _ = app.emit("diffy-changed", ());
        }
        Err(e) => eprintln!("diffy: {e}"),
    }
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_focus();
    }
}

fn run_app(launch: cli::Launch) {
    tauri::Builder::default()
        // The single instance plugin comes first (Tauri documentation). A
        // second instance forwards its arguments and exits here, before `setup`,
        // so it never reads its inputs a second time.
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            forward(app, argv, cwd);
        }))
        .manage(ipc::Open::default())
        .manage(ipc::Repository::default())
        .setup(move |app| {
            use tauri::Manager;
            // Read the inputs into snapshots now (Q74). A bad input is an error.
            let start = match startup_for(launch) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("diffy: {e}");
                    std::process::exit(1);
                }
            };
            if let Ok(mut open) = app.state::<ipc::Open>().0.lock() {
                *open = start.session;
            }
            if let Ok(mut repo) = app.state::<ipc::Repository>().0.lock() {
                *repo = start.repository.map(|mut r| {
                    attach_db(app.handle(), &mut r);
                    r
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::compare_text,
            ipc::open_text,
            ipc::report_ui_error,
            ipc::current_comparison,
            ipc::select_encoding,
            ipc::swap_sides,
            ipc::repository_files,
            ipc::open_repository_file,
            review::mark_file_reviewed,
            review::add_review_finding,
            review::review_findings,
            review::resolve_finding,
            review::findings_markdown
        ])
        .run(tauri::generate_context!())
        .expect("the Diffy window failed to start");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn files_are_read_into_a_session_before_startup_returns() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("a"), "one\n").unwrap();
        std::fs::write(d.path().join("b"), "two\n").unwrap();
        let start = startup_for(cli::Launch::Files {
            left: d.path().join("a"),
            right: d.path().join("b"),
        })
        .unwrap();
        assert!(start.session.is_some() && start.repository.is_none());
        // The inputs are in memory: removing the files does not matter now.
        std::fs::remove_file(d.path().join("a")).unwrap();
        let c = start.session.unwrap().comparison().unwrap();
        assert_eq!((c.base.as_str(), c.head.as_str()), ("one\n", "two\n"));
    }

    #[test]
    fn an_unreadable_input_is_an_error() {
        let d = tempfile::tempdir().unwrap();
        let err = startup_for(cli::Launch::Files {
            left: d.path().join("missing-a"),
            right: d.path().join("missing-b"),
        });
        assert!(err.is_err());
    }

    #[test]
    fn open_and_a_bad_repository_give_a_default_or_an_error() {
        let start = startup_for(cli::Launch::Open).unwrap();
        assert!(start.session.is_none() && start.repository.is_none());
        let d = tempfile::tempdir().unwrap();
        assert!(startup_for(cli::Launch::Repository(d.path().join("missing"))).is_err());
    }

    // The re-run exists on Unix only, and the paths below are Unix paths.
    #[cfg(unix)]
    #[test]
    fn relative_paths_become_absolute_and_options_stay() {
        let cwd = std::path::Path::new("/work");
        let args = ["a.txt", "-h", "/abs/b.txt", "sub/c"].map(String::from);
        assert_eq!(
            absolute_args(&args, cwd),
            ["/work/a.txt", "-h", "/abs/b.txt", "/work/sub/c"].map(String::from)
        );
        // Absolute arguments give the same list, so the run again happens once.
        let again = absolute_args(&absolute_args(&args, cwd), cwd);
        assert_eq!(again, absolute_args(&args, cwd));
    }

    #[test]
    fn the_config_keeps_the_prototype_unfrozen() {
        // With "freezePrototype": true, the page never ran: no command reached
        // Rust, and not even the error report did. Do not turn it on without
        // running the app and checking that the page loads its comparison.
        let config = include_str!("../tauri.conf.json");
        assert!(config.contains(r#""freezePrototype": false"#));
    }
}
