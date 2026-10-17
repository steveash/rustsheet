---
id: T-1802
title: "Probe, then enter QUOTIENT, FACTDOUBLE and SERIESSUM's x, n, m (they refuse a range; probes/scalar-lift-open-a Y19-Y30)"
milestone: M9
lane: functions-math
status: in-progress
priority: P2
size: S
tier: opus
depends_on: [T-1770]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
QUOTIENT, FACTDOUBLE and SERIESSUM's x, n and m are `LiftArrays` entries in the SPEC-0011 table once an array probe shows they lift, and MULTINOMIAL, GCD, LCM and SERIESSUM's coefficients are not lifted.

## Context
SPEC-0011 open question 3 left these without an entry until probed. T-1777 measured (probes/scalar-lift-open-a Y19-Y30, Z19-Z30 the scalar controls): `ROWS(QUOTIENT(A1:A60,2))`, `ROWS(QUOTIENT(10,B1:B60))` and `ROWS(FACTDOUBLE(D1:D60))` are `#VALUE!`, and so are SERIESSUM's x, n and m over a range (Y24-Y26); whether they lift over an array is NOT probed. SERIESSUM's coefficients over a range is read whole (Y27, one value, `SERIESSUM(2,0,1,B1:B3)` = 7, -d W31). MULTINOMIAL, GCD and LCM give one result (ROWS 1; Y22, Y23, Y28-Y30), and only MULTINOMIAL's value was probed: `MULTINOMIAL(B1:B3,2)` = 59.999999999999986 (-d W30). T-1770 owns the table, so the entries must be added with its first consumers; the family is math.

## Acceptance criteria
- [ ] First probe (excel-graph, a case under probes/): `INDEX(QUOTIENT({7;9},2),2)`, `INDEX(FACTDOUBLE({5;6}),2)`, `INDEX(SERIESSUM({1;2},0,1,{1,2}),2)` with ROWS of each, before entering them as `LiftArrays`.
- [ ] The table has QUOTIENT, FACTDOUBLE (`LiftArrays`) and SERIESSUM (x, n, m `LiftArrays`, coefficients `Whole`); MULTINOMIAL, GCD and LCM have no lifted argument.
- [ ] probes/scalar-lift-open-a Y19-Y30 pass (the range refusals; ROWS 1 for MULTINOMIAL, GCD, LCM).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other functions' later arguments.
