---
id: T-3120
title: "IRR far-root results on Excel's 2^-53 grid in v: find where the secant rounds v (t2565, 181 cells; irr-fallback-roots A25-A39)"
milestone: M9
lane: functions-financial
status: in-progress
priority: P2
size: M
tier: opus
campaign: visible
depends_on: []
---

## Goal
IRR at roots far above the guess (v = 1/(1 + r) about 1E-7 to 1E-3) returns Excel's value: the secant's
result with v held on Excel's grid of integer multiples of 2^-53. The fix finds where in the secant step
Excel rounds v, so that one rule gives the three-flow cells and the two-flow cells alike.

## Context
Split from T-2131 by the gap triage of 2026-10-29 (`main` 9235b7c3). T-2565's probe has landed and gives the
evidence T-2131's branch was blocked on. T-2131's branch `agent/functions-financial/T-2131-irr-15-digits` is
still `status: blocked` there, so `next` never offers T-2131. This task owns the far-root part. T-2131 keeps
the small-root last digits, and T-2928 keeps the fallback path.

Evidence (excel-graph), all in eval/:
- probes/t2565-irr-guess-sweeps (T-2565): has 181 cells, and every one is a known failure in eval/baseline/probes.txt.
  180 are visibly off (from the 12th digit on, except P!C157 from the 10th) and 1 (P!C96) is within 15 digits
  (crates/rustsheet-eval/src/closeness.rs `within_fifteen_digits`). The failing cells are P!A1, A25,
  A49, A73, A107, A142 and P!C1-C175 (each listed as an F line). Examples:
  - P!C31 `IRR({-100,2000000,42000000},B31)`: Excel 20019.977996174617, ours 20019.977996176563.
  - P!C49 `IRR({0.0001,-2,-42},B49)`: Excel 20019.977187036908, ours 20019.977187028708.
  - P!C73 `IRR({-2.5,-1,123456789.123},B73)`: Excel 7026.083663633989, ours 7026.083663631957.
- probes/irr-fallback-roots, the far-root cells T-2131 took on 2026-10-27 (financial.md "IRR's far-root bound"). This
  task owns them now:
  - P!A25 `IRR({-1,1E+4})`: Excel 9999.0000000011, ours 9999.000000012204.
  - P!A26 `IRR({-1,1E+5})`: Excel 99999.0000004551, ours 99999.00000156532.
  - P!A28 `IRR({-1,3E+6})`: Excel 2999998.999580666, ours 2999999.000579867.
  - P!A36 `IRR({-1,0,1E+8})`: Excel 9998.99999994559, ours 9998.999999948663.
  - P!A37 `IRR({-1,0,1E+10})`: Excel 99998.9977267184, ours 99998.99772636793.
  - P!A38 `IRR({-1,0,1E+12})`: Excel 999998.9968626199, ours 999998.9969050374.
  - P!A39 `IRR({-1,0,1E+14})`: Excel 9999998.994161328, ours 9999998.996610194.
- docs/excel-semantics/financial.md, "IRR is a secant in v", the T-2565 bullet: Excel's results are integer
  multiples of 2^-53 in v. The fit-lab model's v rounded to that grid matches 173 of 175 C cells. P!C95 is one
  grid step off. P!C157 is an outlier about 100 steps off.

Triage measurements on 2026-10-29 (scratch scripts over `fitlab.kernels.solvers`; not committed). The task
re-runs them in the fit lab:
- Candidate A rounds the result: `irr_secant_v()` as it is, then v rounded to the nearest multiple of 2^-53 and
  1/v - 1 returned. It scores 179/181 t2565 cells, missing P!C95 and C157. It gives P!A36-A39 bit-exact and
  `{-0.0001,2,42}` and `{-2.5,-1,123456789.123}` at guess 0.1 bit-exact, but misses the two-flow cells A25, A26
  and A28. On those the model already sits on a grid point, one step from Excel's. Excel's value there is the
  exact root X - 1 with its v rounded to the grid: rounding 1/X gives 9999.0000000011, 99999.0000004551 and
  2999998.999580666 exactly.
- Candidate B keeps the state as d = 1 - v. The secant runs on d, NPV is evaluated at 1 - d, the same offset and
  stop tests apply, and 1/(1 - d) - 1 is returned. For d in [0.5, 1), d's own spacing is 2^-53, which would
  explain the grid. It gives P!A36-A39 and the two guess-0.1 cells above bit-exact, and keeps
  `IRR({-20,5,5,5,5,5})` = 0.07930826116039724 (m9-fin-irr-rate Calc!A1). It misses A25, A26 and A28 the way
  candidate A does. The triage version also threw on part of the t2565 sweep: it had no retry and no guard on a
  flat secant, so it scored 143/181 and is not a real measure.
