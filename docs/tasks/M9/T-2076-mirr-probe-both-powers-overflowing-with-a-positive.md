---
id: T-2076
title: "MIRR: probe both powers overflowing with a positive flow"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
MIRR with both powers overflowing and a positive flow is evidenced by excel-graph and the engine matches it.

## Context
T-1995 fixed the no-positive-flow case (#DIV/0!); with a positive flow `fn_mirr` returns the reinvest error first and that order is unprobed. See docs/excel-semantics/financial.md "MIRR power overflow and underflow".

## Acceptance criteria
- [ ] a probe of `MIRR({-1,2,3},1E+200,1E+200)`, filled by excel-graph, with a mismatch fixed in `fn_mirr` and recorded in financial.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
