---
id: T-2013
title: "Bench RANK: scalar fill-down and a lifted RANK(range,range)"
milestone: M9
lane: perf
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The bench suite covers RANK, so a regression on its scalar or lifted path shows up.

## Context
T-1999 moved RANK onto the SPEC-0011 driver: a scalar call now goes through `lift_call_costed` (and
an `ArgBuf` read for three arguments) and walks the numbers unsorted; a lifted call sorts them once
(`SortedNumbers`, `LiftCost::Prepared`) and binary-searches each element. The performance review
found no RANK workload in `crates/rustsheet/benches/` (lift.rs and aggregate.rs are the closest).

## Acceptance criteria
- [ ] A bench case for `=RANK(A2,$A$2:$A$10001)` filled down 10,000 rows and one for
  `=RANK(A2:A10001,A2:A10001)`, in the existing bench style.
- [ ] Before/after numbers against the pre-T-1999 commit in the Outcome (release, baseline x86-64).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing RANK's algorithm, unless the numbers show a regression.
