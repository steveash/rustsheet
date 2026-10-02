---
id: T-0007
title: "Crash-only execution of pending cases"
milestone: M0
lane: harness
status: done
priority: P1
size: S
depends_on: []
---

## Goal
Pending cases (no oracle yet) still run through the engine so panics/timeouts are caught immediately.

## Context
Today `run_case` returns early when there are no expectations; generated suites can be large and the oracle is a bottleneck.

## Acceptance criteria
- [x] Pending cases are evaluated; status `pending` unless the engine panics/times out/errors, which is reported as such and fails `check` for gate suites.
- [x] Report shows crash counts per suite.
- [x] Test with a spec that triggers an engine error.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`run_case` no longer returns early for cases without expectations; they run in the isolated
worker thread, so panics/timeouts/load errors surface as `panic`/`timeout`/`engine_error` (which
the baseline records as `!Status` failures, so a new crash fails the gate). Clean runs stay
`pending`. The suites table gained a `crashes` column. Unit tests cover clean, panic and
engine-error pending cases.
