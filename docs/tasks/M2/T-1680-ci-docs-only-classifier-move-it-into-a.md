---
id: T-1680
title: "CI docs-only classifier: move it into a tested script and guard new docs/ readers"
milestone: M2
lane: harness
status: todo
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
- [ ] Classification lives in a `scripts/` file that reads NUL-separated paths on stdin and prints `code=true|false`. `make hygiene` runs its `scripts/test_*.py`, with cases for an excel-semantics file, a mixed docs + crate rename, top-level `CLAUDE.md`, `.github/`, a dotfile and empty input.
- [ ] A hygiene check fails when code under `crates/`, `tools/` or `bindings/` reads a `docs/` or `.claude/` path that is not on the exception list.
- [ ] The three jobs share one condition, for example a single `run_heavy` output.
- [ ] Every fail-safe stays: pushes, workflow_dispatch, a missing base, a failed diff and a failed `changes` job all run everything.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing which paths count as docs-only.
