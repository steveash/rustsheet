---
id: T-2715
title: "CEILING.MATH and FLOOR.MATH are 0 when number/significance is below the smallest normal (calipers E169, F169, E183, F183)"
milestone: M5
lane: functions-math
status: done
priority: P2
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
Each function of the CEILING/FLOOR family returns Excel's value when number/significance is below the smallest normal
double. On the calipers goldens Excel gives 0, and the engine rounds the tiny ratio up to one step. The probe decides
which members of the family behave this way.

## Context
Found by the M5 planner run of 2026-10-25 (calipers fetched, `rustsheet-eval run --suite calipers`). No open task names
these cells.

In `calipers/roundtrip/formulas_math_basic/golden.xlsx` (sheet `Math Tests`) and its twin
`calipers/roundtrip/formulas_datetime/golden.xlsx` (sheet `Sheet2`), C10 is `=10^(-307)` and C11 is `=-10^(-307)`.
8 cells, visibly off (the 4 below in each workbook):

| cell | formula | Excel (desktop golden) | ours |
|---|---|---|---|
| E169 | `CEILING.MATH($C$10,10)` | 0 | 10 |
| F169 | `CEILING.MATH($C$10,100)` | 0 | 100 |
| E183 | `FLOOR.MATH($C$11,10)` | 0 | -10 |
| F183 | `FLOOR.MATH($C$11,100)` | 0 | -100 |

Same rows, already matching: `CEILING.MATH($C$10,1)` is 1, `CEILING.MATH($C$10,0.01)` is 0.01 and
`FLOOR.MATH($C$11,0.01)` is -0.01. In those the ratio (1E-307, 1E-305) is normal. In the failing cells the ratio
(1E-308, 1E-309) is subnormal. So the likely rule is the division flush of numbers.md "Subnormal results become 0" (`/`
flushes a subnormal quotient), applied to the ratio before it is rounded up. But `ROUNDUP($C$10,-2)` is 100 in the same
golden (C143), so the ROUND family does not flush that way, and the rule must not be assumed for every rounding function.

The engine today (scratch run of the same formulas, 2026-10-25): `CEILING(1E-307,10)`, `CEILING.PRECISE`, `ISO.CEILING`
and `ECMA.CEILING` give 10; `FLOOR(-1E-307,10)` and `FLOOR.PRECISE` give -10; `MROUND(1E-307,10)` gives 0. Code:
`snap_ratio` in crates/rustsheet-engine/src/functions/math_and_trigonometry/ceiling_floor.rs returns a subnormal ratio
unflushed. `fn_ceiling_math` and `fn_floor_math` are in the same file; `arithmetic::flush_subnormal` is the existing helper.

The golden is desktop evidence, so probe first with excel-graph (`excel-probe` skill). If Excel for the web disagrees,
record a desktop-only rule and file an M14 follow-up instead of matching the golden (decision 0039).

## Acceptance criteria
- [x] A probes case tagged M5 (excel-graph) covers CEILING, CEILING.MATH (both modes), CEILING.PRECISE, ISO.CEILING,
      ECMA.CEILING, FLOOR, FLOOR.MATH (both modes), FLOOR.PRECISE and MROUND. Numbers are ±1E-307, ±2^-1022 and ±1E-300.
      Significances put the ratio just above, at and below 2^-1022 (for example 1, 4, 4.5, 10, 1E+10). ROUNDUP and
      ROUNDDOWN of the same numbers are controls (moved to their own case, probes/t2715-roundup-near-smallest-normal,
      owned by T-2756; see Outcome).
- [x] The engine matches every cell of the probe. The 8 calipers cells above pass (`rustsheet-eval run --suite calipers`
      after `rustsheet-eval fetch --suite calipers`), and their lines leave eval/baseline/calipers.txt
      (`rustsheet-eval update-baseline --suite calipers`) with no new known failure. (The cells pass; the calipers.txt
      lines are deferred to T-0463's ratchet, see Outcome.)
- [x] numbers.md gets an entry for the rule, per function, with the evidence and `Status: VERIFIED`.
- [x] `make check` green; reviewers approve.

## Visible-parity stage
`semantics`: a semantic rule (a flush of the ratio) causes a wrong number. The task is done when every owned cell has
Excel's value.

## Out of scope
- MOD on the same inputs (T-2714).
- The 16th-digit ratio ties (T-2652 done, T-2687).

## Outcome
- Probe t2715-ceiling-floor-subnormal-ratio (excel-graph, 2026-10-25, M5, 76 pairs × 11 functions + a ratio column):
  every CEILING/FLOOR variant (CEILING, CEILING.MATH both modes, CEILING.PRECISE, ISO.CEILING, ECMA.CEILING, FLOOR,
  FLOOR.MATH both modes, FLOOR.PRECISE) is 0 exactly when number/significance is subnormal; MROUND already matched.
  `snap_ratio` now flushes the ratio with `arithmetic::flush_subnormal`. numbers.md "CEILING and FLOOR of a subnormal
  ratio are 0".
- Found in passing: ROUNDUP(±2^-1022, d) is 0 while ROUNDUP(±1E-307, d) is one step. The controls are their own case
  (probes/t2715-roundup-near-smallest-normal, M2, 10 new-coverage known failures), owned by T-2756.
- calipers: E169, F169, E183 and F183 pass in both workbooks. `update-baseline --suite calipers` still refuses on
  the 36 pv_goldens cells (pv23_007/008/009/056) that T-2714 found failing on `main` too (T-2708, T-0463), so the
  eight `F` lines leave calipers.txt with T-0463's ratchet.
- Eval delta: probes +2 cases (1,184 cells; 1,174 pass, 10 new-coverage known failures in the ROUNDUP case), 0
  regressions, 0 fixed elsewhere.
