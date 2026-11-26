# Q75: single instance tests

Throwaway code for map question Q75. Never merge this branch.

## What it is

- `src-tauri/`: a minimal Tauri 2.12.7 app with tauri-plugin-single-instance 2.5.5. It has no window. Each callback copies each input file into a snapshot folder, as Q74 says, and writes one JSON line to a log.
  - Environment switches: a sleep in the callback (`Q75_SLEEP_MS`), a sleep before the plugin setup (`Q75_START_MS`), a sleep in the app setup hook on the main thread (`Q75_SETUP_MS`), and an open handle on each input (`Q75_HOLD`). `Q75_FIX_OWD` sets the working folder to `OWD` before the plugin setup.
- `harness/run_tests.py`: the tests, on Linux and Windows.
- `.github/workflows/q75-windows.yml`: builds the app on `windows-latest` and runs the tests.

## Hosts

- Linux: a Fedora 44 distrobox on the Bazzite reference host, with the dbus-broker session bus. Git 2.55.0. The tests ran on the release binary and on the AppImage.
- Windows: the GitHub `windows-latest` runner, with the release binary. It is not the Windows 11 notebook.

## Results

### Linux

- Thread: the callback runs on the `zbus::Connection executor` thread, not on the main thread (`results/linux-binary.json`).
- Bus reply timeout: none. With a callback of 30 s and of 130 s, the second instance waited for the full callback and exited with 0 (`results/linux-binary-timeout.json`). The plugin sets no zbus method timeout, and zbus has none by default.
- AppImage working folder: the AppImage runtime starts the app in the mount folder, for example `/tmp/.mount_q75-siOaPGMK/usr`. The folder of the caller is in `OWD`. The plugin forwards `current_dir()`, so the callback gets the mount folder. A relative argument is not found. In `git difftool`, the worktree `$REMOTE` is such a relative path (`f.txt`) (`results/linux-appimage-owd.json`, tests `cwd` and `difftool_share`).
  - With `set_current_dir(OWD)` at the start of `main`, before the plugin setup, the callback gets the folder of the caller. The relative file is found (tests `cwd_owd_fix` and `difftool_share_owd_fix`).
  - The plain binary keeps the folder of the caller (`results/linux-binary.json`).
- Second instance during the start:
  - When two processes start at the same time, one gets the bus name, and the other forwards to it. In each run one process forwarded (`results/linux-binary-race.json`, `results/linux-appimage.json`).
  - When the first process is slow before the plugin setup (2 s), a later process gets the name first. The first process then forwards its own arguments to it. No arguments are lost, but the roles swap.
  - When the app setup hook blocks the main thread after the plugin setup (3 s and 12 s), the second instance waits until the setup ends. Then the callback runs, and the second instance exits with 0 (`results/linux-binary-setup.json`).
- Copy of a 50 MB input in the callback: at most 41 ms on the binary and 36 ms on the AppImage. The first copy is the slowest. The second instance takes 83 to 129 ms on the binary and 323 to 366 ms on the AppImage (`results/linux-binary.json`, `results/linux-appimage.json`).
- `git difftool` with an open handle on each input: Git exits with 0 and deletes `$LOCAL`. The open handle still reads the full content (`results/linux-appimage-owd.json`).

### Windows

The runner has Windows Server 2025, Git 2.56.1.windows.1 and rustc 1.100.0 (`results/windows.json`, run 45845184754).

- Thread: the callback runs on the main thread, in the window procedure of the plugin window.
- Timeout: with a callback of 12 s and of 30 s, the second instance exits with 0 after 10 s, while the callback still runs. The caller gets no error. The callback logs only at its end, and the harness stopped the app before that, so `callback_done` is false.
- Working folder: the binary keeps the folder of the caller. A relative argument is found.
- Second instance during the start:
  - When two processes start at the same time, the first process gets the mutex, and the second forwards to it. Each run forwarded.
  - When the first process is slow before the plugin setup (2 s), the roles swap as on Linux. No arguments are lost.
  - When the app setup hook blocks the main thread for 3 s after the plugin setup, the second instance waits about 2.5 s, then the callback runs.
  - When the setup hook blocks the main thread for 12 s, the second instance exits with 0 after its 10 s timeout. The callback never runs. The arguments are lost.
- Copy of a 50 MB input in the callback: 19 to 20 ms. The second instance takes 40 to 42 ms.
- `git difftool` with an open handle on each input:
  - With the default Rust share mode, which has `FILE_SHARE_DELETE`, Git exits with 0 and deletes `$LOCAL`. The open handle still reads the full content.
  - Without `FILE_SHARE_DELETE`, Git exits with 0 and prints `warning: unable to unlink '...': Invalid argument`. `$LOCAL` and its temporary folder stay on the disk. Git does not wait or ask.

## Run

Linux, from `harness/`, after `cargo build --release` or `tauri build --bundles appimage`:

    python3 -I run_tests.py <app> <work folder> <result json> [test ...]

Windows: push this branch. The workflow runs the tests and uploads `q75-windows.json`.
