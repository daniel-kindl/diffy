#!/usr/bin/env node
// Generate the S1 fixture set for Q54, Q47 and Q46. The output is
// deterministic: the same seed gives the same bytes. The files go to
// fixtures/out/, which Git ignores. fixtures/checksums.txt has the expected
// SHA-256 of each file.
// Usage: node fixtures/gen.mjs [--check]

import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const out = join(here, "out");

// mulberry32: a small seeded PRNG.
function rng(seed) {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

const WORDS = ["value", "count", "index", "buffer", "left", "right", "hunk", "line", "anchor", "review",
  "finding", "snapshot", "offset", "range", "token", "state", "result", "error", "path", "entry"];

// One line of Rust-like source. The average length is about 50 bytes.
function sourceLine(r, n) {
  const w = () => WORDS[Math.floor(r() * WORDS.length)];
  const k = Math.floor(r() * 6);
  const num = Math.floor(r() * 100000);
  switch (k) {
    case 0: return `    let ${w()}_${n} = ${w()}.${w()}(${num});`;
    case 1: return `    if ${w()} > ${num} { return Err(Error::${w()}); }`;
    case 2: return `fn ${w()}_${w()}_${n}(${w()}: u32) -> u32 { ${w()} + ${num} }`;
    case 3: return `    // ${w()} ${w()} ${w()} ${num}`;
    case 4: return `    ${w()}.push(${w()}_${num % 97});`;
    default: return "";
  }
}

// Return [left, right] as arrays of lines, with `blocks` change blocks of
// 1 to `maxSize` lines. Each block is a change, an insertion or a deletion.
// `kinds` limits the block kinds.
function edited(seed, lines, blocks, maxSize, kinds = ["change", "insert", "delete"]) {
  const r = rng(seed);
  const left = Array.from({ length: lines }, (_, i) => sourceLine(r, i));
  const right = [];
  // Block start positions: evenly spaced with jitter, so blocks never touch.
  const gap = Math.floor(lines / blocks);
  if (gap < maxSize + 8) throw new Error("too many blocks for the line count");
  const starts = new Set();
  for (let b = 0; b < blocks; b++) starts.add(b * gap + 4 + Math.floor(r() * (gap - maxSize - 8)));
  let i = 0;
  while (i < left.length) {
    if (!starts.has(i)) {
      right.push(left[i]);
      i++;
      continue;
    }
    starts.delete(i); // one block for each start
    const size = 1 + Math.floor(r() * maxSize);
    const kind = kinds[Math.floor(r() * kinds.length)];
    if (kind === "insert") {
      for (let k = 0; k < size; k++) right.push(`    let added_${i}_${k} = ${Math.floor(r() * 1e6)};`);
      continue; // keep left[i] as the next line
    }
    if (kind === "change") {
      for (let k = 0; k < size; k++) right.push(`${left[i + k] ?? ""} // changed ${Math.floor(r() * 1e6)}`);
    }
    i += size; // "delete" drops the lines
  }
  return [left, right];
}

// Lines for a target size in bytes, at about 35.4 bytes for each line on average.
const linesFor = (bytes) => Math.round(bytes / 35.4);

const FIXTURES = [
  { name: "small", note: "A small change: 120 lines with 6 change blocks.", seed: 1, lines: 120, blocks: 6, maxSize: 4 },
  { name: "1mb", note: "A 1 MB diff: each side is about 1 MB.", seed: 2, lines: linesFor(1_000_000), blocks: 250, maxSize: 6 },
  { name: "10mb", note: "A 10 MB diff: each side is about 10 MB.", seed: 3, lines: linesFor(10_000_000), blocks: 2_500, maxSize: 6 },
  { name: "50mb-stress", note: "A 50 MB stress input: each side is about 50 MB.", seed: 4, lines: linesFor(50_000_000), blocks: 10_000, maxSize: 6 },
  // 2,000 change blocks of exactly 5 lines: 10,000 removed and 10,000 added lines.
  { name: "10k-lines", note: "10,000 changed lines: 2,000 blocks that each change 5 lines.", seed: 5, lines: 60_000, blocks: 2_000, maxSize: 5, fixedSize: true, kinds: ["change"] },
];

function build(f) {
  if (!f.fixedSize) return edited(f.seed, f.lines, f.blocks, f.maxSize, f.kinds);
  // Fixed block size: patch the size draw.
  const [left] = edited(f.seed, f.lines, f.blocks, f.maxSize, f.kinds);
  const r = rng(f.seed * 7919);
  const gap = Math.floor(f.lines / f.blocks);
  const right = left.slice();
  for (let b = 0; b < f.blocks; b++) {
    const start = b * gap + 4 + Math.floor(r() * (gap - f.maxSize - 8));
    for (let k = 0; k < f.maxSize; k++) right[start + k] = `${left[start + k]} // changed ${b}_${k}`;
  }
  return [left, right];
}

const sha = (buf) => createHash("sha256").update(buf).digest("hex");

function main(args) {
  const check = args.includes("--check");
  mkdirSync(out, { recursive: true });
  const manifest = { generator: "fixtures/gen.mjs", fixtures: [] };
  const sums = [];
  for (const f of FIXTURES) {
    const [left, right] = build(f);
    const entry = { name: f.name, note: f.note, seed: f.seed };
    for (const [side, lines] of [["left", left], ["right", right]]) {
      const buf = Buffer.from(lines.join("\n") + "\n", "utf8");
      const file = `${f.name}.${side}.txt`;
      writeFileSync(join(out, file), buf);
      entry[side] = { file, bytes: buf.length, lines: lines.length, sha256: sha(buf) };
      sums.push(`${sha(buf)}  ${file}`);
    }
    manifest.fixtures.push(entry);
    console.log(`${f.name}: left ${entry.left.bytes} bytes, ${entry.left.lines} lines; right ${entry.right.bytes} bytes, ${entry.right.lines} lines`);
  }
  writeFileSync(join(out, "manifest.json"), JSON.stringify(manifest, null, 2) + "\n");
  const text = sums.join("\n") + "\n";
  const sumsPath = join(here, "checksums.txt");
  if (check) {
    const expected = readFileSync(sumsPath, "utf8");
    if (expected !== text) {
      console.error("the fixtures do not match fixtures/checksums.txt");
      return 1;
    }
    console.log("the fixtures match fixtures/checksums.txt");
  } else {
    writeFileSync(sumsPath, text);
  }
  return 0;
}

process.exitCode = main(process.argv.slice(2));
