---
id: T-0138
title: "Probe intersection operator edge cases: whitespace, literal and error operands"
milestone: M1
lane: research
status: done
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
- [x] Each of the 8 groups has oracle evidence recorded in docs/excel-semantics/ (VERIFIED or
  corrected), and the UNVERIFIED entry is removed or narrowed.
- [x] Every engine mismatch found has a fix task filed.
- [x] `make check` green; reviewers approve.

## Out of scope
- Fixing the engine (those are the follow-up tasks).

## Outcome
All 8 groups were probed on excel-graph (2026-10-05). Four new probe cases with oracle sidecars:
intersection-whitespace, intersection-operands, intersection-function-operands and
arith-range-error-precedence. When Excel for the web cannot parse a formula in an xlsx, it refuses the
whole workbook (`FileCorruptTryRepair`). Those formulas were probed one per scratch workbook and are
recorded in docs. The UNVERIFIED entry in references.md is replaced by verified entries on separators
and operands. syntax.md's trailing-input entry is now verified. coercion.md's range-error entry
now covers the multi-cell case. Group 6 (defined names) has scratch evidence only and stays
UNVERIFIED until a committed case exists (T-0043).
Mismatches filed: T-0148 (separators, non-reference operands), T-0149 (`@`, union, `(A1)(A2)`),
T-0042 (spec2xlsx spill references). Eval delta: +4 probe cases, 41 new oracle cells, 35 pass and
6 fail (new coverage, baselined).
