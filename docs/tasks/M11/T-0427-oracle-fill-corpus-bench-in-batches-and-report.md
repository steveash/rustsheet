---
id: T-0427
title: "Oracle-fill corpus-bench in batches and report the formula-cell match rate against the published leaderboard"
milestone: M11
lane: harness
status: in-progress
priority: P1
size: M
tier: opus
depends_on: [T-0426]
---

## Goal
corpus-bench has Excel's values from excel-graph, and the report gives RustSheet's formula-cell
match rate next to xlsx-corpus-bench's published numbers: LibreOffice 96.7–98.9%, witan
99.7–99.8%.

## Context
Owner request (2026-10-15): finish Phase 1's test suite before the Phase 2 re-architecture
(decision 0027). Building test infrastructure is allowed during Phase 1, and this is one of the
suites the milestone exit criteria name that does not exist yet.
- Depends on T-0426.
- Fill with `rustsheet-oracle fill -b excel-graph --suite corpus-bench` in batches, with the
  canary embedded as always.
- Workbooks the canary refuses (SHEETS/INFO, CELL("width")), web-vs-desktop conflicts (decision
  0019) and excel-graph failures stay pending with a recorded reason.
- Compare with the leaderboard's comparator, which uses a 1e-9 relative tolerance, as a second
  number. Our own score stays bit-exact.

## Acceptance criteria
- [ ] Sidecars committed (or stored per the `oracle_dir` convention, decision 0019) for the subset.
  Pending workbooks are listed with reasons.
- [ ] `make eval` reports corpus-bench bit-exact and 1e-9-tolerance match rates. docs/status.md's
  steward table gains the row, through the steward's next run, not this PR.
- [ ] The gap-triager is run on the failures; new tasks are filed and cited.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine fixes.
