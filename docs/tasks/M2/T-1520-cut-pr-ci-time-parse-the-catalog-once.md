---
id: T-1520
title: "Cut PR CI time: parse the catalog once in generator tests, run the eval report only on failure or main"
milestone: M2
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
PR CI finishes in about 3 minutes instead of 12, with no gate weakened, so a busy `main` makes PRs
stale less often (the merge queue is unavailable for this repository, decision 0053).

## Context
Owner request (2026-10-14). Job times on #714: python 12 min (pytest 11), eval 4.7 min, rust
2.5 min. `pytest --durations` showed 17 `tests/test_generator.py` tests at 25-85 s each: they
reparse functions.yaml (0.7 s) for every seed. The eval job's informational `make eval` reruns all
suites after the `make eval-check` gate (2.5 min).

## Acceptance criteria
- [x] `tools/tests/test_generator.py` parses the catalog once; every test still gets its own copy.
- [x] CI skips the informational full eval report on a green PR or merge group, and keeps it on a
      failed gate, on `main` pushes and on manual runs. Decision 0054 records it.
- [x] `make check` green; reviewers approve.

## Out of scope
- Speeding up `make eval-check` or the Rust test build.

## Outcome
pytest in tools/: 672 tests in 30 s locally (was 12 min in CI, 16.5 min locally). The eval job
skips the 2.5 min report on green PRs. Expected PR CI: about 3 min, bounded by eval-check and rust.
