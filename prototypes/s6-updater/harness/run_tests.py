"""Q55 S6 test runner for Linux (AppImage) and Windows (NSIS). Throwaway code.

Usage: python3 -I run_tests.py <release folder> <work folder> <result json> [test ...]

The release folder has the three signed test releases, their .sig files,
wrong.key and test.key (throwaway keys), and the tauri CLI path in tauri.txt.
"""

import ctypes
import hashlib
import json
import os
import shutil
import signal
import socket
import sqlite3
import subprocess
import sys
import time

REL, WORK, RESULT = (os.path.abspath(p) for p in sys.argv[1:4])
ONLY = sys.argv[4:]
WIN = os.name == "nt"
PLATFORM = "windows-x86_64" if WIN else "linux-x86_64"
IDENT = "dev.diffy.q55updater"
PORT = 18555
HANG_PORT = 18556
SERVE = os.path.join(WORK, "serve")
BASE = f"http://127.0.0.1:{PORT}"
TERMINAL = {"no_update", "update_skipped", "check_failed", "install_failed", "migration_failed",
            "no_endpoint", "updater_error"}


def art(version):
    if WIN:
        return os.path.join(REL, f"q55-updater_{version}_x64-setup.exe")
    return os.path.join(REL, f"q55-updater_{version}_amd64.AppImage")


def sha(path):
    try:
        with open(path, "rb") as f:
            return hashlib.sha256(f.read()).hexdigest()
    except OSError as e:
        return f"error: {e}"


def read(path):
    with open(path, encoding="utf-8") as f:
        return f.read()


TAURI = read(os.path.join(REL, "tauri.txt")).strip().split(" ")


def sign(path, key, version=None):
    cmd = TAURI + ["signer", "sign", "-f", os.path.join(REL, key), "-p", ""]
    if version:
        cmd += ["--app-version", version]
    env = dict(os.environ)
    env.pop("TAURI_SIGNING_PRIVATE_KEY", None)
    subprocess.run(cmd + [path], check=True, capture_output=True, env=env,
                   shell=WIN, timeout=120)
    return read(path + ".sig")


def manifest(name, version, file, sig):
    body = {"version": version, "notes": "q55 test", "pub_date": "2026-11-25T00:00:00Z",
            "platforms": {PLATFORM: {"signature": sig, "url": f"{BASE}/{file}"}}}
    with open(os.path.join(SERVE, name), "w", encoding="utf-8") as f:
        json.dump(body, f)


