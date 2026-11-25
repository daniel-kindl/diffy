# Q44 prototype: S3 Git backend

Throwaway code for map question Q44 in `docs/maps/diffy-v0.1.md`. It compares gix and git2 on
the Git fixtures of specification section 19. It is not Diffy code.

Charter question: Which Git backend, gix or git2, reads the fixtures correctly, never writes,
and builds with no installed Git?

## Contents

- `fixtures.sh` builds one repository for each fixture: `index-worktree`, `unusual-names`,
  `renames`, `detached-head`, `submodules` and `binary`. It also builds `untrusted-config`.
  That fixture is not in section 19. It covers the rule of section 04: no hooks, no shell
  commands, and the repository configuration is untrusted.
- `backend-gix` and `backend-git2` each answer one operation as JSON: `head`, `refs`, `status`,
  `index`, `submodules`, `tree <rev>` and `diff <from> <to>`. `model` has the answer types.
- `harness oracle` asks Git each operation, on a copy of each fixture. It uses no system or
  global configuration, the same as Q43, and `git --no-optional-locks`.
- `harness check` runs a backend on a fresh copy of each fixture and compares each answer with
  Git's. It never runs `git`. Before and after each operation, it records each file and
  directory in the copy, `.git` included. The record has the type, the permission bits, the size
  and the modification time. It also has the SHA-256 of the content or of the link target. A directory also gets a hash of its entry
  names, so a lock file that appears and goes away still changes the directory time. With
  `--no-write-permission`, nobody can write to the copy. After each operation the harness also
  lists the marker files that the `untrusted-config` commands leave.
- `no-git.sh` runs a command in a bubblewrap container with no `git` executable: `/usr/bin` has
  each host program except `git*`, and `/usr/libexec/git-core` is empty.
- `controls/git-status-writes.sh` is a negative control. It runs plain `git status`, which
  refreshes the index of a fresh copy, so the read-only check must report a change.
- `run.sh <work-dir>` runs each step and writes `results/`.

## Versions

- gix 0.91.0, features `status`, `revision` and `sha1`, with no default features.
- git2 0.21.0 with `vendored-libgit2` (libgit2-sys 0.18.8+1.9.7), with no default features,
  so no HTTPS and no SSH.
- Git 2.55.0 as the oracle. rustc 1.98.1.
- The `rust-dev` distrobox container (Fedora 44) on the Bazzite 45 host, 2026-11-25.

## How each backend reads a repository

- gix opens with `gix::open::Options::isolated()`, so no system, global or environment
  configuration applies. It then removes each `filter` section from its configuration in memory
  (`Q44_GIX_FILTERS=strip`, the default). See the filter drivers finding.
- git2 sets the system, global, XDG and ProgramData search paths to an empty directory.
- Status: untracked files one by one, staged renames, and submodules with dirty checks, the same
  as `git status --porcelain=v2 --untracked-files=all --ignore-submodules=none`.
- Diff: tree to tree with rename detection at 50 %, the same as `git diff-tree -r -M`.
- Tree and index: each entry with mode and object ID, and the SHA-256 of each blob. That
  compares the bytes that each backend reads with `git cat-file blob`.

## Results

`results/<run>.json` has one record for each operation, with the expected and the actual
answer when they differ. Each run has 7 fixtures and 58 operations.

| Run | Same as Git | Repository unchanged | No configured command |
|-----|-------------|----------------------|-----------------------|
| `gix` | 58 | 58 | 58 |
| `gix-no-write` | 58 | 58 | 58 |
| `git2` | 57 | 58 | 58 |
| `git2-no-write` | 57 | 58 | 58 |
| `gix-filters-trust` | 58 | 58 | 57 |
| `gix-filters-section-filter` | 58 | 51 | 58 |
| `gix-filters-section-filter-no-write` | 57 | 58 | 58 |
| `gix-default-open` | 58 | 58 | 58 |
| `control-git-status` | 0 | 0 | 52 |

`traces.json` lists each program that a traced run started, from `strace -f -e execve,execveat`:

- The build in the no-git container started no `git` and made no attempt to start one.
- Each backend run started only `harness` and the backend. The exception is
  `gix-filters-trust`, which also started `clean.sh`, `touch` and `cat`.

## Findings

1. gix gives the same answer as Git for each operation, on both copies. It changes nothing in the
   repository, runs no configured command and starts no process.
2. git2 differs from Git for one operation. `git add -N ita.txt` is `.A` for Git and `AM` for
   libgit2: a new staged file with a worktree change. Everything else is the same as Git.
   libgit2 has no external filter drivers, so the `untrusted-config` commands do not run.
3. Filter drivers in gix. With its default configuration trust, gix status ran
   `filter.q44.clean`, a command from the repository configuration, once for each `*.txt` file.
   Git also runs it, and the `fsmonitor` command too (`control-git-status`). gix accepts the
   repository configuration of a repository that the user owns.
   - A configuration section filter that refuses the repository configuration stops the command.
     gix then also lacks `core.symlinks`, so it probes the file system: status creates and
     removes a symbolic link `__file_link<n>` in the worktree root. That is a write. With no write
     permission the probe fails, and gix reports the symbolic link that became a file as `.M`,
     where Git reports `.T`.
   - Removing the `filter` sections from the configuration in memory, after opening, leaves no
     driver and writes nothing. That is the default of `backend-gix`.
   - With no driver, a file with a `filter=` attribute compares with no filter. For Git LFS that
     means the raw file against the pointer blob.
4. gix API notes. `Submodule::head_id()` is the commit in the superproject `HEAD` tree, not the
   checked-out commit. The backend opens the submodule repository for that. `Submodule::url()`
   is the URL in `.git/config` after `submodule add`, not the one in `.gitmodules`. The
   comparison leaves the URL out, because Diffy reads no submodule remote.
5. Build:
   - The gix build compiles no C. Each `cc` call in its trace is a link.
   - The git2 build compiles libgit2 with `cc` and links the system `libz.so.1`, found with
     `pkg-config`.
   - Linux x86-64 sizes, stripped: gix 5,820,392 bytes, git2 1,601,376 bytes. With the default gix
     features, the gix binary was 7,429,696 bytes.
   - Windows x64 (`x86_64-pc-windows-gnu`, mingw64-gcc) builds for both: gix 9,844,736 bytes and
     git2 3,320,832 bytes. Each one imports only Windows system DLLs. See `build.json`.

## Not tested

- A run on Windows or macOS, and the `x86_64-pc-windows-msvc` target. Q59 has the Windows
  notebook.
- Conflicts, linked worktrees, sparse checkout, split index, SHA-256 repositories, Git LFS and
  large repositories or timing.
- Git versions other than 2.55.0.
- A fix for the libgit2 intent-to-add difference, for example from the index entry flag.
