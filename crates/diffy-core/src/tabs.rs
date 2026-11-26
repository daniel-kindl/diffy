//! The tabs of the one app instance (Q25): one tab for each comparison or
//! review session. Opening a source that already has a tab selects that tab.

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TabKey {
    /// A comparison of two inputs, by their source labels.
    Comparison { left: String, right: String },
    /// A review session, by its repository and baseline.
    Review {
        repository: String,
        baseline: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TabId(pub u64);

#[derive(Debug, Default)]
pub struct Tabs {
    tabs: Vec<(TabId, TabKey)>,
    active: Option<TabId>,
    next: u64,
}

impl Tabs {
    /// Open a tab, or select the one that exists for the same key.
    pub fn open(&mut self, key: TabKey) -> TabId {
        if let Some((id, _)) = self.tabs.iter().find(|(_, k)| *k == key) {
            let id = *id;
            self.active = Some(id);
            return id;
        }
        self.next += 1;
        let id = TabId(self.next);
        self.tabs.push((id, key));
        self.active = Some(id);
        id
    }

    /// Close a tab. The neighbour on the left becomes active, or the first.
    pub fn close(&mut self, id: TabId) -> bool {
        let Some(pos) = self.tabs.iter().position(|(i, _)| *i == id) else {
            return false;
        };
        self.tabs.remove(pos);
        if self.active == Some(id) {
            self.active = self
                .tabs
                .get(pos.saturating_sub(1))
                .or_else(|| self.tabs.first())
                .map(|(i, _)| *i);
        }
        true
    }

    pub fn activate(&mut self, id: TabId) -> bool {
        let known = self.tabs.iter().any(|(i, _)| *i == id);
        if known {
            self.active = Some(id);
        }
        known
    }

    pub fn active(&self) -> Option<TabId> {
        self.active
    }

    pub fn len(&self) -> usize {
        self.tabs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    pub fn keys(&self) -> impl Iterator<Item = (TabId, &TabKey)> {
        self.tabs.iter().map(|(i, k)| (*i, k))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmp(a: &str, b: &str) -> TabKey {
        TabKey::Comparison {
            left: a.into(),
            right: b.into(),
        }
    }

    #[test]
    fn opening_the_same_source_selects_its_tab() {
        let mut t = Tabs::default();
        let a = t.open(cmp("a", "b"));
        let b = t.open(cmp("c", "d"));
        assert_ne!(a, b);
        assert_eq!(t.active(), Some(b));
        assert_eq!(t.open(cmp("a", "b")), a);
        assert_eq!((t.len(), t.active()), (2, Some(a)));
    }

    #[test]
    fn a_review_and_a_comparison_have_their_own_tabs() {
        let mut t = Tabs::default();
        t.open(cmp("a", "b"));
        t.open(TabKey::Review {
            repository: "/r".into(),
            baseline: "HEAD".into(),
        });
        assert_eq!(t.len(), 2);
    }

    #[test]
    fn closing_the_active_tab_selects_its_left_neighbour() {
        let mut t = Tabs::default();
        let a = t.open(cmp("1", "2"));
        let b = t.open(cmp("3", "4"));
        let c = t.open(cmp("5", "6"));
        assert!(t.close(c));
        assert_eq!(t.active(), Some(b));
        assert!(t.close(a));
        assert_eq!(t.active(), Some(b));
        assert!(t.close(b));
        assert_eq!(t.active(), None);
        assert!(t.is_empty());
        assert!(!t.close(b));
    }

    #[test]
    fn activate_ignores_an_unknown_tab() {
        let mut t = Tabs::default();
        let a = t.open(cmp("1", "2"));
        assert!(!t.activate(TabId(99)));
        assert_eq!(t.active(), Some(a));
    }
}
