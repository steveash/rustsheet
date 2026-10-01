---
id: T-0007
title: "Crash-only execution of pending cases"
milestone: M0
lane: harness
status: todo
priority: P1
size: S
depends_on: []
---

## Goal
Pending cases (no oracle yet) still run through the engine so panics/timeouts are caught immediately.

## Context
Today `run_case` returns early when there are no expectations; generated suites can be large and the oracle is a bottleneck.

## Acceptance criteria
- [ ] Pending cases are evaluated; status `pending` unless the engine panics/times out/errors, which is reported as such and fails `check` for gate suites.
- [ ] Report shows crash counts per suite.
- [ ] Test with a spec that triggers an engine error.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
