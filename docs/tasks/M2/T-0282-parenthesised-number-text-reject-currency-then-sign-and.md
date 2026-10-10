---
id: T-0282
title: "Parenthesised number text: reject currency-then-sign and trailing currency"
milestone: M2
lane: engine
status: in-progress
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
`0+"($-1)"`, `0+"($ -1)"` and `0+"(1$)"` are #VALUE! as in Excel (the engine gives 1, 1 and -1).

## Context
probes/paren-number-text-forms A1, A2, A6 (excel-graph, T-0281); see coercion.md "Parenthesised number text is negative". Code: `parse_formatted_number` in formatter/format.rs. A currency symbol inside parentheses is accepted only before the digits without a sign (`($1)`, `($0)`); `(1$)` is #VALUE!.

## Acceptance criteria
- [ ] probes/paren-number-text-forms passes in full; baseline shrinks; no regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
