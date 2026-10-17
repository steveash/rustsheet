---
id: T-1792
title: "Probe what MAP and SCAN show for an element whose LAMBDA returns a multi-cell range"
milestone: M2
lane: functions-logic
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The engine shows what Microsoft 365 Excel shows for a MAP or SCAN element whose LAMBDA returns a
multi-cell range, backed by oracle evidence in a probes case.

## Context
Today both give `#VALUE!` per element (functions/logical/bycol_byrow.rs,
`Model::lambda_result_value`), which is unprobed. The related verified case is BYROW, where Excel
gives `#CALC!` (docs/excel-semantics/arrays.md, probes/m7-argument-modes; T-1511 covers BYROW/BYCOL
only). T-1769 made the element read only the range's existing cells, on the assumption that the
element's value does not depend on what the range holds; the probe should confirm that too.
Cases to probe (excel-probe skill): `=MAP(A1:A2,LAMBDA(r,B1:B3))`, `=SCAN(0,A1:A2,LAMBDA(a,v,B1:B3))`,
the same with an error cell (`=1/0`) inside B1:B3, a 1x2 row range `B1:C1`, and a one-cell range
`B1:B1` as the control.

## Acceptance criteria
- [ ] A probes case with oracle values for the formulas above (and their element values).
- [ ] The result is recorded in docs/excel-semantics/ (the "Seen-range cap" note in references.md
      cites the element value as unprobed).
- [ ] If Excel differs from today, the engine matches it (or a follow-up task is filed); any
      value that depends on the range's contents means `lambda_result_value` must read values again.
- [ ] `make check` green; reviewers approve.

## Out of scope
- BYROW/BYCOL (T-1511) and MAKEARRAY.
