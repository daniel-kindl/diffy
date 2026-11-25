//! Throwaway S8 scale checks for map question Q46. Never merged.
//!
//! The change list of a comparison comes from gix (ADR 0005): the repository opens isolated, the
//! `filter` sections are removed in memory, and a tree to tree diff with rename detection gives
//! the changed files. The line counts of each file come later, from a bounded pool of worker
//! threads that a token can cancel. The open path of one file uses the canonical diff (Q43, Q61):
//! gix-imara-diff 0.3.4 with the Q60 patch, histogram, `postprocess_lines`.

use gix::diff::tree_with_rewrites::Change;
use gix::ObjectId;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, State};

type R<T> = Result<T, String>;

fn err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

/// The Q46 run, from DIFFY_Q46 (the parts) and DIFFY_Q46_LABEL.
#[tauri::command]
pub fn q46_config() -> Option<serde_json::Value> {
    let parts = std::env::var("DIFFY_Q46").ok()?;
    let env = |k: &str, d: &str| std::env::var(k).unwrap_or_else(|_| d.into());
    Some(serde_json::json!({
        "parts": parts,
        "label": env("DIFFY_Q46_LABEL", "q46"),
        "fixtures": env("DIFFY_Q46_FIXTURES", "files-1000"),
        "runs": env("DIFFY_Q46_RUNS", "21").parse::<u32>().unwrap_or(21),
        "threads": threads(),
        "renames": std::env::var("DIFFY_Q46_RENAMES").map_or(true, |v| v != "0"),
        "pack_cache": std::env::var("DIFFY_Q46_PACK_CACHE").ok(),
    }))
}

fn repos_dir() -> R<std::path::PathBuf> {
    std::env::var_os("DIFFY_Q46_REPOS").map(Into::into).ok_or_else(|| "set DIFFY_Q46_REPOS".into())
}

#[tauri::command]
pub fn q46_repos() -> R<serde_json::Value> {
    let path = repos_dir()?.join("repos.json");
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(err)
}

/// The number of worker threads for the line counts: DIFFY_Q46_THREADS, default 4.
fn threads() -> usize {
    std::env::var("DIFFY_Q46_THREADS").ok().and_then(|v| v.parse().ok()).unwrap_or(4)
}

/// Opens a repository as ADR 0005 says: isolated, then with no `filter` section.
fn open(path: &std::path::Path) -> R<gix::Repository> {
    let mut options = gix::open::Options::isolated();
    // DIFFY_Q46_PACK_CACHE sets core.deltaBaseCacheLimit and gitoxide.core.defaultPackCacheMemoryLimit,
    // the delta cache of each thread-local repository. gix caps it at 96 MB when it is unset.
    if let Ok(v) = std::env::var("DIFFY_Q46_PACK_CACHE") {
        options = options.config_overrides([
            format!("core.deltaBaseCacheLimit={v}"),
            format!("gitoxide.core.defaultPackCacheMemoryLimit={v}"),
        ]);
    }
    let mut repo = gix::open_opts(path, options).map_err(err)?;
    let mut config = repo.config_snapshot_mut();
    let names: Vec<Option<gix::bstr::BString>> = config
        .sections_by_name("filter")
        .into_iter()
        .flatten()
        .map(|s| s.header().subsection_name().map(ToOwned::to_owned))
        .collect();
    for name in &names {
        while config.remove_section("filter", name.as_ref().map(AsRef::as_ref)).is_some() {}
    }
    config.commit().map_err(err)?;
    Ok(repo)
}

struct Item {
    status: char,
    path: String,
    orig: Option<String>,
    old: Option<ObjectId>,
    new: Option<ObjectId>,
}

struct Comparison {
    repo: gix::ThreadSafeRepository,
    items: Vec<Item>,
}

#[derive(Default)]
pub struct Q46 {
    comparisons: Mutex<HashMap<u64, Arc<Comparison>>>,
    tokens: Mutex<HashMap<u64, Arc<AtomicBool>>>,
}

impl Q46 {
    fn comparison(&self, id: u64) -> R<Arc<Comparison>> {
        self.comparisons.lock().unwrap().get(&id).cloned().ok_or_else(|| format!("no comparison {id}"))
    }
    fn token(&self, id: u64) -> Arc<AtomicBool> {
        self.tokens.lock().unwrap().entry(id).or_default().clone()
    }
}

