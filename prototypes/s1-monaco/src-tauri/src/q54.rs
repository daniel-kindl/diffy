//! Throwaway S1 verdict checks for map question Q54. Never merged.
//!
//! The frontend runs the checks. This module gives it the parts that the page
//! cannot do: the memory and CPU time of the process tree, a memory sampler,
//! and a live file that changes on disk while the view is open.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// The Q54 run, from DIFFY_S1_Q54="A" or "B" and DIFFY_S1_Q54_LABEL.
#[tauri::command]
pub fn q54_config() -> Option<serde_json::Value> {
    let option = std::env::var("DIFFY_S1_Q54").ok()?;
    let label = std::env::var("DIFFY_S1_Q54_LABEL").unwrap_or_else(|_| format!("q54-{option}"));
    let parts = std::env::var("DIFFY_S1_Q54_PARTS").unwrap_or_else(|_| "all".into());
    let fixtures = std::env::var("DIFFY_S1_Q54_FIXTURES").unwrap_or_else(|_| "small,1mb,10mb,10k-lines,50mb-stress".into());
    Some(serde_json::json!({ "option": option, "label": label, "parts": parts, "fixtures": fixtures }))
}

/// One process of the tree: this process and each descendant.
struct Proc {
    pid: u32,
    name: String,
    rss_kb: u64,
    /// Q46: the anonymous and the file-backed part of the RSS.
    anon_kb: u64,
    file_kb: u64,
    cpu_ticks: u64,
}

#[cfg(target_os = "linux")]
fn tree() -> Vec<Proc> {
    let me = std::process::id();
    let mut parent: HashMap<u32, u32> = HashMap::new();
    let mut ticks: HashMap<u32, u64> = HashMap::new();
    if let Ok(entries) = fs::read_dir("/proc") {
        for entry in entries.flatten() {
            let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() else { continue };
            let Ok(stat) = fs::read_to_string(entry.path().join("stat")) else { continue };
            let Some((_, rest)) = stat.rsplit_once(") ") else { continue };
            let f: Vec<&str> = rest.split_whitespace().collect();
            // After ") ": state, ppid, ..., utime is field 14 and stime is field 15 of stat (index 11 and 12 here).
            if let (Some(ppid), Some(ut), Some(st)) = (f.get(1), f.get(11), f.get(12)) {
                if let Ok(ppid) = ppid.parse() {
                    parent.insert(pid, ppid);
                }
                ticks.insert(pid, ut.parse::<u64>().unwrap_or(0) + st.parse::<u64>().unwrap_or(0));
            }
        }
    }
    let is_member = |mut pid: u32| {
        if pid == me {
            return true;
        }
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
    let mut out = Vec::new();
    for &pid in parent.keys() {
        if !is_member(pid) {
            continue;
        }
        let status = fs::read_to_string(format!("/proc/{pid}/status")).unwrap_or_default();
        let field = |k: &str| status.lines().find_map(|l| l.strip_prefix(k)).map(|v| v.trim().to_string());
        let kb = |k: &str| -> u64 {
            field(k).and_then(|v| v.split_whitespace().next().and_then(|n| n.parse().ok())).unwrap_or(0)
        };
        let (rss_kb, anon_kb, file_kb) = (kb("VmRSS:"), kb("RssAnon:"), kb("RssFile:"));
        out.push(Proc { pid, name: field("Name:").unwrap_or_default(), rss_kb, anon_kb, file_kb, cpu_ticks: ticks[&pid] });
    }
    out
}

#[cfg(not(target_os = "linux"))]
fn tree() -> Vec<Proc> {
    Vec::new()
}

fn clock_ticks() -> u64 {
    std::process::Command::new("getconf")
        .arg("CLK_TCK")
        .output()
        .ok()
        .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse().ok())
        .unwrap_or(100)
}

/// The resident memory and the CPU time of the process tree, now.
#[tauri::command]
pub fn tree_stats() -> serde_json::Value {
    let t = tree();
    let hz = clock_ticks() as f64;
    serde_json::json!({
        "supported": cfg!(target_os = "linux"),
        "total_rss_kb": t.iter().map(|p| p.rss_kb).sum::<u64>(),
        "total_cpu_ms": t.iter().map(|p| p.cpu_ticks).sum::<u64>() as f64 * 1000.0 / hz,
        "processes": t.iter().map(|p| serde_json::json!({
            "pid": p.pid, "name": p.name, "rss_kb": p.rss_kb, "cpu_ms": p.cpu_ticks as f64 * 1000.0 / hz,
        })).collect::<Vec<_>>(),
    })
}

#[derive(Default)]
pub struct Sampler(Mutex<Option<(Arc<AtomicBool>, std::thread::JoinHandle<serde_json::Value>)>>);

