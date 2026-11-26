//! Saved comparison sessions (C-08).
//!
//! A session keeps the two input sources with their content identity, the
//! profile, the baseline and the navigation state. On restore, each input is
//! checked against the saved identity: a missing or changed file is shown
//! explicitly, never silently replaced.

use crate::StorageError;
use diffy_comparison::profile::Profile;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// One input of a saved session: a file path and the identity it had.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedInput {
    pub source: String,
    pub content_id: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Navigation {
    pub selected_file: Option<String>,
    pub scroll_line: u32,
    pub view: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedSession {
    pub name: String,
    pub left: SavedInput,
    pub right: SavedInput,
    pub profile: Profile,
    pub baseline: String,
    pub navigation: Navigation,
}

/// What a restore finds for one input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputStatus {
    Unchanged,
    /// The file differs from the saved content.
    Changed {
        now: String,
    },
    Missing,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Restored {
    pub session: SavedSession,
    pub left: InputStatus,
    pub right: InputStatus,
}

fn json_err(e: serde_json::Error) -> StorageError {
    StorageError::Sqlite(rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
}

pub fn save(conn: &Connection, s: &SavedSession) -> Result<i64, StorageError> {
    conn.execute(
        "INSERT INTO comparison_sessions
         (name, left_source, left_content_id, right_source, right_content_id,
          profile_json, baseline, navigation_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            s.name,
            s.left.source,
            s.left.content_id,
            s.right.source,
            s.right.content_id,
            serde_json::to_string(&s.profile).map_err(json_err)?,
            s.baseline,
            serde_json::to_string(&s.navigation).map_err(json_err)?,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

fn check(input: &SavedInput) -> InputStatus {
    match std::fs::read(Path::new(&input.source)) {
        Err(_) => InputStatus::Missing,
        Ok(bytes) => {
            let now = diffy_comparison::Snapshot::new(
                diffy_comparison::Source::File(input.source.clone()),
                bytes,
            )
            .id()
            .to_hex();
            if now == input.content_id {
                InputStatus::Unchanged
            } else {
                InputStatus::Changed { now }
            }
        }
    }
}

/// Load a session and check its inputs.
pub fn restore(conn: &Connection, id: i64) -> Result<Option<Restored>, StorageError> {
    let row = conn.query_row(
        "SELECT name, left_source, left_content_id, right_source, right_content_id,
                profile_json, baseline, navigation_json
         FROM comparison_sessions WHERE id = ?1",
        [id],
        |r| {
            Ok((
                r.get::<_, String>(0)?,
                SavedInput {
                    source: r.get(1)?,
                    content_id: r.get(2)?,
                },
                SavedInput {
                    source: r.get(3)?,
                    content_id: r.get(4)?,
                },
                r.get::<_, String>(5)?,
                r.get::<_, String>(6)?,
                r.get::<_, String>(7)?,
            ))
        },
    );
    let (name, left, right, profile, baseline, nav) = match row {
        Ok(v) => v,
        Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    let session = SavedSession {
        name,
        profile: serde_json::from_str(&profile).map_err(json_err)?,
        navigation: serde_json::from_str(&nav).map_err(json_err)?,
        baseline,
        left: left.clone(),
        right: right.clone(),
    };
    Ok(Some(Restored {
        left: check(&left),
        right: check(&right),
        session,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use diffy_comparison::{Snapshot, Source};
    use std::fs;

    fn input(path: &Path, content: &[u8]) -> SavedInput {
        fs::write(path, content).unwrap();
        SavedInput {
            source: path.display().to_string(),
            content_id: Snapshot::new(Source::File("x".into()), content.to_vec())
                .id()
                .to_hex(),
        }
    }

    fn session(dir: &Path) -> SavedSession {
        SavedSession {
            name: "s".into(),
            left: input(&dir.join("l.txt"), b"left\n"),
            right: input(&dir.join("r.txt"), b"right\n"),
            profile: Profile {
                context_lines: 7,
                ..Profile::default()
            },
            baseline: "HEAD".into(),
            navigation: Navigation {
                selected_file: Some("a.rs".into()),
                scroll_line: 42,
                view: "unified".into(),
            },
        }
    }

    #[test]
    fn a_session_round_trips_with_its_profile_baseline_and_navigation() {
        let d = tempfile::tempdir().unwrap();
        let conn = crate::open(&d.path().join("db")).unwrap();
        let s = session(d.path());
        let id = save(&conn, &s).unwrap();
        let r = restore(&conn, id).unwrap().unwrap();
        assert_eq!(r.session, s);
        assert_eq!(
            (r.left, r.right),
            (InputStatus::Unchanged, InputStatus::Unchanged)
        );
        assert!(restore(&conn, 999).unwrap().is_none());
    }

    #[test]
    fn a_changed_or_missing_file_is_shown_explicitly() {
        let d = tempfile::tempdir().unwrap();
        let conn = crate::open(&d.path().join("db")).unwrap();
        let id = save(&conn, &session(d.path())).unwrap();
        fs::write(d.path().join("l.txt"), b"edited\n").unwrap();
        fs::remove_file(d.path().join("r.txt")).unwrap();
        let r = restore(&conn, id).unwrap().unwrap();
        assert!(matches!(r.left, InputStatus::Changed { .. }));
        assert_eq!(r.right, InputStatus::Missing);
        // Restoring never rewrites a file.
        assert_eq!(fs::read(d.path().join("l.txt")).unwrap(), b"edited\n");
    }

    #[test]
    fn migration_two_keeps_the_first_migration_data() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("db");
        {
            let mut c = Connection::open(&p).unwrap();
            crate::migrations().to_version(&mut c, 1).unwrap();
            c.execute("INSERT INTO app_meta VALUES ('k','v')", [])
                .unwrap();
        }
        let c = crate::open(&p).unwrap();
        let v: String = c
            .query_row("SELECT value FROM app_meta", [], |r| r.get(0))
            .unwrap();
        assert_eq!(v, "v");
        assert!(
            crate::backup_path(&p, 1).exists(),
            "a copy is written before the migration"
        );
    }
}
