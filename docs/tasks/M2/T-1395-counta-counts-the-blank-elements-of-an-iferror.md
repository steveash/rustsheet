---
id: T-1395
title: "COUNTA counts the blank elements of an IFERROR array but not those of TAKE of a blank range"
milestone: M2
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
COUNTA counts the empty elements of an array made by IFERROR/IFNA over a range, while still skipping those of TAKE/INDEX over a blank range.

## Context
probes/counta-empty-value D18: `COUNTA(IFERROR(B1:B2,1))` (B1 blank, B2 "x") is 2 in Excel, the engine gives 1. Counting every `ArrayNode::Empty` fixes it but regresses probes/subtotal-reference-operands-c H13 (`COUNTA(TAKE(B3:B3,1))` of a blank is 0), so the two empties must be told apart in the array representation (an IFERROR empty value vs a blank cell). Design question, hence opus.

## Acceptance criteria
- [ ] D18 passes with no regression of subtotal-reference-operands-c H13.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
