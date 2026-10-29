---
id: T-3085
title: "TAKE/DROP of a one-cell name, LET variable or INDEX/OFFSET result keeps the reference"
milestone: M7
lane: functions-lookup
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A TAKE/DROP whose array is a one-cell reference reached other than as a typed cell (a defined name for `$E$7`, a LET variable bound to `E7`, `INDEX(E7:E8,1)`, `OFFSET(E7,0,0)`) behaves as Excel does: probably a reference, as `TAKE(E7,1)` is.

## Context
T-3072 made `take_drop_reference` (functions/lookup_and_reference/drop_take.rs) read a literal `Node::ReferenceKind` array argument as its one-cell range, so `ROWS(E9-TAKE(E7,2,42))` is `#VALUE!` (E7 text) and `ISREF(TAKE(E7,1))` TRUE (probes/t3072-take-drop-one-cell-reference, docs/excel-semantics/lookup.md "TAKE/DROP of a one-cell reference is a reference"). Other one-cell reference forms still take the value path and give a 1x1 array; unprobed (T-3072 Excel-fidelity review).

## Acceptance criteria
- [ ] excel-probe: `ROWS(E9-TAKE(nm,1))` (nm = `Sheet1!$E$7`), `LET(x,E7,ROWS(E9-TAKE(x,1)))`, `ROWS(E9-TAKE(INDEX(E7:E8,1),1))`, `ROWS(E9-TAKE(OFFSET(E7,0,0),1))`, and `ISREF(..)` and `TYPE(1+TAKE(..,1))` of each (E7 text, E9 blank, a number cell for TYPE).
- [ ] The engine matches every probed cell; lookup.md records the findings.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed all four forms (probes/t3085-take-drop-one-cell-forms, 16 cells). LET variables and OFFSET already matched; a defined name and an INDEX result as TAKE/DROP's array now keep the reference (`take_drop_reference`). Findings in lookup.md. Probe case passes 16/16. (An earlier commit on this branch wrongly deleted nine t2065 oracle files; restored.)
