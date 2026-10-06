---
id: T-0986
title: "TEXT General: drop one digit (6 to 5) when decimal exponent magnitude is 99 or more"
milestone: M1
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
TEXT(x,"General") prints 5 significant digits when the decimal exponent magnitude is 99 or more (6 otherwise), as Excel does.

## Context
Rule in docs/excel-semantics/numbers.md (Exponent width); evidence probes/number-text-exponent-digits column D (T-0186). Engine fails D1-D8 and D10-D15.

## Acceptance criteria
- [ ] number-text-exponent-digits column D cells pass; baseline shrinks.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
