//! Throwaway S1 prototype for map question Q41. Never merged.
//!
//! The Rust side gives the frontend the fixture text, the canonical hunks and
//! the facts that the benchmark harness records. Until Q43 selects a diff
//! crate, the canonical hunks come from the ADR 0002 oracle command:
//! `git diff --no-index --histogram` with explicit hunk options and no system
//! or global Git configuration.

use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::Instant;
use tauri::ipc::Response;
use tauri::{AppHandle, State};

mod q42;

/// The hunk options of the oracle command. Q43 pins the final options.
const ORACLE_OPTIONS: [&str; 6] = [
    "--histogram",
    "-U3",
    "--inter-hunk-context=0",
    "--no-color",
    "--no-ext-diff",
    "--no-renames",
];

fn prototype_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn fixtures_dir() -> PathBuf {
    std::env::var_os("DIFFY_S1_FIXTURES")
        .map(PathBuf::from)
        .unwrap_or_else(|| prototype_dir().join("fixtures").join("out"))
}

/// One hunk, as the `@@` header of the oracle output gives it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Hunk {
    pub left_start: u32,
    pub left_count: u32,
    pub right_start: u32,
    pub right_count: u32,
}

/// One change block: consecutive removed and added lines in a hunk.
/// A start is the 1-based number of the first line of the block on that side.
/// When a side has no lines, its start is the number of the next line on that
/// side, so the block comes after line `start - 1`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Block {
    pub left_start: u32,
    pub left_count: u32,
    pub right_start: u32,
    pub right_count: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Parsed {
    pub hunks: Vec<Hunk>,
    pub blocks: Vec<Block>,
    pub added: u32,
    pub removed: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct CanonicalDiff {
    pub source: String,
    pub git_version: String,
    pub command: String,
    pub oracle_ms: f64,
    pub cached: bool,
    #[serde(flatten)]
    pub parsed: Parsed,
}

fn parse_range(text: &str) -> Option<(u32, u32)> {
    match text.split_once(',') {
        Some((start, count)) => Some((start.parse().ok()?, count.parse().ok()?)),
        None => Some((text.parse().ok()?, 1)),
    }
}

fn parse_header(line: &str) -> Option<Hunk> {
    let rest = line.strip_prefix("@@ -")?;
    let (left, rest) = rest.split_once(" +")?;
    let (right, _) = rest.split_once(" @@")?;
    let (left_start, left_count) = parse_range(left)?;
    let (right_start, right_count) = parse_range(right)?;
    Some(Hunk { left_start, left_count, right_start, right_count })
}

/// Parse the unified diff output of the oracle command.
pub fn parse_unified(text: &str) -> Result<Parsed, String> {
    let mut parsed = Parsed::default();
    let mut in_hunk = false;
    let (mut left, mut right) = (0u32, 0u32);
    let mut open: Option<Block> = None;
    let close = |open: &mut Option<Block>, parsed: &mut Parsed| {
        if let Some(block) = open.take() {
            parsed.blocks.push(block);
        }
    };
    for line in text.lines() {
        if line.starts_with("@@ ") {
            close(&mut open, &mut parsed);
            let hunk = parse_header(line).ok_or_else(|| format!("bad hunk header: {line}"))?;
            // A side with count 0 names the line before the hunk.
            left = if hunk.left_count == 0 { hunk.left_start + 1 } else { hunk.left_start };
            right = if hunk.right_count == 0 { hunk.right_start + 1 } else { hunk.right_start };
            parsed.hunks.push(hunk);
            in_hunk = true;
            continue;
        }
        if !in_hunk {
            continue;
        }
        match line.as_bytes().first() {
            Some(b' ') => {
                close(&mut open, &mut parsed);
                left += 1;
                right += 1;
            }
            Some(b'-') => {
                let block = open.get_or_insert(Block { left_start: left, left_count: 0, right_start: right, right_count: 0 });
                block.left_count += 1;
                left += 1;
                parsed.removed += 1;
            }
            Some(b'+') => {
                let block = open.get_or_insert(Block { left_start: left, left_count: 0, right_start: right, right_count: 0 });
                block.right_count += 1;
                right += 1;
                parsed.added += 1;
            }
            Some(b'\\') => {} // "\ No newline at end of file"
            _ => {
                // The header of the next file ends the hunk.
                close(&mut open, &mut parsed);
                in_hunk = false;
            }
        }
    }
    close(&mut open, &mut parsed);
    Ok(parsed)
}

fn oracle_command(left: &Path, right: &Path) -> Command {
    let mut cmd = Command::new("git");
    // Remove each GIT_* variable, then turn off the system and global configuration.
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(key);
        }
    }
    cmd.env("GIT_CONFIG_NOSYSTEM", "1").env("GIT_CONFIG_GLOBAL", "/dev/null").env("LC_ALL", "C");
    // Run outside each repository, so no repository setting changes the hunks.
    cmd.current_dir(std::env::temp_dir());
    cmd.args(["diff", "--no-index"]).args(ORACLE_OPTIONS).arg("--").arg(left).arg(right);
    cmd
}

