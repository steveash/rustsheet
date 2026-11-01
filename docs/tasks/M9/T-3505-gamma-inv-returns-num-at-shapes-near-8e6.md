---
id: T-3505
title: "GAMMA.INV returns #NUM! at shapes near 8E6 where Excel returns a number"
milestone: M9
lane: functions-stat
status: done
priority: P0
size: M
tier: opus
depends_on: []
campaign: semantics
---

## Goal
GAMMA.INV, GAMMAINV, CHISQ.INV, CHISQ.INV.RT and CHIINV return Excel's number, not `#NUM!`, at the shapes below the
huge-shape band (1E9, T-3070) where the engine gives up today.

## Context
- Held-out blind sweep (excel-graph, filled 2026-10-31): `GAMMA.INV(0.5,7990464.81598859,1)` is #NUM! in the engine
  and 7990464.482655259 in Excel. A wrong type is a semantic failure. The other 135 GAMMA.INV cells of the sweep at
  shapes 20 to 1E9 with p from 0.01 to 0.5 were fine.
- The shape lies below `HUGE_SHAPE_MIN` (incomplete_gamma/huge_shape.rs), so the cause is in `gamma_quantile`
  (incomplete_gamma/mod.rs): suspects are the one `MAX_TERMS` term budget its cdfs share, the bracket, or another
  directive-5 cap.
- Root cause: `gamma_quantile` gives all the cdfs of one inversion a single budget of `MAX_TERMS` (2^19) terms
  (directive 5). Where the smaller tail's crossing lies below x = a + 1 the cdf is the power series, about 7.3 sqrt(a)
  terms near the median, and the refinement from the Wilson-Hilferty start takes about 25 cdfs (its first step,
  2^-30 of the start, spans about 2^23 doubles, so about 23 halvings). From about 7E6 that passes 2^19 terms, the
  inversion is NaN and the cell `#NUM!`. From 1E8 every cdf is Temme's (one term), so the band ends there. Where the
  crossing lies above a + 1 (the upper tail is the smaller one) the cdf is the continued fraction, which is cheap.
- Failing region on main 3003a898b (local sweep, release build: 81 shapes log-spaced from 1E5 to 1E9 by 14 p from
  1E-300 to 1-1E-10, GAMMA.INV(p,a,1), CHISQ.INV(p,2a), CHISQ.INV.RT(p,2a); 3,402 cells), all `#NUM!`:
  - shape from 7.94E6 (7.08E6 still a number) up to 8.91E7 (1E8 a number), and only where the lower tail is the
    smaller one: GAMMA.INV and CHISQ.INV with p <= 0.5, CHISQ.INV.RT and CHIINV with p >= 0.5;
  - the lower shape edge moves up as the quantile moves into the tail (the series needs fewer terms there):
    p = 0.5 from 7.9E6, 0.3 from 1.0E7, 0.1 from 1.1E7, 0.01 and 0.001 from 1.6E7, 1E-10 from 3.5E7, 1E-30 from
    7.9E7; never at 1E-100 or 1E-300 below 1E8. For CHISQ.INV.RT the mirror: 0.7 from df 2E7, 0.9 from 2.2E7, 0.99
    from 3.2E7, 0.999999 from 5E7, 1-1E-10 from 7.1E7;
  - GAMMAINV and CHIINV are the same kernel (GAMMA.INV and CHISQ.INV.RT) and fail at the same points
    (probes/t3505-gamma-inv-budget-band).
  - The scale does not matter (the search is at shape a, x times the rate).
