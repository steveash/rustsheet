---
id: T-1128
title: "Probe RATE with an empty guess argument (RATE(48,-200,8000,0,1,))"
milestone: M9
lane: functions-financial
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`RATE(48,-200,8000,0,1,)` (empty sixth argument) matches Excel, whether Excel takes the default guess 0.1 or 0.

## Context
T-1050 let `fn_rate` accept a sixth argument. An empty guess likely goes through `get_number` as 0 rather than the 0.1 default (functions/financial/mod.rs, guess branch); no oracle data covers it (found by the T-1050 fidelity review). Probe it (excel-probe skill), record the finding in `docs/excel-semantics/financial.md`, and fix if it differs.

## Acceptance criteria
- [ ] A probe case with Excel values for an empty guess (and an empty fv/type for contrast) passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
