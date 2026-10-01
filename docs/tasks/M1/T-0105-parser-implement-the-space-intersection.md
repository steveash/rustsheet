---
id: T-0105
title: "Parser: implement the space (intersection) operator and reject trailing input"
milestone: M1
lane: engine
status: todo
priority: P1
size: S
depends_on: []
---

## Goal
`=A1:B2 B1:B3` must evaluate the intersection like Excel, and garbage after a complete expression must be a parse error instead of being silently dropped.

## Context
docs/research/ironcalc-assessment.md §4: parser never checks end of input (parser/mod.rs ~340).

## Acceptance criteria
- [ ] Spec cases in `smoke` for intersection (incl. #NULL! for disjoint ranges) and for invalid trailing input.
- [ ] Hand expectations flagged for oracle confirmation; eval green.
- [ ] `make check` green; reviewers approve.

## Out of scope
Union operator (separate task).
