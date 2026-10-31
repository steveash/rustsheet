---
id: T-3270
title: "Large-shape gamma ratio form at a/10 and a/2: last digits of 8 m9-gamma-dist-a50/a500 cells 1-2 ulp off after T-2638"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-3252]
defer: phase-1b
---

## Goal
The 8 probes-numeric cells below, which T-2638's large-shape prefactor left 1 or 2 ulp from Excel, become bit-exact.
If that is not possible, each one is recorded with its ulp distance and the reason.

## Context
- T-2638 (done) found Excel's prefactor for shapes of 20 and up, away from the mode:
  sqrt(a/2pi) exp(L + (a - x) - correction(a)). It is in docs/excel-semantics/statistics.md, "Incomplete gamma:
  accurate kernel (T-0971)", in the T-2638 paragraph. With that form, 31 cells became bit-exact. T-2638's stage section
  sends "cells left within 4 ulp" to a `defer: phase-1b` follow-up, but none was filed. The cells below are those
  cells. No open task names them; `scripts/exit_owners.py`'s owner rule was checked against origin/main and the
  pushed task branches in gap triage 2026-10-30d.
- Every cell sits at x/a = 1/10 or 1/2. Those are the two ends of the band where the engine switches between the
  ratio form and `rcomp` (`incomplete_gamma::NEAR_MODE` = 0.4, `RATIO_FORM_MIN` = 0.005). T-3252 probes those bounds,
  so this task waits for it. The remaining bit may be where Excel rounds L (`a ln(x/a)` against `-a ln(a/x)`) or
  `correction(a)`.
- Cells, Excel vs ours (ulp = ours - Excel). All are within 15 digits; probes-numeric is ungated, so check them with
  `make eval-show`:
  - probes-numeric/m9-gamma-dist-a50 Calc!A13 `GAMMA.DIST(50,50,2,TRUE)` and Calc!A23 `GAMMA.DIST(25000000,50,1000000,TRUE)`:
    6.953305247616096E-06 vs 6.953305247616095E-06 (-1)
  - probes-numeric/m9-gamma-dist-a500 Calc!A1 `GAMMA.DIST(50,500,1,TRUE)` and Calc!A21
    `GAMMA.DIST(50000000,500,1000000,TRUE)`: 5.364386963522797E-307 vs 5.3643869635227986E-307 (+2)
  - probes-numeric/m9-gamma-dist-a500 Calc!A2 `GAMMA.DIST(50,500,1,FALSE)`: 4.829137127185113E-306 vs
    4.829137127185114E-306 (+1)
  - probes-numeric/m9-gamma-dist-a500 Calc!A12 `GAMMA.DIST(100,500,2,FALSE)`: 2.4145685635925566E-306 vs
    2.414568563592557E-306 (+1)
  - probes-numeric/m9-gamma-dist-a500 Calc!A13 `GAMMA.DIST(500,500,2,TRUE)` and Calc!A23
    `GAMMA.DIST(250000000,500,1000000,TRUE)`: 4.067165681999133E-44 vs 4.0671656819991333E-44 (+1)
- Neighbours: T-3224 owns the near-mode cdf cells of the same probes (A5, A15, A25) and T-2703 owns the subnormal
  density companions (A22, B22).

## Acceptance criteria
- [ ] The 8 cells above are bit-exact. `make eval-show CASE=probes-numeric/m9-gamma-dist-a50.case.json` and
      `-a500` show them. Any cell that is not exact is listed in the Outcome with its ulp distance and the forms tried.
- [ ] No new known-failure line in any baseline. The bit-exact cells of both probes stay bit-exact.
- [ ] statistics.md's T-2638 paragraph records the change, or the forms tried.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The bounds of the ratio form (T-3252) and the shapes below 20 (T-3219).

## Deferred (decision 0114)
`defer: phase-1b`: every cell this task owns is already within 15 significant digits of Excel, so it waits until
Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells
visibly off. In that case, give it a `campaign:` stage instead.
