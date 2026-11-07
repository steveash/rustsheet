---
id: T-4901
title: "Probe MROUND of a formula-returned blank and LET/LAMBDA omissions in the newer roundings (T-4873 follow-up)"
milestone: M5
lane: functions-math
status: done
priority: P3
size: S
tier: sonnet
depends_on: [T-4873]
---

## Goal
MROUND of a blank that a formula returns, and a LET or LAMBDA variable bound to an omission in CEILING.MATH and
FLOOR.PRECISE, give what Excel gives, decided by a small probe.

## Context
- T-4873 (PR #3932) moved the "written-empty" test of the ToolPak functions onto the argument node
  (`Model::is_empty_argument`, `Model::atp_numbers` in cast.rs). As a result, MROUND now reads a formula-returned blank
  (a VLOOKUP or INDEX landing on an empty cell) as 0, where it used to give `#N/A`. That follows SERIESSUM's evidenced
  `x` (gen-m5/74b183ca6fc1d190 Calc!A16), but no cell probes MROUND itself: numbers.md "An empty argument of a ToolPak
  function is #N/A; an empty significance of the newer roundings is 1" marks it UNVERIFIED.
- The same node test makes `LET(e,,CEILING.MATH(2.675,e))` read `e` as omitted (3). Only QUOTIENT's LET row is
  evidenced (probes/t4873-empty-arg-toolpak-roundings A12).
- Review findings on #3932 (security, design and excel-fidelity, all MINOR).

## Acceptance criteria
- [x] A probes case (excel-probe skill, excel-graph) covers `MROUND(VLOOKUP(..blank..),1)`, `MROUND(5,INDEX(<blank>,1))`,
      `MROUND(IF(TRUE,),1)`, plus `LET(e,,CEILING.MATH(2.675,e))`, `LAMBDA(a,b,CEILING.MATH(a,b))(2.675,)`, and the
      same two forms for FLOOR.PRECISE.
- [x] After the fill, numbers.md drops UNVERIFIED and states the corrected rule with its cells. Amended scope: the
      cells that fail (A3, A10, A12:A16) are not fixed here; the engine fix needs the wrapper's condition evaluated
      and is T-4912 (opus), which owns them (see Outcome).
- [x] Amended: the new case's 7 failing cells are recorded as F lines in eval/baseline (made by `make eval-baseline`)
      and owned by T-4912; no existing line changed. `make check` green; reviewers approve.

## Out of scope
- QUOTIENT and SERIESSUM written-empty arguments (T-4873, evidenced).

## Outcome
Probe `probes/t4901-mround-blank-let-lambda-roundings` filled on excel-graph (16 cells). A VLOOKUP/INDEX blank is 0 in
MROUND, CEILING.MATH and FLOOR.PRECISE and the LET/LAMBDA omissions give 3 and -3, all as ours. But a written-empty
branch of IF, CHOOSE or IFERROR hands the omission on (`MROUND(IF(TRUE,),1)` is `#N/A`, `CEILING.MATH(2.675,IF(TRUE,))`
is 3), which ours reads as 0: A3, A10, A12:A16 are off. The engine change needs the wrapper's condition evaluated, so it
is filed as T-4912 (opus), which owns those cells. numbers.md records the rule with its cells and drops the MROUND
UNVERIFIED. Eval delta: one new probe case, 7 new visibly-off cells owned by T-4912, 7 F lines for the new case added to eval/baseline by `make eval-baseline`, no existing line changed.
