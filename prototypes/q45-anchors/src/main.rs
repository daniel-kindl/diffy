//! Q45 throwaway harness: conservative anchor mapping across revisions.
//!
//! Usage:
//!   q45-anchors fixtures <out.json>
//!   q45-anchors property <small|medium> <histories> <out.json>
//!   q45-anchors real <pairs.tsv> <out.json>
//!   q45-anchors timing <pairs.tsv> <runs> <out.json>   (Q65)

mod core;
mod fixtures;
mod history;
mod timing;

use core::{K, METHODS, Pair, canonical_map, lcs_rule, map_all, split};
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn line(p: Option<usize>) -> Value {
    p.map_or(Value::from("outdated"), |n| Value::from(n + 1))
}

#[derive(Default, serde::Serialize)]
struct Score {
    anchors: u64,
    /// The method gives the expected line, or outdated when outdated is expected.
    correct: u64,
    /// The method gives a line, and it is not the expected line (or outdated is expected).
    wrong: u64,
    /// The method gives outdated where a line is expected.
    outdated_where_line_expected: u64,
}

impl Score {
    fn add(&mut self, expected: Option<usize>, got: Option<usize>) -> bool {
        self.anchors += 1;
        match (expected, got) {
            (e, g) if e == g => self.correct += 1,
            (Some(_), None) => self.outdated_where_line_expected += 1,
            _ => {
                self.wrong += 1;
                return true;
            }
        }
        false
    }
}

fn fixtures(out: &str) {
    let mut records = Vec::new();
    let mut pass = BTreeMap::new();
    for f in fixtures::FIXTURES {
        let old = split(f.old);
        let expected = fixtures::expected(f, old.len());
        let (new_path, new_bytes) = f.new.unwrap_or(("", b""));
        let new = split(new_bytes);
        let pair = Pair::new(&old, &new);
        let rule = lcs_rule(&pair.old, &pair.new).expected;
        // A deleted file has no new lines: each anchor is outdated, for each method.
        let deleted = f.new.is_none();
        let rule_agrees = deleted || rule == expected;
        let cmap = canonical_map(f.old, new_bytes);
        let mut methods = serde_json::Map::new();
        for m in METHODS {
            let got = if deleted { vec![None; old.len()] } else { map_all(m, &pair, &cmap) };
            let ok = got == expected;
            *pass.entry(format!("{m:?}")).or_insert(0) += ok as u32;
            let diffs: Vec<Value> = (0..old.len())
                .filter(|i| got[*i] != expected[*i])
                .map(|i| json!({"old_line": i + 1, "expected": line(expected[i]), "got": line(got[i])}))
                .collect();
            methods.insert(format!("{m:?}"), json!({"pass": ok, "differences": diffs}));
        }
        records.push(json!({
            "fixture": f.name,
            "old_path": f.old_path,
            "new_path": if deleted { Value::Null } else { Value::from(new_path) },
            "old_lines": old.len(),
            "expected": f.expected,
            "lcs_rule_agrees": rule_agrees,
            "methods": methods,
        }));
    }
    let summary = json!({"fixtures": fixtures::FIXTURES.len(), "pass": pass});
    write(out, json!({"summary": summary, "fixtures": records}));
}

fn property(profile: &str, histories: u64, out: &str) {
    let p = history::PROFILES.iter().find(|p| p.name == profile).expect("profile");
    let mut scores: BTreeMap<String, Score> = BTreeMap::new();
    let mut wrong_examples: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    let (mut expect_line, mut expect_edited_or_deleted, mut expect_ambiguous) = (0u64, 0u64, 0u64);
    let (mut anchors, mut unchanged_histories) = (0u64, 0u64);
    for seed in 0..histories {
        let h = history::generate(seed, p);
        let old_bytes = h.old.concat();
        let new_bytes = h.new.concat();
        let old = split(&old_bytes);
        let new = split(&new_bytes);
        assert_eq!(old.len(), h.old.len(), "seed {seed}: old split");
        assert_eq!(new.len(), h.new.len(), "seed {seed}: new split");
        unchanged_histories += (old_bytes == new_bytes) as u64;
        let pair = Pair::new(&old, &new);
        let rule = lcs_rule(&pair.old, &pair.new);
        // The survivors are a longest common subsequence, and the rule agrees with each one
        // that it maps.
        let survivors = h.survivor.iter().flatten().count() as u32;
        assert_eq!(rule.len, survivors, "seed {seed}: the survivors are not an LCS");
        for i in 0..old.len() {
            if let Some(j) = rule.expected[i] {
                assert_eq!(h.survivor[i], Some(j), "seed {seed}: rule and history differ at {}", i + 1);
                expect_line += 1;
            } else if h.survivor[i].is_none() {
                expect_edited_or_deleted += 1;
            } else {
                expect_ambiguous += 1;
            }
        }
        anchors += old.len() as u64;
        let cmap = canonical_map(&old_bytes, &new_bytes);
        for m in METHODS {
            let got = map_all(m, &pair, &cmap);
            let s = scores.entry(format!("{m:?}")).or_default();
            for i in 0..old.len() {
                if s.add(rule.expected[i], got[i]) {
                    let ex = wrong_examples.entry(format!("{m:?}")).or_default();
                    if ex.len() < 5 {
                        ex.push(json!({
                            "seed": seed,
                            "old_line": i + 1,
                            "expected": line(rule.expected[i]),
                            "history_survivor": line(h.survivor[i]),
                            "got": line(got[i]),
                            "edits": h.edits,
                            "old": String::from_utf8_lossy(&old_bytes),
                            "new": String::from_utf8_lossy(&new_bytes),
                        }));
                    }
                }
            }
        }
    }
    let summary = json!({
        "profile": p.name,
        "max_old_lines": p.max_lines,
        "max_edits": p.max_edits,
        "histories": histories,
        "seeds": format!("0..{histories}"),
        "unchanged_histories": unchanged_histories,
        "anchors": anchors,
        "expected_line": expect_line,
        "expected_outdated_edited_or_deleted": expect_edited_or_deleted,
        "expected_outdated_ambiguous": expect_ambiguous,
        "window_context_lines": K,
        "methods": scores,
    });
    write(out, json!({"summary": summary, "wrong_examples": wrong_examples}));
}