def prepare_serve():
    shutil.rmtree(SERVE, ignore_errors=True)
    os.makedirs(SERVE)
    a = {}
    for v in ("0.2.0", "0.3.0-pre.1"):
        shutil.copy(art(v), os.path.join(SERVE, f"app-{v}"))
        a[v] = read(art(v) + ".sig")
    # A copy of 0.2.0 with one byte changed after signing.
    data = bytearray(open(art("0.2.0"), "rb").read())
    data[len(data) // 2] ^= 0xFF
    open(os.path.join(SERVE, "app-0.2.0-changed"), "wb").write(data)
    # The 0.2.0 artifact signed with a different throwaway key.
    wrong = os.path.join(SERVE, "app-0.2.0-wrongkey")
    shutil.copy(art("0.2.0"), wrong)
    a["wrongkey"] = sign(wrong, "wrong.key", "0.2.0")
    # The 0.2.0 artifact signed with the test key but with no version.
    nover = os.path.join(SERVE, "app-0.2.0-noversion")
    shutil.copy(art("0.2.0"), nover)
    a["noversion"] = sign(nover, "test.key")
    # A payload that is not an AppImage or an installer, with a valid signature.
    bad = os.path.join(SERVE, "bad-payload")
    with open(bad, "wb") as f:
        f.write(b"q55 test payload that is not an executable\n" * 1000)
    a["bad"] = sign(bad, "test.key", "0.2.0")
    return a


def start_servers():
    here = os.path.dirname(os.path.abspath(__file__))
    srv = subprocess.Popen([sys.executable, "-I", os.path.join(here, "server.py"), SERVE, str(PORT)])
    # A port that accepts the connection and never answers: a black hole.
    hang = socket.socket()
    hang.bind(("127.0.0.1", HANG_PORT))
    hang.listen(64)
    for _ in range(50):
        try:
            socket.create_connection(("127.0.0.1", PORT), 0.2).close()
            break
        except OSError:
            time.sleep(0.1)
    return srv, hang


class Env:
    """One test installation: the app, its data folder and its config."""

    def __init__(self, name):
        step(f"{name}: set up")
        self.root = os.path.join(WORK, name)
        shutil.rmtree(self.root, ignore_errors=True)
        os.makedirs(self.root)
        if WIN:
            self.data = os.path.join(os.environ["APPDATA"], IDENT)
            self.install_dir = os.path.join(os.environ["LOCALAPPDATA"], "q55-updater")
            self.exe = os.path.join(self.install_dir, "q55-updater.exe")
            uninstall = os.path.join(self.install_dir, "uninstall.exe")
            if os.path.exists(uninstall):
                kill_installers()
                step(f"{name}: uninstall")
                subprocess.run([uninstall, "/S"], check=False, timeout=120)
                time.sleep(3)
            shutil.rmtree(self.data, ignore_errors=True)
            step(f"{name}: install 0.1.0")
            subprocess.run([art("0.1.0"), "/S"], check=True, timeout=180)
            for _ in range(60):
                if os.path.exists(self.exe):
                    break
                time.sleep(1)
            time.sleep(2)
        else:
            self.install_dir = os.path.join(self.root, "app")
            os.makedirs(self.install_dir)
            self.exe = os.path.join(self.install_dir, "q55.AppImage")
            shutil.copy(art("0.1.0"), self.exe)
            self.data = os.path.join(self.root, "xdg", IDENT)
        os.makedirs(self.data, exist_ok=True)
        self.start_hash = sha(self.exe)

    def config(self, **kw):
        cfg = {"channel": "stable",
               "endpoints": {"stable": f"{BASE}/f/stable.json", "pre": f"{BASE}/f/pre.json"},
               "timeout_s": 20}
        cfg.update(kw)
        with open(os.path.join(self.data, "q55-config.json"), "w", encoding="utf-8") as f:
            json.dump(cfg, f)

    def events(self):
        try:
            return [json.loads(l) for l in read(os.path.join(self.data, "q55-log.jsonl")).splitlines() if l]
        except OSError:
            return []

    def run(self, timeout=240, fsize=None, on_pause=None):
        before = len(self.events())
        env = dict(os.environ)
        if not WIN:
            env["XDG_DATA_HOME"] = os.path.join(self.root, "xdg")

        def limit():
            import resource
            signal.signal(signal.SIGXFSZ, signal.SIG_IGN)
            resource.setrlimit(resource.RLIMIT_FSIZE, (fsize, fsize))

        step(f"{os.path.basename(self.root)}: run app")
        t0 = time.time()
        p = subprocess.Popen([self.exe], env=env, cwd=self.root,
                             preexec_fn=limit if fsize else None)
        code = None
        paused = False
        while time.time() - t0 < timeout:
            if code is None:
                code = p.poll()
            ev = self.events()[before:]
            if on_pause and not paused and any(x["event"] == "paused" for x in ev):
                paused = True
                step("app paused: add the fault")
                on_pause()
            if ev and ev[-1]["event"] in TERMINAL and code is not None:
                time.sleep(2)
                break
            time.sleep(0.5)
        else:
            p.kill()
            step("app run timed out")
            kill_installers()
        return {"exit_code": code, "seconds": round(time.time() - t0, 1),
                "events": self.events()[before:]}

    def db(self):
        out = {}
        for name in sorted(os.listdir(self.data)):
            if name.endswith(".sqlite"):
                c = sqlite3.connect(os.path.join(self.data, name))
                out[name] = {"user_version": c.execute("PRAGMA user_version").fetchone()[0],
                             "rows": [r[0] for r in c.execute("SELECT body FROM notes")]}
                c.close()
        return out

    def state(self):
        h = sha(self.exe)
        return {"hash": h, "unchanged": h == self.start_hash, "db": self.db()}


def last(ev):
    return ev[-1]["event"] if ev else None


def versions(ev):
    return sorted({e["version"] for e in ev if e["event"] == "start"})


def lock_file(path):
    """Opens the file with no sharing, so no other process can write it (Windows)."""
    k = ctypes.windll.kernel32
    k.CreateFileW.restype = ctypes.c_void_p
    h = k.CreateFileW(path, 0x80000000, 0, None, 3, 0, None)
    return h


def unlock_file(h):
    ctypes.windll.kernel32.CloseHandle(ctypes.c_void_p(h))


def step(msg):
    print(f"  {time.strftime('%H:%M:%S')} {msg}", flush=True)


def kill_installers():
    """Stops each leftover installer or app process, and lists what it stopped."""
    if WIN:
        r = subprocess.run(["powershell", "-NoProfile", "-Command",
                            "Get-Process | Where-Object { $_.Name -like 'q55-updater*' -or $_.Name -like 'Au_' -or $_.Name -like 'Un_A' -or $_.Name -eq 'uninstall' } | "
                            "ForEach-Object { \"$($_.Name) $($_.Path)\"; Stop-Process -Force -Id $_.Id }"],
                           check=False, capture_output=True, text=True, timeout=60)
        if r.stdout.strip():
            step("stopped: " + r.stdout.strip().replace("\n", " | "))


def good_manifests(sigs):
    manifest("stable.json", "0.2.0", "f/app-0.2.0", sigs["0.2.0"])
    manifest("pre.json", "0.3.0-pre.1", "f/app-0.3.0-pre.1", sigs["0.3.0-pre.1"])


def expect_updated(r, to):
    ev = r["events"]
    return to in versions(ev) and last(ev) == "no_update"


def t_update_stable(sigs):
    e = Env("update_stable")
    good_manifests(sigs)
    e.config()
    r = e.run()
    s = e.state()
    backup = s["db"].get("data.pre-m2.sqlite", {})
    ok = (expect_updated(r, "0.2.0") and "0.3.0-pre.1" not in versions(r["events"])
          and s["db"]["data.sqlite"]["user_version"] == 2
          and "user data written by v0.1.0" in s["db"]["data.sqlite"]["rows"]
          and backup.get("user_version") == 1 and not s["unchanged"])
    if not WIN:
        ok = ok and s["hash"] == sha(art("0.2.0"))
    return ok, {"run": r, "state": s}


def t_channel_pre(sigs):
    e = Env("channel_pre")
    good_manifests(sigs)
    e.config(channel="pre")
    r = e.run()
    s = e.state()
    ok = expect_updated(r, "0.3.0-pre.1") and "0.2.0" not in versions(r["events"])
    return ok, {"run": r, "state": s}


def rejected(name, sigs, version, file, sig, extra=None):
    e = Env(name)
    good_manifests(sigs)
    manifest("stable.json", version, file, sig)
    e.config()
    r = e.run()
    s = e.state()
    ok = last(r["events"]) == "install_failed" and s["unchanged"] and versions(r["events"]) == ["0.1.0"]
    # Recovery: the next check with a good manifest updates the app.
    good_manifests(sigs)
    r2 = e.run()
    ok = ok and expect_updated(r2, "0.2.0")
    return ok, {"run": r, "state": s, "recovery_run": r2, **(extra or {})}


def t_wrong_key(sigs):
    return rejected("wrong_key", sigs, "0.2.0", "f/app-0.2.0-wrongkey", sigs["wrongkey"])


def t_changed_artifact(sigs):
    return rejected("changed_artifact", sigs, "0.2.0", "f/app-0.2.0-changed", sigs["0.2.0"])


def t_version_mismatch(sigs):
    return rejected("version_mismatch", sigs, "0.4.0", "f/app-0.2.0", sigs["0.2.0"])


def t_no_signed_version(sigs):
    return rejected("no_signed_version", sigs, "0.2.0", "f/app-0.2.0-noversion", sigs["noversion"])


def t_interrupted_download(sigs):
    return rejected("interrupted_download", sigs, "0.2.0", "cut/app-0.2.0", sigs["0.2.0"])


def t_bad_payload(sigs):
    return rejected("bad_payload", sigs, "0.2.0", "f/bad-payload", sigs["bad"])


def t_no_network(sigs):
    out = {}
    ok = True
    for label, url in (("refused", "http://127.0.0.1:9/latest.json"),
                       ("black_hole", f"http://127.0.0.1:{HANG_PORT}/latest.json")):
        e = Env(f"no_network_{label}")
        e.config(endpoints={"stable": url}, timeout_s=5)
        r = e.run()
        s = e.state()
        ok = ok and last(r["events"]) == "check_failed" and s["unchanged"] and r["exit_code"] == 4
        out[label] = {"run": r, "state": s}
    return ok, out


def t_not_writable(sigs):
    """The install folder is made read-only while the app waits to install."""
    e = Env("not_writable")
    good_manifests(sigs)
    e.config(pause_before_install_s=5)
    user = os.environ.get("USERNAME", "")

    def deny():
        if WIN:
            subprocess.run(["icacls", e.install_dir, "/deny", f"{user}:(OI)(CI)(W,D,DC)"],
                           check=True, timeout=60)
        else:
            os.chmod(e.install_dir, 0o555)

    try:
        r = e.run(timeout=150, on_pause=deny)
    finally:
        kill_installers()
        if WIN:
            subprocess.run(["icacls", e.install_dir, "/remove:d", user], check=True, timeout=60)
        else:
            os.chmod(e.install_dir, 0o755)
    s = e.state()
    listing = sorted(os.listdir(e.install_dir))
    # After the permission is back: the app still starts, then updates.
    e.config()
    r2 = e.run()
    s2 = e.state()
    ok = s["unchanged"] and "0.2.0" not in versions(r["events"]) and expect_updated(r2, "0.2.0")
    return ok, {"run": r, "state": s, "install_dir": listing, "recovery_run": r2,
                "recovery_state": s2}


def t_failed_install(sigs):
    """Linux: the new AppImage write fails (file size limit), so the updater
    restores the old file. Windows: uninstall.exe is locked while the app
    waits to install, so the NSIS installer cannot write it."""
    e = Env("failed_install")
    good_manifests(sigs)
    handle = []
    if WIN:
        e.config(pause_before_install_s=5)
        target = os.path.join(e.install_dir, "uninstall.exe")
        before_hashes = {n: sha(os.path.join(e.install_dir, n)) for n in os.listdir(e.install_dir)}
        try:
            r = e.run(timeout=150, on_pause=lambda: handle.append(lock_file(target)))
        finally:
            kill_installers()
            for h in handle:
                unlock_file(h)
        after_hashes = {n: sha(os.path.join(e.install_dir, n)) for n in os.listdir(e.install_dir)}
        changed = sorted(n for n in after_hashes if before_hashes.get(n) != after_hashes[n])
    else:
        e.config()
        r = e.run(fsize=50 * 1024 * 1024)
        changed = []
    s = e.state()
    # The old or partly new app must still start.
    e.config(install=False)
    r_start = e.run()
    e.config()
    r2 = e.run()
    s2 = e.state()
    ok = (s["unchanged"] and "0.2.0" not in versions(r["events"])
          and last(r_start["events"]) == "update_skipped" and expect_updated(r2, "0.2.0"))
    return ok, {"run": r, "state": s, "changed_files": changed,
                "lock_handles": [h for h in handle], "start_after_fail": r_start,
                "recovery_run": r2, "recovery_state": s2}


def t_failed_migration(sigs):
    e = Env("failed_migration")
    good_manifests(sigs)
    e.config(fail_migration=2)
    r = e.run()
    s = e.state()
    main = s["db"]["data.sqlite"]
    ok = ("0.2.0" in versions(r["events"]) and last(r["events"]) == "migration_failed"
          and main["user_version"] == 1 and "user data written by v0.1.0" in main["rows"]
          and s["db"].get("data.pre-m2.sqlite", {}).get("user_version") == 1)
    # Recovery: the next start without the fault runs the migration again.
    e.config()
    r2 = e.run()
    s2 = e.state()
    ok = ok and expect_updated(r2, "0.2.0") and s2["db"]["data.sqlite"]["user_version"] == 2
    return ok, {"run": r, "state": s, "recovery_run": r2, "recovery_state": s2}


TESTS = [t_update_stable, t_channel_pre, t_wrong_key, t_changed_artifact, t_version_mismatch,
         t_no_signed_version, t_interrupted_download, t_bad_payload, t_no_network,
         t_not_writable, t_failed_install, t_failed_migration]


def main():
    os.makedirs(WORK, exist_ok=True)
    sigs = prepare_serve()
    srv, hang = start_servers()
    results = {"platform": PLATFORM, "tests": {}}
    try:
        for t in TESTS:
            name = t.__name__[2:]
            if ONLY and name not in ONLY:
                continue
            access_before = len(open(os.path.join(SERVE, "access.log")).readlines()) \
                if os.path.exists(os.path.join(SERVE, "access.log")) else 0
            try:
                ok, detail = t(sigs)
            except Exception as ex:  # noqa: BLE001
                import traceback
                ok, detail = False, {"exception": repr(ex), "traceback": traceback.format_exc()}
            log = os.path.join(SERVE, "access.log")
            detail["requests"] = [l.split(" ", 1)[1].strip() for l in open(log).readlines()[access_before:]] \
                if os.path.exists(log) else []
            results["tests"][name] = {"pass": ok, **detail}
            print(f"{name}: {'pass' if ok else 'FAIL'}", flush=True)
            with open(RESULT, "w", encoding="utf-8") as f:
                json.dump(results, f, indent=1)
    finally:
        srv.kill()
        hang.close()
    with open(RESULT, "w", encoding="utf-8") as f:
        json.dump(results, f, indent=1)
    sys.exit(0)


if __name__ == "__main__":
    main()