/// Start a thread that samples the tree memory each `interval_ms`.
#[tauri::command]
pub fn sampler_start(interval_ms: u64, state: tauri::State<'_, Sampler>) -> Result<(), String> {
    let mut slot = state.0.lock().unwrap();
    if slot.is_some() {
        return Err("the sampler runs already".into());
    }
    let stop = Arc::new(AtomicBool::new(false));
    let flag = stop.clone();
    let handle = std::thread::spawn(move || {
        let started = Instant::now();
        let (mut peak, mut samples) = (0u64, 0u64);
        let mut peak_by_name: HashMap<String, u64> = HashMap::new();
        let mut anon_by_name: HashMap<String, u64> = HashMap::new();
        let mut file_by_name: HashMap<String, u64> = HashMap::new();
        while !flag.load(Ordering::Relaxed) {
            let t = tree();
            peak = peak.max(t.iter().map(|p| p.rss_kb).sum());
            for p in &t {
                let e = peak_by_name.entry(p.name.clone()).or_default();
                *e = (*e).max(p.rss_kb);
                let e = anon_by_name.entry(p.name.clone()).or_default();
                *e = (*e).max(p.anon_kb);
                let e = file_by_name.entry(p.name.clone()).or_default();
                *e = (*e).max(p.file_kb);
            }
            samples += 1;
            std::thread::sleep(Duration::from_millis(interval_ms));
        }
        serde_json::json!({
            "interval_ms": interval_ms,
            "samples": samples,
            "duration_ms": started.elapsed().as_secs_f64() * 1000.0,
            "peak_total_rss_kb": peak,
            "peak_rss_kb_by_process_name": peak_by_name,
            "peak_rss_anon_kb_by_process_name": anon_by_name,
            "peak_rss_file_kb_by_process_name": file_by_name,
        })
    });
    *slot = Some((stop, handle));
    Ok(())
}

#[tauri::command]
pub fn sampler_stop(state: tauri::State<'_, Sampler>) -> Result<serde_json::Value, String> {
    let (stop, handle) = state.0.lock().unwrap().take().ok_or("the sampler does not run")?;
    stop.store(true, Ordering::Relaxed);
    handle.join().map_err(|_| "the sampler thread failed".to_string())
}

fn live_dir() -> PathBuf {
    std::env::temp_dir().join(format!("diffy-s1-q54-{}", std::process::id()))
}

pub fn live_file(name: &str) -> PathBuf {
    live_dir().join(format!("{name}.right.txt"))
}

#[derive(serde::Deserialize)]
pub struct LineEdit {
    /// The 1-based line to replace, or to insert before when `insert` is true.
    line: usize,
    text: String,
    #[serde(default)]
    insert: bool,
}

/// Change the live copy of the right side of a fixture on disk, as an editor
/// does: write a temporary file, then rename it over the live file. The first
/// call copies the fixture.
#[tauri::command]
pub fn live_write(name: String, edits: Vec<LineEdit>) -> Result<serde_json::Value, String> {
    let started = Instant::now();
    let path = live_file(&name);
    fs::create_dir_all(live_dir()).map_err(|e| e.to_string())?;
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => fs::read_to_string(crate::fixture_file(&name, "right")?).map_err(|e| e.to_string())?,
    };
    let mut lines: Vec<String> = text.split('\n').map(str::to_string).collect();
    // Apply from the bottom up, so that each line number names the line before the edits.
    let mut sorted: Vec<&LineEdit> = edits.iter().collect();
    sorted.sort_by(|a, b| b.line.cmp(&a.line));
    for e in sorted {
        let i = e.line.checked_sub(1).ok_or("line numbers start at 1")?;
        if i >= lines.len() {
            return Err(format!("line {} is after the end", e.line));
        }
        if e.insert {
            lines.insert(i, e.text.clone());
        } else {
            lines[i] = e.text.clone();
        }
    }
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, lines.join("\n")).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
    Ok(serde_json::json!({ "path": path.display().to_string(), "write_ms": started.elapsed().as_secs_f64() * 1000.0 }))
}

#[tauri::command]
pub fn live_read(name: String) -> Result<tauri::ipc::Response, String> {
    fs::read(live_file(&name)).map(tauri::ipc::Response::new).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn live_reset(name: String) -> Result<(), String> {
    match fs::remove_file(live_file(&name)) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

/// The canonical hunks of the left fixture side against the live right side.
/// Never cached.
#[tauri::command]
pub async fn live_canonical(name: String) -> Result<crate::CanonicalDiff, String> {
    let left = crate::fixture_file(&name, "left")?;
    crate::run_oracle(left, live_file(&name)).await
}