/// The change list of `base` to `head` in a fixture repository. Each earlier comparison is
/// dropped, so only the open comparison holds memory.
#[tauri::command]
pub async fn q46_change_list(name: String, id: u64, state: State<'_, Q46>) -> R<tauri::ipc::Response> {
    let started = Instant::now();
    let dir = repos_dir()?.join(format!("{name}.git"));
    let (comparison, open_ms, diff_ms) = tauri::async_runtime::spawn_blocking(move || -> R<_> {
        let t0 = Instant::now();
        let repo = open(&dir)?;
        let open_ms = t0.elapsed().as_secs_f64() * 1000.0;
        let t1 = Instant::now();
        let tree = |rev: &str| -> R<gix::Tree<'_>> {
            let id = repo.rev_parse_single(rev).map_err(err)?;
            id.object().map_err(err)?.peel_to_tree().map_err(err)
        };
        let old = tree("base")?;
        let new = tree("head")?;
        // DIFFY_Q46_RENAMES=0 turns off rename detection, for the memory check.
        let renames = std::env::var("DIFFY_Q46_RENAMES").map_or(true, |v| v != "0");
        let options = gix::diff::Options::default().with_rewrites(renames.then(gix::diff::Rewrites::default));
        let mut items = Vec::new();
        for change in repo.diff_tree_to_tree(&old, &new, options).map_err(err)? {
            if change.entry_mode().is_tree() {
                continue;
            }
            items.push(match change {
                Change::Addition { location, id, .. } => {
                    Item { status: 'A', path: location.to_string(), orig: None, old: None, new: Some(id) }
                }
                Change::Deletion { location, id, .. } => {
                    Item { status: 'D', path: location.to_string(), orig: None, old: Some(id), new: None }
                }
                Change::Modification { location, previous_id, id, .. } => Item {
                    status: 'M',
                    path: location.to_string(),
                    orig: None,
                    old: Some(previous_id),
                    new: Some(id),
                },
                Change::Rewrite { source_location, location, source_id, id, copy, .. } => Item {
                    status: if copy { 'C' } else { 'R' },
                    path: location.to_string(),
                    orig: Some(source_location.to_string()),
                    old: Some(source_id),
                    new: Some(id),
                },
            });
        }
        items.sort_by(|a, b| a.path.cmp(&b.path));
        drop((old, new));
        let diff_ms = t1.elapsed().as_secs_f64() * 1000.0;
        Ok((Comparison { repo: repo.into_sync(), items }, open_ms, diff_ms))
    })
    .await
    .map_err(err)??;
    // A compact list: one line for each file, "<status>\t<path>[\t<orig>]".
    let mut list = String::new();
    for it in &comparison.items {
        list.push(it.status);
        list.push('\t');
        list.push_str(&it.path);
        if let Some(o) = &it.orig {
            list.push('\t');
            list.push_str(o);
        }
        list.push('\n');
    }
    let header = serde_json::json!({
        "files": comparison.items.len(),
        "open_ms": open_ms,
        "diff_ms": diff_ms,
        "rust_ms": started.elapsed().as_secs_f64() * 1000.0,
    });
    {
        let mut map = state.comparisons.lock().unwrap();
        map.clear();
        map.insert(id, Arc::new(comparison));
    }
    Ok(tauri::ipc::Response::new(format!("{header}\n{list}").into_bytes()))
}

fn blob(repo: &gix::Repository, id: Option<ObjectId>) -> R<Vec<u8>> {
    match id {
        None => Ok(Vec::new()),
        Some(id) => Ok(repo.find_object(id).map_err(err)?.detach().data),
    }
}

fn binary(b: &[u8]) -> bool {
    b[..b.len().min(8000)].contains(&0)
}

/// The canonical change groups of two texts, as 0-based (before, after) ranges.
fn groups(a: &[u8], b: &[u8]) -> Vec<gix_imara_diff::Hunk> {
    let input = gix_imara_diff::InternedInput::new(a, b);
    let mut diff = gix_imara_diff::Diff::compute(gix_imara_diff::Algorithm::Histogram, &input);
    diff.postprocess_lines(&input);
    diff.hunks().collect()
}

