//! Live review state: pause, resume and status (L-05).
//!
//! The view shows one snapshot. While paused it keeps showing that snapshot,
//! even if newer complete results arrive. On resume it shows the newest one.

use crate::publish::Versioned;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LiveStatus {
    /// The view shows the newest result.
    Live,
    /// The user froze the view.
    Paused,
    /// A newer result exists than the one shown, or a refresh is overdue.
    Stale,
    /// The last refresh failed. The view keeps the last complete result.
    Error,
}

pub struct LiveView<T> {
    shown: Option<Versioned<T>>,
    newest: Option<Versioned<T>>,
    paused: bool,
    failed: bool,
    changed: bool,
}

impl<T> Default for LiveView<T> {
    fn default() -> Self {
        Self {
            shown: None,
            newest: None,
            paused: false,
            failed: false,
            changed: false,
        }
    }
}

impl<T> LiveView<T> {
    /// A refresh produced a complete result.
    pub fn receive(&mut self, result: Versioned<T>) {
        self.failed = false;
        self.changed = false;
        if self
            .newest
            .as_ref()
            .is_none_or(|n| n.version < result.version)
        {
            self.newest = Some(result.clone());
        }
        if !self.paused {
            self.shown = self.newest.clone();
        }
    }

    /// A refresh failed. The shown result stays.
    pub fn fail(&mut self) {
        self.failed = true;
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resume and reconcile: show the newest result.
    pub fn resume(&mut self) {
        self.paused = false;
        self.shown = self.newest.clone();
    }

    pub fn shown(&self) -> Option<&Versioned<T>> {
        self.shown.as_ref()
    }

    pub fn status(&self) -> LiveStatus {
        if self.failed {
            return LiveStatus::Error;
        }
        if self.paused {
            return LiveStatus::Paused;
        }
        if self.changed {
            return LiveStatus::Stale;
        }
        match (&self.shown, &self.newest) {
            (Some(s), Some(n)) if s.version == n.version => LiveStatus::Live,
            (None, None) => LiveStatus::Live,
            _ => LiveStatus::Stale,
        }
    }

    /// A change was seen and no result is ready yet: the view is stale.
    pub fn mark_changed(&mut self) {
        self.changed = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn v(version: u64, value: &str) -> Versioned<String> {
        Versioned {
            version,
            value: Arc::new(value.to_owned()),
        }
    }

    #[test]
    fn live_follows_the_newest_result() {
        let mut view = LiveView::default();
        assert_eq!(view.status(), LiveStatus::Live);
        view.receive(v(1, "a"));
        view.receive(v(2, "b"));
        assert_eq!(view.shown().unwrap().version, 2);
        assert_eq!(view.status(), LiveStatus::Live);
    }

    #[test]
    fn pause_freezes_the_snapshot_and_resume_reconciles() {
        let mut view = LiveView::default();
        view.receive(v(1, "a"));
        view.pause();
        view.receive(v(2, "b"));
        assert_eq!(view.status(), LiveStatus::Paused);
        assert_eq!(*view.shown().unwrap().value, "a");
        view.resume();
        assert_eq!(*view.shown().unwrap().value, "b");
        assert_eq!(view.status(), LiveStatus::Live);
    }

    #[test]
    fn a_failure_shows_error_and_keeps_the_last_result() {
        let mut view = LiveView::default();
        view.receive(v(1, "a"));
        view.fail();
        assert_eq!(view.status(), LiveStatus::Error);
        assert_eq!(*view.shown().unwrap().value, "a");
        view.receive(v(2, "b"));
        assert_eq!(view.status(), LiveStatus::Live);
    }

    #[test]
    fn a_change_without_a_result_is_stale() {
        let mut view = LiveView::default();
        view.receive(v(1, "a"));
        view.mark_changed();
        assert_eq!(view.status(), LiveStatus::Stale);
        view.receive(v(2, "b"));
        assert_eq!(view.status(), LiveStatus::Live);
    }
}
