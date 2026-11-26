mod cli;
mod ipc;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // An AppImage sets OWD to the folder it was started in (Q75).
    let cwd = std::env::var_os("OWD")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_default();
    match cli::parse(&args, &cwd) {
        Ok(cli::Launch::Help) => println!("usage: diffy [<left> <right> | <repo-path>]"),
        Ok(cli::Launch::Version) => println!("diffy {}", env!("CARGO_PKG_VERSION")),
        Ok(cli::Launch::Files { left, right }) => {
            // The shell will open a tab from these snapshots. Reading them
            // here checks the inputs before the app starts.
            let initial = cli::read_snapshots(&left, &right)
                .map_err(|e| e.to_string())
                .map(|(l, r)| ipc::Session::new(l, r));
            match initial {
                Ok(s) => run_app(Startup {
                    session: Some(s),
                    repository: None,
                }),
                Err(e) => {
                    eprintln!("diffy: {e}");
                    std::process::exit(1);
                }
            }
        }
        Ok(cli::Launch::Repository(path)) => match ipc::RepoState::open(&path) {
            Ok(r) => run_app(Startup {
                session: None,
                repository: Some(r),
            }),
            Err(e) => {
                eprintln!("diffy: {e}");
                std::process::exit(1);
            }
        },
        Ok(_launch) => run_app(Startup::default()),
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

fn run_app(start: Startup) {
    tauri::Builder::default()
        .manage(ipc::Open(std::sync::Mutex::new(start.session)))
        .manage(ipc::Repository(std::sync::Mutex::new(start.repository)))
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
