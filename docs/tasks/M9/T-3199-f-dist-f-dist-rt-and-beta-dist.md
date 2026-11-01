---
id: T-3199
title: "F.DIST, F.DIST.RT and BETA.DIST: 16 cells where Excel is 5-50 ulp off the true value and T-2219's accurate kernel is visibly off Excel"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
campaign: visible
depends_on: []
defer: phase-1b
---

## Goal
The 16 cells below, where T-2219's accurate half-integer kernel is visibly off Excel (13 of them
it moved there from within 15 digits), agree with Excel to 15 significant digits again, without losing the 172 cells
T-2219 fixed.

## Context
- Decision 0188 R5 (owner's 2026-10-31 rulings): parked as Phase 1b. Every cell this task owns is a BETA.DIST or
  F.DIST density cell or a dense probes-numeric sweep cell, within Gate A's envelope (most agree with Excel to 12 or
  more digits, and Excel and the engine are both close to the math), so R1(c) counts it as Phase 1b evidence. A
  point that Gate A's tables show outside the envelope gets its own task without the key.
T-2219 (docs/excel-semantics/statistics.md "Incomplete beta in the ironcalc-calc workbooks",
bullet "Accurate tails at half-integer shapes from a + b = 10") routes half-integer shapes with
10 <= a + b < MAXGAM to `incomplete_beta.rs::half_integer_tail`, close to correctly rounded. On these
cells Excel itself is off the 60-digit mpmath value (Excel / ours now / Cephes before, in ulp), and
Cephes erred the same way as Excel, so they need Excel's own formulation of the branch bratio
takes there (`bratio::dispatch`), not more accuracy:
- probes-numeric/t2220-f-dist-sweep-rt A89 `F.DIST.RT(10,1,50)` and t2220-f-dist-sweep-cdf A801
  `F.DIST(0.1,50,1,TRUE)`: +13 / +1 / +24
- t2220-f-dist-sweep-rt A299 `F.DIST.RT(10,3,100)` and t2220-f-dist-sweep-cdf A921
  `F.DIST(0.1,100,3,TRUE)`: +13 / -4 / +15
- t2220-f-dist-sweep-rt A288 `F.DIST.RT(5,3,50)`: +6 / -1 / +9
- t2220-f-dist-sweep-d2-1 A754 `F.DIST(0.0356,100,1,TRUE)`: -50 / +5 / -47
- t2220-f-dist-sweep-d2-1 A670 `F.DIST(0.24,50,1,TRUE)`: +10 / -1 / +13
- m9-beta-dist-a100 A45 `BETA.DIST(0.5,100,50,TRUE)`: -35 / 0 / -22
- t0969-t-beta-link E159 `BETA.DIST(D159,B159/2,0.5,TRUE)`: -15 / 0 / -9
- t1745-beta-inv-roundtrip G150: -14 / 0 / -15; H150: -16 / -1 / -20; G158: +5 / 0 / +6;
  G159 (`BETA.DIST(D159,100,10,TRUE)`): +18 / 0 / +13
- Added by gap triage 2026-10-30 (`make unowned-off ALL=1` on main 8f47bb850 listed them as owned
  only by the deferred T-2221): probes-numeric/m9-f-dist-rt A16 `F.DIST.RT(10,10,10)` and
  m9-legacy-fdist A55 `FDIST(10,10,10)`, A56 `F.DIST.RT(10,10,10)`. Excel 0.0005715525434020334,
  ours 0.0005715525434020328, and the 60-digit mpmath value 0.00057155254340203257715. Excel is
  +1.4E-15 relative off it and ours +4E-16, so ours is visibly off Excel. a = b = 5 (a + b = 10)
  is in `half_integer_tail`'s range, the same class as the cells above.
Several have a shape of 1/2 (d2 = 1) or a shape of 50 or more: check which bratio branch each
takes (bgrat, bfrac, basym) and whether T-2508 (bfrac, basym) or T-2512 (bgrat) covers them before
starting; if one does, move the cell there instead.

## Acceptance criteria
- [ ] Each cell above is within half a unit of the 15th digit of Excel, or named in the Outcome with
      the task that owns its branch (non-deferred).
- [ ] None of the 172 cells T-2219 moved to within 15 digits goes visibly off again; no gated
      known-failure line added.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Bit-level fits (T-2221). The F quantiles (T-2511).
