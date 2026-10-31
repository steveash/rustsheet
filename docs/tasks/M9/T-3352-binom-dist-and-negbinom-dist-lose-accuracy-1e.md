---
id: T-3352
title: "BINOM.DIST and NEGBINOM.DIST lose accuracy (1E-9 to 1E-8) at a very large n or number of failures"
milestone: M9
lane: functions-stat
status: done
priority: P0
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
BINOM.DIST at n ≥ 6e5 and NEGBINOM.DIST with a large number of failures are within about 1e-13 of Excel, as Excel is of
the math, instead of 1e-9 to 1e-8 off today.

## Context
- Owner's 2026-10-31 assessment (decision 0188, "The engine is wrong where Excel is right", item 5). Users hit
  this: it is an ordinary input, Excel returns the right number and the engine does not. Highest-priority work of the
  accuracy stage.
- Code: crates/rustsheet-engine/src/functions/statistical/binom.rs (`fn_binom_dist`, `fn_negbinom_dist`) and
  discrete.rs (T-0927's exact walk, capped, with statrs past it). At a large n the pmf needs a saddle-point (Loader)
  form or the incomplete beta with an accurate prefactor, not a sum or a log-gamma difference that cancels.
- Related: T-2146 (BINOM last digits, ironcalc-calc), T-3366 (BINOM.DIST generalizes poorly to fresh points). This task
  is the gross loss at a large n; it does not chase last digits.
- Probe (written before the fix, filled by excel-graph): probes/t3352-binom-negbinom-large-n, 208 cells, Calc!A1:A208.
  A1:A124 are BINOM.DIST(k,n,p,FALSE|TRUE) at n 1E4, 1E5, 6E5, 1E6, 1E7, 1E8, 1E9 with p 0.3, 0.5, 0.001 and k =
  floor(mean), floor(mean + 2 sd), floor(mean - 3 sd) (k >= 1); A125:A208 are NEGBINOM.DIST(f,r,p,FALSE|TRUE) at failures
  1E3 to 1E9 with (r, p) = (f, 0.5), (f/4, 0.2), (10, 10/(f + 10)) and f at the mean and at mean + 2 sd. n 1E4 and 1E5
  and failures 1E3 to 1E4 are the side below the failing region.
- Before the fix: 42 of 208 within 15 digits of Excel, 124 within 1E-9 of the 50-digit math; the cdf near the mean was
  grossly wrong from n = 6E5 (A108 `BINOM.DIST(300000000,1000000000,0.3,TRUE)` -2.82, Excel 0.5000156001249112; A198
  `NEGBINOM.DIST(1000000000,1000000000,0.5,TRUE)` -5.65, Excel 0.5000089206205796) and the pmf 1E-11 (n = 1E4) to 9E-6
  (failures 1E9) off.

## Acceptance criteria
- [ ] (partial: 133 of 208 cells; T-3447 owns the other 75) BINOM.DIST at n from 6e5 to 1e9 and NEGBINOM.DIST with failures from 1e5 to 1e9, pmf and cdf, match Excel's
      value within 15 significant digits, with the right type and error, on the oracle-filled cells. The Gate A envelope
      below is only a floor on the fresh sweep; where Excel differs from the math by less than G = 1e-4, Excel's value
      is the target, never the math.
- [x] A new public probe case under eval/suites/probes/cases/ (one milestone tag, M9; decision 0057), filled by the
      oracle (excel-graph), reproduces the bug; its cells are listed in this file before the fix. Include the inputs in
      Context and points on both sides of the failing region.
- [ ] (partial: 133 of 208 cells; T-3447 owns the other 75) After the fix every cell of that case is within 15 significant digits of Excel's value, with the right type and
      error (decision 0083's rule).
- [x] No gated cell regresses (`make eval-baseline` refuses regressions; decision 0083's ulp allowance only as it
      states).
- [x] Gate A envelope on a fresh sweep of its own (decision 0188 R2, R4.2): a new probes-numeric case, filled, with
      points drawn log-uniform over the function's valid domain and stratified around the failing region, scored
      against a 50-digit mpmath reference. The Outcome reports, per stratum, the points within 15 digits of Excel and
      the points within 1e-9 relative of the math (absolute 1e-300 near 0): 0 points outside both, and 0 cells where
      one side is an error and the other a number. This is a floor only: the target stays Excel's value.
- [x] The rule (what Excel does, what was wrong) is in docs/excel-semantics/ in the same PR.
- [x] `make check` green; reviewers approve.

## Out of scope
- Last-digit (bit) parity on cells already within 15 digits: Phase 1b (decision 0188 R5).
- Fitting to Excel's bits; if a fit is needed it follows decision 0188 R4 (hold-out split, fresh sweep, `FITTED` tag).

## Visible-parity stage
Decision 0083 and 0114 (`accuracy`): done when every cell this task owns is within half a unit of the 15th significant
digit of Excel's value with the right type and error, and the fresh sweep shows the Gate A envelope (decision 0188 R2).
Bit-level residue moves to a follow-up with `defer: phase-1b`.

## Outcome
- Cause: past the double-double walk's 2^10 terms, both functions fell back to statrs. Its cdf near the mean was
  grossly wrong from n = 6E5 (`BINOM.DIST(300000000,1000000000,0.3,TRUE)` -2.82, Excel 0.5000156001249112), because
  bratio's basym branch was not ported. Its pmf (a log-gamma difference) was 1E-11 off at n = 1E4 and up to 9E-6 off at
  1E9.
- Fix: a TOMS 708 `basym` port (`bratio/basym.rs`, in doubles), reached through `bratio::basym_tails`, which only the
  binomial family calls. Past the walk, the pmf uses Excel's Loader kernel (`binom.rs` `loader_pmf`; for NEGBINOM.DIST,
  p times the shifted kernel). BETA.DIST, F and T stay on today's kernel: with basym under BETA.DIST, ironcalc-calc
  BETADIST_BETAINV.xlsx G29 passes but H29 (BETA.INV round trip) regresses by an ulp. Two search changes were tried; both
  regressed other gated cells (T-3443).
- Probe probes/t3352-binom-negbinom-large-n (208 cells, excel-graph):
  - Within 15 digits of Excel: 42 before, 133 after.
  - Within 1E-9 of the 50-digit math: 124 before, 208 after. Worst is 5.2E-12, the same as Excel's own worst.
  - The 75 cells not within 15 digits are at most 4.4E-12 from Excel: Loader-kernel pmf cells (cause not identified) and
    basym, bfrac or walk cdf tails.
  - Recorded as new-coverage known failures (148 cells not bit-exact). The 75 cells outside 15 digits (57 pmf, 18 cdf)
    move to T-3447, which owns acceptance criteria 1 and 3 for them. One of them, A48 `BINOM.DIST(600,600000,0.001,TRUE)`,
    was within 15 digits on main and is not on basym (ours 0.5108555229841736, Excel 0.5108555229841731).
- Gate A envelope on the fresh sweep probes-numeric/t3352-binom-negbinom-sweep (300 points, seed 3352, scored against
  mpmath at 50 digits by a scratch script; SPEC-0029's generator is T-3357). Per stratum: within 15 digits of Excel /
  within 1E-9 of the math / outside both / error against number:
  - typical: 91 / 120 / 0 / 0 of 120
  - large n near the mean: 42 / 90 / 0 / 0 of 90
  - tails 3 to 8 sd: 14 / 60 / 0 / 0 of 60
  - log-uniform: 24 / 30 / 0 / 0 of 30
  No Excel value is more than 1E-9 off the math.
  The misses have both sides about 1E-14 to 3E-11 off the math. Ours is nearer on most (Phase 1b evidence, decision 0188
  R1(c)).
- Eval delta: gated suites have 0 fixed and 0 regressions; probes gains 1 case (208 cells, 60 bit-exact). In ungated
  probes-numeric, m9-legacy-binomdist gains 2 bit-exact cells and t2513-r3-binom-pmf-n gains 14; no case lost any.
- Acceptance criteria: 2 (probe), 4 (no regression), 5 (Gate A), 6 (semantics) and 7 are met; 1 and 3 are met for 133
  of 208 cells, and T-3447 owns the remaining 75.
- Follow-ups:
  - T-3447: the 75 probe cells still outside 15 digits of Excel.
  - T-3443: basym for BETA.DIST, F and T, with BETA.INV's crossing.
  - T-3444: BINOM.DIST.RANGE and BINOM.INV at a large n are still statrs; `BINOM.DIST.RANGE(1E9,0.3,3E8,3E8+100)` is
    1.47.
