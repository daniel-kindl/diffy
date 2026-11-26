# Q55: S6 signed update prototype

Throwaway code for map question Q55. Never merge this branch.

## What it is

- `src-tauri/`: a minimal Tauri 2.12.7 app with tauri-plugin-updater 2.14.1. It has no window. At startup it runs the test database migrations, then it checks one update channel and installs the update.
  - The plugin config has `requireSignedVersion: true` (Q73). It also has `dangerousInsecureTransportProtocol: true`, only because the test server is plain HTTP on 127.0.0.1.
  - Rust selects the endpoint of the channel at run time with `updater_builder().endpoints()` (Q50, Q72).
  - Before each migration, the app writes a copy with `VACUUM INTO`. Each migration runs in one transaction. The test config can make migration 2 fail after its statements ran.
  - The test config can make the app wait before the install, so the harness adds a fault after the app started.
- `harness/server.py`: the test update server. `/f/` serves a file, and `/cut/` sends half the body and closes the connection.
- `harness/run_tests.py`: the 12 tests, on Linux (AppImage) and Windows (NSIS).
- `.github/workflows/q55-windows.yml`: builds the NSIS releases on `windows-latest` and runs the tests, in three jobs.

The test releases are 0.1.0 (schema 1), 0.2.0 (schema 2) and 0.3.0-pre.1 (schema 2). A throwaway key signs them. Linux uses a key made in the scratchpad, and each Windows job makes its own key. No key is in this branch. The Q9 release key is not used.

## Hosts

- Linux: a Fedora 44 distrobox on the Bazzite reference host, WebKitGTK 2.54.2. The AppImage is in a home folder, and `/tmp` is a tmpfs on another device.
- Windows: the GitHub `windows-latest` runner, Windows Server 2025, NSIS 3.13, current user install mode, updater install mode `passive`.

## Results

| Test | Linux | Windows |
| --- | --- | --- |
| update_stable: 0.1.0 updates to 0.2.0, restarts, migrates 1 to 2 | pass | pass |
| channel_pre: the pre-release channel gets 0.3.0-pre.1, the stable channel gets 0.2.0 | pass | pass |
| wrong_key: a signature from another key is rejected | pass | pass |
| changed_artifact: one changed byte after signing is rejected | pass | pass |
| version_mismatch: manifest 0.4.0 with a 0.2.0 signature is rejected | pass | pass |
| no_signed_version: a signature with no version is rejected | pass | pass |
| interrupted_download: half the body, then a closed connection | pass | pass |
| bad_payload: a signed file that is not an AppImage or an installer | pass | pass |
| no_network: a refused port, and a port that never answers (5 s timeout) | pass | pass |
| not_writable: the install folder is made read-only during the wait | pass | pass, with the NSIS dialog |
| failed_install: Linux write fails (file size limit), Windows `uninstall.exe` locked | pass | fail |
| failed_migration: migration 2 fails after the update | pass | pass |

Each rejected update leaves the installed app unchanged, and the next check with a good manifest updates it.

Details:

- Linux not_writable: the move of the AppImage to the backup fails with `Permission denied`, and nothing changes.
- Linux failed_install: the write of the new AppImage fails with `File too large`. The updater moves the old AppImage back, and its hash is unchanged.
- Windows not_writable: the app starts the installer and exits. The installer cannot write `q55-updater.exe` and waits at a modal dialog: "Error opening file for writing", with Abort, Retry and Ignore. See `results/q55-not_writable.png`. After Abort, it shows "Installation Aborted" and waits for the user to close it (`results/q55-not_writable_after_abort.png`). The app does not restart. The old app is unchanged and starts.
- Windows failed_install: with `uninstall.exe` locked, the installer replaces `q55-updater.exe`, skips the uninstaller without an error and restarts the app on 0.2.0. The result is a new app with the old uninstaller.
- failed_migration: SQLite rolls back the transaction. The database stays at user version 1 with the user row, and the `VACUUM INTO` copy is next to it. The next start without the fault migrates to 2.
- An interrupted download gives `error decoding response body` and writes no file.

## Run

Linux, from `harness/`, after a `tauri build --bundles appimage` of each version:

    python3 -I run_tests.py <release folder> <work folder> <result json> [test ...]

The release folder has the AppImages, their `.sig` files, `test.key`, `wrong.key` and `tauri.txt` with the CLI path.
