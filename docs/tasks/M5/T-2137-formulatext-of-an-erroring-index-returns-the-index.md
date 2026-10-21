---
id: T-2137
title: "FORMULATEXT of an erroring INDEX returns the INDEX error, not #ERROR!"
milestone: M5
lane: engine
status: done
priority: P2
size: S
tier: sonnet
campaign: visible
depends_on: []
---

## Goal
`FORMULATEXT(INDEX(range,n))` propagates INDEX's error (`#REF!` out of range, `#VALUE!` for an array-constant source) as Excel does.

## Context
probes/index-reference-args Calc!A4 (Excel `#REF!`, engine `#ERROR!`) and A5 (Excel `#VALUE!`, engine `#ERROR!`). See docs/excel-semantics/references.md "IF and INDEX return references to aggregates" (T-2059). FORMULATEXT reads its argument through the reference path (`evaluate_call_with_reference`), which drops INDEX's error.

## Acceptance criteria
- [ ] probes/index-reference-args A4 and A5 pass and leave eval/baseline/ (`make eval-baseline`).
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083 (gap triage 2026-10-21): every cell this task owns has the wrong type, error, text or boolean, or is a
number further than half a unit in Excel's 15th significant digit, so the task is in the campaign. Its acceptance
criteria above are already the visible-parity stage.

## Out of scope
-

## Outcome
FORMULATEXT of an INDEX argument now returns INDEX's own error (`#REF!` out of range), and `#VALUE!` when
INDEX of an array constant picks a value rather than a cell. Other non-reference arguments keep `#ERROR!`
(including a 3D range). probes/index-reference-args Calc!A4 and A5 pass and left eval/baseline/.