fn git_version() -> String {
    Command::new("git")
        .arg("--version")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|e| format!("unknown ({e})"))
}

#[derive(Default)]
struct Cache(Mutex<HashMap<String, CanonicalDiff>>);

fn manifest() -> Result<serde_json::Value, String> {
    let path = fixtures_dir().join("manifest.json");
    let text = fs::read_to_string(&path)
        .map_err(|e| format!("cannot read {}: {e}. Run `npm run fixtures` first", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("bad manifest: {e}"))
}

/// Return the file of a fixture side. Only names in the manifest are valid.
fn fixture_file(name: &str, side: &str) -> Result<PathBuf, String> {
    if side != "left" && side != "right" {
        return Err("side must be left or right".into());
    }
    let manifest = manifest()?;
    let entry = manifest["fixtures"]
        .as_array()
        .and_then(|list| list.iter().find(|f| f["name"] == name))
        .ok_or_else(|| format!("unknown fixture {name}"))?;
    let file = entry[side]["file"].as_str().ok_or("bad manifest entry")?;
    Ok(fixtures_dir().join(file))
}

#[tauri::command]
fn list_fixtures() -> Result<serde_json::Value, String> {
    manifest()
}

#[tauri::command]
fn read_side(name: String, side: String) -> Result<Response, String> {
    let path = fixture_file(&name, &side)?;
    let bytes = fs::read(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    Ok(Response::new(bytes))
}

#[tauri::command]
async fn canonical_hunks(name: String, cache: State<'_, Cache>) -> Result<CanonicalDiff, String> {
    if let Some(hit) = cache.0.lock().unwrap().get(&name) {
        return Ok(CanonicalDiff { cached: true, ..hit.clone() });
    }
    let left = fixture_file(&name, "left")?;
    let right = fixture_file(&name, "right")?;
    let mut cmd = oracle_command(&left, &right);
    let command = format!(
        "GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null git diff --no-index {} -- {} {}",
        ORACLE_OPTIONS.join(" "),
        left.file_name().unwrap().to_string_lossy(),
        right.file_name().unwrap().to_string_lossy()
    );
    let started = Instant::now();
    let output = tauri::async_runtime::spawn_blocking(move || cmd.output())
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| format!("cannot run git: {e}"))?;
    let oracle_ms = started.elapsed().as_secs_f64() * 1000.0;
    // Exit 0: no difference. Exit 1: differences. Other: error.
    if !matches!(output.status.code(), Some(0 | 1)) {
        return Err(format!("git diff failed: {}", String::from_utf8_lossy(&output.stderr)));
    }
    let parsed = parse_unified(&String::from_utf8_lossy(&output.stdout))?;
    let result = CanonicalDiff {
        source: "ADR 0002 oracle command (until Q43 selects a crate)".into(),
        git_version: git_version(),
        command,
        oracle_ms,
        cached: false,
        parsed,
    };
    cache.0.lock().unwrap().insert(name, result.clone());
    Ok(result)
}

fn read_trim(path: &str) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

fn cpu_model() -> Option<String> {
    let text = fs::read_to_string("/proc/cpuinfo").ok()?;
    text.lines()
        .find(|l| l.starts_with("model name"))
        .and_then(|l| l.split_once(':'))
        .map(|(_, v)| v.trim().to_string())
}

fn meminfo_kb(key: &str) -> Option<u64> {
    let text = fs::read_to_string("/proc/meminfo").ok()?;
    let line = text.lines().find(|l| l.starts_with(key))?;
    line.split_whitespace().nth(1)?.parse().ok()
}

fn os_name() -> Option<String> {
    let text = fs::read_to_string("/etc/os-release").ok()?;
    text.lines()
        .find_map(|l| l.strip_prefix("PRETTY_NAME="))
        .map(|v| v.trim_matches('"').to_string())
}

fn gpus() -> Vec<serde_json::Value> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir("/sys/class/drm") else { return out };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("card") || name.contains('-') {
            continue;
        }
        let dev = entry.path().join("device");
        let driver = fs::read_link(dev.join("driver"))
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()));
        out.push(serde_json::json!({
            "card": name,
            "vendor": read_trim(&dev.join("vendor").to_string_lossy()),
            "device": read_trim(&dev.join("device").to_string_lossy()),
            "driver": driver,
        }));
    }
    out
}

