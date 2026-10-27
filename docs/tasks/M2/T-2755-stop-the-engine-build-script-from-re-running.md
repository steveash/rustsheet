---
id: T-2755
title: "Stop the engine build script from re-running on every cargo call"
milestone: M2
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A second `cargo build`, `cargo test` or `cargo clippy` with nothing changed recompiles nothing,
so local `make` targets and any CI job that runs cargo twice stop rebuilding the whole engine.

## Context
Found in T-2761. crates/rustsheet-engine/build.rs (inherited from IronCalc) prints
`cargo:rerun-if-changed=.git/HEAD` and `.git/refs`. Cargo resolves those against the crate
directory, `crates/rustsheet-engine/.git/HEAD` does not exist, and a missing watched file always
counts as changed (`cargo build -v`: "Dirty rustsheet_engine: the file
`crates/rustsheet-engine/.git/HEAD` is missing"). So every cargo call re-runs the script and
recompiles the engine and everything above it: about 2 min for a release build on a 4-core
runner (PR #1842's evidence-floor step: 1:57 of rebuild). The script sets `GIT_VERSION` for
`INFO("release")`, so the fix must keep that value (`git describe --tags --exact-match`, else
`--dirty --always`) and keep working in a worktree (where `.git` is a file) and in a tarball
without git.

## Acceptance criteria
- [x] Two consecutive `cargo build --release -p rustsheet_eval` runs: the second compiles nothing.
- [x] `INFO("release")` still reports the same `GIT_VERSION` after a commit or checkout (it
      re-runs when HEAD moves), in a normal clone and in a `git worktree`.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing what `INFO("release")` returns.

## Outcome
crates/rustsheet-engine/build.rs asks git for its real directories (`git rev-parse
--path-format=absolute --git-dir --git-common-dir`) and watches only paths that exist: this
worktree's HEAD, the branch's loose ref (or `refs/heads` while the branch is packed),
`packed-refs` and `refs/tags`, plus `build.rs`. Without git it watches only `build.rs`.
Measured locally: a second `cargo build --release -p rustsheet_eval` with nothing changed
compiles nothing (0.1 s; before, it re-ran the script and rebuilt the engine and everything
above it, about 2 min on a CI runner). A commit re-runs the script and `GIT_VERSION` follows
`git describe` (`d5c06299d`). In a `git worktree` (detached HEAD, `.git` a file) it watches
`.git/worktrees/<name>/HEAD` and a second build compiles nothing.
