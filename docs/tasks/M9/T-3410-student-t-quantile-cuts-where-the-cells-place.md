---
id: T-3410
title: "Student-t quantile cuts where the cells place them: T_TAIL_P, T_STATRS_MIN_P and T_CENTRE_C (R4 guards)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
The three Student-t quantile cuts in crates/rustsheet-engine/src/functions/statistical/t_dist.rs sit where Excel's
cells place them, each chosen under decision 0188 R4's guards, with its `FITTED` tag updated.

## Context
T-3395 moved each cut in full eval runs (a temporary env override, reverted; suites probes-numeric, probes, gen-m9,
ironcalc-calc) and scored every changed cell by its ulp distance to Excel (counts are in the doc comments):
- `T_TAIL_P` (1E-5): the tail path beats statrs on net at every q tried from 5E-11 to 5E-4. A cut of 1E-3 makes 21
  cells closer (gen-m9 46ac3ef5e61583ca A293 `TINV(0.001,9)` bit-exact) and 5 further off. A cut of 1E-2 breaks
  5cebac1f02bf30f2 A60 and m9-t-inv-2t A56 (q 5E-3). So the cells favour a cut in 5E-4..5E-3.
- `T_STATRS_MIN_P` (1E-8): only T_DIST.xlsx E66 (`T.INV(0.9999999,10)`, q 1E-7) discriminates, and it needs statrs.
  Below the cut statrs beats bisection on net (to 1E-11: 19 closer, 11 further; to 1E-20: 33 and 19, a4090567758520df
  A201 `T.INV(1E-15,10)` bit-exact). The bisection exists for statrs' hang (1E-100 at df 1000) and panic (about
  1E-200), so the lowest safe cut needs a bound on statrs' time and no panic, not only scores.
- `T_CENTRE_C` (0.075): cells bracket it in 0.05..0.1, and t3349-t-inv-2t-sweep A98 (c 0.0787) and A195 (c 0.0985)
  come closer from the centre path, so the cells favour 0.0985..0.1.
None of these moves passed R4: the scores above are in-sample, on every cell.

## Acceptance criteria
- [ ] For each cut: a hold-out split fixed before scoring (R4.1), train and hold-out counts for the chosen cut and the
      current one, a fresh log-uniform sweep after the freeze (R4.2) and the Gate A envelope result; the chosen cut
      beats the current one on hold-out, or the cut stays and the Outcome says why.
- [ ] A lower `T_STATRS_MIN_P` keeps statrs' search bounded and panic-free at every df up to `T_DF_MAX`
      (a unit test at the new cut's worst case).
- [ ] The `FITTED` tags state the new counts and ranges; `make eval-baseline` records the fixes; no regression.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The df bands and extrapolated fits of T-3364.