#[tauri::command]
fn system_info() -> serde_json::Value {
    let env = |k: &str| std::env::var(k).ok();
    serde_json::json!({
        "hardware": {
            "cpu": cpu_model(),
            "logical_cpus": std::thread::available_parallelism().map(|n| n.get()).ok(),
            "mem_total_kb": meminfo_kb("MemTotal:"),
            "gpus": gpus(),
        },
        "os": {
            "name": os_name(),
            "kernel": read_trim("/proc/sys/kernel/osrelease"),
            "container": Path::new("/run/.containerenv").exists(),
        },
        "webview": tauri::webview_version().ok(),
        "session": {
            "XDG_SESSION_TYPE": env("XDG_SESSION_TYPE"),
            "WAYLAND_DISPLAY": env("WAYLAND_DISPLAY"),
            "DISPLAY": env("DISPLAY"),
            "GDK_BACKEND": env("GDK_BACKEND"),
            "WEBKIT_DISABLE_DMABUF_RENDERER": env("WEBKIT_DISABLE_DMABUF_RENDERER"),
            "WEBKIT_DISABLE_COMPOSITING_MODE": env("WEBKIT_DISABLE_COMPOSITING_MODE"),
        },
        "build": {
            "profile": if cfg!(debug_assertions) { "debug" } else { "release" },
            "tauri": tauri::VERSION,
        },
        "oracle_git": git_version(),
    })
}

/// The resident memory of this process and each descendant, for example the
/// WebKit web process.
#[tauri::command]
fn process_memory() -> serde_json::Value {
    let me = std::process::id();
    let mut parent: HashMap<u32, u32> = HashMap::new();
    if let Ok(entries) = fs::read_dir("/proc") {
        for entry in entries.flatten() {
            let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() else { continue };
            let Ok(stat) = fs::read_to_string(entry.path().join("stat")) else { continue };
            // The field after ") " is the state, then the parent PID.
            let Some((_, rest)) = stat.rsplit_once(") ") else { continue };
            if let Some(ppid) = rest.split_whitespace().nth(1).and_then(|p| p.parse().ok()) {
                parent.insert(pid, ppid);
            }
        }
    }
    let is_descendant = |mut pid: u32| {
        while let Some(&p) = parent.get(&pid) {
            if p == me {
                return true;
            }
            if p <= 1 {
                return false;
            }
            pid = p;
        }
        false
    };
    let mut list = Vec::new();
    let mut total = 0u64;
    for &pid in parent.keys() {
        if pid != me && !is_descendant(pid) {
            continue;
        }
        let status = fs::read_to_string(format!("/proc/{pid}/status")).unwrap_or_default();
        let field = |k: &str| status.lines().find_map(|l| l.strip_prefix(k)).map(|v| v.trim().to_string());
        let rss_kb: u64 = field("VmRSS:")
            .and_then(|v| v.split_whitespace().next().and_then(|n| n.parse().ok()))
            .unwrap_or(0);
        total += rss_kb;
        list.push(serde_json::json!({ "pid": pid, "name": field("Name:"), "rss_kb": rss_kb }));
    }
    serde_json::json!({ "total_rss_kb": total, "processes": list })
}

