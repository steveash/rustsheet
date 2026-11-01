---
id: T-3471
title: "BETA.DIST cdf at a < 1, b = 2.5 and x in [0.29, 0.3): 9 t2464-r2-bratio-x29 cells where Excel is within 1.7 ulp of the math and ours is 4 to 6 ulp off (accuracy)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
The 9 BETA.DIST cdf cells below, at a = 0.75 or 0.9, b = 2.5 and x from 0.2910 to 0.2997, are within half a unit
of the 15th significant digit of Excel's value, through a more accurate kernel on the branch they take (not a fit to
Excel's bits). No cell of t2464-r2-bratio-x29 that is within 15 digits today moves off.

## Context
- Gap triage 2026-10-31i (`make eval` on origin/main 0c2fbf706, excel-graph; `scripts/exit_owners.py --all-off` and
  `--milestone M9`, `rate:BETA.DIST`): these cells have no open owner, deferred or not.
- Signed ulps from mpmath at 60 digits (x and the shapes as the doubles Excel reads), Excel / ours:
  - `BETA.DIST(0.2910,0.75,2.5,TRUE)` A1551: Excel 0.6794939195302907 (-1.4), ours 0.6794939195302913 (+4.0)
  - A1574 (x 0.2933): -0.5 / +4.9; A1597 (0.2956): -0.4 / +5.0; A1609 (0.2968): -1.0 / +4.4;
    A1616 (0.2975): -0.7 / +4.7; A1637 (0.2996): +0.3 / +5.7
  - `BETA.DIST(0.2916,0.9,2.5,TRUE)` A2327: Excel 0.617681544291794 (+1.5), ours 0.6176815442917933 (-4.8)
  - A2393 (0.2982): -0.6 / -6.0; A2408 (0.2997): +1.7 / -4.6
  Excel is within 1.7 ulp of the math on all 9 and ours 4 to 6 ulp off, so this is the `accuracy` stage.
- Root cause (suspected): with min(a, b) <= 1 and x <= 0.5 bratio does not swap, a0 = a < 1, b0 = 2.5, and Excel's
  dispatch cut is x0 >= 0.3, not R's 0.29 (statistics.md "Incomplete beta is TOMS 708 `bratio`", bullet "x0 >= 0.3,
  not R's 0.29"; the engine's `bratio/mod.rs` `dispatch` takes 0.3). So in [0.29, 0.3) these points take the
  a0 <= 1 path of bup with n = 20 terms and then bgrat, and ours carries that branch's error. T-2512's research
  (docs/research/t2464/r3-bgrat/RESULTS.md) found TOMS bgrat stops tens of ulps short because it hands 15·eps to
  grat1; T-3382 builds an accurate bgrat increment for the L140 one-term case. Check which branch each cell takes
  first (`Branch` in `bratio/mod.rs`); if it is bgrat's increment, reuse T-3382's (do not copy it, directive 3) or
  land the shared piece here and point T-3382 at it.
- Excel's cell is near the math, so the target is the math to about 1 ulp, not Excel's bits. If part of the case turns
  out to need Excel's own rounding, that part moves to a `visible` follow-up with `defer: phase-1b` (decision 0188 R5)
  and the R4 guards of reviewer-excel-fidelity check 7.
- Code: crates/rustsheet-engine/src/functions/statistical/bratio/ (`mod.rs`, `bup.rs`, `bgrat.rs`).

Owned cells (one line, all BETA.DIST cdf):
- probes-numeric/t2464-r2-bratio-x29.case.json (9): Calc!A1551, A1574, A1597, A1609, A1616, A1637, A2327, A2393, A2408

## Visible-parity stage
Decision 0083 / 0114 (`accuracy`): met when the 9 cells above are within half a unit of the 15th significant digit
of Excel's value (`make eval-show CASE=probes-numeric/t2464-r2-bratio-x29.case.json`), and no cell of the case that is
within 15 digits today moves off. Bit-level criteria still unmet then move to a follow-up with `defer: phase-1b`.

## Acceptance criteria
- [ ] The 9 cells above are within half a unit of the 15th significant digit of Excel's value, shown in the Outcome
      with `make eval-show` output.
- [ ] On t2464-r2-bratio-x29 (3,080 cells) the count within 15 digits of Excel does not fall, and no probes-numeric or
      gated cell goes from within 15 digits to visibly off; any that does is listed with Excel / new / exact values,
      or the change is narrowed.
- [ ] A unit test pins at least two of the cells at Excel's value, with the case and cell named.
- [ ] No new known-failure line in any eval/baseline/<suite>.txt (`make eval-baseline` refuses regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- L140 bup+bgrat with one bup term at a0 > 15 (T-3382) and the bulk with several bup terms (T-3401, on branch
  agent/functions/tasks-refill-opus when this was filed).
- Excel's bits where Excel is off the math (Phase 1b, decision 0188 R5).
