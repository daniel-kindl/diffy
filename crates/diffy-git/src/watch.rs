//! The watcher infrastructure (spec section 20, L-02, Q27).
//!
//! Native notifications feed a bounded queue. The queue coalesces a burst of
//! events on one path, and when it is full it drops the paths and sets an
//! overflow flag, so the consumer rescans the whole repository instead of
//! missing a change. The watcher respects `.gitignore` and always watches
//! the Git index and the refs.

use ignore::gitignore::{Gitignore, GitignoreBuilder};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

#[derive(Default)]
struct State {
    paths: BTreeSet<PathBuf>,
    overflow: bool,
    /// The time of the last push, for the debounce.
    last_push: Option<Instant>,
}

/// What the consumer takes from the queue.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Batch {
    pub paths: Vec<PathBuf>,
    /// The queue was full. Rescan the whole repository.
    pub overflow: bool,
}

/// A bounded queue that coalesces repeated paths.
#[derive(Clone)]
pub struct CoalescingQueue {
    capacity: usize,
    state: Arc<(Mutex<State>, Condvar)>,
}

impl CoalescingQueue {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            state: Arc::default(),
        }
    }

    pub fn push(&self, path: PathBuf) {
        let (lock, cv) = &*self.state;
        let mut s = lock.lock().unwrap();
        s.last_push = Some(Instant::now());
        if s.overflow {
            // Already a full rescan: the path adds nothing.
        } else if s.paths.contains(&path) {
            // Coalesced.
        } else if s.paths.len() >= self.capacity {
            s.paths.clear();
            s.overflow = true;
        } else {
            s.paths.insert(path);
        }
        cv.notify_all();
    }

    /// Take everything, or wait up to `timeout` for the first event.
    pub fn take(&self, timeout: Duration) -> Batch {
        let (lock, cv) = &*self.state;
        let mut s = lock.lock().unwrap();
        if s.paths.is_empty() && !s.overflow {
            s = cv.wait_timeout(s, timeout).unwrap().0;
        }
        let batch = Batch {
            paths: std::mem::take(&mut s.paths).into_iter().collect(),
            overflow: s.overflow,
        };
        s.overflow = false;
        batch
    }

    /// Wait for a burst to settle, then take it. It returns once no event
    /// arrived for `quiet` (200 to 500 ms), or after `max_wait` since the
    /// first event, so a steady stream still refreshes. If nothing happens for
    /// `reconcile_every`, it returns an overflow batch, so the consumer
    /// reconciles with a full rescan. A burst gives one batch.
    pub fn take_settled(
        &self,
        quiet: Duration,
        max_wait: Duration,
        reconcile_every: Duration,
    ) -> Batch {
        let (lock, cv) = &*self.state;
        let mut s = lock.lock().unwrap();
        let idle_end = Instant::now() + reconcile_every;
        while s.paths.is_empty() && !s.overflow {
            let now = Instant::now();
            if now >= idle_end {
                return Batch {
                    paths: Vec::new(),
                    overflow: true,
                };
            }
            s = cv.wait_timeout(s, idle_end - now).unwrap().0;
        }
        let first = Instant::now();
        loop {
            let since_last = s.last_push.map_or(quiet, |t| t.elapsed());
            if since_last >= quiet || first.elapsed() >= max_wait {
                break;
            }
            let wait = (quiet - since_last).min(max_wait.saturating_sub(first.elapsed()));
            s = cv.wait_timeout(s, wait).unwrap().0;
        }
        let batch = Batch {
            paths: std::mem::take(&mut s.paths).into_iter().collect(),
            overflow: s.overflow,
        };
        s.overflow = false;
        batch
    }
}

/// Is this path one that live mode must see?
fn relevant(root: &Path, ignore: &Gitignore, path: &Path) -> bool {
    let Ok(rel) = path.strip_prefix(root) else {
        return false;
    };
    let mut parts = rel.components().map(|c| c.as_os_str().to_string_lossy());
    if parts.next().as_deref() == Some(".git") {
        // The index, HEAD and the refs. Not the object database.
        let rest: Vec<_> = parts.collect();
        return matches!(
            rest.first().map(|s| &**s),
            Some("index" | "HEAD" | "packed-refs" | "refs")
        );
    }
    !ignore
        .matched_path_or_any_parents(rel, path.is_dir())
        .is_ignore()
}

/// Watches one worktree. Dropping it stops the watcher.
pub struct RepoWatcher {
    _watcher: RecommendedWatcher,
    pub queue: CoalescingQueue,
}

impl RepoWatcher {
    pub fn start(root: &Path, capacity: usize) -> notify::Result<Self> {
        let root = root.canonicalize()?;
        let mut builder = GitignoreBuilder::new(&root);
        builder.add(root.join(".gitignore"));
        builder.add(root.join(".git/info/exclude"));
        let ignore = builder
            .build()
            .map_err(|e| notify::Error::generic(&e.to_string()))?;
        let queue = CoalescingQueue::new(capacity);
        let q = queue.clone();
        let r = root.clone();
        let mut watcher =
            notify::recommended_watcher(move |res: notify::Result<Event>| match res {
                Ok(ev) => {
                    for p in ev.paths {
                        if relevant(&r, &ignore, &p) {
                            q.push(p);
                        }
                    }
                }
                // A notification error may hide events: ask for a rescan.
                Err(_) => {
                    let mut s = q.state.0.lock().unwrap();
                    s.paths.clear();
                    s.overflow = true;
                }
            })?;
        watcher.watch(&root, RecursiveMode::Recursive)?;
        Ok(Self {
            _watcher: watcher,
            queue,
        })
    }
}
