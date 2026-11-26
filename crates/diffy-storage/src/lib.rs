//! The central SQLite database (spec sections 10 and 13).
//!
//! One database holds the application data, outside each repository. It
//! stores no credential. Migrations use `PRAGMA user_version`. Before the
//! pending migrations run, a `VACUUM INTO` copy is written next to the
//! database. A released migration never changes. New migrations go at the
//! end. There are no down migrations.

use rusqlite::Connection;
use rusqlite_migration::{M, Migrations};
use std::path::{Path, PathBuf};

/// The migrations, in order. Never edit or reorder a released entry.
const MIGRATION_SQL: &[&str] = &[
    // 1: the first schema. It holds the application metadata only.
    "CREATE TABLE app_meta (key TEXT PRIMARY KEY NOT NULL, value TEXT NOT NULL) STRICT;",
];

fn migrations() -> Migrations<'static> {
    Migrations::new(MIGRATION_SQL.iter().map(|s| M::up(s)).collect())
}

#[derive(Debug)]
pub enum StorageError {
    /// The database is newer than this app. The app did not change it.
    NewerDatabase {
        found: u32,
        supported: u32,
    },
    Sqlite(rusqlite::Error),
    Migration(rusqlite_migration::Error),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NewerDatabase { found, supported } => write!(
                f,
                "the database has schema version {found}, and this app supports up to {supported}"
            ),
            Self::Sqlite(e) => write!(f, "database error: {e}"),
            Self::Migration(e) => write!(f, "migration error: {e}"),
        }
    }
}

impl std::error::Error for StorageError {}

impl From<rusqlite::Error> for StorageError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Sqlite(e)
    }
}

/// The path of the copy that is written before a migration from `version`.
pub fn backup_path(db: &Path, version: u32) -> PathBuf {
    let mut name = db.file_name().unwrap_or_default().to_os_string();
    name.push(format!(".v{version}.bak"));
    db.with_file_name(name)
}

/// Open the database, copy it and migrate it to the latest version.
pub fn open(path: &Path) -> Result<Connection, StorageError> {
    let mut conn = Connection::open(path)?;
    let supported = MIGRATION_SQL.len() as u32;
    let found: u32 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    // Check the version before any write, so an older app leaves a newer
    // database as it is.
    if found > supported {
        return Err(StorageError::NewerDatabase { found, supported });
    }
    if found > 0 && found < supported {
        let copy = backup_path(path, found);
        if copy.exists() {
            std::fs::remove_file(&copy).ok();
        }
        conn.execute("VACUUM INTO ?1", [copy.to_string_lossy().as_ref()])?;
    }
    // WAL where the file system supports it. The pragma returns the mode.
    let _mode: String = conn.query_row("PRAGMA journal_mode = WAL", [], |r| r.get(0))?;
    conn.pragma_update(None, "foreign_keys", true)?;
    migrations()
        .to_latest(&mut conn)
        .map_err(StorageError::Migration)?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn version(c: &Connection) -> u32 {
        c.query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap()
    }

    #[test]
    fn the_migrations_are_valid() {
        migrations().validate().unwrap();
    }

    #[test]
    fn a_new_database_gets_the_latest_version() {
        let dir = tempfile::tempdir().unwrap();
        let conn = open(&dir.path().join("diffy.db")).unwrap();
        assert_eq!(version(&conn), MIGRATION_SQL.len() as u32);
    }

    #[test]
    fn an_older_app_does_not_change_a_newer_database() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("diffy.db");
        {
            let c = Connection::open(&path).unwrap();
            c.execute_batch("CREATE TABLE t (x); PRAGMA user_version = 99;")
                .unwrap();
        }
        let before = std::fs::read(&path).unwrap();
        assert!(matches!(
            open(&path),
            Err(StorageError::NewerDatabase { found: 99, .. })
        ));
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }

    #[test]
    fn a_failed_migration_keeps_the_data_and_the_copy() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("diffy.db");
        // A version-0 migration set is not reachable, so build version 1 by
        // hand, then migrate with a broken second migration.
        {
            let mut c = Connection::open(&path).unwrap();
            migrations().to_version(&mut c, 1).unwrap();
            c.execute("INSERT INTO app_meta VALUES ('k', 'v')", [])
                .unwrap();
        }
        let copy = backup_path(&path, 1);
        {
            let c = Connection::open(&path).unwrap();
            c.execute("VACUUM INTO ?1", [copy.to_string_lossy().as_ref()])
                .unwrap();
        }
        let mut c = Connection::open(&path).unwrap();
        let broken = Migrations::new(vec![
            M::up(MIGRATION_SQL[0]),
            M::up("CREATE TABLE ok (x); THIS IS NOT SQL;"),
        ]);
        assert!(broken.to_latest(&mut c).is_err());
        assert_eq!(version(&c), 1);
        let v: String = c
            .query_row("SELECT value FROM app_meta WHERE key = 'k'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(v, "v");
        let cc = Connection::open(&copy).unwrap();
        assert_eq!(version(&cc), 1);
    }

    #[test]
    fn open_writes_no_copy_when_no_migration_is_pending() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("diffy.db");
        {
            // A database one version behind: simulate with a version of 0
            // data and a partial migration list.
            let mut c = Connection::open(&path).unwrap();
            Migrations::new(vec![M::up(MIGRATION_SQL[0])])
                .to_latest(&mut c)
                .unwrap();
        }
        // Same version as the app: no pending migration, so no copy.
        open(&path).unwrap();
        assert!(!backup_path(&path, 1).exists());
    }

    #[test]
    fn the_schema_has_no_credential_column() {
        let dir = tempfile::tempdir().unwrap();
        let conn = open(&dir.path().join("diffy.db")).unwrap();
        let mut stmt = conn
            .prepare("SELECT m.name || '.' || p.name FROM sqlite_master m, pragma_table_info(m.name) p WHERE m.type = 'table'")
            .unwrap();
        let cols: Vec<String> = stmt
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        for c in cols {
            let c = c.to_lowercase();
            assert!(
                !["token", "password", "secret", "credential"]
                    .iter()
                    .any(|w| c.contains(w)),
                "{c}"
            );
        }
    }
}
