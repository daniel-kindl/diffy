//! The changed-file explorer model (R-02, Q29, Q81).
//!
//! It lists the changed files with a status, a generated flag and a reviewed
//! flag. Search and filters hide files from the list. A hidden file stays
//! counted and discoverable: the counts include it and `hidden` lists it.
//! No file content is read to decide that a file is generated.

use globset::{Glob, GlobSet, GlobSetBuilder};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Added,
    Deleted,
    Modified,
    Renamed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChangedFile {
    pub path: String,
    pub status: Status,
    /// The value of the `linguist-generated` attribute, if the file has one.
    pub generated_attribute: Option<bool>,
    pub reviewed: bool,
}

/// A short built-in list of generated paths (Q81).
const BUILTIN_GENERATED: &[&str] = &[
    "**/package-lock.json",
    "**/yarn.lock",
    "**/pnpm-lock.yaml",
    "**/Cargo.lock",
    "**/*.min.js",
    "**/*.min.css",
    "**/*.map",
    "**/*.pb.go",
];

pub struct GeneratedRules {
    builtin: GlobSet,
    user: GlobSet,
}

impl GeneratedRules {
    pub fn new(user_globs: &[String]) -> Result<Self, globset::Error> {
        let build = |pats: &mut dyn Iterator<Item = &str>| -> Result<GlobSet, globset::Error> {
            let mut b = GlobSetBuilder::new();
            for p in pats {
                b.add(Glob::new(p)?);
            }
            b.build()
        };
        Ok(Self {
            builtin: build(&mut BUILTIN_GENERATED.iter().copied())?,
            user: build(&mut user_globs.iter().map(String::as_str))?,
        })
    }

    /// The attribute wins over the built-in list and the user's globs.
    pub fn is_generated(&self, path: &str, attribute: Option<bool>) -> bool {
        match attribute {
            Some(v) => v,
            None => self.builtin.is_match(path) || self.user.is_match(path),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Filter {
    /// A case-insensitive substring of the path.
    pub search: String,
    pub status: Option<Status>,
    pub reviewed: Option<bool>,
    pub show_generated: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Counts {
    pub total: usize,
    pub visible: usize,
    pub hidden: usize,
    pub generated: usize,
    pub reviewed: usize,
}

#[derive(Debug)]
pub struct Listing<'a> {
    pub visible: Vec<&'a ChangedFile>,
    /// The files that a filter hides. They stay discoverable.
    pub hidden: Vec<&'a ChangedFile>,
    pub counts: Counts,
}

pub fn list<'a>(files: &'a [ChangedFile], rules: &GeneratedRules, filter: &Filter) -> Listing<'a> {
    let needle = filter.search.to_lowercase();
    let (mut visible, mut hidden, mut generated, mut reviewed) = (Vec::new(), Vec::new(), 0, 0);
    for f in files {
        let is_gen = rules.is_generated(&f.path, f.generated_attribute);
        generated += usize::from(is_gen);
        reviewed += usize::from(f.reviewed);
        let shown = (filter.show_generated || !is_gen)
            && (needle.is_empty() || f.path.to_lowercase().contains(&needle))
            && filter.status.is_none_or(|s| s == f.status)
            && filter.reviewed.is_none_or(|r| r == f.reviewed);
        if shown {
            visible.push(f)
        } else {
            hidden.push(f)
        }
    }
    visible.sort_by(|a, b| a.path.cmp(&b.path));
    hidden.sort_by(|a, b| a.path.cmp(&b.path));
    let counts = Counts {
        total: files.len(),
        visible: visible.len(),
        hidden: hidden.len(),
        generated,
        reviewed,
    };
    Listing {
        visible,
        hidden,
        counts,
    }
}

/// A path hierarchy node.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Node {
    pub dirs: BTreeMap<String, Node>,
    pub files: Vec<String>,
}

/// Build the folder tree of the visible files.
pub fn tree(listing: &Listing<'_>) -> Node {
    let mut root = Node::default();
    for f in &listing.visible {
        let mut parts: Vec<&str> = f.path.split('/').collect();
        let name = parts.pop().unwrap_or_default().to_owned();
        let mut node = &mut root;
        for d in parts {
            node = node.dirs.entry(d.to_owned()).or_default();
        }
        node.files.push(name);
    }
    root
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(path: &str, status: Status, reviewed: bool) -> ChangedFile {
        ChangedFile {
            path: path.into(),
            status,
            generated_attribute: None,
            reviewed,
        }
    }

    fn files() -> Vec<ChangedFile> {
        vec![
            f("src/main.rs", Status::Modified, true),
            f("src/lib.rs", Status::Added, false),
            f("Cargo.lock", Status::Modified, false),
            f("docs/a.md", Status::Deleted, false),
        ]
    }

    #[test]
    fn generated_files_are_hidden_but_counted_and_listed() {
        let rules = GeneratedRules::new(&[]).unwrap();
        let fs = files();
        let l = list(&fs, &rules, &Filter::default());
        assert_eq!(
            l.counts,
            Counts {
                total: 4,
                visible: 3,
                hidden: 1,
                generated: 1,
                reviewed: 1
            }
        );
        assert_eq!(l.hidden[0].path, "Cargo.lock");
        let all = list(
            &fs,
            &rules,
            &Filter {
                show_generated: true,
                ..Filter::default()
            },
        );
        assert_eq!(all.counts.visible, 4);
    }

    #[test]
    fn the_attribute_wins_over_the_list_and_the_globs() {
        let rules = GeneratedRules::new(&["docs/**".into()]).unwrap();
        assert!(rules.is_generated("Cargo.lock", None));
        assert!(!rules.is_generated("Cargo.lock", Some(false)));
        assert!(rules.is_generated("docs/a.md", None));
        assert!(rules.is_generated("src/x.rs", Some(true)));
        assert!(!rules.is_generated("src/x.rs", None));
    }

    #[test]
    fn search_status_and_reviewed_filters_hide_without_losing() {
        let rules = GeneratedRules::new(&[]).unwrap();
        let fs = files();
        let by = |filter: Filter| list(&fs, &rules, &filter).counts;
        assert_eq!(
            by(Filter {
                search: "SRC/".into(),
                ..Filter::default()
            })
            .visible,
            2
        );
        assert_eq!(
            by(Filter {
                status: Some(Status::Added),
                ..Filter::default()
            })
            .visible,
            1
        );
        assert_eq!(
            by(Filter {
                reviewed: Some(false),
                ..Filter::default()
            })
            .visible,
            2
        );
        let c = by(Filter {
            search: "zzz".into(),
            ..Filter::default()
        });
        assert_eq!((c.total, c.visible, c.hidden), (4, 0, 4));
    }

    #[test]
    fn the_tree_follows_the_path_hierarchy() {
        let rules = GeneratedRules::new(&[]).unwrap();
        let fs = files();
        let l = list(&fs, &rules, &Filter::default());
        let t = tree(&l);
        assert_eq!(t.dirs["src"].files, vec!["lib.rs", "main.rs"]);
        assert_eq!(t.dirs["docs"].files, vec!["a.md"]);
        assert!(t.files.is_empty(), "Cargo.lock is hidden as generated");
    }

    #[test]
    fn a_bad_user_glob_is_an_error() {
        assert!(GeneratedRules::new(&["[".into()]).is_err());
    }
}
