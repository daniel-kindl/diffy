// Q84 prototype: rusqlite_migration 2.9.0 with rusqlite 0.43.0. Throwaway code. Never merge.
use rusqlite::Connection;
use rusqlite_migration::{M, Migrations};
use std::path::Path;

const M1: &str = "CREATE TABLE notes(id INTEGER PRIMARY KEY, body TEXT NOT NULL);";
const M2: &str = "ALTER TABLE notes ADD COLUMN tag TEXT; UPDATE notes SET tag = 'm2';";
const M3: &str = "CREATE TABLE findings(id INTEGER PRIMARY KEY, note INTEGER REFERENCES notes(id));";
const BAD: &str = "CREATE TABLE findings(id INTEGER PRIMARY KEY); SELECT * FROM no_such_table;";

fn migrations(n: usize, bad_last: bool) -> Migrations<'static> {
    let mut v = vec![M::up(M1), M::up(M2), M::up(M3)];
    v.truncate(n);
    if bad_last {
        *v.last_mut().unwrap() = M::up(BAD);
    }
    Migrations::new(v)
}

fn version(c: &Connection) -> i64 {
    c.query_row("PRAGMA user_version", [], |r| r.get(0)).unwrap()
}

fn rows(c: &Connection) -> i64 {
    c.query_row("SELECT count(*) FROM notes", [], |r| r.get(0)).unwrap()
}

// The Q55 recovery copy: VACUUM INTO before the pending migrations.
fn copy(c: &Connection, dest: &Path) -> rusqlite::Result<()> {
    let _ = std::fs::remove_file(dest);
    c.execute("VACUUM INTO ?1", [dest.to_str().unwrap()]).map(|_| ())
}

fn check(name: &str, ok: bool, detail: String) -> bool {
    println!("{{\"test\": \"{name}\", \"pass\": {ok}, \"detail\": {detail:?}}}");
    ok
}

fn main() {
    let dir = std::env::args().nth(1).expect("work folder");
    std::fs::create_dir_all(&dir).unwrap();
    let db = Path::new(&dir).join("data.sqlite");
    let pre = Path::new(&dir).join("data.pre.sqlite");
    let _ = std::fs::remove_file(&db);
    let mut all = true;

    // 1. The migration list is valid.
    let r = migrations(3, false).validate();
    all &= check("validate", r.is_ok(), format!("{r:?}"));

    // 2. A new database migrates to version 1, gets a user row, and WAL works.
    let mut c = Connection::open(&db).unwrap();
    let mode: String = c.query_row("PRAGMA journal_mode=WAL", [], |r| r.get(0)).unwrap();
    migrations(1, false).to_latest(&mut c).unwrap();
    c.execute("INSERT INTO notes(body) VALUES ('user data')", []).unwrap();
    all &= check("fresh_to_v1", version(&c) == 1 && mode == "wal", format!("user_version {} journal {mode}", version(&c)));

    // 3. A failing migration 3 after a good migration 2: one transaction holds
    //    both, so both roll back. The copy keeps version 1.
    copy(&c, &pre).unwrap();
    let r = migrations(3, true).to_latest(&mut c);
    let cols: i64 = c.query_row("SELECT count(*) FROM pragma_table_info('notes') WHERE name = 'tag'", [], |r| r.get(0)).unwrap();
    let pc = Connection::open(&pre).unwrap();
    all &= check("failed_m3_rolls_back_m2", r.is_err() && version(&c) == 1 && cols == 0 && rows(&c) == 1
        && version(&pc) == 1 && rows(&pc) == 1,
        format!("err {:?}; user_version {}; tag column {cols}; rows {}; copy user_version {}", r.err().map(|e| e.to_string()), version(&c), rows(&c), version(&pc)));

    // 4. The next start without the fault migrates to version 3.
    let r = migrations(3, false).to_latest(&mut c);
    all &= check("retry_to_v3", r.is_ok() && version(&c) == 3 && rows(&c) == 1, format!("{r:?} user_version {}", version(&c)));

    // 5. An older app (2 migrations) opens the version 3 database.
    let r = migrations(2, false).to_latest(&mut c);
    let far = matches!(r, Err(rusqlite_migration::Error::MigrationDefinition(
        rusqlite_migration::MigrationDefinitionError::DatabaseTooFarAhead)));
    all &= check("older_app_too_far_ahead", far && version(&c) == 3 && rows(&c) == 1, format!("{r:?} user_version {}", version(&c)));

    // 6. The same version: no work, no error.
    let r = migrations(3, false).to_latest(&mut c);
    all &= check("same_version_noop", r.is_ok() && version(&c) == 3, format!("{r:?}"));

    println!("{{\"all_pass\": {all}}}");
}
