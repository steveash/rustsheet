---
id: T-1669
title: "Probe BYROW/BYCOL array results: BYCOL, array-constant source, 1x1 array"
milestone: M2
lane: research
status: todo
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
- [ ] Probe with oracle values added; arrays.md updated; engine fixed or a fix task filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