/// Starts the line counts of comparison `id`. A pool of worker threads counts each file, and a
/// sender thread emits `q46-stats` batches each 50 ms, then one `q46-stats-done`. Cancel token
/// `token` stops the workers before their next file.
#[tauri::command]
pub fn q46_stats_start(app: AppHandle, id: u64, token: u64, state: State<'_, Q46>) -> R<()> {
    let comparison = state.comparison(id)?;
    let cancel = state.token(token);
    let next = Arc::new(AtomicUsize::new(0));
    let batch: Arc<Mutex<Vec<(u32, i64, i64)>>> = Default::default();
    let started = Instant::now();
    let n = threads();
    let workers: Vec<_> = (0..n)
        .map(|_| {
            let (comparison, cancel, next, batch) = (comparison.clone(), cancel.clone(), next.clone(), batch.clone());
            std::thread::spawn(move || -> R<u64> {
                let repo = comparison.repo.to_thread_local();
                let mut done = 0u64;
                loop {
                    if cancel.load(Ordering::Relaxed) {
                        return Ok(done);
                    }
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    let Some(it) = comparison.items.get(i) else { return Ok(done) };
                    let a = blob(&repo, it.old)?;
                    let b = blob(&repo, it.new)?;
                    let (added, removed) = if binary(&a) || binary(&b) {
                        (-1, -1)
                    } else {
                        groups(&a, &b).iter().fold((0i64, 0i64), |(x, y), h| {
                            (x + h.after.len() as i64, y + h.before.len() as i64)
                        })
                    };
                    batch.lock().unwrap().push((i as u32, added, removed));
                    done += 1;
                }
            })
        })
        .collect();
    std::thread::spawn(move || {
        let flush = |final_: bool| {
            let items = std::mem::take(&mut *batch.lock().unwrap());
            if !items.is_empty() || final_ {
                let _ = app.emit("q46-stats", serde_json::json!({ "id": id, "token": token, "items": items }));
            }
        };
        while !workers.iter().all(|w| w.is_finished()) {
            std::thread::sleep(Duration::from_millis(50));
            flush(false);
        }
        let mut counted = 0u64;
        let mut error = None;
        for w in workers {
            match w.join() {
                Ok(Ok(d)) => counted += d,
                Ok(Err(e)) => error = Some(e),
                Err(_) => error = Some("a worker thread failed".into()),
            }
        }
        flush(true);
        let _ = app.emit("q46-stats-done", serde_json::json!({
            "id": id,
            "token": token,
            "cancelled": cancel.load(Ordering::Relaxed),
            "counted": counted,
            "files": comparison.items.len(),
            "threads": n,
            "ms": started.elapsed().as_secs_f64() * 1000.0,
            "error": error,
        }));
    });
    Ok(())
}

/// Sets a cancel token and returns at once. The command is async, so it does not wait for the
/// main thread.
#[tauri::command]
pub async fn q46_cancel(token: u64, state: State<'_, Q46>) -> R<()> {
    state.token(token).store(true, Ordering::Relaxed);
    Ok(())
}

/// One side of one changed file, as bytes. Empty for the missing side of an added or deleted file.
#[tauri::command]
pub async fn q46_blob(id: u64, index: usize, side: String, token: u64, state: State<'_, Q46>) -> R<tauri::ipc::Response> {
    let comparison = state.comparison(id)?;
    let cancel = state.token(token);
    tauri::async_runtime::spawn_blocking(move || {
        let it = comparison.items.get(index).ok_or("bad index")?;
        let repo = comparison.repo.to_thread_local();
        let data = blob(&repo, if side == "left" { it.old } else { it.new })?;
        if cancel.load(Ordering::Relaxed) {
            return Err("cancelled".to_string());
        }
        Ok(tauri::ipc::Response::new(data))
    })
    .await
    .map_err(err)?
}

/// The canonical blocks of one changed file, in the format of `crate::Block`. The token is
/// checked before and after the diff: the diff itself cannot stop, but its result is dropped.
#[tauri::command]
pub async fn q46_blocks(id: u64, index: usize, token: u64, state: State<'_, Q46>) -> R<serde_json::Value> {
    let comparison = state.comparison(id)?;
    let cancel = state.token(token);
    tauri::async_runtime::spawn_blocking(move || {
        let started = Instant::now();
        let it = comparison.items.get(index).ok_or("bad index")?;
        let repo = comparison.repo.to_thread_local();
        let a = blob(&repo, it.old)?;
        let b = blob(&repo, it.new)?;
        if cancel.load(Ordering::Relaxed) {
            return Err("cancelled".to_string());
        }
        let blocks: Vec<crate::Block> = groups(&a, &b)
            .into_iter()
            .map(|h| crate::Block {
                left_start: h.before.start + 1,
                left_count: h.before.len() as u32,
                right_start: h.after.start + 1,
                right_count: h.after.len() as u32,
            })
            .collect();
        if cancel.load(Ordering::Relaxed) {
            return Err("cancelled".to_string());
        }
        Ok(serde_json::json!({ "blocks": blocks, "rust_ms": started.elapsed().as_secs_f64() * 1000.0 }))
    })
    .await
    .map_err(err)?
}
