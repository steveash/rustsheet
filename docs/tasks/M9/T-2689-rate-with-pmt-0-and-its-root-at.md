---
id: T-2689
title: "RATE with pmt 0 and its root at -1: Excel returns the 200th secant iterate at nper 24 but #NUM! at 12 and 60 (calipers formulas_financial F141:F143)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
RATE gives Excel's value or Excel's `#NUM!` when pmt is 0 and fv is 0. In that case the residual is pv·(1+r)^n,
whose only root is r = -1, so the secant creeps toward -1 until its 200-step cap. The cap's return rule is pinned
down by these cells and implemented.

## Context
Gap triage 2026-10-25 (`make eval`, agent/engine/T-2632-typed-arg-counts at 4dbb3c62).
`calipers/roundtrip/formulas_financial/golden.xlsx` was saved by Microsoft Excel 16.0300. Its block
"6. RATE (Nper × PV, PMT=-100)", Sheet1!B139:G144, calls `RATE(nper, $C$17..$C$21, $C$22)`. So the second
argument (pmt) is the column's value and the third (pv) is -100. Column F has pmt = 0:

| cell | formula | Excel | ours | model (below) |
|---|---|---|---|---|
| F139 | `RATE(1,0,-100)` | #NUM! | #NUM! | the first iterate is at or below -1 |
| F140 | `RATE(6,0,-100)` | -0.9999993377532328 | passes | converges at step 113 with the same bits |
| F141 | `RATE(12,0,-100)` | #NUM! | #NUM! | cap: 200th step -3.87e-7, f -3.4e-61 |
| F142 | `RATE(24,0,-100)` | **-0.9970141220397587** | **#NUM!** | cap: 200th iterate **-0.9970141220397587**, step -8.94e-5, f -2.5e-59 |
| F143 | `RATE(60,0,-100)` | #NUM! | #NUM! | cap: 200th step -1.25e-3 |
| F144 | `RATE(360,0,-100)` | #NUM! | #NUM! | cap: 200th step -1.44e-3 |

F142 has the known-failure line `F calipers/roundtrip/formulas_financial/golden.xlsx Sheet1!F142` in
eval/baseline/calipers.txt. The F140 line there no longer fails.

The "model" column is a scratch Python run by the triager of financial.md "RATE is a secant on the FV residual"
(T-2005): r0 = 0.1, r1 = r0 - 0.001·sign(f0), the secant step, the stop test |dr| < 1e-7 and |f| < 1e-7, and an
iterate at or below -1 is `#NUM!`. It used plain double `1+r` and square-and-multiply `^`, not the engine's
`extended_sum` and `power`. Excel's F142 value equals that run's 200th iterate bit for bit. But that iterate
fails the step test (8.9e-5), while F141's 200th step (3.9e-7) is smaller and Excel gives `#NUM!`.
financial.md says the 200th iterate "is returned on the step test alone" (probes/rate-step-cap A30 returned with
step < 1e-7 and |f| 7e-5; A31, with a 200th step of 8E-7, is `#NUM!`). Neither that rule nor an |f| test
separates F141 from F142. Open hypotheses:
- (a) Excel's `1+r` (with its `+`) or its `^` changes the n = 12 and n = 60 trajectories, so they end at or below
  -1, or flat, before the cap. The engine uses Excel's `+` and `^` here, and gives `#NUM!` at n = 24, so first
  trace the engine's own iterates against the plain-double run.
- (b) The cap's return test is something else, for example relative to r or to the previous step.
- (c) The cap is not a fixed 200 for a creeping run.

This is the same open question as T-2527's A583 bullet (does the cap's return also test |f|?). Whatever rule this
task finds must keep rate-step-cap A30/A31 and m9-fin-irr-rate A583 as they are, or say why they move. Coordinate
with T-2527 if it is in flight.

Code: `compute_rate`, `rate_residual` (functions/financial/financial_util.rs); `rate_of` (financial/annuity.rs).

