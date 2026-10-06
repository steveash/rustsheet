---
id: T-0187
title: "Probe union operator edge cases (precedence, operands, errors, cross-sheet, ISREF, FORMULATEXT, multi-area names)"
milestone: M1
lane: research
status: todo
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
- [ ] Each of the 8 groups has oracle evidence recorded in docs/excel-semantics/references.md
  (VERIFIED or corrected), and the UNVERIFIED entry is removed or narrowed.
- [ ] Every engine mismatch found has a fix task filed (multi-area values go to T-0133).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing the engine (follow-up tasks; multi-area values are spec T-0133).
- Unions as function arguments (`SUM((A1,B2))`, AREAS, INDEX area_num): T-0133's probe list.
