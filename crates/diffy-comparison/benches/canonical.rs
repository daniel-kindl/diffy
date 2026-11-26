//! Benchmark of the canonical diff and the anchor mapping (F-07).
//!
//! Run: `cargo bench -p diffy-comparison --bench canonical`
//!
//! It builds two synthetic inputs, times each run, and prints one JSON line per
//! case with the median and the maximum time and the peak memory (Linux:
//! `VmHWM`). The inputs are deterministic, so two runs compare. The reference
//! hardware is the Fedora host. A run elsewhere shows relative changes only.

use diffy_comparison::anchor::map_anchors;
use diffy_comparison::canonical::canonical_hunks;
use std::time::Instant;

/// A small deterministic generator, so the inputs are the same on each run.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 33
    }
}

/// Lines of about `width` bytes, until the text has `bytes` bytes.
fn text(bytes: usize, width: usize, seed: u64) -> Vec<String> {
    let mut rng = Lcg(seed);
    let mut lines = Vec::new();
    let mut total = 0;
    while total < bytes {
        let n = rng.next();
        let line = format!("{:0>w$x}\n", n ^ (n << 17), w = width - 1);
        total += line.len();
        lines.push(line);
    }
    lines
}

/// Change `count` evenly spread lines.
fn edit(lines: &[String], count: usize) -> Vec<String> {
    let mut out = lines.to_vec();
    let step = (lines.len() / count.max(1)).max(1);
    for i in (0..lines.len()).step_by(step).take(count) {
        out[i] = format!("changed {i}\n");
    }
    out
}

fn peak_kib() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("VmHWM:"))
                .and_then(|l| l.split_whitespace().nth(1)?.parse().ok())
        })
        .unwrap_or(0)
}

fn run(name: &str, base: &[String], head: &[String], runs: usize) {
    let (b, h) = (base.concat().into_bytes(), head.concat().into_bytes());
    let mut times = Vec::new();
    let mut hunks = 0;
    for _ in 0..runs {
        let t = Instant::now();
        hunks = canonical_hunks(&b, &h).len();
        times.push(t.elapsed().as_secs_f64() * 1000.0);
    }
    let t = Instant::now();
    let mapped = map_anchors(&b, &h).iter().filter(|m| m.is_some()).count();
    let map_ms = t.elapsed().as_secs_f64() * 1000.0;
    times.sort_by(|a, b| a.total_cmp(b));
    println!(
        "{{\"case\":\"{name}\",\"bytes\":{},\"lines\":{},\"hunks\":{hunks},\"canonical_ms_median\":{:.1},\"canonical_ms_max\":{:.1},\"anchor_map_ms\":{map_ms:.1},\"anchors_mapped\":{mapped},\"peak_rss_kib\":{}}}",
        b.len(),
        base.len(),
        times[times.len() / 2],
        times[times.len() - 1],
        peak_kib()
    );
}

fn main() {
    let runs = std::env::var("DIFFY_BENCH_RUNS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5);
    let small = text(1 << 20, 80, 1);
    run("1mb_100_changes", &small, &edit(&small, 100), runs);
    let lines = text(100_000 * 80, 80, 2);
    run("10000_changed_lines", &lines, &edit(&lines, 10_000), runs);
    let big = text(50 << 20, 80, 3);
    run("50mb_1000_changes", &big, &edit(&big, 1_000), runs);
}
