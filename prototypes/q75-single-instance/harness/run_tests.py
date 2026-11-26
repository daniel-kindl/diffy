"""Q75 single instance tests. Throwaway code. Never merge.

Usage: python -I run_tests.py <app> <work folder> <result json> [test ...]

<app> is the release binary, or the AppImage on Linux.
"""

import json
import os
import platform
import shutil
import subprocess
import sys
import time

APP, WORK, OUT = (os.path.abspath(a) for a in sys.argv[1:4])
ONLY = sys.argv[4:]
WIN = os.name == "nt"
RESULTS = {"platform": platform.platform(), "app": os.path.basename(APP), "tests": {}}


def read_log(path):
    if not os.path.exists(path):
        return []
    with open(path, encoding="utf-8") as f:
        out = []
        for line in f:
            try:
                out.append(json.loads(line))
            except ValueError:
                pass  # a line that is still being written
        return out


def wait_for(path, pred, timeout):
    end = time.time() + timeout
    while time.time() < end:
        events = read_log(path)
        if pred(events):
            return events
        time.sleep(0.05)
    return read_log(path)


def fresh(name):
    d = os.path.join(WORK, name)
    shutil.rmtree(d, ignore_errors=True)
    os.makedirs(d)
    return d


def env_for(log, **extra):
    env = dict(os.environ)
    env["Q75_LOG"] = log
    for k, v in extra.items():
        env[k] = str(v)
    return env


