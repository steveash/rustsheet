---
id: T-2027
title: "BINOM.INV and CRITBINOM: probability_s 0 is #NUM!; Z.TEST of a typed TRUE is #DIV/0!"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Three edge cells return Excel's error: BINOM.INV and CRITBINOM with `probability_s` = 0, and Z.TEST whose
array argument is a typed TRUE.

## Context
Gap triage 2026-10-19 (one verdict class: the wrong error or a value where Excel gives an error, on an
edge argument). Evidence is in eval (probes-numeric is not gated, so these are report cells, not baseline
lines):
- probes-numeric/m9-legacy-critbinom Calc!A23 `CRITBINOM(10,0,0.5)` and A24 `BINOM.INV(10,0,0.5)`: Excel
  #NUM!, ours 0. The rule is already recorded: docs/excel-semantics/statistics.md "Legacy distribution names
  sweep", "BINOM.INV/CRITBINOM: alpha 0 or 1 is #NUM!, and probability 0 or 1 is #NUM!". Probability 1
  (A25, A26) already passes. Code: `binom_inv_of` in
  crates/rustsheet-engine/src/functions/statistical/binom.rs (`!(0.0..1.0).contains(&p)` lets 0 through).
- probes-numeric/t1049-ztest Calc!A51 `Z.TEST(TRUE,0.5,1)`: Excel #DIV/0!, ours #VALUE!. A typed TRUE is a
  one-value array (and a one-value array is #DIV/0!: A39-A42), while typed text stays #VALUE! (A50,
  `Z.TEST("a",4)`, passes). Rule: statistics.md, the Z.TEST bullet under the ERFC kernel entry. Code:
  `fn_z_test` in crates/rustsheet-engine/src/functions/statistical/z_test.rs.

## Acceptance criteria
- [ ] In `make eval`, probes-numeric/m9-legacy-critbinom A23, A24 and probes-numeric/t1049-ztest A51 pass;
      no other cell of those two cases, and no line of any baseline, starts failing.
- [ ] Unit tests for the three cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Z.TEST with TRUE inside a range or an array literal (already passing: A46), BINOM.INV's search.
