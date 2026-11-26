//! Q65: the time of the M3 mapping on the Q46 fixtures, and the anchors that are outdated
//! because their segment has more than `SEGMENT_CELLS` cells.

use crate::core::{Method, Pair, SEGMENT_CELLS, canonical_map, map_all, split};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::time::Instant;

/// The M3 segments, as `brackets` in `core.rs` builds them: (old start, old end, new start, new end).
fn segments(pair: &Pair, cmap: &[Option<usize>]) -> Vec<(usize, usize, usize, usize)> {
    let mut count = vec![(0u32, 0u32); pair.tokens as usize];
    for t in &pair.old {
        count[*t as usize].0 += 1;
    }
    for t in &pair.new {
        count[*t as usize].1 += 1;
    }
    let mut marks: Vec<(isize, isize)> = (0..pair.old.len())
        .filter(|p| count[pair.old[*p] as usize] == (1, 1))
        .filter_map(|p| cmap[p].map(|q| (p as isize, q as isize)))
        .collect();
    marks.push((pair.old.len() as isize, pair.new.len() as isize));
    let mut prev = (-1isize, -1isize);
    let mut out = Vec::new();
    for (p1, q1) in marks {
        out.push(((prev.0 + 1) as usize, p1 as usize, (prev.1 + 1) as usize, q1 as usize));
        prev = (p1, q1);
    }
    out
}

fn pct(v: &mut [f64], p: f64) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    if v.is_empty() {
        return 0.0;
    }
    let i = ((p / 100.0) * v.len() as f64).ceil() as usize;
    v[i.clamp(1, v.len()) - 1]
}

