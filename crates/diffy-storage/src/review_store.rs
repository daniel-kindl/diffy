//! Storage of review sessions (R-03 to R-05). A session is saved as a whole in
//! one transaction, so a failed save leaves the old one. Findings keep their
//! original anchor. No credential is stored.

use crate::StorageError;
use diffy_comparison::ContentId;
use diffy_review::{
    Anchor, Baseline, Category, Checkpoint, Finding, FindingId, RepositoryId, ReviewSession,
    SessionParts, Severity, SourceState,
};
use rusqlite::{Connection, params};
use std::collections::BTreeMap;

fn baseline_parts(b: &Baseline) -> (&'static str, &str) {
    match b {
        Baseline::Head => ("head", ""),
        Baseline::Commit(v) => ("commit", v),
        Baseline::Branch(v) => ("branch", v),
        Baseline::PullRequestBase(v) => ("pull_request_base", v),
        Baseline::LastReviewedSnapshot(v) => ("last_reviewed_snapshot", v),
    }
}

fn baseline_from(kind: &str, value: String) -> Option<Baseline> {
    Some(match kind {
        "head" => Baseline::Head,
        "commit" => Baseline::Commit(value),
        "branch" => Baseline::Branch(value),
        "pull_request_base" => Baseline::PullRequestBase(value),
        "last_reviewed_snapshot" => Baseline::LastReviewedSnapshot(value),
        _ => return None,
    })
}

fn severity_name(s: Severity) -> &'static str {
    match s {
        Severity::Blocker => "blocker",
        Severity::Major => "major",
        Severity::Minor => "minor",
        Severity::Nit => "nit",
    }
}

fn severity_from(s: &str) -> Option<Severity> {
    Some(match s {
        "blocker" => Severity::Blocker,
        "major" => Severity::Major,
        "minor" => Severity::Minor,
        "nit" => Severity::Nit,
        _ => return None,
    })
}

const CATEGORIES: [(&str, Category); 7] = [
    ("correctness", Category::Correctness),
    ("security", Category::Security),
    ("performance", Category::Performance),
    ("design", Category::Design),
    ("tests", Category::Tests),
    ("docs", Category::Docs),
    ("style", Category::Style),
];

fn category_name(c: Category) -> &'static str {
    CATEGORIES
        .iter()
        .find(|(_, v)| *v == c)
        .map_or("style", |(n, _)| n)
}

fn category_from(s: &str) -> Option<Category> {
    CATEGORIES.iter().find(|(n, _)| *n == s).map(|(_, v)| *v)
}

fn corrupt(what: &str) -> StorageError {
    StorageError::Sqlite(rusqlite::Error::InvalidParameterName(format!(
        "corrupt {what}"
    )))
}

fn content(hex: &str) -> Result<ContentId, StorageError> {
    ContentId::from_hex(hex).ok_or_else(|| corrupt("content id"))
}

/// Save a session. It replaces the stored session for the same repository and
/// baseline, in one transaction.
pub fn save(conn: &mut Connection, session: &ReviewSession) -> Result<(), StorageError> {
    let p = session.to_parts();
    let (kind, value) = baseline_parts(&p.baseline);
    let tx = conn.transaction()?;
    tx.execute(
        "DELETE FROM review_sessions WHERE repository = ?1 AND baseline_kind = ?2 AND baseline_value = ?3",
        params![p.repository.0, kind, value],
    )?;
    tx.execute(
        "INSERT INTO review_sessions (repository, baseline_kind, baseline_value, head_revision, next_finding_id)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![p.repository.0, kind, value, p.source.head_revision, p.next_finding_id as i64],
    )?;
    let sid = tx.last_insert_rowid();
    for (path, id) in &p.files {
        tx.execute(
            "INSERT INTO review_files VALUES (?1, ?2, ?3)",
            params![sid, path, id.to_hex()],
        )?;
    }
    for f in &p.findings {
        tx.execute(
            "INSERT INTO findings VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                sid,
                f.id.0 as i64,
                f.anchor.path,
                f.anchor.revision,
                f.anchor.lines.start,
                f.anchor.lines.end,
                severity_name(f.severity),
                category_name(f.category),
                f.notes,
                f.resolved,
            ],
        )?;
    }
    for c in &p.checkpoints {
        tx.execute(
            "INSERT INTO checkpoints VALUES (?1, ?2)",
            params![sid, c.number],
        )?;
        for (path, id) in &c.files {
            tx.execute(
                "INSERT INTO checkpoint_files VALUES (?1, ?2, ?3, ?4)",
                params![sid, c.number, path, id.to_hex()],
            )?;
        }
    }
    tx.commit()?;
    Ok(())
}

