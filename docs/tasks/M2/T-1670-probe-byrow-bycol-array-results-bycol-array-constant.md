---
id: T-1670
title: "Probe BYROW/BYCOL array results: BYCOL, array-constant source, 1x1 array"
milestone: M2
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's result is known for BYROW, BYCOL and array-constant sources when a LAMBDA returns an array, including a 1x1 array.

## Context
T-1664 made BYROW over a range give one `#CALC!` when the LAMBDA returns a multi-cell value (evidence: eta-reduced-names E49/E101, m7-argument-modes F45:H45). Unprobed: BYCOL, `BYROW({1;2;3},LAMBDA(r,r))`, `LAMBDA(r,{7})`, `LAMBDA(r,SEQUENCE(1))` (expected scalar), and a mix of scalar and array rows. Ours: bycol_byrow.rs `makes_calc_error`. Use the `excel-probe` skill; record in docs/excel-semantics/arrays.md.

## Acceptance criteria
- [x] Probe with oracle values added; arrays.md updated; engine fixed or a fix task filed.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/byrow-bycol-array-results` (16 formulas, excel-graph) shows a 1x1 array result counts as its value and any larger result gives one `#CALC!`. Fixed `calc_result_to_array_node` in bycol_byrow.rs; recorded in arrays.md. Probe cells 8/32 -> 32/32.
