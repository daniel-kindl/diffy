//! Q43 throwaway harness: compare the change groups of Rust diff crates with
//! the `git diff --histogram` oracle (ADR 0002). Never merged.
//!
//! The canonical hunks are the `-U3` hunks. Each crate gives change groups,
//! and `hunks` groups them with the Git rule.
//!
//! Usage: q43-parity <pairs.tsv> <result.json>
//! Each line of pairs.tsv is `<label>\t<left path>\t<right path>`.

use std::fs;
use std::io::Write;
use std::ops::Range;
use std::process::Command;

use imara_diff::{Algorithm, Diff, InternedInput};

/// One change group: a removed range of left lines and an added range of
/// right lines, 0-based, half-open. A pure insertion has an empty left range
/// that starts at the insertion point, and the reverse for a pure deletion.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Group {
    left: Range<u32>,
    right: Range<u32>,
}

/// The context lines of the canonical hunks.
const CONTEXT: u32 = 3;

/// The hunk options that the oracle gives on the command line, with
/// `-U<n>` and the indent heuristic option.
const ORACLE_ARGS: &[&str] = &[
    "diff",
    "--no-index",
    "--histogram",
    "--inter-hunk-context=0",
    "--no-color",
    "--no-ext-diff",
    "--no-renames",
    "--no-textconv",
];

/// The oracle result: the hunk ranges and the change groups in the hunks.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Oracle {
    hunks: Vec<Group>,
    groups: Vec<Group>,
}

fn oracle(left: &str, right: &str, indent_heuristic: bool) -> Option<Oracle> {
    oracle_u(left, right, indent_heuristic, 0)
}

/// Run the oracle with `-U<context>`. Return the hunk ranges, and the change
/// groups from the `-` and `+` lines of the hunk bodies.
fn oracle_u(left: &str, right: &str, indent_heuristic: bool, context: u32) -> Option<Oracle> {
    let heuristic = if indent_heuristic {
        "--indent-heuristic"
    } else {
        "--no-indent-heuristic"
    };
    let out = Command::new("git")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("HOME", "/nonexistent")
        .env_remove("GIT_DIR")
        .current_dir(std::env::temp_dir())
        .args(ORACLE_ARGS)
        .arg(format!("-U{context}"))
        .arg(heuristic)
        .arg("--")
        .arg(left)
        .arg(right)
        .output()
        .expect("git runs");
    // 0: no difference, 1: difference. Each other code is an error.
    match out.status.code() {
        Some(0) | Some(1) => {}
        _ => panic!("git failed: {}", String::from_utf8_lossy(&out.stderr)),
    }
    let text = out.stdout;
    let lines: Vec<&[u8]> = text.split(|b| *b == b'\n').collect();
    // Git prints "Binary files ... differ" in place of the hunks.
    let has_hunk = lines.iter().any(|l| l.starts_with(b"@@ -"));
    if !has_hunk && lines.iter().any(|l| l.starts_with(b"Binary files ")) {
        return None;
    }
    let mut hunks = Vec::new();
    let mut groups: Vec<Group> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if !lines[i].starts_with(b"@@ -") {
            i += 1;
            continue;
        }
        let s = std::str::from_utf8(lines[i]).expect("hunk header is ASCII");
        let mut parts = s.split(' ');
        parts.next();
        let lr = parse_range(&parts.next().unwrap()[1..]);
        let rr = parse_range(&parts.next().unwrap()[1..]);
        hunks.push(Group {
            left: lr.clone(),
            right: rr.clone(),
        });
        // Read the body: exactly the line counts of the header.
        let (mut l, mut r) = (lr.start, rr.start);
        let mut open: Option<Group> = None;
        i += 1;
        while l < lr.end || r < rr.end {
            let line = lines[i];
            i += 1;
            match line.first() {
                Some(b' ') => {
                    if let Some(g) = open.take() {
                        groups.push(g);
                    }
                    l += 1;
                    r += 1;
                }
                Some(b'-') => {
                    let g = open.get_or_insert(Group {
                        left: l..l,
                        right: r..r,
                    });
                    g.left.end = l + 1;
                    l += 1;
                }
                Some(b'+') => {
                    let g = open.get_or_insert(Group {
                        left: l..l,
                        right: r..r,
                    });
                    g.right.end = r + 1;
                    r += 1;
                }
                Some(b'\\') => {}
                _ => panic!("unexpected hunk line in {left}"),
            }
        }
        if let Some(g) = open.take() {
            groups.push(g);
        }
    }
    Some(Oracle { hunks, groups })
}

/// Convert a unified diff range `start,count` (1-based) to a 0-based range.
/// With count 0, `start` is the line before the empty range.
fn parse_range(s: &str) -> Range<u32> {
    let (start, count) = match s.split_once(',') {
        Some((a, b)) => (a.parse::<u32>().unwrap(), b.parse::<u32>().unwrap()),
        None => (s.parse::<u32>().unwrap(), 1),
    };
    if count == 0 {
        start..start
    } else {
        start - 1..start - 1 + count
    }
}

