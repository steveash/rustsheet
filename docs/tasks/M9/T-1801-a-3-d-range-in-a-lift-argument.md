---
id: T-1801
title: "A 3-D range in a Lift argument is #REF!, and in a LiftArrays argument #VALUE! (probes/scalar-lift-open-b, -d)"
milestone: M9
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-1770]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
A 3-D range (`Sheet1:Sheet2!A1:A2`) in a `Lift` argument gives `#REF!`, as Excel does, and in a `LiftArrays` argument `#VALUE!`.

## Context
SPEC-0011 open question 2 assumed `#VALUE!` in both, as `lift_with_cost` gives today. T-1777 (probes/scalar-lift-open-b W2-W11, -d W17-W29) measured: `ROWS(PMT(Sheet1:Sheet2!C1:C2,10,100))` and `ROWS(NORM.DIST(Sheet1:Sheet2!C1:C2,0,1,TRUE))` are `#REF!`, also for one cell per sheet (`PMT(Sheet1:Sheet2!C1,10,100)`, -d W21-W22), in a later argument (`PMT(0.1,3D,100)`, -b W6) and beside an array (`INDEX(PMT({0.1;0.2},10,3D),1)` and 2, -d W17-W18; `INDEX(PMT(3D,{10;20},100),1)`, -d W25). In EDATE (`LiftArrays`) the 3-D range is `#VALUE!`, alone (-b W4-W5, W7) and beside an array (elements 1 and 2 `#VALUE!`, -d W19-W20, W26). The shape beside an array is the array's (-b W8-W9: ROWS 2). `ISREF(3D)` is FALSE and `3D+0` is `#REF!` (-d W24, W27-W28), while `SUM(3D)` works (-b W1) and `ROWS(3D)` is `#VALUE!` (-b W12). Also `BITAND(13,3D)` (a `Lift` argument) is `#REF!` (-d W29). T-1770 (driver read phase) must account for it, and T-1372, T-1383, T-1290 for their families.

## Acceptance criteria
- [ ] probes/scalar-lift-open-b W2-W11 and probes/scalar-lift-open-d W17-W29 pass.
- [ ] A test covers a 3-D range in a `Lift` and in a `LiftArrays` argument, alone and beside an array.
- [ ] `make check` green; reviewers approve.

## Out of scope
- 3-D ranges in aggregates or in any non-lifted function.
