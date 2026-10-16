---
id: T-1680
title: "CI docs-only classifier: move it into a tested script and guard new docs/ readers"
milestone: M2
lane: harness
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The CI step that decides whether a PR is docs-only (decision 0049) is covered by tests. A new build-time or test-time reader of `docs/` or `.claude/` cannot slip past it unnoticed.

## Context
Found by the 2026-10-16 steward spot-audit of PR #606 (T-1415). The opus security, architecture and quality reviewers each flagged it as MINOR.
- The classifier is an inline `case` statement in `.github/workflows/ci.yml` (the `changes` job). It has no test, and a wrong skip looks exactly like a pass.
- It also relies on convention: decision 0049 says new readers of `docs/` must add an exception, and nothing checks that.
- The long `if:` condition is copied into three jobs.
This is a protected path, so the change needs a decision record (or an amendment to 0049) and explicit approval from reviewer-architecture and reviewer-security.

## Acceptance criteria
- [x] Classification lives in a `scripts/` file that reads NUL-separated paths on stdin and prints `code=true|false`. `make hygiene` runs its `scripts/test_*.py`, with cases for an excel-semantics file, a mixed docs + crate rename, top-level `CLAUDE.md`, `.github/`, a dotfile and empty input.
- [x] A hygiene check fails when code under `crates/`, `tools/` or `bindings/` reads a `docs/` or `.claude/` path that is not on the exception list.
- [x] The three jobs share one condition, for example a single `run_heavy` output.
- [x] Every fail-safe stays: pushes, workflow_dispatch, a missing base, a failed diff and a failed `changes` job all run everything.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing which paths count as docs-only.

## Outcome
- `scripts/ci_changes.py` holds the docs-only rules (unchanged), reads NUL-separated paths and
  prints `code=true|false`. CI runs the copy from the PR's base commit (decision 0066, amends
  0049). `scripts/test_ci_changes.py` (12 tests, run by `make hygiene`) covers excel-semantics,
  a docs + crate rename, top-level `CLAUDE.md`, `.github/`, dotfiles, nested Markdown, empty and
  undecodable input, and newlines in file names.
- `scripts/check_docs_readers.py` (in `make hygiene`) fails when a string literal in crates/,
  tools/, bindings/ or fuzz/ names a `docs/` or `.claude/` path outside `CODE_READ_PATHS`.
  `CODE_READ_PATHS` is the one exception list, and the classifier and the check share it.
- The heavy jobs share `needs.changes.outputs.run_heavy != 'false'`. Every fail-safe stays.
- Eval delta: none (no engine change). Reviewer MINORs: the literal scan is a heuristic
  (documented in decision 0066); no follow-up task.