## Acceptance criteria
- [x] A probe case (excel-graph, oracle-filled, at least 30 cells) covers the following, and is gated or baselined:
      `RATE(n,0,pv)` for n = 1 to 40 and a few larger n, at pv = -100 and -1;
      pmt = 0 with a nonzero fv (a root away from -1) as a control;
      and guesses 0 and 0.5 at n = 24.
      If the web disagrees with the Mac cells, desktop evidence already in the repo stays the target (decision 0030).
      Record the conflict and add a re-check to T-1075.
- [x] financial.md "RATE is a secant on the FV residual" states the cap's return rule (or the trajectory difference)
      with the cells that bound it, and the UNVERIFIED note on the cap rule is updated.
- [x] (deferred to the steward's calipers ratchet, see Outcome; the cell passes) `F calipers/roundtrip/formulas_financial/golden.xlsx Sheet1!F142` disappears from eval/baseline/calipers.txt
      (`rustsheet-eval update-baseline --suite calipers`). F139, F141, F143, F144 and the new probe cells pass,
      and no baseline gains a line.
- [x] `make check` green; reviewers approve.

## Out of scope
- RATE's last-ulp misses and the nested `RATE(n,PMT(..),..)` cells (T-2527).
- PMT's bits (T-1528, T-2243).

## Visible-parity stage
Decision 0083: this task is done when every cell it owns is within half a unit of the 15th significant digit of
Excel's value (an expected 0 must be exactly 0), or has the right type, error and text. Criteria above that ask
for bit-exact or ulp-level matches and are still unmet then move to a follow-up task (no campaign key), named in
the Outcome. Regressions that stay within 4 ulp follow decision 0083's allowance.

Owned visibly-off cells (2026-10-25 eval run): calipers formulas_financial Sheet1!F142 (RATE, `#NUM!` where Excel
has a number). Stage `semantics`: wrong type, decided by the solver's stopping rule.

## Progress
Superseded by the Outcome. The in-flight notes traced the plain-double secant over rate-pmt-zero-cap and found its
200th iterate equals Excel's value bit for bit on every returned cell, so only the cap's return rule was open; the
step-window and trajectory hypotheses they weighed were ruled out by probes/rate-cap-return-rule.

## Outcome
Excel's 200-step cap has its own, looser return test: the 200th iterate is returned when |r200 - r199| < 1e-4,
|f(r200)| < 1e-4 and 1 + r200 > 1e-4, all absolute; otherwise `#NUM!`. `compute_rate` used to return it on
|dr| < 1e-7 alone. Evidence: new probe `probes/rate-cap-return-rule` (74 cells, excel-graph) next to
`probes/rate-pmt-zero-cap` (58). Guesses that move 1 + r200 across 1e-4 at fixed n (A1-A19) put the lower pmt-0 band
on 1 + r, not on the step ratio; loan runs from far guesses pin the |f| edge (rate-step-cap A30 at 7.3e-5 returns,
A53 at 1.06e-4 is `#NUM!`) and, scaled by 2^-10 to 2^-40 (same trajectory, smaller f), show the step test is
absolute (A72: step 7.8e-5, 4e-4 of the rate, returns; A70: 1.3e-4 is `#NUM!`). financial.md "RATE is a secant on
the FV residual" states the rule.

Eval delta: probes +132 cells (both cases 100%, `C` lines in eval/baseline/probes.txt); rate-step-cap stays 46/46;
calipers formulas_financial Sheet1!F142 passes (F139-F144 all match); probes-numeric m9-fin-irr-rate A583
`RATE(10,-1E6,1E7)` now `#NUM!` as in Excel (481 -> 482; this is T-2527's A583 bullet, its |f| is 0.1-0.4 at the cap).
The calipers baseline is not rewritten in this PR: `update-baseline --suite calipers` on this branch sees 1448
fixed lines and 36 unrelated ulp-level misses (pv23_007/008/009/056 cash-flow and IRR cells, no RATE call) from
drift since the last calipers ratchet, and refuses; the steward's next calipers ratchet drops the F142 line and
the F140 line, which also passes now.