fn ms(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

#[derive(Default)]
struct Fixture {
    pairs: u64,
    anchors: u64,
    mapped: u64,
    outdated: u64,
    capped_segments: u64,
    capped_anchors: u64,
    capped_kept_by_diff: u64,
    largest_segment_cells: u64,
    panics: Vec<String>,
    file_ms: Vec<f64>,
    diff_ms: Vec<f64>,
    m3_ms: Vec<f64>,
    run_total_ms: Vec<f64>,
}

pub fn run(tsv: &str, runs: usize, out: &str) {
    let list = std::fs::read_to_string(tsv).expect("pairs");
    let rows: Vec<(String, String, String)> = list
        .lines()
        .map(|r| {
            let c: Vec<&str> = r.split('\t').collect();
            (c[0].to_string(), c[1].to_string(), c[2].to_string())
        })
        .collect();
    let mut fx: BTreeMap<String, Fixture> = BTreeMap::new();
    for run in 0..runs {
        let mut totals: BTreeMap<String, f64> = BTreeMap::new();
        for (name, left, right) in &rows {
            let fixture = name.split('/').next().unwrap().to_string();
            let a = std::fs::read(left).expect("left");
            let b = std::fs::read(right).expect("right");
            let t0 = Instant::now();
            let cmap = canonical_map(&a, &b);
            let t1 = Instant::now();
            let (old, new) = (split(&a), split(&b));
            let pair = Pair::new(&old, &new);
            let got = std::panic::catch_unwind(|| map_all(Method::HunkBrackets, &pair, &cmap));
            let t2 = Instant::now();
            let f = fx.entry(fixture.clone()).or_default();
            let (d, m) = ((t1 - t0).as_secs_f64() * 1e3, (t2 - t1).as_secs_f64() * 1e3);
            f.diff_ms.push(d);
            f.m3_ms.push(m);
            f.file_ms.push(d + m);
            *totals.entry(fixture).or_default() += d + m;
            if run > 0 {
                continue;
            }
            f.pairs += 1;
            f.anchors += old.len() as u64;
            match got {
                Err(_) => f.panics.push(name.clone()),
                Ok(got) => {
                    f.mapped += got.iter().flatten().count() as u64;
                    f.outdated += got.iter().filter(|g| g.is_none()).count() as u64;
                }
            }
            for (os, oe, ns, ne) in segments(&pair, &cmap) {
                let cells = ((oe - os) * (ne - ns)) as u64;
                f.largest_segment_cells = f.largest_segment_cells.max(cells);
                if cells > SEGMENT_CELLS as u64 {
                    f.capped_segments += 1;
                    f.capped_anchors += (oe - os) as u64;
                    f.capped_kept_by_diff += cmap[os..oe].iter().flatten().count() as u64;
                }
            }
        }
        for (k, v) in totals {
            fx.get_mut(&k).unwrap().run_total_ms.push(v);
        }
        eprintln!("run {} of {runs} done", run + 1);
    }
    let mut res = serde_json::Map::new();
    for (k, mut f) in fx {
        let v: Value = json!({
            "pairs": f.pairs,
            "anchors": f.anchors,
            "m3_mapped": f.mapped,
            "m3_outdated": f.outdated,
            "segments_over_cap": f.capped_segments,
            "anchors_in_segments_over_cap": f.capped_anchors,
            "of_them_kept_by_canonical_diff": f.capped_kept_by_diff,
            "largest_segment_cells": f.largest_segment_cells,
            "panics": f.panics,
            "per_file_ms": {
                "p50": ms(pct(&mut f.file_ms, 50.0)), "p95": ms(pct(&mut f.file_ms, 95.0)), "max": ms(pct(&mut f.file_ms, 100.0)),
                "canonical_diff_p50": ms(pct(&mut f.diff_ms, 50.0)), "canonical_diff_p95": ms(pct(&mut f.diff_ms, 95.0)),
                "m3_after_diff_p50": ms(pct(&mut f.m3_ms, 50.0)), "m3_after_diff_p95": ms(pct(&mut f.m3_ms, 95.0)), "m3_after_diff_max": ms(pct(&mut f.m3_ms, 100.0)),
            },
            "fixture_total_ms_one_thread": {
                "runs": f.run_total_ms.len(), "p50": ms(pct(&mut f.run_total_ms, 50.0)), "max": ms(pct(&mut f.run_total_ms, 100.0)),
            },
        });
        res.insert(k, v);
    }
    let doc = json!({
        "note": "Q65: M3 (HunkBrackets) on each modified or renamed file of the Q46 fixtures, each old line an anchor. Times: canonical diff, then split, interning and M3. One thread, file reads not timed.",
        "segment_cells_cap": SEGMENT_CELLS,
        "runs": runs,
        "fixtures": res,
    });
    std::fs::write(out, serde_json::to_string_pretty(&doc).unwrap() + "\n").expect("write");
}

/// The cost of one segment at the cap: the LCS rule on `n` old and `m` new lines with
/// `n * m = SEGMENT_CELLS`, from 4 different line texts (many repeats), as a bound.
pub fn bound(out: &str) {
    let mut r = crate::history::Rng::new(65);
    let mut rows = Vec::new();
    for (n, m) in [(1000usize, 1000usize), (100, 10_000), (10, 100_000), (1, 1_000_000)] {
        let a: Vec<u32> = (0..n).map(|_| r.range(0, 4) as u32).collect();
        let b: Vec<u32> = (0..m).map(|_| r.range(0, 4) as u32).collect();
        let mut times = Vec::new();
        let mut panicked = false;
        for _ in 0..5 {
            let t = Instant::now();
            let ok = std::panic::catch_unwind(|| crate::core::lcs_rule(&a, &b)).is_ok();
            times.push(t.elapsed().as_secs_f64() * 1e3);
            panicked |= !ok;
        }
        rows.push(json!({"old_lines": n, "new_lines": m, "cells": n * m, "panic": panicked,
            "ms_p50": ms(pct(&mut times, 50.0)), "ms_max": ms(pct(&mut times, 100.0))}));
    }
    let doc = json!({"note": "Q65: one M3 segment at the cap, 5 runs each, 4 line texts.", "segments": rows});
    std::fs::write(out, serde_json::to_string_pretty(&doc).unwrap() + "\n").expect("write");
}
