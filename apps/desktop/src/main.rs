mod cli;

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
            if let Err(e) = cli::read_snapshots(&left, &right) {
                eprintln!("diffy: {e}");
                std::process::exit(1);
            }
        }
        Ok(_launch) => {}
        Err(e) => {
            eprintln!("diffy: {e}");
            std::process::exit(2);
        }
    }
}
