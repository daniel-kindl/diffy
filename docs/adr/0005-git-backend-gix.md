# ADR 0005: Diffy reads Git repositories with gix

Status: Accepted
Affects: crates/diffy-git

The owner decided this on 2026-11-25. The Git backend exists, so the Status is Accepted.

## Context

Diffy reads the index, the worktree, refs, commits, trees and blobs, renames and submodules (specification sections 04 and 12). It must never write to a repository, and it must run with no installed Git. It must also treat the repository configuration as untrusted, with no hooks and no shell commands (section 04). The backend is under each local review feature, so a change after M0 is expensive.

Spike S3 (map question Q44) compared gix 0.91.0 and git2 0.21.0 (libgit2 1.9.7) on 7 fixtures with 58 operations. The oracle was Git 2.55.0. The results are on the tag `prototype/diffy-v0.1-q44`.

## Options considered

- gix: same answer as Git for each operation. It wrote nothing, also on a copy with no write permission. The build compiles no C. The Linux x86-64 binary is 5.8 MB. With its default trust, gix status runs a filter driver command from the repository configuration, as Git does.
- git2: same answer as Git for 57 of 58 operations. libgit2 reports an intent-to-add file (`git add -N`) as `AM`, where Git reports `.A`. It wrote nothing and runs no filter driver. The build compiles libgit2 with a C compiler and links the system zlib. The Linux x86-64 binary is 1.6 MB.

## Decision

Diffy reads Git repositories with gix. It opens each repository with `gix::open::Options::isolated()`, so no system, global or environment configuration applies. After it opens the repository, it removes each `filter` section from the configuration in memory, so no filter driver command runs. See [the v0.1 map](../maps/diffy-v0.1.md), decision Q44.

## Consequences

- The Git backend tests compare gix answers with the `git` CLI on the fixtures, with no system or global configuration. Git is necessary only in the test environment.
- Each operation test checks that the fixture repository is unchanged, and that no command from the repository configuration runs.
- A configuration section filter that refuses the repository configuration is not a replacement. gix then probes the file system with a symbolic link in the worktree, which is a write.
- A file with a `filter=` attribute, for example a Git LFS file, compares with no filter. Map decision Q63 says how Diffy shows such files.
