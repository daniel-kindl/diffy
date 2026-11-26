//! Versioned, complete results (L-03).
//!
//! A refresh runs as a job with a version number. A newer job cancels the
//! older one through its [`CancelToken`]. Only a job that finishes, and that
//! is still the newest, publishes. A reader sees the last complete result or
//! nothing, never a partial one.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// A cooperative cancel flag. A job checks it between small steps, so a
/// cancel is acknowledged quickly.
#[derive(Clone, Debug, Default)]
pub struct CancelToken(Arc<AtomicBool>);

impl CancelToken {
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

/// A complete result and the version of the job that made it.
#[derive(Debug, PartialEq, Eq)]
pub struct Versioned<T> {
    pub version: u64,
    pub value: Arc<T>,
}

impl<T> Clone for Versioned<T> {
    fn clone(&self) -> Self {
        Self {
            version: self.version,
            value: Arc::clone(&self.value),
        }
    }
}

pub struct Publisher<T> {
    next: AtomicU64,
    current: Mutex<Option<CancelToken>>,
    latest: Mutex<Option<Versioned<T>>>,
}

impl<T> Default for Publisher<T> {
    fn default() -> Self {
        Self {
            next: AtomicU64::new(1),
            current: Mutex::new(None),
            latest: Mutex::new(None),
        }
    }
}

/// A started job. Finish it with [`Publisher::finish`].
pub struct Job {
    pub version: u64,
    pub cancel: CancelToken,
}

impl<T> Publisher<T> {
    /// Start a job. It cancels the job that is still running.
    pub fn start(&self) -> Job {
        let cancel = CancelToken::default();
        let mut cur = self.current.lock().unwrap();
        if let Some(old) = cur.replace(cancel.clone()) {
            old.0.store(true, Ordering::Release);
        }
        Job {
            version: self.next.fetch_add(1, Ordering::AcqRel),
            cancel,
        }
    }

    /// Publish the result of a finished job. A cancelled or superseded job
    /// publishes nothing. Returns whether it published.
    pub fn finish(&self, job: Job, value: T) -> bool {
        let mut latest = self.latest.lock().unwrap();
        if job.cancel.is_cancelled() {
            return false;
        }
        if latest.as_ref().is_some_and(|l| l.version > job.version) {
            return false;
        }
        *latest = Some(Versioned {
            version: job.version,
            value: Arc::new(value),
        });
        true
    }

    /// The last complete result.
    pub fn latest(&self) -> Option<Versioned<T>> {
        self.latest.lock().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn a_new_job_cancels_the_old_one_and_only_the_new_one_publishes() {
        let p = Publisher::<&str>::default();
        let a = p.start();
        let b = p.start();
        assert!(a.cancel.is_cancelled());
        assert!(!b.cancel.is_cancelled());
        assert!(!p.finish(a, "old"));
        assert!(p.latest().is_none(), "no partial or stale result");
        assert!(p.finish(b, "new"));
        assert_eq!(*p.latest().unwrap().value, "new");
    }

    #[test]
    fn a_late_older_result_never_replaces_a_newer_one() {
        let p = Publisher::<u32>::default();
        let a = p.start();
        // Simulate: job `a` finished before `b` started.
        assert!(p.finish(a, 1));
        let b = p.start();
        assert!(p.finish(b, 2));
        assert_eq!(p.latest().unwrap().version, 2);
    }

    #[test]
    fn a_job_that_checks_between_small_steps_acknowledges_a_cancel_within_250_ms() {
        let p = Arc::new(Publisher::<u32>::default());
        let job = p.start();
        let cancel = job.cancel.clone();
        let worker = std::thread::spawn(move || {
            // Work in 5 ms steps and check the token after each one.
            for _ in 0..10_000 {
                if cancel.is_cancelled() {
                    return Some(Instant::now());
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            None
        });
        std::thread::sleep(Duration::from_millis(50));
        let sent = Instant::now();
        let _newer = p.start();
        let acked = worker.join().unwrap().expect("the job saw the cancel");
        assert!(acked.duration_since(sent) < Duration::from_millis(250));
    }
}
