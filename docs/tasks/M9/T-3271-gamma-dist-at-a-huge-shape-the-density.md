---
id: T-3271
title: "GAMMA.DIST at a huge shape: the density's #NUM! cut at a tiny x and the cdf's above 1E76 (T-2435 rules, T-3070 rest)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-3070]
---

## Goal
GAMMA.DIST and GAMMADIST give Excel's `#NUM!` for a density at a huge shape and a tiny x / beta, and for the cdf
above a shape of about 1E76, where ours is 0.

## Context
- docs/excel-semantics/statistics.md "GAMMA.DIST at a huge shape: #NUM! for a density at a tiny x, for the cdf above
  about 1E+76 (T-2435)": the density cut lies between (alpha - 1)|LN(x/beta)| of 6.9E+10 (0) and 6.9E+11 (`#NUM!`);
  the cdf is 0 up to a shape of 1E+40 at x/beta = 2E-10 and `#NUM!` at 1.85E+76. Both cuts are bracketed, not
  pinned: probe them first (excel-graph refused workbooks holding `GAMMA.DIST(2,a,1E10,TRUE)` for a from 1E40 to
  1E100 and densities at shapes 1E10-1E11 at a tiny x, FileCorruptTryRepair, T-2982: try one cell a workbook).
- Cells (moved from T-3070, which did the gamma inversions):
  - probes/t2435-dist-extremes A25, A26, A101, A102;
  - gen-m9/2801c1c11c21e134 Calc!A241 `GAMMADIST(1E-300,10000000001,3,...)` (Excel `#NUM!`, ours 0);
  - gen-m9/40425341c5034eed Calc!A265 `GAMMADIST(2,...,10000000000.5,"TRUE")` (Excel `#NUM!`, ours 0).
- Split from T-3219, which lists the two gen-m9 cells among the type mismatches other causes may own: their cause
  is this huge-shape rule, not T-3219's small-shape prefactor.
- Code: `statistical/gamma.rs` `fn_gamma_dist` (density `tiny_ratio_density`/`gamma_density`, cdf `gamma_cdf`).

## Acceptance criteria
- [ ] The six cells above pass; the cuts are pinned by a probe (or bracketed and named UNVERIFIED) and recorded in
      statistics.md; no cell becomes visibly off.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The gamma inversions (T-3070, T-3272).
