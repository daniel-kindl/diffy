# Q84: SQLite migration tool

Throwaway prototype for map question Q84. See `docs/maps/diffy-v0.1.md` on `main`. Never merge this branch.

## What it is

- `rm/`: rusqlite 0.43.0 (bundled SQLite) with rusqlite_migration 2.9.0. `src/main.rs` runs six checks and prints one JSON line for each.
- `refinery/`: rusqlite 0.43.0 with refinery 0.10.0 and its `rusqlite` feature. It has no code. It only checks that Cargo can resolve the dependencies.
- The hand-written option is the Q55 code on the tag `prototype/diffy-v0.1-q55`: `PRAGMA user_version`, a `VACUUM INTO` copy and one transaction for each migration.

## Results

Host: the Fedora 44 container on the Bazzite reference host (`results/host.txt`).

rusqlite_migration (`results/q84-rusqlite-migration.jsonl`), each check passes:

- `validate`: `Migrations::validate` accepts the list.
- `fresh_to_v1`: a new database migrates to user version 1 in WAL mode.
- `failed_m3_rolls_back_m2`: from version 1, migration 2 succeeds and migration 3 fails. `to_latest` runs both in one transaction, so both roll back. The database stays at version 1 with the user row. The `VACUUM INTO` copy from before the run is at version 1 with the user row.
- `retry_to_v3`: the next run without the fault migrates to version 3.
- `older_app_too_far_ahead`: an app with 2 migrations opens the version 3 database. `to_latest` returns `DatabaseTooFarAhead` and does not change the database.
- `same_version_noop`: an app at the same version does nothing.

refinery (`results/q84-refinery-resolve.txt`): Cargo cannot resolve rusqlite 0.43.0 with refinery 0.10.0. refinery-core 0.10.0 accepts rusqlite `>= 0.23, <= 0.40`, and two versions of `libsqlite3-sys` cannot be linked.

## Run

    cd rm && cargo run --release -- <work folder>
    cd refinery && cargo generate-lockfile
