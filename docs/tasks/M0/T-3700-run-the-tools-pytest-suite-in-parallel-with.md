---
id: T-3700
title: "Run the tools/ pytest suite in parallel with pytest-xdist"
milestone: M0
lane: harness
status: todo
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
`make py-check` runs the tools/ pytest suite across all cores, so the python CI job (and every
agent's local `make check`) stops spending about 2-4 minutes running 1,248 tests one at a time.

## Context
Owner request (2026-11-01, CI-speed investigation). The python job takes p50 306 s, of which `make
py-check` is p50 242 s (10 sampled heavy PR runs of 2026-11-01). Locally (4 cores) ruff takes
0.3 s, mypy 6 s and pytest 117 s, all on one process; the slowest tests are fitlab solver scores
(up to 10 s each) and generator coverage tests (about 4 s each). Once T-3699 shortens the Rust
shards, the python job is the next critical path.

## Acceptance criteria
- [ ] `pytest-xdist` is a dev dependency of tools/ (pyproject and uv.lock), and `make py-check`
      runs pytest with `-n auto` (or an equivalent worker count).
- [ ] The same 1,248 tests pass in parallel and serially; any test that shares state across tests
      (temp files, caches, the release rustsheet-eval binary build) is made safe for parallel
      runs, not skipped.
- [ ] Before/after times for `make py-check` locally and in CI, in the decision record (protected
      path: Makefile; new dependency).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Rewriting slow tests; changing which tests run.
