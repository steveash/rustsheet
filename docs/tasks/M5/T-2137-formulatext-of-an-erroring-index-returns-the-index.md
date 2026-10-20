---
id: T-2137
title: "FORMULATEXT of an erroring INDEX returns the INDEX error, not #ERROR!"
milestone: M5
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`FORMULATEXT(INDEX(range,n))` propagates INDEX's error (`#REF!` out of range, `#VALUE!` for an array-constant source) as Excel does.

## Context
probes/index-reference-args Calc!A4 (Excel `#REF!`, engine `#ERROR!`) and A5 (Excel `#VALUE!`, engine `#ERROR!`). See docs/excel-semantics/references.md "IF and INDEX return references to aggregates" (T-2059). FORMULATEXT reads its argument through the reference path (`evaluate_call_with_reference`), which drops INDEX's error.

## Acceptance criteria
- [ ] probes/index-reference-args A4 and A5 pass and leave eval/baseline/ (`make eval-baseline`).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
