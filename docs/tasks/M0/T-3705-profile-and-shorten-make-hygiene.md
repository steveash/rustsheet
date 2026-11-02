---
id: T-3705
title: "Profile and shorten make hygiene"
milestone: M0
lane: harness
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`make hygiene` (the critical path of every docs-only PR, and part of every heavy run) takes
noticeably less than its current 74-107 s, with every ratchet and check still run.

## Context
Owner request (2026-11-01, CI-speed investigation). Docs-only PR runs take p50 2.7 min, bounded by
the hygiene job (116 s in run 41288141328: checkout 23 s with full history, `make hygiene` 72 s,
deny 9 s); in heavy runs `make hygiene` took p50 104 s. It runs nine scripts plus the scripts'
unit tests; nobody has measured which of them dominates.

## Acceptance criteria
- [x] Per-step times of `make hygiene` (locally and in one CI run) recorded in the Outcome.
- [x] The slowest steps are made faster (caching parsed task/decision files, running independent
      checks in parallel, a shallower checkout where full history is not needed) without
      dropping any check; before/after CI times recorded.
- [x] A decision record if the Makefile, scripts/ or .github/ change; `make check` green;
      reviewers approve.

## Out of scope
- Removing or weakening any ratchet.

## Outcome
Profile, 2026-11-01, local, on a machine busy with other jobs:
- The nine ratchet scripts take about 10 s: `check_duplication.py` 6 s, the rest under 1.2 s each.
- `unittest discover` over `scripts/` (28 modules, 464 tests) takes 455 s serial. The merge-train
  modules dominate: `test_merge_train_red` 142 s, `test_merge_train` 115 s, `test_merge_ready` 61 s.
- CI `make hygiene` was p50 104 s.

Change (decision 0241): `make hygiene` runs one `unittest discover -p <file>` per module, one
process per CPU, largest file first. The same 464 tests pass, and `xargs` fails when any module
fails. Locally that took 144 s (the bound is the slowest module), against 455 s serial.

The checkout of full history (about 25 s) stays: governance and `ci_changes` need the merge base.
CI (run 41301094082, PR #2742): `make hygiene` 46 s against a p50 of 104 s, and the hygiene job
88 s against 152 s. The hygiene job bounds docs-only runs, so they drop by about a minute.
