---
id: T-1935
title: "Legacy IF arm passes a range whole into a range argument (MATCH(2,IF(B1:B6>0,B1:B6),0))"
milestone: M2
lane: engine
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A legacy (no `cm`) `=MATCH(2,IF(B1:B6>0,B1:B6),0)` matches Excel: FORMULATEXT `=MATCH(2,IF(@B1:B6>0,B1:B6),0)`, 2 on row 2 and `#VALUE!` on row 10 (probes/legacy-match-lookup-array-intersection J2, Y2).

## Context
Found by T-1862. In a legacy cell the engine puts an automatic `@` on IF's value arm when IF sits in MATCH's `lookup_array` (a range argument), because IF's arms are scalar in the signature walk (`add_legacy_intersection` -> `add_argument_intersection`, expressions/parser/static_analysis.rs and static_analysis/legacy.rs). Excel keeps the arm whole there and hands the range to MATCH. docs/excel-semantics/references.md "Legacy MATCH, VLOOKUP and HLOOKUP intersect an operation in their lookup table" records the known mismatch. Probe the same shape under SUM, COUNT and VLOOKUP's table before generalizing (`SUM(IF(TRUE,B1:B6+0))` does intersect its arm's arithmetic: references.md "Legacy formulas intersect the ranges in value arguments").

## Acceptance criteria
- [ ] probes/legacy-match-lookup-array-intersection J2 and Y2 pass; no other eval cell regresses.
- [ ] Any new shape probed is recorded in docs/excel-semantics/references.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
