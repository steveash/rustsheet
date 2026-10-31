---
id: T-3350
title: "CHISQ.INV and CHISQ.INV.RT return #NUM! at df of 2E7 and above"
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
CHISQ.INV and CHISQ.INV.RT (and the legacy CHIINV) return Excel's quantile at df ≥ 2e7, mostly with p near 1, where the
engine returns `#NUM!` today.

## Context
- Owner's 2026-10-31 assessment (decision 0188, "The engine is wrong where Excel is right", item 4). Users hit
  this: it is an ordinary input, Excel returns the right number and the engine does not. Highest-priority work of the
  accuracy stage.
- Code: `fn_chisq_inv`, `fn_chisq_inv_rt` in crates/rustsheet-engine/src/functions/statistical/chisq.rs and the gamma
  quantile under statistical/incomplete_gamma/ (the huge-shape path, `huge_shape.rs`). Find which bound or search
  refuses the large shape.
- Excel's CHISQ.INV.RT is about 5e-9 off the math in places (decision 0188 R3): that is below G, so Excel's value is
  still the target there.
- Probe (filled by excel-graph on 2026-10-31, before the fix): probes/t3350-chisq-inv-huge-df, 372 cells: CHISQ.INV and
  CHISQ.INV.RT at p of 1E-300, 1E-100, 1E-30, 1E-10, 0.001, 0.1, 0.5, 0.9, 0.999, 1-1E-6, 1-1E-10 and 1-1E-15 and df of
  1E6, 2E6, 5E6, 1E7, 2E7, 5E7, 1E8, 2E8, 5E8, 1E9, 2E9, 5E9, 1E10, 2E10 and 1E11, and CHIINV at 2E7, 1E8 and 1E9.
  The engine was `#NUM!` and Excel a number on 46 cells: CHISQ.INV A40, A41, A51-A55, A63-A67, A74-A79;
  CHISQ.INV.RT A225-A228, A235-A240, A247-A252, A259-A264; CHIINV A362-A364, A366-A368. That is df 1E7 to 1E8 (shape
  5E6 to 5E7) wherever the smaller tail is the lower one: CHISQ.INV at p up to 0.5, and CHISQ.INV.RT at p of 0.5 and
  above, the "p near 1" of the assessment. Both sides of the region (df 1E6 to 5E6, and 2E8 up) were already numbers.
- Also probes/t3350-gamma-inv-band-lower-edge (160 cells: GAMMA.INV(1E-k,a,1), k 40 to 250, at shapes 1E9 to 1E10,
  and CHISQ.INV at df 2a). Before the fix 26 of them had an error on one side and a number on the other, at the band's
  lower edge (the old fit `offset - z < 39`). One of them is the same kind of cell as the fresh sweep's A16.
- Related: T-3294 (deep lower tail at shape 20 and above), T-3337 (df 1 right tail), T-3336 (its df 1 right-tail
  sweep), T-2860 (done: the df 2 closed forms). Neither covers df ≥ 2e7.

## Acceptance criteria
- [ ] CHISQ.INV and CHISQ.INV.RT at df from 2e7 to the documented limit (1e10) with p across (0,1), p near 1
      included, match Excel's result (number or error, within 15 significant digits for a number) on the oracle-filled cells, with the right type and error. The Gate A envelope below is only a floor on the fresh sweep;
      where Excel differs from the math by less than G = 1e-4, Excel's value is the target, never the math.
      Not met in full; handed off (decision 0083): the `#NUM!` bug is fixed on every cell, with no error against a
      number left. The visibly-off numbers left are A229-A289 (`CHISQ.INV.RT(1E-300,df)`, df 2E7 to 1E9: Excel returns
      its start x0, owner ruling to match it, T-3409) and A301-A351 (df 2E9 and up, the existing R3 closed-form
      branch: T-3272 for the digits, T-3364 for the move to the math).
- [x] A new public probe case under eval/suites/probes/cases/ (one milestone tag, M9; decision 0057), filled by the
      oracle (excel-graph), reproduces the bug; its cells are listed in this file before the fix. Include the inputs in
      Context and points on both sides of the failing region.
