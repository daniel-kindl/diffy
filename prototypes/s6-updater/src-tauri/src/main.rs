// Q55 S6 prototype: a minimal Tauri app that migrates a test database at
// startup and then checks one update channel. Throwaway code, never merged.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::{Value, json};
use tauri::{AppHandle, Manager};
use tauri_plugin_updater::UpdaterExt;

// The schema version of this build. The newer test release is built with
// Q55_SCHEMA=2, so it has one more migration.
const SCHEMA: u32 = match option_env!("Q55_SCHEMA") {
    Some(s) => s.as_bytes()[0] as u32 - b'0' as u32,
    None => 1,
};

const MIGRATIONS: [&str; 2] = [
    "CREATE TABLE notes (id INTEGER PRIMARY KEY, body TEXT NOT NULL);
     INSERT INTO notes (body) VALUES ('user data written by v0.1.0');",
    "ALTER TABLE notes ADD COLUMN tag TEXT NOT NULL DEFAULT '';
     CREATE TABLE seen (id INTEGER PRIMARY KEY);
     UPDATE notes SET tag = 'migrated by m2';",
];

fn log(dir: &Path, mut v: Value) {
    v["version"] = json!(env!("CARGO_PKG_VERSION"));
    v["pid"] = json!(std::process::id());
    v["time_ms"] = json!(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    );
    if let Ok(mut f) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("q55-log.jsonl"))
    {
        let _ = writeln!(f, "{v}");
    }
}

/// Runs the pending migrations. Before each migration it writes a copy of the
/// database with VACUUM INTO, and each migration runs in one transaction.
fn migrate(dir: &Path, fail_at: Option<u32>) -> Result<Value, String> {
    let db_path = dir.join("data.sqlite");
    let mut conn = rusqlite::Connection::open(&db_path).map_err(|e| e.to_string())?;
    let mut current: u32 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    let start = current;
    while current < SCHEMA {
        let next = current + 1;
        if current > 0 {
            let backup = dir.join(format!("data.pre-m{next}.sqlite"));
            let _ = std::fs::remove_file(&backup);
            conn.execute("VACUUM INTO ?1", [backup.to_string_lossy()])
                .map_err(|e| format!("backup before m{next}: {e}"))?;
        }
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        tx.execute_batch(MIGRATIONS[(next - 1) as usize])
            .map_err(|e| format!("m{next}: {e}"))?;
        if fail_at == Some(next) {
            // The transaction drops without a commit, so SQLite rolls it back.
            return Err(format!("m{next}: forced failure after the statements ran"));
        }
        tx.pragma_update(None, "user_version", next)
            .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        current = next;
    }
    let rows: Vec<String> = conn
        .prepare("SELECT body FROM notes ORDER BY id")
        .and_then(|mut s| {
            s.query_map([], |r| r.get::<_, String>(0))?
                .collect::<Result<Vec<_>, _>>()
        })
        .map_err(|e| e.to_string())?;
    Ok(json!({ "from": start, "to": current, "rows": rows }))
}

fn read_config(dir: &Path) -> Value {
    std::fs::read_to_string(dir.join("q55-config.json"))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(json!({}))
}

async fn run(app: AppHandle, dir: PathBuf) -> i32 {
    let cfg = read_config(&dir);
    log(
        &dir,
        json!({
            "event": "start",
            "schema": SCHEMA,
            "exe": std::env::current_exe().ok(),
            "appimage": std::env::var("APPIMAGE").ok(),
            "args": std::env::args().collect::<Vec<_>>(),
        }),
    );

    let fail_at = cfg["fail_migration"].as_u64().map(|n| n as u32);
    match migrate(&dir, fail_at) {
        Ok(v) => log(&dir, json!({ "event": "migrated", "result": v })),
        Err(e) => {
            log(&dir, json!({ "event": "migration_failed", "error": e }));
            return 3;
        }
    }

    // Diffy selects the channel in Rust (Q50). Each channel has its own
    // endpoint (Q72).
    let channel = cfg["channel"].as_str().unwrap_or("stable").to_string();
    let Some(url) = cfg["endpoints"][&channel].as_str() else {
        log(&dir, json!({ "event": "no_endpoint", "channel": channel }));
        return 0;
    };
    let timeout = cfg["timeout_s"].as_u64().unwrap_or(30);
    let updater = match app
        .updater_builder()
        .endpoints(vec![url.parse().expect("endpoint URL")])
        .and_then(|b| b.timeout(Duration::from_secs(timeout)).build())
    {
        Ok(u) => u,
        Err(e) => {
            log(&dir, json!({ "event": "updater_error", "error": e.to_string() }));
            return 4;
        }
    };

    let update = match updater.check().await {
        Ok(Some(u)) => u,
        Ok(None) => {
            log(&dir, json!({ "event": "no_update", "channel": channel }));
            return 0;
        }
        Err(e) => {
            log(
                &dir,
                json!({ "event": "check_failed", "channel": channel, "error": e.to_string() }),
            );
            return 4;
        }
    };
    log(
        &dir,
        json!({ "event": "update_found", "channel": channel, "new": update.version }),
    );
    if cfg["install"].as_bool() == Some(false) {
        log(&dir, json!({ "event": "update_skipped" }));
        return 0;
    }
    // A test pause, so the harness can add a fault after the app started.
    if let Some(p) = cfg["pause_before_install_s"].as_u64() {
        log(&dir, json!({ "event": "paused", "seconds": p }));
        tokio::time::sleep(Duration::from_secs(p)).await;
    }

    let mut got = 0usize;
    let result = update
        .download_and_install(|n, _| got += n, || {})
        .await;
    match result {
        Ok(()) => {
            // Windows does not get here: the updater starts the NSIS installer
            // and exits the app. On Linux the app restarts itself.
            log(&dir, json!({ "event": "installed", "bytes": got }));
            log(&dir, json!({ "event": "restarting" }));
            app.restart();
        }
        Err(e) => {
            log(
                &dir,
                json!({ "event": "install_failed", "bytes": got, "error": e.to_string() }),
            );
            4
        }
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let handle = app.handle().clone();
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            tauri::async_runtime::spawn(async move {
                let code = run(handle.clone(), dir).await;
                handle.exit(code);
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("run the Q55 app");
}
