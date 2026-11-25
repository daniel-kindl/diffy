# Q45 prototype: S5 anchor mapping

Throwaway code for map question Q45 in `docs/maps/diffy-v0.1.md`. It is not Diffy code.

Charter question: Which method moves a finding to the correct line of a new revision, and marks
it outdated when the mapping is not certain?

## Terms

- An anchor is one line of the old revision of a file. The prototype anchors each old line.
- A line is its bytes with its line terminator. A change to any byte is an edit, also a change of
  whitespace or of the line ending.
- The canonical diff is the Q43 and Q61 diff: gix-imara-diff 0.3.4 with the Q60 patch,
  `Algorithm::Histogram`, then `postprocess_lines`. `setup.sh` downloads the crate, checks its
  SHA-256 and applies `gix-imara-diff-0.3.4-q60.patch` from the tag `prototype/diffy-v0.1-q60`.
- The file pair comes from the Git backend (Q44). A renamed file is one pair with two paths. A
  deleted file has no new revision, so each anchor in it is outdated.

## The LCS rule: the expected result

A longest common subsequence (LCS) of two files is a longest list of lines that both files
have in the same order. An old line maps to new line `j` only when each LCS matches it to `j`. Otherwise the anchor is outdated. Either the line
was edited or deleted, or another edit of the same size explains the change in another way. Two
identical blocks where one is deleted are an example.

`lcs_rule` in `src/core.rs` computes the rule with two dynamic programming tables. A unit test
compares it with a brute force list of each LCS, on 20,000 random pairs of up to 7 lines.

## Methods

- M1 `HunkMap`: the line moves with the canonical diff. A line in a change group is outdated.
- M2 `HunkWindow`: M1, and then a check. The line and 3 lines above and below must be equal in both
  files, and this window must occur once in each file. The start and the end of the file count as
  lines of the window.
- M3 `HunkBrackets`: M1, and then the LCS rule inside a segment. A bracket is a line that occurs
  once in each file and that the canonical diff keeps. The start and the end of the file are also
  brackets. The anchor maps only when the LCS rule on the segment between the two nearest brackets
  gives the same line as the canonical diff. A bracket line maps as the canonical diff maps it. A
  segment with more than 1,000,000 cells (old lines times new lines) is outdated.

A first M3 needed an unchanged run from the anchor to each bracket. It had no wrong mapping, but
it failed 2 fixtures with an outdated result where a line was expected. The segment rule replaced
it.

## Evidence

- `fixtures`: 17 revision pairs in `src/fixtures.rs`, each with one expected result for each
  old line.
  - Line unchanged, lines added above, a line added directly above, line edited, line deleted.
  - Duplicate blocks: one deleted, a copy inserted, one edited, and a slide.
  - A common line between unique lines.
  - File renamed and file deleted.
  - Whitespace on the anchor line and on another line.
  - LF to CRLF for the file and for one line, and no newline at the end. The LCS rule gives the expected result for
  each fixture (`lcs_rule_agrees`).
- `property`: `src/history.rs` generates an old file and an edit history from a seed. The old file
  has unique lines, common lines such as `}` and repeats of up to 3 blocks. The edits insert,
  delete and edit lines. They also change whitespace and line endings, and remove the newline at
  the end. The generator records each old line that survives unedited, and its new line.
- Each line that an edit creates has a text that no old line has. So the survivors are an LCS. For
  each history, the harness checks that the survivors are an LCS. It also checks that each line
  the LCS rule maps is a survivor at the same new line. A survivor that the LCS rule does not map
  is ambiguous.
- `real`: the 3,732 Q43 real pairs from `corpus.sh`, with the Q43 commits of git/git, tokio and
  ripgrep. A real pair has no edit history, so the LCS rule alone is the expected result.

A wrong mapping is a line where the expected result is outdated or another line.

## Results

`./run.sh <work-dir>` writes `results/`. rustc 1.98.1 and Git 2.55.0, in the `rust-dev`
distrobox container (Fedora 44), 2026-11-25.

