---
id: T-0427
title: "Oracle-fill corpus-bench in batches and report the formula-cell match rate against the published leaderboard"
milestone: M11
lane: harness
status: done
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
- [x] Sidecars committed (or stored per the `oracle_dir` convention, decision 0019) for the subset.
  Pending workbooks are listed with reasons.
- [x] `make eval` reports corpus-bench bit-exact and 1e-9-tolerance match rates. docs/status.md's
  steward table gains the row, through the steward's next run, not this PR.
- [x] The gap-triager is run on the failures; new tasks are filed and cited.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine fixes.

## Outcome
Filled 2026-10-17 (`rustsheet-oracle fill -b excel-graph --suite corpus-bench`, one pass over all 912 workbooks, about 35 min).
Sidecars are committed under `eval/suites/corpus-bench/oracle/` (new `oracle_dir`; the workbooks themselves are fetched into
the cache). Results: 187 workbooks have sidecars, 539 have no formula cells, 186 are pending with reasons in
`eval/suites/corpus-bench/PENDING.md` (182 blocked by a canary bug on zip directory entries, T-0432; 2 canary refusals;
2 excel-graph session errors). 1,651 cells in 48 workbooks are left pending as web-vs-desktop conflicts (decision 0019).

Rates (`make eval`, the suites table now has a `1e-9 %` column, `SuiteSummary::cell_close_rate`): 142 scored
workbooks, 45,444 cells, **97.47% bit-exact, 98.29% within 1e-9 relative** (the leaderboard's comparator; published:
LibreOffice 96.7–98.9%, witan 99.7–99.8%). Baseline: corpus-bench known failures 5 → 21 (16 new-coverage, 0 regressions).
Caveat: 373 of the near-miss cells hold the workbook's 15-digit cached value, not the web value, because the fill
keeps a cached Microsoft Excel value within 1e-9 (decision 0030 rule 1); T-0433 decides whether that applies here.

Gap-triager on the failures: T-1788/T-1770 (month name glued to digits, 623 cells), T-0433 (cached value over web
value, 373), T-1771 (VLOOKUP into a cached external workbook, 115), T-1236 (EOMONTH array lift, 26, extended),
T-0434 (volatile detection through table references, 6), T-1772 (three array/legacy cells, 4), T-1089 (bare
IMAGE, 1, extended). The 5 timeouts stay with T-0430. The steward adds the status.md row.
