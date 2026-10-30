---
id: T-3272
title: "Gamma quantile closed form and band crossings at a huge shape: the last digits (T-3070 rest)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-3070]
---

## Goal
The gamma quantiles from a shape of 1E9 (GAMMA.INV, GAMMAINV, CHISQ.INV, CHISQ.INV.RT, CHIINV) give Excel's last
digits: the Wilson-Hilferty closed form's and the band's true-quantile crossings.

## Context
- T-3070 gave every probed cell Excel's type and, but for one, its 15 digits (docs/excel-semantics/statistics.md
  "GAMMA.INV and CHISQ.INV at a huge shape"; `incomplete_gamma/huge_shape.rs`). Left: the closed form is a fit (s and
  three terms inside the cube, within 4 ulps from shape 1.5E10 to 1E300 but 9 to 33 ulps at p = 1E-100 in the right
  tail at 5E9 to 2E10: probes/t3070-gamma-inv-band A371 `CHISQ.INV.RT(1E-100,1E10)` 10005667524.01516 against Excel's
  10005667524.015223 is just visibly off, A214, A229, A110); the band's true quantiles are mostly 1 ulp off (the
  crossing's even end against Excel's search's own end, as `GAMMA.INV(0.5,1E9,1)`, incomplete-gamma-cap E19).
- Every known-failure line of probes/t3070-gamma-inv-band is this task's (decision 0136: the case named alone).
- probes/t2982-gamma-inv-huge-shape A1:A4, A7:A9, A11, A13, A16, A19, A22:A26, A29, A30, A34, A36:A41, A44, A60, A61,
  A63, A66, A68:A70, A72:A75, A77:A85, A89, A92, A93 (1 to 5 ulps); probes/incomplete-gamma-cap E5, E14, E19 (1 ulp).
- Finding Excel's actual search (the band's edges are fitted too: `BAND_UPPER_*`, `BAND_LOWER_EDGE`) would settle
  both; the probes give the cells to check it against.

## Acceptance criteria
- [ ] A371 within half a unit of the 15th digit; the other cells pass or are named with their ulp in the Outcome; no
      cell becomes visibly off.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Temme's cdf tails at the 1E8 cut (t2982 A144:A146) and E13 (T-0971's kernel accuracy).
