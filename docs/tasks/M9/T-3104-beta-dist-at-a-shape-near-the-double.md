---
id: T-3104
title: "BETA.DIST at a shape near the double range: bpser #NUM! once 1/B overflows, and bgrat/bfrac values where ours are #NUM!"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
BETA.DIST at a shape near the top of the double range matches Excel on probes/t3007-beta-dist-ln-x-overflow: `#NUM!`
where bratio takes bpser with both shapes above 1 and 1/B(a, b) overflows, and Excel's 0 or 1 where today's kernel
answers `#NUM!` on bgrat and bfrac.

## Context
- statistics.md "Incomplete beta at a shape near the double range" (T-3007) has the rule and the cells.
- The 11 off cells of probes/t3007-beta-dist-ln-x-overflow (excel-graph 2026-10-29):
  - bpser for both shapes > 1, Excel `#NUM!`, ours 0: A5 `BETA.DIST(0.1,1E+290,2,TRUE)`, A15 (a = 1.4E+154, b = 2),
    A16 (1E+200, 2), A18 (1E+290, 1.5), A19 (1E+290, 9); and the density A20 `BETA.DIST(0.1,1E+200,2,FALSE)`;
  - ours `#NUM!` where Excel has a value: A3 `BETA.DIST(0.9,1E+290,0.5,TRUE)` 0 and A4 `BETA.DIST(0.1,0.5,1E+290,TRUE)`
    1 (bgrat), A12 `BETA.DIST(0.1,1E+200,1E+200,TRUE)` 0 (bfrac), A21 `BETA.DIST(0.1,2,1E+200,TRUE)` 1 and A22
    `BETA.DIST(0.9,1E+200,2,TRUE)` 0 (L140 bup+bgrat).
- The bpser cut sits between B(a, 2) at a = 1.3E+154 (0) and 1.4E+154 (`#NUM!`), and at a = 1E+290 between b = 1.01 (0)
  and 1.5 (`#NUM!`): probe sqrt(DBL_MAX) = 1.3407807929942596E+154 and its neighbours, and a shape b off 2, to tell
  1/B in doubles from a^b before coding the cut. `bratio::ported_tails` returns `None` for "today's kernel", so a
  `#NUM!` from a ported branch needs its own signal.
- Excel for the web refuses a workbook holding BETA.DIST with a shape of 1E+295 or more: keep such cells out of cases.

- T-2528 (large-shape expansion, a >= 1E6, b < 40) fixed probes/t3007-beta-dist-ln-x-overflow S!A3, A4, A21 and A22;
  7 of the case's 27 cells are left.

## Acceptance criteria
- [ ] The 11 cells above match Excel, or the ones left are recorded with their reason in statistics.md; no gated
      cell regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Shapes of 1E+295 and above (no excel-graph value); the bgrat/bfrac ports themselves (T-2508, T-2509, T-2510).
