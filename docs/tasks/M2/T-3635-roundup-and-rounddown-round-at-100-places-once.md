---
id: T-3635
title: "ROUNDUP and ROUNDDOWN round at 100 places once num_digits overflows 16 bits"
milestone: M2
lane: functions-math
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-3555]
---

## Goal
ROUNDUP and ROUNDDOWN return Excel's values when the narrowed `num_digits` has magnitude 32,768 or more. Today the
engine rounds at the narrowed value itself.

## Context
- docs/excel-semantics/numbers.md, "ROUNDUP and ROUNDDOWN round at 100 places past a 16-bit num_digits" (T-3555,
  VERIFIED). Narrowing gives k, the sign kept and the magnitude modulo 65,536. When |k| >= 32,768, Excel rounds the
  15-digit cut at 100 places: 10^-100 for positive k, 10^100 for negative k. ROUNDUP's unit 10^-k is added as a
  double, so:
  - for positive k it is 0, and ROUNDUP returns the rounded-down value;
  - for negative k it overflows, and ROUNDUP is `#NUM!` whenever a digit is dropped.
  - `ROUNDDOWN(1.23456789012345E-95,65535)` = 1.23456E-95, `ROUNDUP(…)` the same;
  - `ROUNDDOWN(1.5E+80,-65535)` = 0, `ROUNDUP(1.5E+120,-65535)` = 1.5E+120 (nothing dropped);
  - `ROUNDUP(1.23456789012345E+105,-32768)` = `#NUM!`; k = 32,767 rounds at 32,767 places as before.
- Evidence: probes/t3555-roundup-rounddown-digits-by-exponent and probes/t3555-roundup-rounddown-huge-digits-place
  (excel-graph, 2026-11-01). Both are committed with T-3555's PR, so `make eval` lists their failing cells.
- Gate A: nine known failures in `eval/accuracy/known-failures.tsv` are owned by this task:
  - `accuracy/ROUNDUP/s1!Calc!A110`, A116, A117, A118, A119 and A120;
  - `accuracy/ROUNDDOWN/s1!Calc!A113`, A115 and A117.
  Remove the lines when they pass.
- Code: `effective_digits` and `round_decimal` in `crates/rustsheet-engine/src/decimal_round.rs`. TRUNC shares the
  ROUNDDOWN path. Its huge-digit rows (numbers.md, "TRUNC is ROUNDDOWN") should follow, but TRUNC was not probed
  past 32,767. Check its cells in `make eval` and say in the Outcome what you found.

## Acceptance criteria
- [ ] The probe cells of both t3555 cases pass (`make eval-show CASE=probes/t3555-roundup-rounddown-huge-digits-place`).
- [ ] The nine Gate A known-failures lines are removed and `rustsheet-eval accuracy` reports 0 failing.
- [ ] Unit tests in decimal_round.rs cover positive and negative k at 32,768 and 65,535, and 32,767.
- [ ] `make eval-baseline` records the fixed cells; `make check` green; reviewers approve.

## Out of scope
- ROUND, whose `num_digits` narrowing is different and matches every probed cell.