- So the open question is the two-flow cells. NPV is linear in v there, so a single secant step should reach the
  root. That Excel lands on the grid point nearest the exact root points at how NPV is evaluated
  (c_i/x^i with x = 1/v against Horner in v, near v = 1E-4) or at the step's operand order. A plain
  rounding of the result does not explain it.

Code: `compute_irr`, `irr_secant`, `irr_npv`, `irr_from_discount` (crates/rustsheet-engine/src/functions/financial/financial_util.rs);
fit lab `tools/src/rustsheet_tools/fitlab/kernels/solvers.py` (`irr_secant_v`, `irr_candidates`),
`fitlab/irr_guess_sweep.py`.

Decision 0056 applies: each attempt is a fit-lab run of named candidates against the IRR cells (the
`rustsheet-fitlab irr` set plus the cells above). The task records each candidate's bit-exact and within-4-ulp
counts. After two attempts that do not raise the bit-exact count, the task is blocked as time-boxed.

## Acceptance criteria
- [ ] The fit lab (`uv run rustsheet-fitlab irr`, or a new kernel next to `irr_secant_v`) scores candidate A, candidate B and any
      other candidate tried, on the t2565 cells, the seven irr-fallback-roots cells above and the existing IRR fit
      set. The task's Outcome records each candidate's bit-exact and within-4-ulp counts, including rejected ones.
- [ ] The shipped rule does not lower the existing IRR fit set's bit-exact count (189/263, financial.md).
- [ ] Visible parity: at least 178 of the 180 visibly-off t2565 lines (P!C95 and C157 may stay, each with a
      sentence in financial.md) and irr-fallback-roots P!A25, A26, A28, A36, A37, A38 and A39 disappear from
      eval/baseline/probes.txt (`make eval-baseline`), with no new known-failure lines in any baseline
      (decision 0083's allowance otherwise).
- [ ] The gen-m9 far-root cells T-2131 lists (c0681a8b150ce301 Calc!A218, 67a4c03afd63f988 Calc!A3, A199, A346,
      1dce4774f5c0e736 Calc!A285) are re-measured. The Outcome says which now pass. They stay T-2131's either way.
- [ ] financial.md's "IRR is a secant in v" and "IRR's far-root bound" record where v is rounded, with the cells,
      and replace the "Unexplored: where in the step the rounding happens" note.
- [ ] `make check` green; reviewers approve.

## Out of scope
- IRR's fallback path and its digits (T-2928, T-1686), IRR's `#DIV/0!` cells (T-2802).
- Small-root last digits and the IRR.xlsx NPV residues (T-2131).
- RATE and XIRR. Check whether their solvers keep the same grid only if it costs nothing, and file a follow-up if it does.

## Visible-parity stage
Visible stage (decision 0114): Excel's own iterate is the target, not the exact root. The task is done when the
t2565 and irr-fallback-roots cells above are within half a unit of the 15th significant digit of Excel's value,
except P!C95 and C157 as the acceptance criteria allow.
Any bit-level residue goes to a follow-up with `defer: phase-1b`.

## Progress notes (2026-10-30, opus run, not a handoff)
Work in flight on this branch: `on_discount_grid` in functions/financial/financial_util.rs rounds the secant's
converged v to the nearest multiple of 2^-53 below v = 0.5 (no-op above), in the engine, not yet in the fit lab.
Measured with `rustsheet-eval check --suite probes --suite gen-m9 --suite ironcalc-calc`:
- `round()` (ties away from 0): fixed probes 184, gen-m9 15; REGRESSIONS gen-m9 9a1fa5366bc50918 A302 and
  abb7d565af89b849 A9 (`IRR({-1,0,0,0,0,0,0,0,0,1000})`, 1 ulp: our iterate is exactly half a grid step and Excel
  has the lower v).
- `round_ties_even()`: the same two regress. `trunc()`: fixed 74, ironcalc IRR.xlsx General!L47 regresses.
- ties to the smaller v (the branch's current code): fixed probes 183, gen-m9 16; REGRESSION ironcalc-calc
  IRR.xlsx General!L47 `IRR(A47:F47)` (Excel 1.3333765085060993, ours ...0998, 1 ulp: our iterate is a tie and
  Excel has the larger v).
So the tie direction is not one rule: every off-grid iterate in v in [0.25, 0.5) is a tie, and Excel's own iterate
is on the grid already, which points at candidate B (the secant run on d = 1 - v, where d's spacing is 2^-53).
Next: try candidate B in the engine with the retry and flat-secant guards, and score it in the fit lab (decision
0056) against the IRR fit set (189/263); not yet done: the fit-lab scoring, A25/A26/A28 (two-flow cells), docs.

Resumed 2026-10-30 05:35 UTC by an opus run (branch idle over 2 h).
