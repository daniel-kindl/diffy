//! The property test generator: an old file and an edit history that gives the new file.
//!
//! The generator records each old line that survives, unedited, and its new line. Each line
//! that an edit creates has a text that no old line has. So the lines that survive are a longest
//! common subsequence, and the LCS rule can tell each survivor from an ambiguous one.

pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed)
    }
    /// splitmix64.
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
    /// A number in lo..=hi.
    pub fn range(&mut self, lo: usize, hi: usize) -> usize {
        lo + (self.next() % (hi - lo + 1) as u64) as usize
    }
    pub fn chance(&mut self, percent: u64) -> bool {
        self.next() % 100 < percent
    }
}

pub struct Profile {
    pub name: &'static str,
    pub max_lines: usize,
    pub max_edits: usize,
}

pub const PROFILES: [Profile; 2] = [
    Profile { name: "small", max_lines: 40, max_edits: 6 },
    Profile { name: "medium", max_lines: 200, max_edits: 20 },
];

/// Lines that real code repeats.
const COMMON: [&str; 7] = ["}\n", "\n", "    }\n", "        }\n", "    return;\n", "end\n", "        break;\n"];

pub struct History {
    pub old: Vec<Vec<u8>>,
    pub new: Vec<Vec<u8>>,
    /// For each old line: its new line when the line survives unedited.
    pub survivor: Vec<Option<usize>>,
    pub edits: Vec<String>,
}

struct Gen {
    r: Rng,
    fresh: usize,
    old_texts: std::collections::HashSet<Vec<u8>>,
}

impl Gen {
    fn unique(&mut self) -> Vec<u8> {
        self.fresh += 1;
        let indent = ["", "    ", "        "][self.r.range(0, 2)];
        format!("{indent}let v{} = f({});\n", self.fresh, self.r.range(0, 9)).into_bytes()
    }
    fn common(&mut self) -> Vec<u8> {
        COMMON[self.r.range(0, COMMON.len() - 1)].as_bytes().to_vec()
    }
    /// A text that no old line has: a variant of `base` or a new unique line.
    fn new_text(&mut self, base: &[u8]) -> Vec<u8> {
        let lf = base.ends_with(b"\n") && !base.ends_with(b"\r\n");
        let candidate = match self.r.range(0, 5) {
            0 if base.ends_with(b"\n") => [b"    ", base].concat(),
            1 if lf => [&base[..base.len() - 1], b" \n"].concat(),
            2 if lf => [&base[..base.len() - 1], b"\r\n"].concat(),
            3 => COMMON[self.r.range(0, COMMON.len() - 1)].replace('\n', "\t\n").into_bytes(),
            _ => Vec::new(),
        };
        self.checked(candidate)
    }
    /// `t` when no old line has it, else a new unique line.
    fn checked(&mut self, t: Vec<u8>) -> Vec<u8> {
        if !t.is_empty() && !self.old_texts.contains(&t) {
            return t;
        }
        loop {
            let u = self.unique();
            if !self.old_texts.contains(&u) {
                return u;
            }
        }
    }
}

pub fn generate(seed: u64, p: &Profile) -> History {
    let mut g = Gen { r: Rng::new(seed), fresh: 0, old_texts: Default::default() };

    // The old file: unique lines, common lines and repeats of a few blocks.
    let target = g.r.range(0, p.max_lines);
    let mut bank: Vec<Vec<Vec<u8>>> = Vec::new();
    let mut old: Vec<Vec<u8>> = Vec::new();
    while old.len() < target {
        match g.r.range(0, 9) {
            0..=3 => old.push(g.unique()),
            4..=5 => old.push(g.common()),
            _ => {
                if bank.is_empty() || (bank.len() < 3 && g.r.chance(30)) {
                    let len = g.r.range(1, 5);
                    let block = (0..len).map(|_| if g.r.chance(50) { g.unique() } else { g.common() }).collect();
                    bank.push(block);
                }
                let b = g.r.range(0, bank.len() - 1);
                old.extend(bank[b].iter().cloned());
            }
        }
    }
    old.truncate(target);
    if old.last().is_some_and(|l| l.len() > 1) && g.r.chance(10) {
        old.last_mut().unwrap().pop();
    }
    g.old_texts = old.iter().cloned().collect();

    // The edit history. Each line is (text, old position when it survives unedited).
    let mut cur: Vec<(Vec<u8>, Option<usize>)> =
        old.iter().cloned().enumerate().map(|(i, t)| (t, Some(i))).collect();
    let mut edits = Vec::new();
    let steps = g.r.range(0, p.max_edits);
    for _ in 0..steps {
        let len = cur.len();
        match g.r.range(0, 9) {
            0..=2 => {
                let at = g.r.range(0, len);
                let count = g.r.range(1, 4);
                let lines: Vec<_> = (0..count)
                    .map(|_| {
                        let base = if g.r.chance(50) { g.common() } else { g.unique() };
                        (g.new_text(&base), None)
                    })
                    .collect();
                edits.push(format!("insert {count} at {}", at + 1));
                cur.splice(at..at, lines);
            }
            3..=5 if len > 0 => {
                let at = g.r.range(0, len - 1);
                let count = g.r.range(1, 5).min(len - at);
                edits.push(format!("delete {count} at {}", at + 1));
                cur.drain(at..at + count);
            }
            6..=7 if len > 0 => {
                let at = g.r.range(0, len - 1);
                let base = cur[at].0.clone();
                edits.push(format!("edit {}", at + 1));
                cur[at] = (g.new_text(&base), None);
            }
            8 if len > 0 && g.r.chance(30) => {
                edits.push("crlf all".into());
                for line in cur.iter_mut() {
                    let t = &line.0;
                    if t.ends_with(b"\n") && !t.ends_with(b"\r\n") {
                        let c = [&t[..t.len() - 1], b"\r\n"].concat();
                        *line = (g.checked(c), None);
                    }
                }
            }
            9 if len > 0 => {
                let last = cur.last().unwrap().0.clone();
                if last.ends_with(b"\n") {
                    edits.push("no newline at end".into());
                    let t = g.checked(last[..last.len() - 1].to_vec());
                    *cur.last_mut().unwrap() = (t, None);
                }
            }
            _ => {}
        }
        // Only the last line can lack a terminator. A line that is no longer last gets one.
        let n = cur.len();
        for k in 0..n.saturating_sub(1) {
            if !cur[k].0.ends_with(b"\n") {
                let t = [cur[k].0.as_slice(), b"\n"].concat();
                cur[k] = (g.checked(t), None);
            }
        }
    }
    let mut survivor = vec![None; old.len()];
    for (j, (_, o)) in cur.iter().enumerate() {
        if let Some(i) = o {
            survivor[*i] = Some(j);
        }
    }
    let new = cur.into_iter().map(|(t, _)| t).collect();
    History { old, new, survivor, edits }
}
