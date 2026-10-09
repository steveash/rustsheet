---
id: T-1249
title: "FACT and FACTDOUBLE differ from Excel in the last digits for large n (FACT(170), FACTDOUBLE(297..300))"
milestone: M5
lane: functions-math
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
FACT and FACTDOUBLE return Excel's exact double for large n (bit-identical or within the documented tolerance).

## Context
Probes/fact-large-arguments (T-1233) A6 `FACT(170.9)` expects 7.257415615308004e306, engine gives 7.257415615307994e306; A9 `FACTDOUBLE(300)` expects 8.154414069380598e307, engine 8.154414069380588e307 (also 297, 298 likely). Same family as MULTINOMIAL inexactness (T-1190): Excel's factorial is not the left-to-right product (probably a Gamma/table-based computation). Needs numeric fitting against the probe values; opus. Code: `fn_fact`, `fn_factdouble` in functions/math_and_trigonometry/mathematical.rs.

## Acceptance criteria
- [ ] The new known failures of probes/fact-large-arguments leave eval/baseline/probes.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
