---
id: T-0138
title: "Probe intersection operator edge cases: whitespace, literal and error operands"
milestone: M1
lane: research
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel evidence (excel-probe skill) for the intersection-operator behaviours T-0105 had to choose
without evidence, so that each entry in docs/excel-semantics/references.md "Intersection
operator: implementation choices (UNVERIFIED)" becomes VERIFIED or is fixed in the engine.

## Context
gen-m1 confirms only references separated by one U+0020 space. In T-0105 the engine chose the
rest; its results below were checked with a scratch suite during review (2026-10-05):
1. Characters: LF, CR+LF, TAB, NBSP (U+00A0), U+3000 and runs of spaces all act as the operator
   (`=A1<ws>A1:A2` = 1). Excel may accept only spaces, and a line break may be formatting.
2. Literal operands: `=1 A1` and `=(1) A1` are parse errors; `=A1 (1)` is `#VALUE!`; `=A1 TRUE`
   is a parse error. Parse error or `#VALUE!` in Excel, consistently?
3. Error operands: `=#N/A A1` gives `#N/A`, `=A1 #REF!` gives `#REF!`, but `=A1 #DIV/0!` is a
   parse error; `=(1/0) A1` and `=A1 (1/0)` give `#DIV/0!`. Does an operand's error win over
   `#VALUE!`/`#NULL!`, and which error wins when both operands are errors?
4. `=Data!A1 Other!Z99` (different sheets, no shared coordinates): the engine gives `#VALUE!`
   (the cross-sheet check runs first). Could Excel give `#NULL!`?
5. `=A1 @A1` (parse error in the engine) and spill references `A1# B1:B3` as operands.
6. A defined name whose formula is an intersection: is it a reference?
7. Trailing input (`=1 2`, `=SUM(A1))`): does the oracle backend reject the workbook or store
   something evaluable? This settles the `#ERROR!` hand expectations in
   eval/suites/smoke/cases/trailing-input.case.json, including B5 `=A1 1` and B6 `=(A1)(A2)`.
8. The multi-cell form of left-error precedence, `=Data!C8:C9/Data!D5` (also noted in T-0137).
Probe in one workbook per group, since Excel for the web may reject a whole workbook it cannot
load.

## Acceptance criteria
- [ ] Each of the 8 groups has oracle evidence recorded in docs/excel-semantics/ (VERIFIED or
  corrected), and the UNVERIFIED entry is removed or narrowed.
- [ ] Every engine mismatch found has a fix task filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing the engine (those are the follow-up tasks).
