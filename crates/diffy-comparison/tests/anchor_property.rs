//! A property test for the anchor mapping (Q45, Q64).
//!
//! It builds an old file and an edit history from a seed. Each new line that an
//! edit makes has a text that no old line has, so the lines that survive are
//! known. The mapping must never move an anchor to a wrong line: a surviving
//! line maps to its new line or is outdated, and an edited or deleted line is
//! outdated.

use diffy_comparison::anchor::map_anchors;

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 33
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

/// The old lines: unique lines, lines that repeat (such as `}`) and blocks
/// that occur more than once.
fn old_file(rng: &mut Rng) -> Vec<String> {
    let n = 3 + rng.below(30) as usize;
    let mut lines = Vec::new();
    while lines.len() < n {
        match rng.below(5) {
            0 => lines.push("}".to_owned()),
            1 => lines.push(String::new()),
            2 => {
                // A block of up to 3 lines that may repeat.
                let block: Vec<String> = (0..1 + rng.below(3))
                    .map(|i| format!("blk{} {i}", rng.below(3)))
                    .collect();
                for _ in 0..1 + rng.below(3) {
                    lines.extend(block.iter().cloned());
                }
            }
            _ => lines.push(format!("unique {}", lines.len())),
        }
    }
    lines
}

/// Edit the old lines. Returns the new lines and, for each old line, its new
/// index when it survived unchanged.
fn edit(rng: &mut Rng, old: &[String]) -> (Vec<String>, Vec<Option<usize>>) {
    let mut new = Vec::new();
    let mut truth = vec![None; old.len()];
    let mut fresh = 0;
    let mut new_line = |rng: &mut Rng| {
        fresh += 1;
        // Never equal to an old line, so no edit can accidentally match.
        format!("new {fresh} {}", rng.below(1000))
    };
    for (i, line) in old.iter().enumerate() {
        match rng.below(8) {
            0 => {}                       // delete
            1 => new.push(new_line(rng)), // edit
            2 => {
                // insert a line before, keep the old one
                new.push(new_line(rng));
                truth[i] = Some(new.len());
                new.push(line.clone());
            }
            _ => {
                truth[i] = Some(new.len());
                new.push(line.clone());
            }
        }
    }
    if rng.below(3) == 0 {
        new.push(new_line(rng));
    }
    (new, truth)
}

fn join(lines: &[String]) -> Vec<u8> {
    lines
        .iter()
        .map(|l| format!("{l}\n"))
        .collect::<String>()
        .into_bytes()
}

#[test]
fn the_mapping_never_moves_an_anchor_to_a_wrong_line() {
    let mut rng = Rng(0xD1FF);
    let (mut survivors, mut mapped) = (0u64, 0u64);
    for case in 0..20_000 {
        let old = old_file(&mut rng);
        let (new, truth) = edit(&mut rng, &old);
        let got = map_anchors(&join(&old), &join(&new));
        assert_eq!(got.len(), old.len());
        for (i, (g, t)) in got.iter().zip(&truth).enumerate() {
            survivors += u64::from(t.is_some());
            match (g, t) {
                (None, _) => {}
                (Some(j), Some(k)) if j == k => mapped += 1,
                _ => panic!(
                    "case {case}: old line {i} ({:?}) mapped to {g:?}, truth {t:?}\nold: {old:?}\nnew: {new:?}",
                    old[i]
                ),
            }
        }
    }
    // The mapping is useful: it maps most surviving lines, not none.
    assert!(
        mapped * 10 > survivors * 8,
        "mapped {mapped} of {survivors} surviving lines"
    );
}

#[test]
fn an_edit_with_no_ambiguity_maps_every_surviving_line() {
    // All lines unique: each unchanged line must map.
    let old: Vec<String> = (0..50).map(|i| format!("line {i}")).collect();
    let mut new = old.clone();
    new[10] = "edited".into();
    new.remove(20);
    new.insert(30, "inserted".into());
    let got = map_anchors(&join(&old), &join(&new));
    assert_eq!(got[10], None);
    assert_eq!(got[20], None);
    assert_eq!(got[0], Some(0));
    assert_eq!(got[49], Some(49));
}

/// A naive mapper: each old line goes to the first equal new line. It is wrong
/// for repeated lines, so the property above has teeth.
fn naive(old: &[String], new: &[String]) -> Vec<Option<usize>> {
    old.iter()
        .map(|l| new.iter().position(|n| n == l))
        .collect()
}

#[test]
fn the_property_catches_a_naive_mapper() {
    let mut rng = Rng(0xD1FF);
    let mut wrong = 0;
    for _ in 0..2_000 {
        let old = old_file(&mut rng);
        let (new, truth) = edit(&mut rng, &old);
        for (g, t) in naive(&old, &new).iter().zip(&truth) {
            wrong += usize::from(
                matches!((g, t), (Some(j), Some(k)) if j != k) || (g.is_some() && t.is_none()),
            );
        }
    }
    assert!(
        wrong > 100,
        "the naive mapper made only {wrong} wrong mappings"
    );
}
