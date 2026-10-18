---
id: T-1777
title: "Probe the SPEC-0011 open lifting questions (an error beside a refused range, 3-D ranges, the other Analysis ToolPak functions)"
milestone: M9
lane: research
status: done
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
10. FORMULATEXT and spill of a dynamic `=EDATE({45000;45031},A1:A60)`, as a hint to how Excel saves it.
11. A stacked result past the grid: `ROWS(VSTACK(A:A,1:1))` and `ROWS(HSTACK(A:A,1:1))` (RustSheet gives `#VALUE!` through `check_array_size`, T-1780).

## Acceptance criteria
- [x] The probe case and its oracle sidecar are committed, and the findings are in arrays.md (and financial.md and engineering.md for question 5) with cell citations. The "unverified" notes they answer are rewritten.
- [x] A follow-up task is filed for each question where Excel differs from the design's choice, and SPEC-0011's open-questions list is updated.
- [x] `make check` green.
- [ ] Reviewers approve.

## Out of scope
- Implementing the findings.

## Outcome
Probes: eval/suites/probes/cases/scalar-lift-open-a (Q1-7, 9, 10), -b (Q8), -c, -c2, -c3 (Q11), -d (follow-ups of a and b); excel-graph, 2026-10-17. Findings: docs/excel-semantics/arrays.md (and financial.md, engineering.md, dates.md); SPEC-0011's open questions are updated.
1. Error beside a refused range: the kernel's own check order. Leftmost for EDATE, EFFECT, ACCRINTM (`EDATE(NA(),A1:A60)` `#N/A`, `EDATE(A1:A60,NA())` `#VALUE!`; -a W1-W4, -d W8-W15). Not for YEARFRAC, which checks basis, start, end (-d W1-W5; -a W6, W7). Differs from the spec for YEARFRAC: T-1800.
2. Error precedence: `PMT({#NUM!},{#N/A},100)` `#NUM!`, reversed `#N/A`, EDATE the same (-a W14-W17). Matches the spec (leftmost).
3. Broadcast scalar error or text: elements are `#N/A`, `#VALUE!`, `#VALUE!` (-a W18-W23). Matches.
4. ATP array plus range: all elements `#VALUE!`, on and off the range's rows; `EDATE({45000,45031},M1:M60)` is 1x2 (-a W24-W30, W70-W73). Matches.
5. Later ATP arguments over a range: all `#VALUE!` (PRICE, COUPDAYS, COUPNUM, ACCRINT, CONVERT, COMPLEX, IMSUB, IMDIV, IMPOWER, DOLLARDE, DOLLARFR; -a Y1-Y18, controls Z). Matches the ATP rule.
6. Function-returned multi-cell reference in a worksheet function: lifts, 60 rows (IF, INDEX, OFFSET, INDIRECT, CHOOSE; -a W31-W36). Matches.
7. FORMULATEXT of legacy `=SUM(PMT({0.1;0.2},10,100))`: the written text, no `@` (-a AB1). Matches.
8. 3-D range: `#REF!` in a `Lift` argument (PMT, NORM.DIST, BITAND, also one cell per sheet and beside an array), `#VALUE!` in a `LiftArrays` one (EDATE) (-b, -d W17-W29). Differs from the spec's `#VALUE!` for Lift: T-1801.
9. Other ATP-era functions: QUOTIENT, FACTDOUBLE and SERIESSUM x, n, m refuse a range; MULTINOMIAL, GCD, LCM aggregate; SERIESSUM coefficients whole (-a Y19-Y30). The spec had no entry: T-1802 adds them.
10. Dynamic `=EDATE({45000;45031},A1:A60)`: spills 2 rows of `#VALUE!`, FORMULATEXT is the written text (-a AA3, AB3, AC3). Matches the static shape; how Excel saves it is not visible through the Graph backend (M14).
11. Stacked result past the grid: rows past it are `#NUM!` (spec: `#VALUE!`), columns past 16384 are allowed (-c, -c2). HSTACK(A:A,1:1) gave ROWS 0 in the backend (-c3), left unsettled. Differs: T-1803.
Follow-ups: T-1800 (driver error precedence, T-1770/T-1372/T-1383), T-1801 (3-D `#REF!`, T-1770), T-1802 (math entries, T-1770), T-1803 (stack past the grid, T-1780's bound).
