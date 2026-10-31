---
id: T-3390
title: "GAMMA.DIST and GAMMADIST are #NUM! at a huge shape: the density at a tiny x/beta and the cdf near 1E+76 (8 probe and 2 gen-m9 cells, ours 0)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
GAMMA.DIST and GAMMADIST return Excel's `#NUM!` where Excel refuses a huge shape. Today ours is 0 there. The cases are
the density at a tiny x/beta with a shape of 1E+9 or more, and the cdf at a shape near 1E+76.

## Context
- The rule is recorded in docs/excel-semantics/statistics.md, "GAMMA.DIST at a huge shape: #NUM! for a density at a
  tiny x, for the cdf above about 1E+76 (T-2435)". Both cuts are bracketed but not pinned:
  - density: 0 at (alpha - 1)|LN(x/beta)| of 6.9E+10 (shape 1E+8 at x = 1E-300; shape 1E+9 at x = 1E-10 or 1),
    `#NUM!` at 6.9E+11 (shape 1E+9 or 1E+10 at x = 1E-300, beta 3);
  - cdf: 0 for shapes up to 1E+40 at x/beta = 2E-10, `#NUM!` at 1.85E+76.
- This is a semantic rule (an error kind), not a fit. It has moved from task to task: T-2435, T-1891, T-2982,
  T-3070. Each of those was working the gamma quantiles or the incomplete-gamma cap, so the rule never landed.
  T-3070 (`campaign: accuracy`, quantiles from shape 1E9) and T-2811 (`campaign: visible`, last digits of the T-2435
  probe) both list the cells below today, and T-3219 lists the two gen-m9 cells under "Type mismatches ... (other
  causes may own them; check first)". This task takes them over, so those tasks are left with their numeric work.
- Gap triage 2026-10-30d. `make eval` on origin/main ran, then `report.json` cells with a wrong type or error,
  checked against the open owners. These are the only gamma-distribution type mismatches at a huge shape:
  - probes/t2435-dist-extremes S!A25 `GAMMA.DIST(1E-300,1E+10,3,FALSE)`: Excel `#NUM!`, ours 0 (B25, `(..)=0`:
    Excel `#NUM!`, ours TRUE)
  - probes/t2435-dist-extremes S!A26 `GAMMA.DIST(1E-300,1E+9,3,FALSE)`: Excel `#NUM!`, ours 0 (B26 likewise)
  - probes/t2435-dist-extremes S!A101 `GAMMA.DIST(2,1.85E+76,1E+10,TRUE)`: Excel `#NUM!`, ours 0 (B101 likewise)
  - probes/t2435-dist-extremes S!A102 `GAMMADIST(2,1.85E+76,1E+10,TRUE)`: Excel `#NUM!`, ours 0 (B102 likewise)
  - gen-m9/2801c1c11c21e134 Calc!A241 `GAMMADIST(1E-300,10000000001,3,(IMCSC("2i")<Data!E$5))`: Excel `#NUM!`, ours 0
  - gen-m9/40425341c5034eed Calc!A265 `GAMMADIST(2,(IFS(TRUE,Data!E9,FALSE,Data!E$10)*(Data!$A4^42)),10000000000.5,"TRUE")`
    (shape 123456789.123 * 42^42, about 1.85E+76): Excel `#NUM!`, ours 0
- These neighbours pass today and must keep passing: probes/t2435-dist-extremes A27:A32 (0 at the smaller shapes),
  A33, A34 and A20:A24 (the numbers T-1891 settled; A23 and A24 have 1-ulp last digits that T-2811 owns).
- Code: crates/rustsheet-engine/src/functions/statistical/gamma.rs `gamma_dist_of` already has the other
  `#NUM!` rules (subnormal and overflowing x/beta, T-2738 and T-2982). The density goes through
  `tiny_ratio_density`, `gamma_density` and statrs' pdf; the cdf goes through `gamma_dist_cdf`.
- The brackets are a factor of 10 (density) and 1E+36 (cdf) wide, so a threshold picked inside them is a guess.
  Probe first with the `excel-probe` skill:
  - density at x = 1E-300, beta 3, shapes from 1E+8 to 1E+9 in steps of 1E+8;
  - density at shape 1E+9 for x from 1E-30 to 1E-300;
  - cdf at x/beta = 2E-10 with shapes 1E+40, 1E+50, 1E+60, 1E+70, 1E+75, 1E+76, 1E+77.
  Then look for the mechanism, for example a term that overflows to infinity or a loop bound, rather than a bare
  constant. Record the cut and its evidence in statistics.md.

## Acceptance criteria
- [ ] A probe case pinning both cuts lands with oracle values, and statistics.md states the rule, cites the cells and
      names the mechanism, or says plainly that none was found and the cut is a fitted constant.
- [ ] These 10 known-failure lines disappear, with no new line in any baseline: eval/baseline/probes.txt
      t2435-dist-extremes S!A25, S!B25, S!A26, S!B26, S!A101, S!B101, S!A102, S!B102; eval/baseline/gen-m9.txt
      2801c1c11c21e134 Calc!A241 and 40425341c5034eed Calc!A265.
- [ ] Every cell of the new probe passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- GAMMA.INV, CHISQ.INV, CHISQ.INV.RT and CHIINV at a huge shape or df (T-3070).
- The last digits of finite GAMMA.DIST values at a huge shape (T-2811).
- BETA.DIST's `#NUM!` near the double range (T-3104).

## Visible-parity stage
Decision 0083/0114: every cell this task owns has the wrong type (Excel's `#NUM!` against our 0, or `#NUM!` against
TRUE in the B companions), so the stage is `semantics`. The target is that each owned cell is the right type and
error. No bit-level work is left after that.
