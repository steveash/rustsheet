---
id: T-3316
title: "Probe GAMMA.DIST's huge-shape #NUM! cuts one cell per workbook and pin them (T-3271 brackets)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
campaign: semantics
size: S
tier: opus
depends_on: [T-3271]
---

## Goal
GAMMA.DIST's two huge-shape `#NUM!` cuts sit where Excel's do, not merely at the smallest `#NUM!` probed so far.

## Context
- T-3271 (PR #2466; this task depends on it) puts both cuts at the smallest `#NUM!` Excel has shown:
  `HUGE_SHAPE_DENSITY_CUT` = 6.9187E+11 for (alpha - 1)·(-LN(x/beta)), A26's product (1E+9 - 1)·691.87, and
  `HUGE_SHAPE_CDF_CUT` = 1.85E+76 for the shape (`statistical/gamma.rs` `huge_shape_num_error`). It records the cut
  positions as UNVERIFIED in docs/excel-semantics/statistics.md "GAMMA.DIST at a huge shape". The brackets are
  density 6.9E+10 (0) to 6.9187E+11 (`#NUM!`) and cdf 1E+40 (0) to 1.85E+76 (`#NUM!`)
  (probes/t2435-dist-extremes A26, A92, A100, A101).
- excel-graph refused workbooks that held several of these cells (FileCorruptTryRepair, T-2982). Probe one cell per
  workbook and bisect each bracket. Also check whether the density cut depends on (alpha - 1)·LN alone, by varying
  x and beta at a fixed product, and whether the cdf cut depends on x/beta.
- Opus: the evidence is unclear, and the probing may need a workaround for the refusal.
- **Wrong-type cells, from T-3530's fit audit (probes-numeric/fit-audit-*, excel-graph; scored on main 4e2f5f3df after T-3350;
  docs/research/fit-audit/README.md)** (fit-audit-t3271-density-cut, 50 cells): the density cut's variable is not
  (alpha - 1)·(-LN(x/beta)). Excel is 0 at A43 `GAMMA.DIST(0.5,1E+15,1,FALSE)`, where the engine is `#NUM!`; and
  Excel is `#NUM!` at A9 `GAMMA.DIST(2.17173678289098E-300,1E+9,1,FALSE)` (product 6.9E11) and A48
  `GAMMA.DIST(1E-300,1E+9,1E-6,FALSE)` (6.77E11), where the engine is 0. The scale does not count: A45
  `GAMMA.DIST(1E-200,1E+9,1E+100,FALSE)` is 0. Every probed cell (with t2435 A25, A26, A92) fits `#NUM!` where
  x / shape is below a cut in (2.17E-309, 1E-308], else 0 (UNVERIFIED as a form: probe smaller shapes, x/shape
  between those values). That bracket is the subnormal range, so the cut may be "x/shape is not a normal double".
- The cdf cut (`HUGE_SHAPE_CDF_CUT` 1.85E76) is refuted too: fit-audit-t3271-cdf-cut (38 of 42 cells, filled one cell
  per workbook) has 21 wrong-type cells. Excel is `#NUM!` where the engine is 0 at A4, A6, A7, A9, A10, A11, A13, A14,
  A15, A16, A17 and A40 (`GAMMA.DIST(2,1E+60,1E+10,TRUE)` up to 1.8E76; `GAMMADIST` too). Excel is 0 where the engine
  is `#NUM!` at A28, A29, A32, A34, A35, A36, A37, A38 and A39 (beta = 1, shapes 1.85E76 to 1E200). Excel is not
  monotone in the shape: A12 (1E76) and A18 (1.84E76) are 0 between `#NUM!`s, and the refill agreed. So look for an
  intermediate of Excel's arithmetic that overflows, not for a cut.

## Acceptance criteria
- [ ] Each cut is pinned (or narrowed by at least 10x, with the remaining bracket recorded) by excel-graph probe cells; statistics.md is updated; the constants move if the evidence says so; `make eval-check` shows no regressions and the visible-parity lock does not rise.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The gamma inversions (T-3070, T-3272).
