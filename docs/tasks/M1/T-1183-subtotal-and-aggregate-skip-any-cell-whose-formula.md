---
id: T-1183
title: "SUBTOTAL and AGGREGATE skip any cell whose formula contains SUBTOTAL or AGGREGATE"
milestone: M1
lane: functions-math
status: done
priority: P2
size: S
tier: sonnet
depends_on: [T-1036]
---

## Goal
SUBTOTAL (1-11 and 101-111) and AGGREGATE options 0-3 skip every cell whose formula contains a SUBTOTAL or AGGREGATE call anywhere, and SUBTOTAL skips AGGREGATE cells.

## Context
probes/subtotal-hidden-filtered Nest sheet (excel-graph, T-1036): with C2 `=1+SUBTOTAL(9,A1:A2)`, C5 `=IF(TRUE,SUBTOTAL(..))`, C6 `=AGGREGATE(9,0,A1:A2)`, C8 `=SUM(SUBTOTAL(..))`, C9 `=-SUBTOTAL(..)`, C10 `=LET(x,SUBTOTAL(..),x)` and so on, `SUBTOTAL(9,Ck)`, `SUBTOTAL(109,Ck)` and `AGGREGATE(9,0,Ck)` are all 0, while `AGGREGATE(9,4,Ck)` keeps the value and a plain `=SUM(A1:A2)` (C12) is counted. The engine's `cell_formula_is_one_of` (functions/subtotal.rs) matches only the outermost call, and `cell_is_subtotal` lists only SUBTOTAL. Failing cells: Nest!E2-E3, E5-E11, F2-F3, F5-F11, G2-G3, G5, G7-G11. Whether a SUBTOTAL name inside a string or a defined name counts is unprobed. Keep the walk cheap: it runs per visited formula cell (cache per formula if needed).

## Acceptance criteria
- [x] Every Nest cell of probes/subtotal-hidden-filtered passes; `make eval-baseline` records them.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`cell_formula_is_one_of` now asks `Node::contains_function` (iterative tree walk, expressions/parser/mod.rs) instead of looking at the outermost call, and SUBTOTAL skips AGGREGATE cells too. Every Nest cell of probes/subtotal-hidden-filtered passes. Eval delta: probes known failures 360 -> 334 (26 fixed), 0 regressions. Not cached per formula: the walk is linear in formula size and runs only for visited formula cells inside a SUBTOTAL/AGGREGATE range.