/// Real pairs: the expected result is the LCS rule, with no edit history.
fn real(tsv: &str, out: &str) {
    const MAX_CELLS: usize = 100_000_000;
    let list = std::fs::read_to_string(tsv).expect("pairs");
    let mut scores: BTreeMap<String, Score> = BTreeMap::new();
    let mut wrong: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    let (mut pairs, mut skipped, mut expect_line, mut expect_outdated) = (0u64, Vec::new(), 0u64, 0u64);
    let mut diff_not_lcs_pairs = 0u64;
    // For each method: wrong anchors, of them in a pair where the diff is not an LCS, of them
    // where the method gives the line of the canonical diff, and the pairs.
    let mut wrong_where: BTreeMap<String, (u64, u64, u64, Vec<String>)> = BTreeMap::new();
    for row in list.lines() {
        let mut cols = row.split('\t');
        let (name, left, right) = (cols.next().unwrap(), cols.next().unwrap(), cols.next().unwrap());
        let a = std::fs::read(left).expect("left");
        let b = std::fs::read(right).expect("right");
        let (old, new) = (split(&a), split(&b));
        if (old.len() + 1) * (new.len() + 1) > MAX_CELLS {
            skipped.push(json!({"pair": name, "old_lines": old.len(), "new_lines": new.len()}));
            continue;
        }
        pairs += 1;
        let pair = Pair::new(&old, &new);
        let lcs = lcs_rule(&pair.old, &pair.new);
        let rule = lcs.expected;
        expect_line += rule.iter().flatten().count() as u64;
        expect_outdated += rule.iter().filter(|r| r.is_none()).count() as u64;
        let cmap = canonical_map(&a, &b);
        // The canonical diff keeps fewer lines than a longest common subsequence.
        let diff_not_lcs = (cmap.iter().flatten().count() as u32) < lcs.len;
        diff_not_lcs_pairs += diff_not_lcs as u64;
        for m in METHODS {
            let got = map_all(m, &pair, &cmap);
            let s = scores.entry(format!("{m:?}")).or_default();
            for i in 0..old.len() {
                if s.add(rule[i], got[i]) {
                    let w = wrong_where.entry(format!("{m:?}")).or_default();
                    w.0 += 1;
                    w.1 += diff_not_lcs as u64;
                    w.2 += (cmap[i] == got[i]) as u64;
                    if w.3.last() != Some(&name.to_string()) {
                        w.3.push(name.to_string());
                    }
                    let ex = wrong.entry(format!("{m:?}")).or_default();
                    if ex.len() < 20 {
                        ex.push(json!({
                            "pair": name,
                            "old_line": i + 1,
                            "text": String::from_utf8_lossy(old[i]),
                            "expected": line(rule[i]),
                            "got": line(got[i]),
                        }));
                    }
                }
            }
        }
    }
    let summary = json!({
        "pairs": pairs,
        "skipped_large_pairs": skipped.len(),
        "anchors": expect_line + expect_outdated,
        "expected_line": expect_line,
        "expected_outdated": expect_outdated,
        "window_context_lines": K,
        "pairs_where_canonical_diff_keeps_fewer_lines_than_lcs": diff_not_lcs_pairs,
        "methods": scores,
        "wrong": wrong_where.iter().map(|(k, w)| (k.clone(), json!({
            "anchors": w.0,
            "in_pairs_where_diff_is_not_lcs": w.1,
            "at_the_canonical_diff_line": w.2,
            "pairs": w.3.len(),
        }))).collect::<serde_json::Map<_, _>>(),
    });
    let pairs_of: BTreeMap<_, _> = wrong_where.into_iter().map(|(k, w)| (k, w.3)).collect();
    write(out, json!({"summary": summary, "skipped": skipped, "wrong_pairs": pairs_of, "wrong_examples": wrong}));
}

fn write(out: &str, v: Value) {
    let mut s = serde_json::to_string_pretty(&v).expect("json");
    s.push('\n');
    std::fs::write(out, s).expect("write");
    println!("{}", serde_json::to_string(&v["summary"]).unwrap());
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    match a.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        ["fixtures", out] => fixtures(out),
        ["property", profile, n, out] => property(profile, n.parse().expect("histories"), out),
        ["real", tsv, out] => real(tsv, out),
        ["bound", out] => timing::bound(out),
        ["timing", tsv, runs, out] => timing::run(tsv, runs.parse().expect("runs"), out),
        _ => {
            eprintln!("usage: q45-anchors fixtures OUT | property small|medium N OUT | real PAIRS.TSV OUT | timing PAIRS.TSV RUNS OUT | bound OUT");
            std::process::exit(2);
        }
    }
}
