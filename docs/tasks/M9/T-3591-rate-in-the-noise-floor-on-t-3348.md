---
id: T-3591
title: "RATE in the noise floor on T-3348's fresh sweep: 4 digit cells beyond 15 digits, inside Gate A (T-3389's hold-out)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-3389]
defer: phase-1b
---

## Goal
The 4 RATE cells of T-3348's fresh sweep that have the right type but are beyond 15 digits of Excel match Excel's
value to 15 digits. They are the out-of-sample check of the residual model T-3389 builds for the same noise floor.

## Context
- Gap triage 2026-11-01 (`make eval` on origin/main 408444f20): these cells had no open owner. T-3424 owns the
  sweep's type misses, and T-3389 owns the 4 digit misses of probes/t3348-rate-fv-nonzero.
- Values: Excel, ours, then the 50-digit math from `rustsheet-accuracy math`. The sidecar comes with T-3587.
  - probes-numeric/t3348-rate-gate-a-fresh Calc!A277 `RATE(1,7825886523.68455,4438564695.24373,-12454588133.5585,1)`:
    0.01550309192283962 / 0.015503091922844301 / 0.015503091922839038
  - Calc!A280 `RATE(2,-31530007.9645075,90814054.3979302,-29338998.8658121,1)`: 0.017989535216979366 /
    0.017989535216973353 / 0.017989535216975964
  - Calc!A284 `RATE(22,368004296.87717,-30770197366.4562,120137665952.442,1)`: 0.07120577478278645 /
    0.0712057747827863 / 0.071205774782786333
  - Calc!A285 `RATE(1,-106163915967.863,10335094.4157681,106151868687.97,0)`: 0.16566713455945356 /
    0.16566713456092994 / 0.16566713455708883
- Both Excel and the engine are within 1e-9 of the math but up to 1.4e-11 relative from each other. With T-3587's
  sidecar R1(c) reads them as Phase 1b evidence (decision 0188 R5), hence `defer: phase-1b`. They are T-3389's
  kind: the secant ends in the residual's rounding noise (|pv|, |fv| up to 1.2E11). Excel is not the correctly rounded
  root either, so a more accurate kernel will not match it. Only Excel's residual noise will.
- The sweep was drawn after T-3348 froze its model (financial.md "RATE holds on a flat secant and runs on below -1").
  It has not seen T-3389's residual form, so it is a fresh hold-out for that form (decision 0188 R4.2).
- Code: `compute_rate`, `rate_residual` (crates/rustsheet-engine/src/functions/financial/financial_util.rs); fit lab
  `rate`.

## Acceptance criteria
- [ ] probes-numeric/t3348-rate-gate-a-fresh.case.json (4): Calc!A277, A280, A284, A285 are within half a unit of the
      15th significant digit of Excel. If not, the Outcome lists each with the residual forms tried.
- [ ] R4 guards (reviewer-excel-fidelity check 7, decision 0188) for any change to the residual:
      - hold-out counts, with these 4 cells and the rest of the fresh sweep as the test split;
      - a fresh sweep after the freeze if anything is refitted here;
      - the Gate A envelope result;
      - the `FITTED` tag and its range;
      - the reproducibility floor;
      - no keep-matched branch.
- [ ] RATE's fit set stays bit-exact (960 cells, as T-3348 counts it). No gated cell regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The sweep's type misses (T-3424).
