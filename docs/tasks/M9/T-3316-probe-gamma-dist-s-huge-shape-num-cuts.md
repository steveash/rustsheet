---
id: T-3316
title: "Probe GAMMA.DIST's huge-shape #NUM! cuts one cell per workbook and pin them (T-3271 brackets)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-3271]
---

## Goal
GAMMA.DIST's two huge-shape `#NUM!` cuts sit where Excel's do, not merely at the smallest `#NUM!` probed so far.

## Context
- T-3271 put both cuts at the smallest `#NUM!` Excel has shown: `HUGE_SHAPE_DENSITY_CUT` = 6.9187E+11 for
  (alpha - 1)·(-LN(x/beta)), and `HUGE_SHAPE_CDF_CUT` = 1.85E+76 for the shape
  (`statistical/gamma.rs` `huge_shape_num_error`). The true cuts lie in two brackets that are UNVERIFIED in
  docs/excel-semantics/statistics.md "GAMMA.DIST at a huge shape": density 6.9E+10 (0) to 6.9187E+11 (`#NUM!`),
  cdf 1E+40 (0) to 1.85E+76 (`#NUM!`) (probes/t2435-dist-extremes A26, A92, A100, A101).
- excel-graph refused workbooks that held several of these cells (FileCorruptTryRepair, T-2982). Probe one cell per
  workbook and bisect each bracket. Also check whether the density cut depends on (alpha - 1)·LN alone, by varying
  x and beta at a fixed product, and whether the cdf cut depends on x/beta.
- Opus: the evidence is unclear, and the probing may need a workaround for the refusal.

## Acceptance criteria
- [ ] Each cut is pinned (or narrowed by at least 10x, with the remaining bracket recorded) by excel-graph probe cells; statistics.md is updated; the constants move if the evidence says so, and no cell becomes visibly off.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The gamma inversions (T-3070, T-3272).
