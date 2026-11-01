---
id: T-3743
title: "T-3444 follow-up: point stale BINOM.DIST.RANGE passages at the T-3444 entry; cite small-n evidence; measure RANGE's past-walk cost"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-3444]
---

## Goal
docs/excel-semantics/statistics.md and T-3444's task file describe BINOM.DIST.RANGE and BINOM.INV as T-3444 left
them, and the BINOM.DIST.RANGE lift weight's cost claim is measured, not estimated.

## Context
- The round-2 Excel-fidelity and security reviews of PR #2752 (T-3444) approved with these MINORs, left for a follow-up
  so the approved head did not move.
- statistics.md: the T-3352 entry ("BINOM.DIST and NEGBINOM.DIST at a large n: basym and Loader") still says
  BINOM.DIST.RANGE and BINOM.INV take statrs past the walk (`BINOM.INV(1000000000,0.3,0.5)` 299998246), and the T-0927
  "Ours" paragraph says BINOM.DIST.RANGE walks "from the cheaper end". Since T-3444, one-point ranges and ranges from 0
  take BINOM.DIST's own pmf and cdf, and past the walk the range is the cdf difference ("BINOM.DIST.RANGE and BINOM.INV
  at a large n sit on BINOM.DIST's own cdf (T-3444)").
- Small-n evidence the T-3444 entry does not cite (fidelity reviewer's measurement): the ungated
  probes-numeric/t2145-binom-range-sweep went from 424/790 on main to 589/790 (165 fixed, 0 newly failing).
- T-3444's acceptance criterion 1 is ticked "within 15 digits of Excel" while 266 of the probe's 756 cells miss (owned
  by T-3447 and T-3740): reword it as met for BINOM.INV/CRITBINOM (252/252) and the identified RANGE forms.
- BINOM.DIST.RANGE keeps `Weighted(4096)` (`lift/kinds/statistical.rs` `DISCRETE_SUM_3_OF_4`, budget about 65 us an
  element) but past the walk now runs up to two `excel_binom_cdf` calls (bratio, else a 2^10-term walk) after its own
  walk. Measure it in release over a grid like T-3444's BINOM.INV grid (n 1E3 to 2^31 - 2, p 1E-9 to 1 - 1E-6, ranges
  near and past the mean) and raise the weight, with its DEVIATION entry, `lift-kinds.json`
  (`rustsheet-eval lift-kinds --write`) and unit test, if it exceeds the budget.

## Acceptance criteria
- [ ] The two stale statistics.md passages point at the T-3444 entry, which cites the t2145-binom-range-sweep delta.
- [ ] T-3444's criterion 1 says what was met and who carries the rest.
- [ ] The BINOM.DIST.RANGE weight comment records a release measurement; the weight is raised if it is over budget.
- [ ] `make check` green; reviewers approve.

## Out of scope
- BINOM.DIST's last digits (T-3447) and the upper-tail form (T-3740).