- Overlap: T-3350 (PR #2597, approved and marked ready for the merge train on 2026-10-31) fixes the same root cause
  from the CHISQ.INV side: from shape 1E5 the quantile's cdfs are Temme's, one term each (`QUANTILE_TEMME_SHAPE_MIN`),
  and it retires the DEVIATION in statistics.md "Incomplete gamma: per-call cap and lifting".

## Acceptance criteria
- [x] The failing region is measured and recorded (shape and p ranges, per function).
- [x] A probe case under eval/suites/probes/cases/ (tag M9), filled by the oracle (excel-graph), covers the region and
      points on both sides of it.
- [x] Every cell of that case has Excel's type, and its numbers are within 15 significant digits of Excel's (decision
      0083's rule), or the residue is listed.
- [x] Per-call work stays bounded (directive 5): no cap removed.
- [x] The finding is recorded in docs/excel-semantics/statistics.md.
- [x] No gated regression and no cell newly visibly off (decisions 0083, 0111); main-vs-HEAD per-cell comparison in
      the Outcome.
- [x] `make check` green; reviewers approve.

## Out of scope
- The huge-shape band from 1E9 (T-3070) and the last-digit work of T-3269, T-3285 and T-3311.

## Handoff
- Done: root cause found (see Context: the inversion's one 2^19-term budget runs out on series cdfs from about shape
  7.9E6 to 8.9E7 where the lower tail is the smaller one, about 25 cdfs from the Wilson-Hilferty start's 2^-30 step).
  Failing region measured (Context). Probe probes/t3505-gamma-inv-budget-band (525 cells, oracle-filled excel-graph
  2026-10-31; main: 120 of them `#NUM!`).
- WIP fix in this branch (incomplete_gamma/mod.rs, special.rs): from shape 3E5 (`NARROW_SEARCH_SHAPE_MIN`, derived:
  129 cdfs * 7.4 sqrt(a) fits 2^19 below it) to 1E8 the search starts at the crossing of Temme's expansion
  (`temme_crossing`) with a first step of 2^-50 (`refine_increasing_near`), about 5 cdfs. The budget and cap are
  unchanged; the result is still the crossing of the accurate series cdf. Measured with release copies (not
  `make check`): the full eval run main-vs-HEAD changed exactly 120 cells, all in the new probe, `#NUM!` to 50 Pass and
  70 within 4 ulp; no other cell in any suite changed value. A 3,402-cell local sweep (shapes 1E5-1E9): every cell
  that was a number on main is bit-identical; every `#NUM!` below 1E9 is fixed. Six probe cells at p = 1E-300
  (A1, A13, A133, A145, A85, A97) are unchanged from main and up to 2.2E-14 from Excel (the engine is the true
  quantile per mpmath; Excel is off), visibly off at A13, A85, A97: not caused by this task, a follow-up (T-3294 area).
- Overlap: T-3350 (PR #2597, ready-to-merge in the train) fixes the same root cause by making the quantile's cdfs
  Temme's from shape 1E5, and retires the statistics.md DEVIATION. Its binary scores the probe identically (same
  verdicts; 1 cell, A103, differs by an ulp, both within 4 ulp). Recommended: once #2597 merges, merge origin/main,
  take main's side in incomplete_gamma/mod.rs and special.rs (drop this WIP fix), keep the probe as added evidence,
  `make eval-baseline` for its p = 1E-300 cells, note it in statistics.md, then review. If #2597 does not land,
  finish this fix instead: unit test (`gamma_quantile(7990464.81598859,1.0,0.5,Lower)` = 7990464.482655259 within 4
  ulp), retire the DEVIATION in statistics.md "Incomplete gamma: per-call cap and lifting", `make check`.
- Left: statistics.md entry, unit test, `make check`, baseline, Outcome, review.

## Outcome
Resolved by T-3350 (PR #2597, merged 2026-10-31), which fixed the same root cause: from shape 1E5 every cdf of a
gamma inversion is Temme's expansion, one term each, so the shared 2^19-term budget no longer runs out. This branch's
own WIP fix (a Temme-crossing start from shape 3E5) was dropped in the merge of origin/main, and the PR has no engine
changes. It adds:
- the probe probes/t3505-gamma-inv-budget-band (525 cells, excel-graph): GAMMA.INV, GAMMAINV, CHISQ.INV,
  CHISQ.INV.RT and CHIINV across the old failing band at shapes 2.9E5 to 1E8, at scale 1 and at shapes that are not a
  whole df/2, which T-3350's probe did not cover;
- unit test `gamma_inv_at_scale_one_in_the_budget_band_is_excel_s` (incomplete_gamma/tests.rs): A104, A234 (the
  held-out sweep's `GAMMA.INV(0.5,7990464.81598859,1)` = 7990464.482655259), A231 and A252, each within 4 ulp;
- a note in statistics.md "GAMMA.INV and CHISQ.INV at a huge shape".

Main against the probe: 525 of 525 cells have Excel's type (on main before T-3350, 120 were `#NUM!`). 228 are
bit-exact, 291 within 4 ulp, and 6 within Gate A but not within 4 ulp. 5 of those 6 are visibly off, all GAMMA.INV at
p = 1E-300 (A1, A13, A85, A97, A145). There Excel is not the true quantile, so they belong to T-3294, whose Context
lists them. The owner approved recording them on 2026-11-01. The baseline change is additive only: 297 new-coverage
known failures in eval/baseline/probes.txt, and one visible-parity line (5) for the new case. No other cell in any
suite changed: the branch has no engine code. `make eval-check`: 0 regressions.