/// Load the session of a repository and a baseline.
pub fn load(
    conn: &Connection,
    repository: &RepositoryId,
    baseline: &Baseline,
) -> Result<Option<ReviewSession>, StorageError> {
    let (kind, value) = baseline_parts(baseline);
    let row = conn.query_row(
        "SELECT id, head_revision, next_finding_id FROM review_sessions
         WHERE repository = ?1 AND baseline_kind = ?2 AND baseline_value = ?3",
        params![repository.0, kind, value],
        |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
            ))
        },
    );
    let (sid, head_revision, next_id) = match row {
        Ok(v) => v,
        Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    let mut files = BTreeMap::new();
    let mut stmt =
        conn.prepare("SELECT path, content_id FROM review_files WHERE session_id = ?1")?;
    for r in stmt.query_map([sid], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
    })? {
        let (path, hex) = r?;
        files.insert(path, content(&hex)?);
    }
    let mut findings = Vec::new();
    let mut stmt = conn.prepare(
        "SELECT id, path, revision, line_start, line_end, severity, category, notes, resolved
         FROM findings WHERE session_id = ?1 ORDER BY id",
    )?;
    let rows = stmt.query_map([sid], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, u32>(3)?,
            r.get::<_, u32>(4)?,
            r.get::<_, String>(5)?,
            r.get::<_, String>(6)?,
            r.get::<_, String>(7)?,
            r.get::<_, bool>(8)?,
        ))
    })?;
    for r in rows {
        let (id, path, revision, a, b, sev, cat, notes, resolved) = r?;
        findings.push(Finding {
            id: FindingId(id as u64),
            anchor: Anchor {
                path,
                revision,
                lines: a..b,
            },
            severity: severity_from(&sev).ok_or_else(|| corrupt("severity"))?,
            category: category_from(&cat).ok_or_else(|| corrupt("category"))?,
            notes,
            resolved,
        });
    }
    let mut checkpoints = Vec::new();
    let mut stmt =
        conn.prepare("SELECT number FROM checkpoints WHERE session_id = ?1 ORDER BY number")?;
    let numbers: Vec<u32> = stmt
        .query_map([sid], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    for number in numbers {
        let mut files = BTreeMap::new();
        let mut stmt = conn.prepare(
            "SELECT path, content_id FROM checkpoint_files WHERE session_id = ?1 AND number = ?2",
        )?;
        for r in stmt.query_map(params![sid, number], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })? {
            let (path, hex) = r?;
            files.insert(path, content(&hex)?);
        }
        checkpoints.push(Checkpoint { number, files });
    }
    Ok(Some(ReviewSession::from_parts(SessionParts {
        repository: repository.clone(),
        baseline: baseline_from(kind, value.to_owned()).ok_or_else(|| corrupt("baseline"))?,
        source: SourceState { head_revision },
        files,
        findings,
        next_finding_id: next_id as u64,
        checkpoints,
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use diffy_comparison::{Snapshot, Source};

    fn id(s: &str) -> ContentId {
        Snapshot::new(Source::Text("t".into()), s.as_bytes().to_vec()).id()
    }

    fn session() -> ReviewSession {
        let mut s = ReviewSession::new(
            RepositoryId("/repo".into()),
            Baseline::Branch("main".into()),
            SourceState {
                head_revision: "abc".into(),
            },
        );
        s.mark_reviewed("a.rs", id("a1"));
        let f = s.add_finding(
            Anchor {
                path: "a.rs".into(),
                revision: "abc".into(),
                lines: 3..5,
            },
            Severity::Major,
            Category::Security,
            "multi\nline note".into(),
        );
        s.add_finding(
            Anchor {
                path: "b.rs".into(),
                revision: "abc".into(),
                lines: 0..1,
            },
            Severity::Nit,
            Category::Docs,
            "x".into(),
        );
        s.set_resolved(f, true);
        s.create_checkpoint();
        s
    }

    #[test]
    fn a_session_round_trips_with_findings_files_and_checkpoints() {
        let d = tempfile::tempdir().unwrap();
        let mut conn = crate::open(&d.path().join("db")).unwrap();
        let s = session();
        save(&mut conn, &s).unwrap();
        let back = load(&conn, s.repository(), s.baseline()).unwrap().unwrap();
        assert_eq!(back, s);
        // The next ID continues after a restore: IDs stay stable and unique.
        let mut back = back;
        let new = back.add_finding(
            Anchor {
                path: "c".into(),
                revision: "abc".into(),
                lines: 0..1,
            },
            Severity::Minor,
            Category::Tests,
            "n".into(),
        );
        assert_eq!(new, FindingId(2));
    }

    #[test]
    fn saving_again_replaces_the_session_and_another_baseline_is_separate() {
        let d = tempfile::tempdir().unwrap();
        let mut conn = crate::open(&d.path().join("db")).unwrap();
        let mut s = session();
        save(&mut conn, &s).unwrap();
        s.mark_reviewed("z.rs", id("z"));
        save(&mut conn, &s).unwrap();
        let back = load(&conn, s.repository(), s.baseline()).unwrap().unwrap();
        assert_eq!(back, s);
        let n: i64 = conn
            .query_row("SELECT count(*) FROM review_sessions", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 1);
        assert!(
            load(&conn, s.repository(), &Baseline::Head)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn a_failed_save_keeps_the_old_session() {
        let d = tempfile::tempdir().unwrap();
        let mut conn = crate::open(&d.path().join("db")).unwrap();
        let s = session();
        save(&mut conn, &s).unwrap();
        // A second finding with the same ID breaks the primary key.
        let mut bad = s.to_parts();
        bad.findings.push(bad.findings[0].clone());
        assert!(save(&mut conn, &ReviewSession::from_parts(bad)).is_err());
        let back = load(&conn, s.repository(), s.baseline()).unwrap().unwrap();
        assert_eq!(back, s);
    }
}