- [ ] After the fix every cell of that case is within 15 significant digits of Excel's value, with the right type and
      error (decision 0083's rule). Not met in full; handed off: every type and error now matches, and the numbers left
      off are A25 (df 5E6, Excel 1.5E-14 off the math: T-3294), A205-A289 (T-3409) and A301-A351 (T-3272, T-3364).
- [x] No gated cell regresses (`make eval-baseline` refuses regressions; decision 0083's ulp allowance only as it
      states).
- [x] Gate A envelope on a fresh sweep of its own (decision 0188 R2, R4.2): a new probes-numeric case, filled, with
      points drawn log-uniform over the function's valid domain and stratified around the failing region, scored
      against a 50-digit mpmath reference. The Outcome reports, per stratum, the points within 15 digits of Excel and
      the points within 1e-9 relative of the math (absolute 1e-300 near 0): 0 cells where one side is an error and the
      other a number, and 0 points outside both, except 3 (A122, A127, A155: `CHISQ.INV.RT` at about 1E-230, df 2.4E9
      to 7.2E9). Those 3 sit on the existing R3 closed-form branch (shape 1E9 and up, decision 0188 R3). They are
      unchanged base to head (ours is Excel's closed form, 2.66E-4 from the math there) and are owned by T-3364. This is a floor only: the target stays Excel's value.
- [x] The rule (what Excel does, what was wrong) is in docs/excel-semantics/ in the same PR.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Last-digit (bit) parity on cells already within 15 digits: Phase 1b (decision 0188 R5).
- Fitting to Excel's bits; if a fit is needed it follows decision 0188 R4 (hold-out split, fresh sweep, `FITTED` tag).

## Visible-parity stage
Decision 0083 and 0114 (`accuracy`): done when every cell this task owns is within half a unit of the 15th significant
digit of Excel's value with the right type and error, and the fresh sweep shows the Gate A envelope (decision 0188 R2).
Bit-level residue moves to a follow-up with `defer: phase-1b`.

## Outcome
- **Root cause.** One gamma inversion shares one budget of 2^19 series terms (directive 5). From about shape 5E6
  (df 1E7) to 1E8, statrs' search and the refinement take over 140 cdfs, each about 7.3 sqrt(a) terms near the median.
  So the budget ran out and the quantile was NaN (`#NUM!`) wherever the smaller tail is the lower one. From shape 1E8
  every cdf was already Temme's expansion. Fix: from shape 1E5 (`QUANTILE_TEMME_SHAPE_MIN`, incomplete_gamma/mod.rs)
  the quantile's cdfs are Temme's (one term each) and it halves the whole range, as it already did from 1E9.
  GAMMA.INV and GAMMAINV share the path.
- **Band lower edge (same region, df 2E9 to 1E10).** The 2026-10-31 fill of probes/t3350-gamma-inv-band-lower-edge
  showed the old fit `offset - z < 39` was wrong in 21 of the 549 huge-shape quantile cells in eval/, with an error
  against a number either way (the 549 include the probe itself; the hold-out record below splits them). It is replaced by ln P(a, x0) - ln p < 205.3 (`BAND_LOWER_EDGE`), which reads as
  Excel's Newton steps giving up after about 200 (UNVERIFIED as the algorithm). That rule separates 548 of the 549;
  the miss, `GAMMA.INV(1E-80,1E10,1)`, goes to T-3396. The cut was fitted on the probe only, and the fresh sweep is
  its hold-out: sweep A16, `#NUM!` in Excel, is now `#NUM!` here too.
- **Hold-out record for the lower-edge cut (decision 0188 R4, reviewer check 7).** `BAND_LOWER_EDGE` = 205.3 has one free
  parameter, the threshold on d = ln P(a, x0) - ln p, with the form fixed beforehand (the cdf ratio). It was chosen
  from the training probe alone: the probe pins the edge in (204.9, 205.8), and 205.3 lies inside that interval.
  The other cells were scored only after it was set.

  | split | cells | old cut (o - z < 39) | new cut (d < 205.3) | near the edge | nearest value / `#NUM!` |
  |---|---|---|---|---|---|
  | train: probes/t3350-gamma-inv-band-lower-edge | 160 (154 band-relevant) | 134/160 | 159/160 | 37 within 30 of 205.3 | value 204.9 (A72); `#NUM!` 205.8 (A11), miss A122 at 184.2 |
  | test: pre-existing huge-shape cells (t2982, t3070, others) | 117 band-relevant | 117/117 | 117/117 | 12 within 30 | value 204.9 (t3070 A30, the same input as train A72); `#NUM!` 225.2 (t3070 A44) |
  | test: fresh sweep probes-numeric/t3350-r4-chisq-inv-sweep | 4 band-relevant | 3/4 | 4/4 | 1 (A16) | value 171.4 (A27); `#NUM!` 210.6 (A16) |

  - "Band-relevant" means a huge-shape quantile inside the band's upper edge whose start's density does not underflow.
    The other 6 training cells are `CHISQ.INV` at df above 1E10, `#NUM!` on both sides by the df limit.
  - Acceptance margin: the edge's uncertainty in training is 0.9 wide (204.9 to 205.8). On the held-out cells the
    nearest value is 0.4 below 205.3 and the nearest `#NUM!` 5.3 above it (A16).
  - Honestly: this margin was not set in advance, and only A16 tests the edge on an input the fit did not see. The
    nearest held-out value is the training input again, and the nearest held-out `#NUM!` is 20 above the edge. The
    pre-existing cells do not tell the two cuts apart (both 117/117); only A16 does (old cut a value, Excel and the
    new cut `#NUM!`).
  - Out-of-range policy: the cut is pinned at shapes 1E9 to 7E9 and measured from 1E9 to 1E10. At 7E9 to 1E10 its form
    is known wrong: there the start lies above the mean, and A122 `GAMMA.INV(1E-80,1E10,1)` was right under the old
    cut (`#NUM!`) but is now a value where Excel gives `#NUM!`. Falling back to the math or to the start outside the
    pinned range would not help, because the cut decides between a value and an error, not between two values: there
    is no "safer" answer to fall back to, and either side is just as wrong for that cell. T-3396 owns the edge there,
    and T-3409 (a model of Excel's search) may replace the cut altogether.
  - Net: on the lower-edge probe, error-against-number cells fell from 26 to 1 (21 fixed by the cut, A122 broken by
    it, 6 already right). On the fresh sweep they fell from 30 to 0 (review's base-to-head count; almost all from the
    budget fix, 1 (A16) from the cut).
- **Probe cells** (probes/t3350-chisq-inv-huge-df): all 46 cells listed in Context are now within 4 ulps of Excel
  (passed 168 -> 181 bit-exact, `#NUM!` mismatches 46 -> 0). The visibly-off cells left are not this task's bug:
  - A25 `CHISQ.INV(1E-300,5E6)`, df 5E6: ours is the math, Excel is 1.5E-14 off it. Owner T-3294.
  - A205-A289 `CHISQ.INV.RT(1E-300,df)`, df 5E6 to 1E9: Excel returns its closed-form start, 2.6E-4 from the math.
    Ours is the math. These cells are not in R3's closed-form case (its region starts at shape 1E9), and the owner
    ruled not to widen it, so Excel's value is the target there: T-3409.
  - A301-A351, df 2E9 and up: the known closed form, about 1E-13 from Excel's value. Owners T-3272 (the digits) and
    T-3364 (the move to the math).
  The lower-edge probe went from 98 to 120 bit-exact and from 26 error-vs-number cells to 1 (A122, T-3396).
- **Gate A floor on the fresh sweep** (probes-numeric/t3350-r4-chisq-inv-sweep: 240 points, seed 3350; filled by
  excel-graph and scored by docs/research/t3350/sweep.py against a 50-digit mpmath reference: dps 60, Newton to
  1E-52 in log x). "Both" is within 15 digits of Excel and within 1E-9 of the math; "math only" is within 1E-9 of
  the math only.

  | stratum | points | both | both errors | math only | outside |
  |---|---|---|---|---|---|
  | CHISQ.INV full | 40 | 35 | 3 | 2 | 0 |
  | CHISQ.INV region | 50 | 50 | 0 | 0 | 0 |
  | CHISQ.INV region near 1 | 30 | 30 | 0 | 0 | 0 |
  | CHISQ.INV.RT full | 40 | 33 | 4 | 0 | 3 |
  | CHISQ.INV.RT region | 50 | 50 | 0 | 0 | 0 |
  | CHISQ.INV.RT region near 1 | 30 | 30 | 0 | 0 | 0 |

  - 0 cells have an error on one side and a number on the other.
  - The 2 "math only" points are small df (A4 at df 7, A38 at df 147), where Excel is 2E-15 off: Phase 1b.
  - The 3 "outside" points, A122, A127 and A155, are `CHISQ.INV.RT` at about 1E-230 and df 2.4E9 to 7.2E9. They lie
    on the existing R3 closed-form branch (shape at least 1E9): ours is Excel's closed form, 2.66E-4 from the math.
    The move to the math is T-3364.
- **Eval delta.** No regressions and no fixed gated cells (no gated cell covered the region). eval/baseline/probes.txt
  gains the two new probe cases (231 new-coverage lines, mostly cells within 4 ulps). Baselines were recorded with a
  release binary verified to hold this branch's engine: touched sources, `-v` build showing this worktree's
  rustsheet_engine, the binary copied into the worktree, and `show` on the new probes.
- Follow-ups: T-3396 (lower edge at shape 1E10), T-3409 (P1: model Excel's search at a huge shape. The owner ruled
  not to widen R3, so the target at A205-A289 is Excel's value).
- Review round 1: merged origin/main. T-3131 had replaced statrs' search with a Wilson-Hilferty start; this branch keeps
  that start and adds Temme cdfs from shape 1E5, since even from the start the series cdfs would use up the budget
  from about shape 5E6.
