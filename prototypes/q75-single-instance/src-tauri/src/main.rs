// Q75 single instance prototype. Throwaway code. Never merge.
//
// Environment:
//   Q75_LOG           JSON lines log file (required)
//   Q75_SNAP          snapshot folder for the copies (default: next to the log)
//   Q75_SLEEP_MS      the callback sleeps this long before it returns
//   Q75_START_MS      the first instance sleeps this long before the plugin setup
//   Q75_SETUP_MS      the app setup hook blocks the main thread this long
//   Q75_FIX_OWD       set the working folder to OWD before the plugin setup
//   Q75_HOLD          "share" keeps each input open with the default Rust share mode,
//                     "noshare" keeps it open without FILE_SHARE_DELETE (Windows)
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

static MAIN_THREAD: OnceLock<std::thread::ThreadId> = OnceLock::new();
static HELD: Mutex<Vec<(PathBuf, File)>> = Mutex::new(Vec::new());
static COUNT: Mutex<u64> = Mutex::new(0);

fn now_ms() -> f64
{
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64() * 1000.0
}

fn log(v: serde_json::Value)
{
    let path = std::env::var("Q75_LOG").expect("Q75_LOG");
    let mut f = OpenOptions::new().create(true).append(true).open(path).unwrap();
    // One write with O_APPEND, so lines of two processes do not mix.
    f.write_all(format!("{v}\n").as_bytes()).unwrap();
}

fn env_ms(name: &str) -> u64
{
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(0)
}

fn open_hold(path: &Path) -> std::io::Result<File>
{
    #[cfg(windows)]
    if std::env::var("Q75_HOLD").as_deref() == Ok("noshare")
    {
        use std::os::windows::fs::OpenOptionsExt;
        // FILE_SHARE_READ | FILE_SHARE_WRITE, no FILE_SHARE_DELETE.
        return OpenOptions::new().read(true).share_mode(0x1 | 0x2).open(path);
    }
    File::open(path)
}

fn snap_dir() -> PathBuf
{
    match std::env::var("Q75_SNAP")
    {
        Ok(v) => PathBuf::from(v),
        Err(_) => PathBuf::from(std::env::var("Q75_LOG").unwrap()).with_extension("snap"),
    }
}

// Copies each existing input file into the snapshot folder, as Q74 says.
fn handle(args: &[String], cwd: &str) -> Vec<serde_json::Value>
{
    let mut out = Vec::new();
    for arg in args.iter().skip(1)
    {
        let p = Path::new(arg);
        let full = if p.is_absolute() { p.to_path_buf() } else { Path::new(cwd).join(p) };
        let meta = match fs::metadata(&full)
        {
            Ok(m) if m.is_file() => m,
            _ =>
            {
                out.push(serde_json::json!({"arg": arg, "file": false}));
                continue;
            }
        };
        let n = {
            let mut c = COUNT.lock().unwrap();
            *c += 1;
            *c
        };
        fs::create_dir_all(snap_dir()).unwrap();
        let dest = snap_dir().join(format!("{n}.bin"));
        let t = Instant::now();
        let copied = fs::copy(&full, &dest);
        let copy_ms = t.elapsed().as_secs_f64() * 1000.0;
        let mut entry = serde_json::json!({
            "arg": arg, "file": true, "size": meta.len(), "copy_ms": copy_ms,
            "copy_ok": copied.is_ok(), "copy_err": copied.err().map(|e| e.to_string()),
        });
        if std::env::var("Q75_HOLD").is_ok()
        {
            match open_hold(&full)
            {
                Ok(f) =>
                {
                    HELD.lock().unwrap().push((full.clone(), f));
                    entry["held"] = true.into();
                }
                Err(e) => entry["held_err"] = e.to_string().into(),
            }
        }
        out.push(entry);
    }
    out
}

fn main()
{
    MAIN_THREAD.set(std::thread::current().id()).unwrap();
    if std::env::var("Q75_FIX_OWD").is_ok()
    {
        // The AppImage runtime starts the app in the mount folder and keeps the
        // folder of the caller in OWD. The plugin forwards current_dir().
        if let Some(owd) = std::env::var_os("OWD")
        {
            let _ = std::env::set_current_dir(owd);
        }
    }
    let start = now_ms();
    let args: Vec<String> = std::env::args().collect();
    let cwd = std::env::current_dir().map(|p| p.display().to_string()).unwrap_or_default();
    log(serde_json::json!({
        "event": "start", "t": start, "pid": std::process::id(), "args": args, "cwd": cwd,
        "owd": std::env::var("OWD").ok(), "appimage": std::env::var("APPIMAGE").ok(),
    }));
    let start_ms = env_ms("Q75_START_MS");
    if start_ms > 0
    {
        std::thread::sleep(std::time::Duration::from_millis(start_ms));
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            let t0 = now_ms();
            if argv.iter().any(|a| a == "--q75-read-held")
            {
                log(serde_json::json!({"event": "held", "t": t0, "read": read_held()}));
                return;
            }
            if argv.iter().any(|a| a == "--q75-exit")
            {
                log(serde_json::json!({"event": "exit", "t": t0}));
                app.exit(0);
                return;
            }
            let thread = std::thread::current();
            let files = handle(&argv, &cwd);
            let sleep = env_ms("Q75_SLEEP_MS");
            if sleep > 0
            {
                std::thread::sleep(std::time::Duration::from_millis(sleep));
            }
            log(serde_json::json!({
                "event": "callback", "t": t0, "t_end": now_ms(), "pid": std::process::id(),
                "argv": argv, "cwd": cwd, "files": files,
                "main_thread": Some(&thread.id()) == MAIN_THREAD.get(),
                "thread_name": thread.name(), "thread_id": format!("{:?}", thread.id()),
            }));
        }))
        .setup(move |_app| {
            let setup_ms = env_ms("Q75_SETUP_MS");
            if setup_ms > 0
            {
                // Blocks the main thread after the plugin setup.
                std::thread::sleep(std::time::Duration::from_millis(setup_ms));
            }
            log(serde_json::json!({
                "event": "ready", "t": now_ms(), "pid": std::process::id(),
                "after_start_ms": now_ms() - start,
            }));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("run");
}

fn read_held() -> Vec<(String, usize)>
{
    let mut out = Vec::new();
    for (p, f) in HELD.lock().unwrap().iter_mut()
    {
        let mut buf = Vec::new();
        let n = f.read_to_end(&mut buf).unwrap_or(0);
        out.push((p.display().to_string(), n));
    }
    out
}
