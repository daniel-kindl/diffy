# diffy

Diffy is a local-first diff viewer and code review workbench for Windows and Linux. It is for developers who supervise changes from humans and from AI coding agents. It works without Git, an account, internet access or an AI service.

## Status

Diffy is in early development and has no release. The domain code, the Tauri shell and a first renderer exist. Much of the v0.1 plan does not exist yet. Pull request review, the GitHub and OneDev providers, live review and the packages are not built. The plan is in [docs/maps/diffy-v0.1.md](docs/maps/diffy-v0.1.md).

## Install and run

There is nothing to install yet. The first pre-release is a Linux AppImage after milestone M1. See map decision Q11.

To build and run from source on Linux you need:

- A stable Rust toolchain and Node.js 24 (the version that was tested).
- The WebKitGTK 4.1 development packages, for example `libwebkit2gtk-4.1-dev` on Ubuntu.

```sh
npm --prefix apps/desktop/ui ci
npm --prefix apps/desktop/ui run build
cargo run -p diffy-desktop --features tauri/custom-protocol -- <left> <right>
```

Without the `tauri/custom-protocol` feature, a debug build loads the development server instead of the built page.

## Use

The binary is named `diffy-desktop` for now. The commands below call it `diffy`.

- `diffy <left> <right>` compares two files, or two folders.
- `diffy <repo-path>` opens a Git repository and lists its changed files. Select a file to compare it with `HEAD`.
- `diffy --help` and `diffy --version` print the usage and the version.

Diffy only reads. It does not change a file or a repository.

For a side, you can select a legacy encoding. A byte order mark sets UTF-8 or UTF-16. With no mark and no selection, a file is UTF-8, and Diffy shows where invalid bytes start. Diffy never guesses an encoding. The Swap button swaps the sides, and each source stays with its input.

| Key | Action |
|---|---|
| F3, Alt+Down | Next change |
| Shift+F3, Alt+Up | Previous change |
| Ctrl+F or Cmd+F | Find |
| Ctrl+G or Cmd+G | Go to line |
| F6 | Switch the side |

Tab always moves the focus, so there is no keyboard trap.

## Known limitations

- Syntax highlighting is not active. No Monaco grammar is bundled yet.
- `git difftool` passes two paths, so it should work with `diffy <local> <remote>`. This has not been tested.
- A comparison opens only from the command line. There is no file dialog and no paste input yet.
- The window has been built and started on Linux only. The Windows build compiles in CI, and nobody has run it.
- The canonical diff matches `git diff --histogram`. On a large file with many scattered changes this algorithm is slow. See [the benchmarks](docs/benchmarks.md).

## Contribute

- Read [AGENTS.md](AGENTS.md). The project follows the [dkstack](https://github.com/daniel-kindl/dkstack) rules.
- Each change has one issue, one branch and one pull request. The pull request title is a conventional commit with 50 characters or fewer.
- Pull requests merge with squash only, after the `validate` check passes and a reviewer approves.
- Report a vulnerability as [SECURITY.md](SECURITY.md) says.

## License

Diffy is licensed under either of these licenses, at your option:

- [MIT](LICENSE-MIT)
- [Apache-2.0](LICENSE-APACHE)

See [ADR 0001](docs/adr/0001-license-mit-or-apache-2.md).
