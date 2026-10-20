---
id: T-2050
title: "BINOM.INV and CRITBINOM with probability_s 0 are #NUM!, not 0"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`BINOM.INV(10,0,0.5)` and `CRITBINOM(10,0,0.5)` are `#NUM!` as in Excel, not 0.

## Context
- Known failures (gap triage 2026-10-19): probes-numeric/m9-legacy-critbinom Calc!A23
  `CRITBINOM(10,0,0.5)` and A24 `BINOM.INV(10,0,0.5)`: Excel (excel-graph) `#NUM!`, ours 0.
  probability_s 1 is already `#NUM!` (A25, A26 pass), and so are -0.5 and 1.5 (A31, A32).
- The rule is already recorded and not implemented: docs/excel-semantics/statistics.md (legacy
  names entry, "BINOM.INV/CRITBINOM: alpha 0 or 1 is #NUM!, and probability 0 or 1 is #NUM!, even
  though BINOMDIST accepts them").
- Code: `binom_inv_of` in crates/rustsheet-engine/src/functions/statistical/binom.rs tests
  `!(0.0..1.0).contains(&p)`, which lets p = 0 through; the bound must exclude 0 as well.

## Acceptance criteria
- [ ] probes-numeric/m9-legacy-critbinom Calc!A23 and A24 pass (ungated suite; `make eval`), with no
      new known-failure lines in any suite.
- [ ] A unit test covers p = 0 for both names (`#NUM!`) and keeps `BINOM.INV(50,1E-06,0.9)` = 0 (A20).
- [ ] `make check` green; reviewers approve.

## Out of scope
- BINOM.INV kernel accuracy (T-0927).
