//! The Q62 inputs: a line that occurs many times (the seed), where the Q60 copy
//! differed from Git. The merged gitoxide code (commit 2bcb31d) must match
//! Git 2.55.0 on each of them.

mod common;

use common::oracle;
use diffy_comparison::canonical::canonical_hunks;
use std::fs;

/// One line for each token, so each token is one line.
fn lines(tokens: &[&str]) -> Vec<u8> {
    tokens
        .iter()
        .map(|t| format!("{t}\n"))
        .collect::<String>()
        .into_bytes()
}

fn check(tmp: &std::path::Path, name: &str, a: &[u8], b: &[u8]) {
    assert_eq!(canonical_hunks(a, b), oracle(tmp, a, b), "{name}");
}

#[test]
fn the_maintainer_input_matches_git() {
    // `B` 4 times, `A A B A`, then `B` 61 times, against `B A B A`.
    let mut old = vec!["B"; 4];
    old.extend(["A", "A", "B", "A"]);
    old.extend(vec!["B"; 61]);
    let new = ["B", "A", "B", "A"];
    let tmp = std::env::temp_dir().join(format!("diffy-q62-{}", std::process::id()));
    fs::create_dir_all(&tmp).unwrap();
    check(&tmp, "the maintainer input", &lines(&old), &lines(&new));
    fs::remove_dir_all(&tmp).ok();
}

#[test]
fn the_seed_count_family_matches_git() {
    // The same shape with the seed `B` occurring 60 to 110 times in total.
    let tmp = std::env::temp_dir().join(format!("diffy-q62f-{}", std::process::id()));
    fs::create_dir_all(&tmp).unwrap();
    for total in 60..=110usize {
        for lead in [0, 1, 4, 10] {
            let tail = total.saturating_sub(lead + 1);
            let mut old = vec!["B"; lead];
            old.extend(["A", "A", "B", "A"]);
            old.extend(vec!["B"; tail]);
            check(
                &tmp,
                &format!("seed {total} lead {lead}"),
                &lines(&old),
                &lines(&["B", "A", "B", "A"]),
            );
            check(
                &tmp,
                &format!("reverse {total} lead {lead}"),
                &lines(&["B", "A", "B", "A"]),
                &lines(&old),
            );
        }
    }
    fs::remove_dir_all(&tmp).ok();
}
