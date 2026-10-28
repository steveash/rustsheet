---
id: T-3004
title: "Give the 10 FORECAST.ETS cells of the stat argument-modes probe to T-2425 so the Phase 1 rate leaves them out (decision 0143)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The ten FORECAST.ETS-family cells of the stat argument-modes probe (Sheet1 column H, rows 266-281: excel-graph `#N/A`,
ours `#N/IMPL!`) are owned only by T-2425 (M14), so decision 0143 leaves them out of the Phase 1 rate and the four
FORECAST.ETS functions meet it.

## Context
- Found by T-0085 (2026-10-28 eval on main 81569726, `make eval`): after decision 0143 the Phase 1 rate leaves 453
  counted FORECAST.ETS cells out, but FORECAST.ETS keeps 2/20, FORECAST.ETS.CONFINT 2/12, FORECAST.ETS.SEASONALITY
  3/20 and FORECAST.ETS.STAT 3/10 visibly-off counted cells, all ten in that probe. Excel for the web never computes the
  family (statistics.md "FORECAST.ETS family: excel-graph never computes it"), so they are desktop-evidence work like
  T-2425's gen-m9 cells.
- Two things keep them out of `eval/runs/desktop-owned.tsv` (`scripts/desktop_owned.py`): T-2425 does not name them, and
  the M9 tasks T-0969 (its TDIST and TTEST tally lines) and T-0972 (its CHIINV tally line) name the probe's case on a
  line with no A1 token, which exit_owners.py's `owned` rule (decision 0136) reads as owning every cell of the case.
- This file names the probe only by a phrase, not its stem, so it owns none of its cells itself.

## Acceptance criteria
- [ ] T-2425 lists the ten cells (case stem plus Sheet1 A1 names) in its Cells list.
- [ ] The tally lines of T-0969 and T-0972 that name the probe stop claiming the whole case: each names the cell it
      counts (from `make eval-show CASE=...` or report.json `attributed_off`), and `make unowned-off ALL=1` lists no new
      unowned cell because of the change.
- [ ] `python3 scripts/desktop_owned.py | grep -c` over the probe's stem gives 10, and on `make eval` the four
      FORECAST.ETS rows show 0 visibly-off Phase 1 cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing the FORECAST.ETS family (T-2425).
- Changing the ownership rule.
