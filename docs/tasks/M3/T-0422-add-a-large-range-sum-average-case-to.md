---
id: T-0422
title: "Add a large-range SUM/AVERAGE case to the edit bench"
milestone: M3
lane: perf
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Add a large-range SUM and AVERAGE recalc case (non-integer cells) to crates/rustsheet/benches/edit.rs.

## Context
The bench only covers chained integer `+`; T-1499 added a per-addition twice-rounding step to SUM, AVERAGE, `+` and `-` that only a large non-integer range exercises (performance review of T-1499 measured it by hand: 200k cells, 20 formulas, ~88 ms vs ~87 ms on main).

## Acceptance criteria
- [ ] The bench case exists and its numbers are recorded in the task Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
