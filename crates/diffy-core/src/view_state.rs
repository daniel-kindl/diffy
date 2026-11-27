//! The place of the user across a refresh (L-04).
//!
//! A refresh gives a new snapshot. The selected file, the scroll position and
//! the selected finding stay when they still exist, and move to the nearest
//! valid place when they do not. The reviewed status is not part of this state:
//! it is invalidated by a change of the content (`ReviewSession::needs_review`).

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ViewState {
    pub selected_file: Option<String>,
    /// The first visible line, 0-based, in the selected file.
    pub scroll_line: u32,
    pub selected_finding: Option<u64>,
}

/// What the new snapshot has.
pub struct Snapshot<'a> {
    pub files: &'a [String],
    /// The number of lines of the selected file in the new snapshot, if it exists.
    pub selected_file_lines: Option<u32>,
    pub finding_ids: &'a [u64],
}

impl ViewState {
    /// The state after a refresh. Nothing is invented: a file that is gone
    /// clears the selection and the scroll, a shorter file clamps the scroll,
    /// and an unknown finding clears the finding selection.
    pub fn after_refresh(&self, new: &Snapshot<'_>) -> ViewState {
        let file_kept = self
            .selected_file
            .as_ref()
            .is_some_and(|f| new.files.iter().any(|n| n == f));
        let (selected_file, scroll_line) = if file_kept {
            let max = new.selected_file_lines.unwrap_or(0).saturating_sub(1);
            (self.selected_file.clone(), self.scroll_line.min(max))
        } else {
            (None, 0)
        };
        let selected_finding = self
            .selected_finding
            .filter(|id| new.finding_ids.contains(id));
        ViewState {
            selected_file,
            scroll_line,
            selected_finding,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> ViewState {
        ViewState {
            selected_file: Some("a.rs".into()),
            scroll_line: 120,
            selected_finding: Some(7),
        }
    }

    fn files(names: &[&str]) -> Vec<String> {
        names.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn selection_scroll_and_finding_stay_when_they_still_exist() {
        let f = files(&["a.rs", "b.rs"]);
        let new = Snapshot {
            files: &f,
            selected_file_lines: Some(500),
            finding_ids: &[7, 8],
        };
        assert_eq!(state().after_refresh(&new), state());
    }

    #[test]
    fn a_shorter_file_clamps_the_scroll_to_its_last_line() {
        let f = files(&["a.rs"]);
        let new = Snapshot {
            files: &f,
            selected_file_lines: Some(50),
            finding_ids: &[7],
        };
        assert_eq!(state().after_refresh(&new).scroll_line, 49);
        let empty = Snapshot {
            files: &f,
            selected_file_lines: Some(0),
            finding_ids: &[7],
        };
        assert_eq!(state().after_refresh(&empty).scroll_line, 0);
    }

    #[test]
    fn a_removed_file_clears_the_selection_and_the_scroll() {
        let f = files(&["b.rs"]);
        let new = Snapshot {
            files: &f,
            selected_file_lines: None,
            finding_ids: &[7],
        };
        let got = state().after_refresh(&new);
        assert_eq!((got.selected_file, got.scroll_line), (None, 0));
        assert_eq!(
            got.selected_finding,
            Some(7),
            "the finding is kept: findings are not tied to the view"
        );
    }

    #[test]
    fn an_unknown_finding_clears_its_selection() {
        let f = files(&["a.rs"]);
        let new = Snapshot {
            files: &f,
            selected_file_lines: Some(500),
            finding_ids: &[1],
        };
        assert_eq!(state().after_refresh(&new).selected_finding, None);
    }

    #[test]
    fn an_empty_state_stays_empty() {
        let new = Snapshot {
            files: &[],
            selected_file_lines: None,
            finding_ids: &[],
        };
        assert_eq!(
            ViewState::default().after_refresh(&new),
            ViewState::default()
        );
    }
}
