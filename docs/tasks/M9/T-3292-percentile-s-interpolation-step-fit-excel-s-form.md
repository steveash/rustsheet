---
id: T-3292
title: "PERCENTILE's interpolation step: fit Excel's form where a + h(b - a) misses (gen-m9 5395b8a427764a3f A269, the one cell T-1811's rule leaves)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-3291]
defer: phase-1b
---

## Goal
PERCENTILE, PERCENTILE.INC, QUARTILE and QUARTILE.INC form the interpolated value with Excel's own operation order, so
gen-m9/5395b8a427764a3f Calc!A269 is Excel's to the bit and every PERCENTILE.INC cell that passes today still passes.

## Context
- Gap triage 2026-10-30 (`make eval` on origin/main 9efff2847): the cell is a known failure in eval/baseline/gen-m9.txt.
  It is 1 ulp off and within 15 digits, so this task is deferred (decision 0114). No open task named it: T-1811
  closed and left it in the baseline.
  - gen-m9/5395b8a427764a3f Calc!A269 `=PERCENTILE(Data!A9:D11,1E-16)`: Excel -2.499999999999999, ours
    -2.4999999999999987. The numbers are -2.5, 1E-15, 0.1, 0.2, 100 and 123456789.123, so n = 6 and h = 4.440892098500626E-16.
- The rule (statistics.md "PERCENTILE and PERCENTILE.INC interpolate from a one-based rank 1 + k(n-1)", T-1811) and
  the engine (`functions/statistical/percentile.rs` `percentile_inc_impl`) return
  `sorted[low] + (rank - whole) * (sorted[high] - sorted[low])`. That matches 39 of the 40 T-1811 gen-m9 cells.
  In doubles, `(1 - h)*a + h*b` and `(a - h*a) + h*b` give Excel's A269 (-2.5·(1 - h) is a rounding tie that goes to
  even), but statistics.md records that `(1-h)*a + h*b` misses 4 other cells. So Excel's form is neither as stated.
  Candidates: a choice of form by the sign of a or of b - a, x87 extended intermediates, or `a*(1 - h) + b*h` with
  `1 - h` formed from r (`(whole + 1) - rank`).
- Needs an oracle sweep: the cached cells hold one discriminating point, so the fit waits for T-3291's
  probes-numeric/t3291-percentile-tiny-k case. Score every candidate on that case and on the T-1811 gen-m9 cells and
  record the counts (decision 0056: time-boxed, attempt 1).
- QUARTILE and QUARTILE.INC share `percentile_inc_impl`. PERCENTILE.EXC's zero-based `interpolate` is out of scope
  unless the sweep shows it uses the same last step.

Cells this task owns:
- gen-m9/5395b8a427764a3f.case.json (1): Calc!A269

## Acceptance criteria
- [ ] gen-m9/5395b8a427764a3f Calc!A269 passes bit-exact and its line leaves eval/baseline/gen-m9.txt, with no new
      line in any eval/baseline/*.txt (`make eval-baseline` refuses regressions); or the task records each candidate
      with its bit-exact count on t3291-percentile-tiny-k and the T-1811 cells, and is time-boxed (decision 0056).
- [ ] Every PERCENTILE.INC, PERCENTILE, QUARTILE and QUARTILE.INC cell of probes-numeric/t3291-percentile-tiny-k that
      the chosen form predicts passes, and the Outcome counts the rest.
- [ ] statistics.md's PERCENTILE entry states the fitted form and cites the sweep cells that pin it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The one-based rank itself (T-1811's rule stands); PERCENTILE.EXC and QUARTILE.EXC unless the sweep ties them in.
- PERCENTRANK.
