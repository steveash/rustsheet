---
id: T-0199
title: "Number-to-text: 14 mantissa digits when decimal exponent magnitude is 99 or more"
milestone: M1
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Number text with a decimal exponent magnitude of 99 or more prints 14 significant digits, as Excel does.

## Context
Rule VERIFIED in docs/excel-semantics/numbers.md (Exponent width), evidence probes/number-text-exponent-digits (T-0186). Engine prints 15 digits there (known failure probes/arith-overflow Calc!A16:B16 and number-text-exponent-digits cells).

## Acceptance criteria
- [ ] number-text-exponent-digits (column B/C/F `&""`/CONCAT cells; TEXT General column D is a separate task, see numbers.md) and arith-overflow A16:B16 pass; baseline shrinks.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
