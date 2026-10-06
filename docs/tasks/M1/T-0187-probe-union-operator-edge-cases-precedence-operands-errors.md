---
id: T-0187
title: "Probe union operator edge cases (precedence, operands, errors, cross-sheet, ISREF, FORMULATEXT, multi-area names)"
milestone: M1
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel evidence (excel-probe skill) for the union-operator behaviours T-0134 chose without
evidence, so that each item of docs/excel-semantics/references.md "Union: parsing and evaluation
the engine chose without evidence" becomes VERIFIED or is fixed in the engine.

## Context
gen-m1/c940d3cfb0d3b083 confirms only unions of plain references in a value context (`#VALUE!`)
and a union cut down to one cell by an intersection. T-0134 chose the rest; the engine's current
result is in brackets:
1. Precedence against arithmetic: `=(A1+B1,C1)` [`A1+(B1,C1)`, `#VALUE!`] and `=(-A1,B1)`
   [parse error, the operand is not a reference]. Accepted? Value? FORMULATEXT?
2. Non-reference operands: `=(1,A1)`, `=(A1,"a")`, `=(A1,#N/A)`, `=(A1,SUM(B1))`,
   `=(A1,(B1+1))` [all parse errors]. Does Excel for the web refuse the workbook?
3. Error order: `=(A1,#REF!)` [`#REF!`] and `=(#REF!,A1) A1` [`#REF!`].
4. Cross-sheet: `=(Data!A1:A3,Other!A1:A3) Data!A2` [`#VALUE!`] and `=(Data!A1,Other!A1)`
   [`#VALUE!`].
5. `=ISREF((A1,B2))` [TRUE].
6. FORMULATEXT of `=((Data!A2,Data!E8),Data!B6)` [the engine prints `(Data!A2,Data!E8,Data!B6)`].
7. Multi-area defined names, including `_xlnm.Print_Area` (`Sheet1!$A$1,Sheet1!$B$2`), used as
   `=SUM(name)` and `=name` [the name body does not parse].
8. Low priority: the de-DE `;` separator as the union operator, `=(A1;B2)`.
Probe in one workbook per group, since Excel for the web refuses a whole workbook it cannot
parse (T-0138).

## Acceptance criteria
- [x] Each of the 8 groups has oracle evidence recorded in docs/excel-semantics/references.md
  (VERIFIED or corrected), and the UNVERIFIED entry is removed or narrowed.
- [x] Every engine mismatch found has a fix task filed (multi-area values go to T-0133).
- [x] `make check` green; reviewers approve.

## Out of scope
- Fixing the engine (follow-up tasks; multi-area values are spec T-0133).
- Unions as function arguments (`SUM((A1,B2))`, AREAS, INDEX area_num): T-0133's probe list.

## Outcome
Probed with excel-graph (2026-10-06). Three new cases are kept: probes/union-accepted (9 cells),
union-neg-operand (2) and union-defined-name (5). Six scratch workbooks were refused by Excel
(`FileCorruptTryRepair`): `(A1+B1,C1)` and all five non-reference operands. That confirms the
engine's syntax errors for the five operands, but the engine wrongly accepts `(A1+B1,C1)`
(T-0193). Groups 3-5 match the engine, except that `ISREF((A1,#REF!))` is FALSE
(T-0194). Group 1: `(-A1,B1)` loads and gives #VALUE!, but the engine parses it as `-(A1,B1)`. Group
6: Excel keeps nested union parentheses in FORMULATEXT, but the engine flattens them (T-0193 covers
both). Group 7: multi-area names work in SUM, COUNT, ISREF and intersection, but the engine refuses
the name body (spec T-0133). Only a user-defined multi-area name was probed; `_xlnm.Print_Area`
itself is left to T-0133's probe list. Group 8: not probeable through a file, since xlsx stores `,`.
references.md "Union: parsing and evaluation" is now VERIFIED, apart from the parts marked as engine
choices. Eval: probes new coverage 16 cells, 4 known failures added (the mismatches above,
baselined as new coverage), 0 regressions.
