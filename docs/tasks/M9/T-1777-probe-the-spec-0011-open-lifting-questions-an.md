---
id: T-1777
title: "Probe the SPEC-0011 open lifting questions (an error beside a refused range, 3-D ranges, the other Analysis ToolPak functions)"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
The questions SPEC-0011 leaves open are answered by excel-graph probes, recorded in docs/excel-semantics/arrays.md, and turned into table or kernel changes in follow-up tasks where Excel differs from the design's choice.

## Context
Spec: docs/specs/SPEC-0011-scalar-argument-lifting.md, "Open questions". Write one probe case (excel-probe skill), tagged M9, with the formulas on the ranges' rows as in probes/scalar-lift-shapes. Fill only it with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes`. Questions:
1. An error argument left or right of a refused range: `EDATE(NA(),A1:A60)`, `EDATE(A1:A60,NA())`, `EFFECT(1/0,C1:C60)`, and the same next to an array. The design assumes the leftmost wins.
2. Error precedence within an element, which today rests on H33 alone: `INDEX(PMT({#NUM!},{#N/A},100),1)` and the reversed `INDEX(PMT({#N/A},{#NUM!},100),1)`, plus one ATP case (`INDEX(EDATE({#NUM!},{#N/A}),1)`).
3. Element values of a broadcast scalar error or text: `INDEX(EDATE(NA(),{1;2}),2)`, `INDEX(PMT("x",{10;20},100),2)`, `INDEX(EDATE("x",{1;2}),2)`.
4. The missing elements of an ATP call with an array and a range: `INDEX(EDATE(A1:A60,{1;2}),2)`, and both elements of `EDATE({45000,45031},M1:M60)` (on and off the range's rows).
5. Later ATP arguments over a range, sampled:
   - PRICE's and COUP*'s (COUPDAYS, COUPNUM) `frequency`;
   - ACCRINT's `basis`;
   - CONVERT's `from_unit` and `to_unit`;
   - COMPLEX's `suffix`;
   - an IM* second argument (IMSUB, IMDIV, IMPOWER);
   - DOLLARDE/DOLLARFR's `fraction`.
6. A function-returned multi-cell reference in a worksheet function's argument: `ROWS(PMT(IF(TRUE,C1:C60),10,100))`, `ROWS(PMT(INDEX(C1:C60,0,1),10,100))`, `ROWS(NORM.DIST(OFFSET(E1,0,0,60),0,1,TRUE))`.
7. FORMULATEXT of the legacy `=SUM(PMT({0.1;0.2},10,100))` (probes/scalar-lift-shapes J3).
8. A 3-D range (`Sheet1:Sheet2!A1:A2`) in a `Lift` and in a `LiftArrays` argument. The design gives `#VALUE!`.
9. Whether the other Analysis ToolPak-era functions refuse a range: QUOTIENT, FACTDOUBLE, MULTINOMIAL, SERIESSUM, and GCD/LCM (catalog `array`, check only).
10. A stacked result past the grid: `ROWS(VSTACK(A:A,1:1))` and `ROWS(HSTACK(A:A,1:1))` (RustSheet gives `#VALUE!` through `check_array_size`, T-1780).
10. FORMULATEXT and spill of a dynamic `=EDATE({45000;45031},A1:A60)`, as a hint to how Excel saves it.

## Acceptance criteria
- [ ] The probe case and its oracle sidecar are committed, and the findings are in arrays.md (and financial.md and engineering.md for question 5) with cell citations. The "unverified" notes they answer are rewritten.
- [ ] A follow-up task is filed for each question where Excel differs from the design's choice, and SPEC-0011's open-questions list is updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing the findings.