#[tauri::command]
fn save_result(label: String, json: String) -> Result<String, String> {
    if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return Err("bad label".into());
    }
    let dir = prototype_dir().join("bench-results");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{label}.json"));
    fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(path.canonicalize().unwrap_or(path).display().to_string())
}

/// The automatic run, from DIFFY_S1_AUTORUN="A:small,B:1mb" and DIFFY_S1_RUNS.
#[tauri::command]
fn autorun_config() -> Option<serde_json::Value> {
    let spec = std::env::var("DIFFY_S1_AUTORUN").ok()?;
    let runs: u32 = std::env::var("DIFFY_S1_RUNS").ok().and_then(|v| v.parse().ok()).unwrap_or(5);
    let label = std::env::var("DIFFY_S1_LABEL").unwrap_or_else(|_| "autorun".into());
    Some(serde_json::json!({ "spec": spec, "runs": runs, "label": label }))
}

#[tauri::command]
fn quit(app: AppHandle, code: i32) {
    app.exit(code);
}

pub fn run() {
    tauri::Builder::default()
        .manage(Cache::default())
        .invoke_handler(tauri::generate_handler![
            list_fixtures,
            read_side,
            canonical_hunks,
            system_info,
            process_memory,
            save_result,
            autorun_config,
            quit,
            q42::s2_config,
            q42::input,
            q42::ext_clipboard_write,
            q42::ext_clipboard_read,
            q42::focus_main,
            q42::ime_engine,
            q42::set_preedit,
            q42::font_match,
            q42::gpu_report,
            q42::snapshot_check
        ])
        .run(tauri::generate_context!())
        .expect("error while running the prototype");
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIFF: &str = "\
diff --git a/l b/r
--- a/l
+++ b/r
@@ -2,4 +2,5 @@ ctx
 a
-b
-c
+B
 d
+e
+f
 g
@@ -10,2 +11,0 @@
-x
-y
@@ -20,0 +20,2 @@
+p
+q
";

    #[test]
    fn parses_hunks_and_blocks() {
        let p = parse_unified(DIFF).unwrap();
        assert_eq!(p.hunks.len(), 3);
        assert_eq!(p.hunks[1], Hunk { left_start: 10, left_count: 2, right_start: 11, right_count: 0 });
        assert_eq!(
            p.blocks,
            vec![
                Block { left_start: 3, left_count: 2, right_start: 3, right_count: 1 },
                Block { left_start: 6, left_count: 0, right_start: 5, right_count: 2 },
                Block { left_start: 10, left_count: 2, right_start: 12, right_count: 0 },
                Block { left_start: 21, left_count: 0, right_start: 20, right_count: 2 },
            ]
        );
        assert_eq!((p.added, p.removed), (5, 4));
    }

    #[test]
    fn oracle_matches_fixture_stats_when_fixtures_exist() {
        let Ok(small) = fixture_file("small", "left") else { return };
        if !small.exists() {
            return;
        }
        let right = fixture_file("small", "right").unwrap();
        let out = oracle_command(&small, &right).output().unwrap();
        let p = parse_unified(&String::from_utf8_lossy(&out.stdout)).unwrap();
        // git diff --shortstat gives 10 insertions and 8 deletions for "small".
        assert_eq!((p.added, p.removed), (10, 8));
        let left_lines = fs::read_to_string(&small).unwrap().lines().count() as u32;
        let right_lines = fs::read_to_string(&right).unwrap().lines().count() as u32;
        let removed: u32 = p.blocks.iter().map(|b| b.left_count).sum();
        let added: u32 = p.blocks.iter().map(|b| b.right_count).sum();
        assert_eq!(left_lines - removed, right_lines - added);
    }
}