def start_first(log, cwd, **extra):
    p = subprocess.Popen([APP, "first"], cwd=cwd, env=env_for(log, **extra),
                         stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    events = wait_for(log, lambda e: any(x["event"] == "ready" for x in e), 60)
    ready = [x for x in events if x["event"] == "ready"]
    return p, (ready[0] if ready else None)


def second(log, args, cwd, timeout=600, **extra):
    t = time.time()
    try:
        r = subprocess.run([APP] + args, cwd=cwd, env=env_for(log, **extra),
                           capture_output=True, text=True, timeout=timeout)
        code, err = r.returncode, r.stderr[-2000:]
    except subprocess.TimeoutExpired:
        code, err = "timeout", ""
    return {"exit": code, "wall_ms": round((time.time() - t) * 1000), "stderr": err}


def stop(p, log, cwd):
    if p.poll() is None:
        second(log, ["--q75-exit"], cwd, timeout=30)
        try:
            p.wait(10)
        except subprocess.TimeoutExpired:
            p.kill()
            p.wait()


def callbacks(log):
    return [x for x in read_log(log) if x["event"] == "callback"]


def make_file(path, size):
    with open(path, "wb") as f:
        left = size
        while left:
            n = min(left, 1 << 20)
            f.write(os.urandom(n))
            left -= n


def t_thread():
    d = fresh("thread")
    log = os.path.join(d, "log.jsonl")
    p, ready = start_first(log, d)
    s = second(log, ["x"], d)
    cb = callbacks(log)
    stop(p, log, d)
    return {"second": s, "callback": cb[0] if cb else None,
            "pass": bool(cb) and s["exit"] == 0}


def t_timeout(sleep_ms):
    d = fresh(f"timeout-{sleep_ms}")
    log = os.path.join(d, "log.jsonl")
    p, ready = start_first(log, d, Q75_SLEEP_MS=sleep_ms)
    s = second(log, ["x"], d, timeout=sleep_ms / 1000 + 120)
    cb = callbacks(log)
    stop(p, log, d)
    return {"sleep_ms": sleep_ms, "second": s,
            "callback_ms": round(cb[0]["t_end"] - cb[0]["t"]) if cb else None,
            "callback_done": bool(cb)}


def t_cwd(fix=False):
    d = fresh("cwd-fix" if fix else "cwd")
    extra = {"Q75_FIX_OWD": 1} if fix else {}
    a, b = os.path.join(d, "a"), os.path.join(d, "b")
    os.makedirs(a)
    os.makedirs(b)
    with open(os.path.join(b, "input.txt"), "w") as f:
        f.write("hello\n")
    log = os.path.join(d, "log.jsonl")
    p, ready = start_first(log, a, **extra)
    s = second(log, ["input.txt"], b, **extra)
    cb = callbacks(log)
    starts = [x for x in read_log(log) if x["event"] == "start"]
    stop(p, log, a)
    found = bool(cb) and cb[0]["files"] and cb[0]["files"][0].get("file")
    return {"first_cwd_expected": a, "second_cwd_expected": b, "starts": starts,
            "callback_cwd": cb[0]["cwd"] if cb else None, "relative_file_found": found,
            "second": s, "pass": bool(cb) and cb[0]["cwd"] == b and bool(found)}


def t_race(start_ms, delays, setup_ms=0):
    runs = []
    for delay in delays:
        d = fresh(f"race-{start_ms}-{setup_ms}-{delay}")
        log = os.path.join(d, "log.jsonl")
        env = env_for(log, Q75_START_MS=start_ms, Q75_SETUP_MS=setup_ms)
        t0 = time.time()
        p = subprocess.Popen([APP, "first"], cwd=d, env=env,
                             stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        time.sleep(delay / 1000)
        s2 = subprocess.Popen([APP, "race-arg"], cwd=d, env=env_for(log),
                              stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        t2 = time.time()
        wait_for(log, lambda e: any(x["event"] == "ready" for x in e), 60)
        wait_for(log, lambda e: any(x["event"] == "callback" for x in e), 20)
        try:
            s2.wait(15)
        except subprocess.TimeoutExpired:
            pass
        s = {"exit": s2.poll(), "wall_ms": round((time.time() - t2) * 1000)}
        time.sleep(1)
        events = read_log(log)
        cb = [x for x in events if x["event"] == "callback"]
        readies = [x for x in events if x["event"] == "ready"]
        primary = ["first" if r["pid"] == p.pid else "second" if r["pid"] == s2.pid else r["pid"]
                   for r in readies]
        for _ in readies:
            second(log, ["--q75-exit"], d, timeout=10)
        for proc in (p, s2):
            try:
                proc.wait(10)
            except subprocess.TimeoutExpired:
                proc.kill()
        runs.append({"delay_ms": delay, "second": s, "forwarded": bool(cb), "primary": primary,
                     "forwarded_args": [c["argv"][1:] for c in cb],
                     "instances_ready": len(readies),
                     "first_ready_ms": round(readies[0]["t"] - t0 * 1000) if readies else None})
    return {"start_ms": start_ms, "setup_ms": setup_ms, "runs": runs}


def t_copy(size, reps):
    d = fresh("copy")
    log = os.path.join(d, "log.jsonl")
    src = os.path.join(d, "input.bin")
    make_file(src, size)
    p, ready = start_first(log, d)
    seconds = [second(log, [src], d) for _ in range(reps)]
    cb = callbacks(log)
    stop(p, log, d)
    copy = sorted(c["files"][0]["copy_ms"] for c in cb)
    return {"size": size, "copy_ms": [round(x, 1) for x in copy],
            "second_wall_ms": [s["wall_ms"] for s in seconds],
            "exits": [s["exit"] for s in seconds], "pass": len(cb) == reps}


def t_difftool(hold, fix=False):
    d = fresh(f"difftool-{hold}" + ("-fix" if fix else ""))
    extra = {"Q75_FIX_OWD": 1} if fix else {}
    repo = os.path.join(d, "repo")
    os.makedirs(repo)
    g = lambda *a: subprocess.run(["git", "-C", repo] + list(a), check=True,
                                  capture_output=True, text=True)
    g("init", "-q")
    g("config", "user.email", "q75@example.invalid")
    g("config", "user.name", "q75")
    with open(os.path.join(repo, "f.txt"), "w") as f:
        f.write("one\n")
    g("add", "f.txt")
    g("commit", "-q", "-m", "one")
    with open(os.path.join(repo, "f.txt"), "w") as f:
        f.write("two\n")
    log = os.path.join(d, "log.jsonl")
    p, ready = start_first(log, d, Q75_HOLD=hold, **extra)
    tool = APP.replace("\\", "/")
    t = time.time()
    try:
        r = subprocess.run(["git", "-C", repo, "difftool", "-y", "-x", f'"{tool}"', "HEAD"],
                           env=env_for(log, Q75_HOLD=hold, **extra), capture_output=True, text=True,
                           stdin=subprocess.DEVNULL, timeout=180)
        git = {"exit": r.returncode, "stderr": r.stderr[-2000:], "stdout": r.stdout[-500:]}
    except subprocess.TimeoutExpired:
        git = {"exit": "timeout"}
    git["wall_ms"] = round((time.time() - t) * 1000)
    cb = callbacks(log)
    held_paths = [os.path.join(cb[0]["cwd"], f["arg"]) if not os.path.isabs(f["arg"]) else f["arg"]
                  for f in (cb[0]["files"] if cb else []) if f.get("file")]
    time.sleep(2)
    exists = {h: os.path.exists(h) for h in held_paths}
    second(log, ["--q75-read-held"], d, timeout=30)
    held = [x for x in read_log(log) if x["event"] == "held"]
    stop(p, log, d)
    return {"hold": hold, "git": git, "callback": cb[0] if cb else None,
            "exists_after_git": exists, "held_read": held[0]["read"] if held else None}


TESTS = {
    "thread": t_thread,
    "timeout_30s": lambda: t_timeout(30_000),
    "timeout_130s": lambda: t_timeout(130_000),
    "timeout_12s": lambda: t_timeout(12_000),
    "cwd": t_cwd,
    "cwd_owd_fix": lambda: t_cwd(True),
    "difftool_share_owd_fix": lambda: t_difftool("share", True),
    "race_natural": lambda: t_race(0, [0, 50, 100, 200, 400]),
    "race_slow_start": lambda: t_race(2000, [0, 500, 1000]),
    "race_slow_setup_3s": lambda: t_race(0, [500], 3000),
    "race_slow_setup_12s": lambda: t_race(0, [500], 12000),
    "copy_50mb": lambda: t_copy(50 * 1024 * 1024, 5),
    "difftool_share": lambda: t_difftool("share"),
    "difftool_noshare": lambda: t_difftool("noshare"),
}

os.makedirs(WORK, exist_ok=True)
for name, fn in TESTS.items():
    if ONLY and name not in ONLY:
        continue
    print("running", name, flush=True)
    try:
        RESULTS["tests"][name] = fn()
    except Exception as e:  # keep going, record the error
        RESULTS["tests"][name] = {"error": repr(e)}
    print(json.dumps(RESULTS["tests"][name])[:600], flush=True)
    with open(OUT, "w", encoding="utf-8") as f:
        json.dump(RESULTS, f, indent=2)
