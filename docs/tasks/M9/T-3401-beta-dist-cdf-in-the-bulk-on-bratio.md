---
id: T-3401
title: "BETA.DIST cdf in the bulk on bratio's L140 bup+bgrat with several bup terms: BETADIST_BETAINV G5 is 150 ulps off where Excel is correctly rounded (accuracy)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-3400]
campaign: accuracy
---

## Goal
BETA.DIST's cdf, and every caller of the same incomplete-beta kernel, is within 15 significant digits of Excel in the
bulk of the distribution when one shape is small (b0 from 2 to 40 after bratio's swap) and the other large (a0 > 15).
Where T-3400 shows Excel close to correctly rounded, that means an accurate kernel, not a fit to Excel's bits.

## Context
- Owned cell (gap triage 2026-10-31, `make eval` on main 83d35563). It is visibly off in ironcalc-calc, which counts
  for M9's exit row and for Gate B (decision 0188 R1(b)). Before this task, deferred T-3374 "owned" it only through an
  out-of-scope line.
  - ironcalc-calc/STATISTICAL/BETADIST_BETAINV.xlsx (1): Sheet1!G5
    `BETA.DIST(33587,3,600,TRUE,2000,5489582)`: Excel 0.6734553797493481, ours 0.6734553797493648, mpmath
    0.67345537974934809577. Excel is within 1.3E-17 relative of the math and ours is 2.5E-14 off (about 150 ulps). The
    unbounded `BETA.DIST(31587/5487582,3,600,TRUE)` gives the same value in the engine, so the bound mapping is not the
    cause. T-2119 had already recorded this cell as "160 ulp off where Excel is correctly rounded"; it was never owned.
- Root cause (suspected): a0 = 600, b0 = 3, x0 = 1 - t ≈ 0.994 send the cell to L140 (`bratio/mod.rs`
  `Branch::L140BupBgrat`), which runs bup for n = 2 terms with b0 = 1 and then bgrat. T-2512's research (branch
  agent/research/T-2512-incomplete-beta-open-pieces, docs/research/t2464/r3-bgrat/RESULTS.md) found that TOMS bgrat
  passes 15·eps to grat1, whose continued fraction stops tens of ulps short. Its follow-up T-3382 adds an accurate
  increment for one bup term only and leaves "n > 1 bup terms (2 <= b < 40)" out of scope, unprobed. The same
  inaccurate increment is the likely cause here.
- Engine scale (triager's scratch grid; T-3400 commits the evidence): 212 of 300 bulk points (p 0.05 to 0.95, a 1.5 to
  39, b 20 to 1E5) are visibly off the math. Examples, engine against math:
  - `BETA.DIST(0.00599787067238201,3,600,TRUE)` (p 0.7): -127 ulps;
  - `BETA.DIST(0.20108341703179983,20,50,TRUE)` (p 0.05): -261 ulps;
  - `BETA.DIST(2.4391745389681467E-05,2,1E5,TRUE)` (p 0.7): 0.7000000000035214 against 0.69999999999999999, 31,718 ulps.
    The b = 1E5 column is worst at p 0.7 for every a.
- Do not move the cells where Excel itself is inaccurate and the engine already matches it, for example
  probes-numeric/m9-beta-dist-a50 and -a100 at x 0.1 with b 2 to 10 (bpser, Excel 57 to 117 ulps off the math). Scope
  the change to the branch and region T-3400 shows Excel to be accurate in.
- Stage: `accuracy` (decision 0114), on G5's evidence. If T-3400 shows Excel as far off as the engine in part of the
  region, that part is a fit: it moves to a `visible` follow-up, and the guards below apply to it.
- Code: crates/rustsheet-engine/src/functions/statistical/bratio/ (`mod.rs`, `bup.rs`, `bgrat.rs`; `bgrat_half` is
  the b = 1/2 template: extend it, do not copy it, directive 3), beta.rs `fn_beta_dist`. The engine must not read
  docs/ (decision 0066).
- Related: T-3382 and T-3383 (one bup term, and Excel's bits; both on the T-2512 branch), T-3402 (BETA.INV on the
  same workbook, H5 chains on G5), T-2511 (the crossing search, in flight).

## Acceptance criteria
- [ ] The known-failure line `F	ironcalc-calc/STATISTICAL/BETADIST_BETAINV.xlsx	Sheet1!G5` leaves
      eval/baseline/ironcalc-calc.txt (`make eval-baseline`), and no new known-failure line appears in any baseline.
- [ ] On T-3400's case, for each (a, b) where Excel is within 4 ulp of the math, the engine is within 15 digits of
      Excel at every point. The Outcome gives the before and after counts per (a, b).
- [ ] No probes-numeric cell that is within 15 digits of Excel today goes visibly off. Any that does is listed with
      Excel, new and math values, or the change is narrowed.
- [ ] Unit tests pin G5 and a few T-3400 cells, naming the case and the cell.
- [ ] If any part of the change is chosen by scoring candidates against Excel cells (a fit, decision 0188 R4,
      reviewer-excel-fidelity check 7), its Outcome shows: train and hold-out counts side by side against the correctly
      rounded candidate, on T-3400's hold-out; a fresh oracle-filled sweep generated after the model was frozen; the
      Gate A envelope result on that sweep against mpmath, with every point outside it listed; a
      `FITTED(T-3401, n_cells, range)` tag on each fitted constant or cut, applied only inside its range, with accurate
      math outside it; no step fitting differences below the oracle's reproducibility floor (4 ulp web against desktop,
      decision 0112); and no branch whose domain is the set of cells that already matched.
- [ ] `make check` green; reviewers approve.

## Out of scope
- BETA.INV's landing (T-3402) and its crossing search (T-2511).
- Excel's bits in L140 (T-3383) and one bup term (T-3382).

## Visible-parity stage
Decision 0083: done when G5 and every T-3400 point where Excel is close to correctly rounded are within half a unit of
the 15th significant digit of Excel's value. Cells still off are named in the Outcome with an owner. Bit-level work
moves to a follow-up filed without the campaign key and with `defer: phase-1b`.
