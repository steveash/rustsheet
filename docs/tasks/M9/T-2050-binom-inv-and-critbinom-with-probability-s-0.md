---
id: T-2050
title: "BINOM.INV and CRITBINOM with probability_s 0 are #NUM!, not 0"
milestone: M9
lane: functions-stat
status: done
priority: P3
size: S
tier: sonnet
campaign: visible
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

Visibly-off cells (gap triage 2026-10-21, decision 0083): 2 cells, 0 numeric past half a unit in the 15th digit and 2 with a wrong type, error or text, blamed on CRITBINOM 1, BINOM.INV 1. Every one is listed above.

## Acceptance criteria
- [ ] Visible parity: the 2 visibly-off cells listed under Context (2026-10-21) have Excel's value to 15 significant digits, or its type, error and text: the 2 cells of the ungated probes-numeric suite pass in `make eval`, with no new known-failure lines in any baseline.
- [ ] probes-numeric/m9-legacy-critbinom Calc!A23 and A24 pass (ungated suite; `make eval`), with no
      new known-failure lines in any suite.
- [ ] A unit test covers p = 0 for both names (`#NUM!`) and keeps `BINOM.INV(50,1E-06,0.9)` = 0 (A20).
- [ ] `make check` green; reviewers approve.

## Out of scope
- BINOM.INV kernel accuracy (T-0927).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (0 where Excel is `#NUM!`), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it.

## Outcome
BINOM.INV and CRITBINOM now reject probability_s 0 as `#NUM!` (the rule was already in statistics.md). Unit test added; m9-legacy-critbinom A23 and A24 pass.
