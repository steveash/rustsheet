---
id: T-0089
title: "Milestone exit rows for M1 and M11 in the eval report (decision 0146 items 1 and 5), with rows that read several suites"
milestone: M0
lane: harness
status: done
priority: P1
size: M
tier: opus
depends_on: [T-0088, T-0092]
---

## Goal
The eval report has `milestone_exit` rows for M1 and M11 measured as decision 0146 items 1 and 5 define, and the
row model reads more than one suite per milestone, so `make eval` says whether M1 and M11 exit and
`scripts/exit_owners.py --milestone M1` (and `M11`) lists their failing exit cells. The M8, M9 and M10 rows build
on this one (T-0096, T-0095, T-0097).

## Context
- `crates/rustsheet-eval/src/milestone_exit.rs` covers M2, M5, M6 and M7 (decisions 0085, 0135, 0140): each row
  reads exactly one generated suite `gen-<milestone>`, and `scripts/exit_owners.py` reads only that suite's
  `gen_visibly_off_cells`. Decision 0146 (T-0088, PR #2144) defines the rest; at visible parity, `n/a` (never met)
  on a missing input, null verdict on a measure shown but not required:
  - **M1:** gen-m1 ≥ 3,000 scored cells, 0 visibly off; smoke 0 visibly off; the `m1-probes` list (not a probe
    row) 0 visibly off, read from the list row's visibly-off count and cells (T-0092).
  - **M11:** corpus-bench ≥ 200,000 scored Excel-backed formula cells with ≥ 0.998 not visibly off (allowance
    `floor(0.002 * scored)`; `n/a` when corpus-bench was not fetched); the M11 probe row 0 visibly off.
  - Coverage and ironcalc-calc are `n/a` for both.
- M8 needs two generated suites (gen-m8a, gen-m8b), M1 a generated suite plus smoke, and M11 a suite that is not
  generated: this task replaces the single `gen-<milestone>` lookup with a list of required suites per milestone,
  each with its own floor and rate, and keeps the M2/M5/M6/M7 rows' output identical.
- This task was split from the original T-0089 (all of M1 and M8-M11, size L) on 2026-10-28: the report inputs are
  T-0092 (list visibly-off cells), T-0093 (probe-row text items) and T-0094 (poi M8 subset); the rows are this
  task (M1, M11), T-0095 (M9), T-0096 (M8) and T-0097 (M10). `scripts/exit_owners.py` is a protected path:
  amend decision 0146's Consequences (or add a record) in the same PR.
- On decision 0146's numbers M1 is met and M11 is met on corpus-bench (99.834% of 205,366 cells) and its probes.
- The progress page (tools/src/rustsheet_tools/progress_report/) shows the rows in its Milestones table; check it
  renders the new rows.

## Acceptance criteria
- [x] `milestone_exit` reads a list of required suites per milestone (name, floor, rate); unit tests show the M2,
      M5, M6 and M7 rows are unchanged, and cover a met and an unmet M1 row (gen-m1 short of 3,000 cells; a
      visibly-off smoke cell; a visibly-off `m1-probes` cell) and a met, an unmet and an `n/a` (corpus-bench
      absent) M11 row.
- [x] report.md and report.json show the M1 and M11 rows; `make eval` reports M1 met on `main`.
- [x] `python3 scripts/exit_owners.py --milestone M1` and `--milestone M11` run and list every visibly-off cell
      of each required suite and list; `scripts/test_exit_owners.py` covers a cell
      from each source.
- [x] `make progress-report` renders the new rows; `make check` green; reviewers approve.

## Out of scope
- The M8, M9 and M10 rows (T-0096, T-0095, T-0097).
- Making the milestones pass (their tasks).

## Outcome
- `milestone_exit` reads a list of required suites per milestone (`SuiteNeed`: name, floor, rate). The first
  suite fills the row's `gen_*` fields and is named in `gen_suite`. The rest are `more_suites`, each a
  `SuiteExit` with its own measure. M2, M5, M6 and M7 read and print as before; their unit tests are unchanged.
- M1 (decision 0146 item 1): gen-m1 with at least 3,000 scored cells and smoke, both with 0 visibly off.
  The probe measure is the `m1-probes` list row (`probe_row` names it), `n/a` when the run did not execute
  the list. M11 (item 5): corpus-bench with at least 200,000 scored cells at 0.998 (`M11_RATE_NUM`), `n/a`
  when not fetched, plus the M11 probe row. Neither milestone has a coverage or ironcalc-calc criterion.
- `scripts/exit_owners.py` lists every required suite's visibly-off cells and the `m1-probes` list's
  (decision 0146 Consequences amended). The progress page names a suite that is not `gen-m<k>` and shows
  `more_suites`.
- `make eval` on this branch: M1 met (gen-m1 3,750 cells, smoke 46, list 0 visibly off); M11 `n/a`
  (corpus-bench not fetched here; 99.834% of 205,366 on decision 0146's run). `exit_owners.py --milestone
  M1` and `--milestone M11` list 0 cells.
- Eval delta: none (report-only change).
