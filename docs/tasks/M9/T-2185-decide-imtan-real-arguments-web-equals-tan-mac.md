---
id: T-2185
title: "Decide IMTAN real arguments: web equals TAN, Mac cache N6 does not"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
IMTAN and IMCOT of a real argument match Excel, or the web/desktop split is recorded as a decision.

## Context
probes/eng-imtan-overflow A20:A25 (excel-graph): IMTAN of a real equals TAN's value at 6 points (`IMTAN("1.57079632")` = 147169271.760705, `IMTAN("5")` = -3.38051500624659), but ironcalc-calc/ENGINEERING/COMPLEXs.xlsx N6 (Mac cache) has IMTAN(5) = -3.38051500624658, which the double-angle form gives. Ours uses the double-angle form except where its denominator is 0, so A20, A21, A24, A25 fail (near pi/2 the form cancels). Using `real_tan` for every y == 0 fixes the web cells but regresses N6; needs a decision (web golden, desktop to M14 per decision 0039). Code: `tan_parts` in complex.rs. See engineering.md 'IMTAN and IMCOT use the double-angle form'.

## Acceptance criteria
- [ ] Real-argument IMTAN decided and implemented; the probe cells A20:A25 pass or are recorded as web/desktop deviations.
- [ ] `make check` green; reviewers approve.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
