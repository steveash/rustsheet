---
id: T-1802
title: "QUOTIENT, FACTDOUBLE and SERIESSUM's x, n, m refuse a range; MULTINOMIAL, GCD and LCM are aggregates (probes/scalar-lift-open-a Y19-Y30)"
milestone: M9
lane: functions-math
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-1770]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
QUOTIENT, FACTDOUBLE and SERIESSUM's x, n and m are `LiftArrays` entries in the SPEC-0011 table, and MULTINOMIAL, GCD, LCM and SERIESSUM's coefficients are not lifted.

## Context
SPEC-0011 open question 3 left these without an entry until probed. T-1777 measured (probes/scalar-lift-open-a Y19-Y30, Z19-Z30 the scalar controls): `ROWS(QUOTIENT(A1:A60,2))`, `ROWS(QUOTIENT(10,B1:B60))` and `ROWS(FACTDOUBLE(D1:D60))` are `#VALUE!`, and so are SERIESSUM's x, n and m over a range (Y24-Y26); they lift over an array like the other Analysis ToolPak functions. SERIESSUM's coefficients over a range is read whole (Y27, one value, `SERIESSUM(2,0,1,B1:B3)` = 7, -d W31). MULTINOMIAL, GCD and LCM take a range as an aggregate: `ROWS` is 1 (Y22, Y23, Y28-Y30), and `MULTINOMIAL(B1:B3,2)` = 59.999999999999986 (-d W30). T-1770 owns the table, so the entries must be added with its first consumers; the family is math.

## Acceptance criteria
- [ ] The table has QUOTIENT, FACTDOUBLE (`LiftArrays`) and SERIESSUM (x, n, m `LiftArrays`, coefficients `Whole`); MULTINOMIAL, GCD and LCM have no lifted argument.
- [ ] probes/scalar-lift-open-a Y19-Y30 pass (arrays lifting aside).
- [ ] `make check` green; reviewers approve.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other functions' later arguments.
