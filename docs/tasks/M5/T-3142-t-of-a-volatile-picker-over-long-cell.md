---
id: T-3142
title: "T of a volatile picker over long cell text re-evaluates the pick"
milestone: M5
lane: functions-text
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`T(CHOOSE(RANDBETWEEN(1,2),D1,REPT("x",256)))` agrees with Excel (computed branch `#VALUE!`, cell branch "") on the branch the first evaluation took.

## Context
T-3107 `reads_cell_text` (t_function.rs) calls `is_reference`, which evaluates a picker again; a volatile pick can differ from the first pass. Fix by recording whether the first evaluation came through a reference, not evaluating twice. Probe first if Excel evidence is missing.

## Acceptance criteria
- [ ] The pick used for the value is the pick used for the reference test; `make check` green.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