#[derive(Clone, Copy, Debug)]
enum Variant {
    ImaraRaw,
    ImaraNoHeuristic,
    ImaraIndentHeuristic,
    SimilarHistogram,
    GixRaw,
    GixNoHeuristic,
    GixIndentHeuristic,
}

const VARIANTS: &[Variant] = &[
    Variant::ImaraRaw,
    Variant::ImaraNoHeuristic,
    Variant::ImaraIndentHeuristic,
    Variant::SimilarHistogram,
    Variant::GixRaw,
    Variant::GixNoHeuristic,
    Variant::GixIndentHeuristic,
];

fn crate_groups(variant: Variant, left: &[u8], right: &[u8]) -> Vec<Group> {
    match variant {
        Variant::ImaraRaw | Variant::ImaraNoHeuristic | Variant::ImaraIndentHeuristic => {
            let input = InternedInput::new(left, right);
            let mut diff = Diff::compute(Algorithm::Histogram, &input);
            match variant {
                Variant::ImaraNoHeuristic => diff.postprocess_no_heuristic(&input),
                Variant::ImaraIndentHeuristic => diff.postprocess_lines(&input),
                _ => {}
            }
            diff.hunks()
                .map(|h| Group {
                    left: h.before,
                    right: h.after,
                })
                .collect()
        }
        Variant::GixRaw | Variant::GixNoHeuristic | Variant::GixIndentHeuristic => {
            let input = gix_imara_diff::InternedInput::new(left, right);
            let mut diff =
                gix_imara_diff::Diff::compute(gix_imara_diff::Algorithm::Histogram, &input);
            match variant {
                Variant::GixNoHeuristic => diff.postprocess_no_heuristic(&input),
                Variant::GixIndentHeuristic => diff.postprocess_lines(&input),
                _ => {}
            }
            diff.hunks()
                .map(|h| Group {
                    left: h.before,
                    right: h.after,
                })
                .collect()
        }
        Variant::SimilarHistogram => {
            let l = split_lines(left);
            let r = split_lines(right);
            let ops = similar::capture_diff_slices(similar::Algorithm::Histogram, &l, &r);
            groups_from_ops(&ops)
        }
    }
}

/// Split into lines, each with its line terminator, as Git and imara-diff do.
fn split_lines(data: &[u8]) -> Vec<&[u8]> {
    let mut out = Vec::new();
    let mut start = 0;
    for (i, b) in data.iter().enumerate() {
        if *b == b'\n' {
            out.push(&data[start..=i]);
            start = i + 1;
        }
    }
    if start < data.len() {
        out.push(&data[start..]);
    }
    out
}

/// Merge adjacent delete and insert operations into change groups.
fn groups_from_ops(ops: &[similar::DiffOp]) -> Vec<Group> {
    let mut groups: Vec<Group> = Vec::new();
    for op in ops {
        let (l, r) = match *op {
            similar::DiffOp::Equal { .. } => continue,
            similar::DiffOp::Delete {
                old_index,
                old_len,
                new_index,
            } => (old_index..old_index + old_len, new_index..new_index),
            similar::DiffOp::Insert {
                old_index,
                new_index,
                new_len,
            } => (old_index..old_index, new_index..new_index + new_len),
            similar::DiffOp::Replace {
                old_index,
                old_len,
                new_index,
                new_len,
            } => (
                old_index..old_index + old_len,
                new_index..new_index + new_len,
            ),
        };
        let g = Group {
            left: l.start as u32..l.end as u32,
            right: r.start as u32..r.end as u32,
        };
        if let Some(last) = groups.last_mut() {
            if last.left.end == g.left.start && last.right.end == g.right.start {
                last.left.end = g.left.end;
                last.right.end = g.right.end;
                continue;
            }
        }
        groups.push(g);
    }
    groups
}

/// Group change groups into hunks with Git's rule for `-U<ctx>` and
/// `--inter-hunk-context=0`: git 2.55.0 `xdiff/xemit.c:46-100` (merge) and
/// `xdiff/xemit.c:186-243` (context). No change is ignorable here.
fn hunks(groups: &[Group], ctx: u32, nl: u32, nr: u32) -> Vec<Group> {
    let max_common = 2 * ctx;
    let mut out = Vec::new();
    let mut i = 0;
    while i < groups.len() {
        let first = &groups[i];
        let mut j = i;
        while j + 1 < groups.len() && groups[j + 1].left.start - groups[j].left.end <= max_common {
            j += 1;
        }
        let last = &groups[j];
        let s1 = first.left.start.saturating_sub(ctx);
        let s2 = first.right.start.saturating_sub(ctx);
        let lctx = ctx.min(nl - last.left.end).min(nr - last.right.end);
        out.push(Group {
            left: s1..last.left.end + lctx,
            right: s2..last.right.end + lctx,
        });
        i = j + 1;
    }
    out
}

