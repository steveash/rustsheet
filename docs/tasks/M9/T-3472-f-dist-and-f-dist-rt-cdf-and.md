---
id: T-3472
title: "F.DIST and F.DIST.RT cdf, and BETA.DIST(0.5,100,100,TRUE): 6 cells where Excel is within 1.6 ulp of the math and ours is 3.6 to 13.8 ulp off (accuracy)"
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
The 6 incomplete-beta cdf cells below (5 F.DIST / F.DIST.RT on the T-2220 sweeps and BETA.DIST(0.5,100,100,TRUE)) are
within half a unit of the 15th significant digit of Excel's value, through a more accurate kernel on the bratio branch
each takes (not a fit to Excel's bits). No cell of those cases that is within 15 digits today moves off.

## Context
- Gap triage 2026-10-31i (`make eval` on origin/main 0c2fbf706, excel-graph; `scripts/exit_owners.py --all-off` and
  `--milestone M9`, `rate:F.DIST`, `rate:F.DIST.RT`, `rate:BETA.DIST`): no open owner, deferred or not. They are the
  cells of that unowned group where Excel is close to correctly rounded; the rest (Excel itself 3 to 620 ulp off the
  math) are T-3474's and T-3475's, Phase 1b.
- Signed ulps from mpmath at 60 digits (F as I_x(d1/2, d2/2) with x = d1·X/(d1·X + d2), the right tail as
  I_y(d2/2, d1/2)), Excel / ours:
  - t2220-f-dist-sweep-d2-1 A400 `F.DIST(0.24,7,1,TRUE)`: Excel 0.0805651348300428 (-0.2), ours 0.08056513483004273 (-5.3)
  - t2220-f-dist-sweep-rt A467 `F.DIST.RT(3,5,10)`: Excel 0.06555756209384413 (+1.2), ours 0.06555756209384406 (-3.8)
  - A504 `F.DIST.RT(1,7,1)`: Excel 0.6493833371797926 (+1.1), ours 0.6493833371797919 (-5.2)
  - A954 `F.DIST.RT(1,100,7)`: Excel 0.5640919724091795 (+1.6), ours 0.5640919724091789 (-3.8)
  - A984 `F.DIST.RT(1,100,50)`: Excel 0.5108794921920699 (+0.6), ours 0.5108794921920683 (-13.8)
  - m9-beta-dist-a100 A51 `BETA.DIST(0.5,100,100,TRUE)`: exactly 1/2 by symmetry; Excel 0.4999999999999999 (one
    double below), ours 0.5000000000000004 (two doubles above)
- Root cause (suspected, one kernel, several branches): every caller makes one bratio call (statistics.md
  "Incomplete beta is TOMS 708 `bratio`"). By the dispatch, A400 swaps to a0 = 0.5, b0 = 3.5, x0 = 0.373 (w1 =
  bpser(b0, a0, y0) at x0 >= 0.3); A467 swaps at the mean to a0 = 5, b0 = 2.5, x0 = 0.4 (L140: bup with n = 2 then
  bpser, x0 <= 0.7); A504 swaps to a0 = 0.5, b0 = 3.5, x0 = 0.125 (bup with n = 20 then bgrat); A954 and A984 have
  a = 50 against b = 3.5 or 25, near the mean (check the swap, then L140 bup+bgrat); A51 is a0 = b0 = 100, bfrac's b0 >= 40.
  Confirm each with `bratio/mod.rs` `Branch` before changing anything. F goes through `fisher.rs` (closed forms at a
  df of 2, then `bratio::closed_or_ported_tails` / `kernel_tails`, T-2847), and a branch the port has not taken yet
  may still fall back to `incomplete_beta.rs`, so the first question is which code path each cell takes today.
- Related open work: T-3382 (accurate bgrat increment, L140 one bup term), T-3471 (a0 < 1 bup+bgrat at x0 in
  [0.29, 0.3)), T-3383 (bfrac and bup arithmetic). Share pieces with them rather than duplicating (directive 3).
- Excel is near the math, so the target is the math to about 1 ulp. If a cell turns out to need Excel's own rounding,
  it moves to a `visible` follow-up with `defer: phase-1b` (decision 0188 R5) and the R4 guards of
  reviewer-excel-fidelity check 7.
- Code: crates/rustsheet-engine/src/functions/statistical/bratio/, `fisher.rs`, `incomplete_beta.rs`.

Owned cells (all cdf or right tail):
- probes-numeric/t2220-f-dist-sweep-d2-1.case.json (1): Calc!A400
- probes-numeric/t2220-f-dist-sweep-rt.case.json (4): Calc!A467, A504, A954, A984
- probes-numeric/m9-beta-dist-a100.case.json (1): Calc!A51

## Visible-parity stage
Decision 0083 / 0114 (`accuracy`): met when the 6 cells above are within half a unit of the 15th significant digit
of Excel's value, and no cell of the three cases that is within 15 digits today moves off. Bit-level criteria still
unmet then move to a follow-up with `defer: phase-1b`.

## Acceptance criteria
- [ ] The 6 cells above are within half a unit of the 15th significant digit of Excel's value (`make eval-show` output
      for each case in the Outcome), with the branch each takes named.
- [ ] On t2220-f-dist-sweep-d2-1, t2220-f-dist-sweep-rt and m9-beta-dist-a100 the count within 15 digits of Excel does
      not fall, and no probes-numeric or gated cell goes from within 15 digits to visibly off; any that does is listed
      with Excel / new / exact values, or the change is narrowed.
- [ ] Unit tests pin the fixed cells at Excel's value, with the case and cell named.
- [ ] No new known-failure line in any eval/baseline/<suite>.txt (`make eval-baseline` refuses regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The F.DIST density (T-3173, T-3174, T-3474) and every cell where Excel is off the math (T-3474, T-3475; Phase 1b).
- F.INV and F.INV.RT (T-3398), which invert whatever cdf this lands.
