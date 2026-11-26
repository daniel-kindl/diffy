//! The 17 Q45 fixtures against `map_anchors` (Q45, Q64). Each fixture is an
//! old and a new revision with the expected new line, or outdated, for each
//! old line. The mapping compares lines without the terminator (Q64), and
//! the canonical diff keeps each byte, so a change of the line ending is an
//! edit.

#[path = "data/q45_fixtures.rs"]
mod fixtures;

use diffy_comparison::anchor::map_anchors;

fn old_line_count(old: &[u8]) -> usize {
    old.split_inclusive(|b| *b == b'\n').count()
}

#[test]
fn each_fixture_gives_the_expected_result() {
    let mut failures = Vec::new();
    for f in fixtures::FIXTURES {
        let want = fixtures::expected(f, old_line_count(f.old));
        let got = match f.new {
            Some((_, new)) => map_anchors(f.old, new),
            // A deleted file has no new revision: each anchor is outdated.
            None => vec![None; old_line_count(f.old)],
        };
        if got != want {
            failures.push(format!(
                "{} ({}): got {got:?}, want {want:?}",
                f.name, f.old_path
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} fixtures differ:\n{}",
        failures.len(),
        fixtures::FIXTURES.len(),
        failures.join("\n")
    );
}

#[test]
fn there_are_17_fixtures() {
    assert_eq!(fixtures::FIXTURES.len(), 17);
}
