---
id: T-1991
title: "Share one places-coercion helper across the nine base conversions"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The nine base conversions in functions/engineering/number_basis.rs coerce `places` through one shared helper instead of nine copies of the same block.

## Context
T-1816 moved the `places` coercion before the number coercion in all nine kernels (Excel reads the second argument first, probes/eng-first-vs-second-error). The block is repeated verbatim, so the ordering rule lives in nine places. A pure refactor; no behavior change.

## Acceptance criteria
- [ ] One helper owns the `places` coercion; eval output is unchanged (no baseline edits).
- [ ] Duplicated-line ratchet goes down (`scripts/check_duplication.py --update`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any change to which error wins (T-1816).
