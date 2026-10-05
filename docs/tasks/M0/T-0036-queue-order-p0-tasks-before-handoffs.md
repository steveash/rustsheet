---
id: T-0036
title: "Queue order: P0 tasks before handoffs"
milestone: M0
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
A P0 task is never queued behind a handoff of lower priority.

## Context
`scripts/tasks.py::available` sorted handoffs first (decision 0018), so on 2026-10-05 T-0931 (P0, stack overflow on nested formulas) waited behind the T-0125 handoff (P2). Owner asked for the fix. Decision 0023.

## Acceptance criteria
- [x] Sort key: P0 first, then handoffs, then milestone/priority/id; unit test `test_p0_comes_before_a_handoff`.
- [x] `make next-task TIER=opus` returns T-0931 on main's task set (was T-0125).
- [x] work-loop and agent-workflow describe the new order.
- [x] Duplicate decision 0021 fixed (status-log record renumbered 0024, references updated); `scripts/check_decisions.py` (in `make hygiene`) fails on duplicate numbers or bad record names (unit tests).
- [x] `make check` green; reviewers approve.

## Out of scope
- Routine prompt text (behavior already follows from `make next-task`; see decision 0023).

## Outcome
P0 tasks now sort before handoffs (decision 0023); T-0931 is next for opus. Decision-number collision fixed and guarded. Eval delta none.
