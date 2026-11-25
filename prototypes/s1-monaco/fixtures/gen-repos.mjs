// Throwaway Q46 fixture repositories. Never merged.
// Usage: node fixtures/gen-repos.mjs <out dir>
//
// Each repository is bare and has two branches: `base` and `head`. The comparison is base to
// head, a tree to tree diff. git fast-import writes each repository from a stream, with fixed
// names and dates, so the object IDs are the same on each run.
import { spawnSync } from "node:child_process";
import { closeSync, mkdirSync, openSync, readFileSync, rmSync, writeFileSync, writeSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const out = process.argv[2];
if (!out) throw new Error("usage: gen-repos.mjs <out dir>");

// splitmix32-like PRNG, deterministic.
function rng(seed) {
  let s = seed >>> 0;
  return () => {
    s = (s + 0x9e3779b9) >>> 0;
    let z = s;
    z = Math.imul(z ^ (z >>> 16), 0x85ebca6b) >>> 0;
    z = Math.imul(z ^ (z >>> 13), 0xc2b2ae35) >>> 0;
    return (z ^ (z >>> 16)) >>> 0;
  };
}

function fileText(id, lines) {
  const r = rng(id * 7919 + 1);
  const rows = [];
  for (let k = 0; k < lines; k++) {
    rows.push(`    let v${k} = compute_${id}_${k}(${r() % 1000}, ${r() % 1000}); // ${r().toString(16)}`);
  }
  return rows;
}

function changed(rows, id) {
  const r = rng(id * 104729 + 3);
  const copy = [...rows];
  for (let n = 0; n < 3; n++) {
    const at = r() % copy.length;
    copy[at] = `    let changed_${n} = edited_${id}(${r() % 1000});`;
  }
  const at = r() % copy.length;
  copy.splice(at, 0, `    // inserted ${id} a`, `    // inserted ${id} b`);
  return copy;
}

const enc = (rows) => rows.join("\n") + "\n";

// Writes the fast-import stream to a file, one blob at a time, and returns the base content bytes.
function writeStream(spec, file) {
  const fd = openSync(file, "w");
  let mark = 0, bytes = 0;
  const put = (s) => writeSync(fd, s);
  const blob = (text) => {
    mark += 1;
    const data = Buffer.from(text);
    put(`blob\nmark :${mark}\ndata ${data.length}\n`); put(data); put("\n");
    return mark;
  };
  const commit = (ref, msg, time, from, ops) => {
    put(`commit ${ref}\ncommitter Q46 Fixture <q46@example.invalid> ${time} +0000\ndata ${msg.length}\n${msg}\n` +
      (from ? `from ${from}\n` : "") + ops.join("") + "\n");
  };
  const baseOps = [];
  let files = 0;
  for (const [path, text] of spec.base()) {
    bytes += Buffer.byteLength(text);
    files += 1;
    baseOps.push(`M 100644 :${blob(text)} ${path}\n`);
  }
  commit("refs/heads/base", "base", 1700000000, null, baseOps);
  const headOps = [];
  for (const op of spec.head()) {
    if (op[0] === "D") headOps.push(`D ${op[1]}\n`);
    else headOps.push(`M 100644 :${blob(op[2])} ${op[1]}\n`);
  }
  commit("refs/heads/head", "head", 1700000100, "refs/heads/base", headOps);
  closeSync(fd);
  return { bytes, files };
}

// total: files in the base tree. counts: { M, A, D, R } changed files. lines: lines per file.
// The text of each file comes from its number, so nothing is kept in memory.
function manyFiles(total, counts, lines) {
  const dirs = 40;
  const path = (i) => `src/d${String(i % dirs).padStart(2, "0")}/file_${String(i).padStart(5, "0")}.rs`;
  return {
    changed: counts.M + counts.D + counts.R + counts.A,
    *base() {
      for (let i = 0; i < total; i++) yield [path(i), enc(fileText(i, lines))];
    },
    *head() {
      let i = 0;
      for (let n = 0; n < counts.M; n++, i++) yield ["M", path(i), enc(changed(fileText(i, lines), i))];
      for (let n = 0; n < counts.D; n++, i++) yield ["D", path(i)];
      for (let n = 0; n < counts.R; n++, i++) {
        yield ["D", path(i)];
        yield ["M", path(i).replace("file_", "moved_"), enc(changed(fileText(i, lines), i))];
      }
      for (let n = 0; n < counts.A; n++) {
        const id = total + n;
        yield ["M", `src/new/added_${String(id).padStart(5, "0")}.rs`, enc(fileText(id, lines))];
      }
    },
  };
}

function onePair(name) {
  const dir = process.env.DIFFY_S1_FIXTURES ?? join(here, "out");
  const left = readFileSync(join(dir, `${name}.left.txt`), "utf8");
  const right = readFileSync(join(dir, `${name}.right.txt`), "utf8");
  const base = [[`data/${name}.txt`, left]];
  for (let i = 0; i < 10; i++) base.push([`src/other_${i}.rs`, enc(fileText(i, 60))]);
  return { base: () => base, head: () => [["M", `data/${name}.txt`, right]], changed: 1 };
}

const FIXTURES = {
  "files-1000": () => manyFiles(2000, { M: 900, A: 40, D: 40, R: 20 }, 60),
  "files-1000-large": () => manyFiles(2000, { M: 900, A: 40, D: 40, R: 20 }, 3000),
  "files-2000-large": () => manyFiles(4000, { M: 1800, A: 80, D: 80, R: 40 }, 3000),
  "files-5000": () => manyFiles(10000, { M: 4500, A: 200, D: 200, R: 100 }, 60),
  "lines-10k": () => onePair("10k-lines"),
  "stress-50mb": () => onePair("50mb-stress"),
};

const env = { ...process.env, GIT_CONFIG_NOSYSTEM: "1", GIT_CONFIG_GLOBAL: "/dev/null", LC_ALL: "C" };
const git = (args, input) => {
  const r = spawnSync("git", args, { stdio: [input ?? "ignore", "pipe", "pipe"], env, maxBuffer: 1 << 30 });
  if (r.status !== 0) throw new Error(`git ${args.join(" ")}: ${r.stderr}`);
  return r.stdout.toString().trim();
};

mkdirSync(out, { recursive: true });
const manifest = [];
for (const [name, make] of Object.entries(FIXTURES)) {
  const spec = make();
  const repo = join(out, `${name}.git`);
  rmSync(repo, { recursive: true, force: true });
  git(["init", "-q", "--bare", repo]);
  const streamFile = join(out, `${name}.fast-import`);
  const { bytes, files } = writeStream(spec, streamFile);
  const fd = openSync(streamFile, "r");
  git(["-C", repo, "fast-import", "--quiet"], fd);
  closeSync(fd);
  rmSync(streamFile);
  manifest.push({
    name,
    repo: `${name}.git`,
    base: git(["-C", repo, "rev-parse", "base"]),
    head: git(["-C", repo, "rev-parse", "head"]),
    base_files: files,
    base_bytes: bytes,
    changed_files: spec.changed,
    shortstat: git(["-C", repo, "diff", "--shortstat", "--histogram", "-M", "base", "head"]),
  });
  console.log(name, manifest.at(-1).shortstat);
}
writeFileSync(join(out, "repos.json"), JSON.stringify({ fixtures: manifest }, null, 2) + "\n");
