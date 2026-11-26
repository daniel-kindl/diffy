mod cli;
mod ipc;

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

fn main() {
    // An AppImage sets OWD to the folder it was started in. Change to it before
    // anything else, so relative paths and the plugin setup see that folder (Q75).
    if let Some(owd) = std::env::var_os("OWD") {
        let _ = std::env::set_current_dir(owd);
    }
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cwd = std::env::current_dir().unwrap_or_default();
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
            if let (Some(r), Ok(mut repo)) =
                (start.repository, app.state::<ipc::Repository>().0.lock())
            {
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
                *repo = start.repository;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::compare_text,
            ipc::current_comparison,
            ipc::select_encoding,
            ipc::swap_sides,
            ipc::repository_files,
            ipc::open_repository_file
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
}