Fixtures (`results/fixtures.json`), the number of pairs with the expected result for each line:

| Method | Pass (17) |
|--------|-----------|
| M1 `HunkMap` | 14 |
| M2 `HunkWindow` | 5 |
| M3 `HunkBrackets` | 17 |

- M1 gives a line where outdated is expected in 3 duplicate block fixtures.
- M2 gives a line where outdated is expected in `duplicate-blocks-slide`. It gives outdated where
  a line is expected in 11 other fixtures.

Property test (`results/property-small.json` and `results/property-medium.json`):

| Profile | Histories | Anchors | Line expected | Outdated: edited or deleted | Outdated: ambiguous |
|---------|-----------|---------|---------------|-----------------------------|---------------------|
| small: up to 40 lines and 6 edits | 1,000,000 | 19,999,115 | 15,350,359 | 4,206,710 | 442,046 |
| medium: up to 200 lines and 20 edits | 100,000 | 10,018,505 | 6,700,572 | 3,202,315 | 115,618 |

| Method | small: wrong | small: outdated where a line is expected | medium: wrong | medium: outdated where a line is expected |
|--------|--------------|------------------------------------------|---------------|-------------------------------------------|
| M1 | 463,746 | 21,316 | 120,012 | 4,678 |
| M2 | 47,257 | 5,954,755 | 11,359 | 2,129,017 |
| M3 | 0 | 36,680 | 0 | 7,776 |

Real pairs (`results/real.json`): 3,714 pairs and 3,126,559 anchors. 18 pairs are skipped, because
the LCS tables would have more than 100,000,000 cells. They are Git `po/*.po` files. The LCS rule
maps 3,104,511 anchors.

| Method | Not the LCS rule result | Pairs | Outdated where the LCS rule maps |
|--------|-------------------------|-------|----------------------------------|
| M1 | 4,069 | 1,376 | 267 |
| M2 | 304 | 103 | 157,377 |
| M3 | 185 | 46 | 408 |

## Findings

1. Only M3 qualifies: each fixture passes, and the property test has no wrong mapping. M3 is
   outdated where a line is expected for 0.24 % (small) and 0.12 % (medium) of these anchors.
   M1 has 21,316 (small) and 4,678 (medium) of them too. The canonical diff puts these lines in
   a change group, but the LCS rule keeps them.
2. M1 follows the canonical diff into ambiguous duplicates. When the diff keeps one of two
   identical blocks, M1 maps each anchor of both blocks.
3. M2 is too strict and not safe. A change within 3 lines of an anchor makes it outdated. A window
   with a unique line in it can still be at an ambiguous line, as in `duplicate-blocks-slide`.
4. On real pairs, M3 maps 185 anchors where the LCS rule gives outdated. In each case M3 gives the
   line where the canonical diff, the same as Git, keeps the line. In 79 of them, the canonical
   diff keeps fewer lines than an LCS. In the other 106, the canonical diff is one of several LCSs,
   and another LCS moves or drops the line. The property test has no moved lines, so it does not
   test this case.
   - An example is `loose.c` in git/git commit `e8bf3190858e`. Part of `loose_object_map_load`
     moves into a new function above it. The canonical diff keeps the line
     `int loose_object_map_load(struct odb_source_loose *loose)`, which occurs once in each file.
     M3 maps it from line 66 to line 107.
   - An LCS matches the moved lines instead and drops that line, so the LCS rule gives outdated.
     `git diff --minimal` also shows line 66 as changed.
5. The LCS rule is quadratic in the file length. M3 uses it only inside a segment between
   brackets, so its cost depends on the largest segment.

## Not tested

- Moved blocks and reverted edits in the property test. The generator never creates a line with
  the text of an old line. The fixtures `duplicate-blocks-copy-inserted` and the real pairs cover
  copies and moves.
- Ranges of more than one line, and findings on the old side of a diff.
- Mapping across more than one revision. Each run maps the original revision to the new one.
- Timing on large files, for example the S8 scale fixtures (Q46).
- An anchor that ignores whitespace or line endings. Each byte change is an edit here.