fn json_str(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

fn first_diff(a: &[Group], b: &[Group]) -> String {
    for i in 0..a.len().max(b.len()) {
        if a.get(i) != b.get(i) {
            return format!("group {i}: git {:?}, crate {:?}", a.get(i), b.get(i));
        }
    }
    String::new()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let pairs = fs::read_to_string(&args[1]).expect("pairs file");
    let mut result = fs::File::create(&args[2]).expect("result file");
    let mut total = 0usize;
    let mut binary = 0usize;
    // [oracle heuristic][variant] -> (pass, fail)
    let mut counts = [[(0usize, 0usize); 7]; 2];
    let mut fails: Vec<String> = Vec::new();
    // -U3 checks: (rule on git -U0 groups == git -U3, gix indent heuristic grouped == git -U3)
    let (mut rule_ok, mut rule_bad) = (0usize, 0usize);
    for line in pairs.lines() {
        let mut f = line.split('\t');
        let (label, lp, rp) = (f.next().unwrap(), f.next().unwrap(), f.next().unwrap());
        let left = fs::read(lp).expect("left");
        let right = fs::read(rp).expect("right");
        // The oracle is the -U3 hunk list. Git trims the common tail of the
        // input when the context is 0 (git 2.55.0 xdiff-interface.c:127), so a
        // -U0 run can place a change group at other lines than a -U3 run.
        let Some(git_ih) = oracle_u(lp, rp, true, CONTEXT) else {
            binary += 1;
            continue;
        };
        let git_noih = oracle_u(lp, rp, false, CONTEXT).expect("text");
        total += 1;
        let nl = split_lines(&left).len() as u32;
        let nr = split_lines(&right).len() as u32;
        // Side check: do the -U0 change groups of Git, grouped with the -U3
        // rule, give the -U3 hunks? A fail shows the tail trim effect.
        let git_u0 = oracle(lp, rp, true).expect("text");
        if git_u0.groups == git_ih.groups {
            rule_ok += 1
        } else {
            rule_bad += 1;
            fails.push(format!("{{\"pair\":{},\"variant\":\"git-U0\",\"oracle\":\"indent-heuristic\",\"first\":{}}}",
                json_str(label), json_str(&first_diff(&git_ih.groups, &git_u0.groups))));
        }
        for (vi, v) in VARIANTS.iter().enumerate() {
            let groups = crate_groups(*v, &left, &right);
            let got = Oracle {
                hunks: hunks(&groups, CONTEXT, nl, nr),
                groups,
            };
            for (oi, want) in [&git_ih, &git_noih].iter().enumerate() {
                if got == **want {
                    counts[oi][vi].0 += 1;
                } else {
                    counts[oi][vi].1 += 1;
                    if counts[oi][vi].1 <= 100 {
                        let oracle_name = if oi == 0 {
                            "indent-heuristic"
                        } else {
                            "no-indent-heuristic"
                        };
                        fails.push(format!(
                            "{{\"pair\":{},\"variant\":\"{:?}\",\"oracle\":\"{}\",\"first\":{}}}",
                            json_str(label),
                            v,
                            oracle_name,
                            json_str(&if want.groups != got.groups {
                                first_diff(&want.groups, &got.groups)
                            } else {
                                format!("hunks: {}", first_diff(&want.hunks, &got.hunks))
                            })
                        ));
                    }
                }
            }
        }
    }
    let git_version = String::from_utf8(
        Command::new("git")
            .arg("--version")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    writeln!(result, "{{").unwrap();
    writeln!(result, "  \"git\": {},", json_str(git_version.trim())).unwrap();
    writeln!(
        result,
        "  \"oracle_args\": {},",
        json_str(&ORACLE_ARGS.join(" "))
    )
    .unwrap();
    writeln!(result, "  \"text_pairs\": {total},").unwrap();
    writeln!(result, "  \"binary_pairs_skipped\": {binary},").unwrap();
    writeln!(
        result,
        "  \"git_u0_groups_equal_git_u3_groups\": {{\"pass\":{rule_ok},\"fail\":{rule_bad}}},"
    )
    .unwrap();
    println!("git -U0 groups == git -U3 groups: pass {rule_ok} fail {rule_bad}");
    writeln!(result, "  \"results\": [").unwrap();
    let mut rows = Vec::new();
    for (oi, oracle_name) in ["--indent-heuristic", "--no-indent-heuristic"]
        .iter()
        .enumerate()
    {
        for (vi, v) in VARIANTS.iter().enumerate() {
            let (p, fl) = counts[oi][vi];
            rows.push(format!(
                "    {{\"oracle\":\"{oracle_name}\",\"variant\":\"{v:?}\",\"pass\":{p},\"fail\":{fl}}}"
            ));
            println!("{oracle_name:22} {v:22?} pass {p:6} fail {fl:6}");
        }
    }
    writeln!(result, "{}", rows.join(",\n")).unwrap();
    writeln!(result, "  ],").unwrap();
    writeln!(
        result,
        "  \"first_fails\": [\n    {}\n  ]",
        fails.join(",\n    ")
    )
    .unwrap();
    writeln!(result, "}}").unwrap();
}
