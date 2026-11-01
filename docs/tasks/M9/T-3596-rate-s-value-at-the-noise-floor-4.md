---
id: T-3596
title: "RATE's value at the noise floor: 4 t3348-rate-gate-a-fresh cells where Excel and the engine return different roots (all within 1e-9 of the math)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-3424]
campaign: visible
defer: phase-1b
---

## Goal
On the 4 owned cells, RATE returns Excel's number. Excel and the engine both return a number there, inside Gate A's
envelope, but they differ by 3e-13 to 1e-11 relative. The fix comes from the model of Excel's secant noise that T-3424
builds for the type of these cells, not from a rule for these 4.

## Context
- Gap triage 2026-11-01 (`make eval` on main 408444f20, excel-graph). `exit_owners.py --milestone M9` lists these 4
  cells of T-3348's fresh Gate A sweep under `rate:RATE`. T-3424 (in flight on
  `agent/functions-financial/T-3424-rate-noise-floor-type`) owns the sweep's type mismatches (A59, A70, ..., A292).
  financial.md says A285 is "a noise-floor cell, T-3424's kind" but no task names it. No open task names A277, A280
  or A284.
- Excel vs ours, with the relative error against the 50-digit root (mpmath `findroot` on
  pv·(1+r)^n + pmt·(1+r·type)·((1+r)^n - 1)/r + fv, computed in this triage):
  - Calc!A277 `RATE(1,7825886523.68455,4438564695.24373,-12454588133.5585,1)`: Excel 0.01550309192283962 (4.0e-14),
    ours 0.015503091922844301 (3.4e-13); root 0.015503091922839004424.
  - Calc!A280 `RATE(2,-31530007.9645075,90814054.3979302,-29338998.8658121,1)`: Excel 0.017989535216979366 (1.9e-13),
    ours 0.017989535216973353 (1.5e-13); root 0.017989535216975979097.
  - Calc!A284 `RATE(22,368004296.87717,-30770197366.4562,120137665952.442,1)`: Excel 0.07120577478278645 (1.6e-15),
    ours 0.0712057747827863 (5.3e-16); root 0.071205774782786331396.
  - Calc!A285 `RATE(1,-106163915967.863,10335094.4157681,106151868687.97,0)`: Excel 0.16566713455945356 (1.7e-11),
    ours 0.16566713456092994 (2.6e-11); root 0.16566713455656912583.
  All 8 values are within 1e-9 of the root, so R1(c) counts the cells as Phase 1b evidence (decision 0188 R5). Hence
  `defer: phase-1b`. The terms are 1e7 to 1e11, so the iterates stop in the noise of f, as in T-3424's cells. Which
  stop each side takes (|f| < 1e-7, the cap's |f| < 1e-4, or T-3348's flat-secant hold) is not yet recorded.
- Rule: financial.md "RATE holds on a flat secant and runs on below -1" and decision 0198 (no rule on the hold).
  Code: `fn_rate` and the secant in crates/rustsheet-engine/src/functions/financial/. `OFFSET` and `TOLERANCE` carry
  T-3348's `FITTED` tags.
- Cells owned:
  - probes-numeric/t3348-rate-gate-a-fresh.case.json (4): Calc!A277, A280, A284, A285

## Acceptance criteria
- [ ] The Outcome records, for each owned cell, the step and stop at which the engine returns and, under T-3424's
      noise model, at which Excel returns.
- [ ] Each owned cell is within half a unit of the 15th significant digit of Excel's value. Or it is named in the
      Outcome with the reason T-3424's model does not reach it, and stays inside Gate A's envelope.
- [ ] Any constant or model change follows decision 0188 R4 (reviewer-excel-fidelity check 7), shown in the Outcome:
      hold-out counts against the correctly rounded root on the 1,796 RATE oracle cells (R4.1); a fresh sweep
      oracle-filled after the freeze (R4.2); its Gate A envelope result, 0 points outside and 0 error-against-number
      cells (R2); `FITTED(T-3596, n_cells, range)` on any new or refitted constant with accurate math outside the
      range (R4.3); no step below the reproducibility floor (R4.4); no keep-matched branch (decisions 0056, 0082).
- [ ] No gated cell regresses (`make eval-baseline`); `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083 and 0114 (`visible`): done when the 4 owned cells are within half a unit of the 15th significant digit
of Excel's value. Bit-level residue moves to a follow-up with `defer: phase-1b`.

## Out of scope
- RATE's type at the noise floor (T-3424). RATE at the spurious root r = -1 (T-3547).
