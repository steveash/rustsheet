---
id: T-3635
title: "ROUNDUP and ROUNDDOWN round at 100 places once num_digits overflows 16 bits"
milestone: M2
lane: functions-math
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-3627]
---

## Goal
ROUNDUP and ROUNDDOWN return Excel's values when the narrowed `num_digits` has magnitude 32,768 or more. Today the
engine rounds at the narrowed value itself.

## Context
- docs/excel-semantics/numbers.md, "ROUNDUP and ROUNDDOWN round at 100 places past a 16-bit num_digits" (T-3627,
  VERIFIED). Narrowing gives k, the sign kept and the magnitude modulo 65,536. When |k| >= 32,768, Excel rounds the
  15-digit cut at 100 places: 10^-100 for positive k, 10^100 for negative k. ROUNDUP's unit 10^-k is added as a
  double, so:
  - for positive k it is 0, and ROUNDUP returns the rounded-down value;
  - for negative k it overflows, and ROUNDUP is `#NUM!` whenever a digit is dropped.
  - `ROUNDDOWN(1.23456789012345E-95,65535)` = 1.23456E-95, `ROUNDUP(…)` the same;
  - `ROUNDDOWN(1.5E+80,-65535)` = 0, `ROUNDUP(1.5E+120,-65535)` = 1.5E+120 (nothing dropped);
  - `ROUNDUP(1.23456789012345E+105,-32768)` = `#NUM!`; k = 32,767 rounds at 32,767 places as before.
- Evidence: probes/t3627-roundup-rounddown-digits-by-exponent, probes/t3627-roundup-rounddown-huge-digits-place and
  probes/t3627-roundup-rounddown-negative-x (negative x: the magnitude rounds, the sign is kept)
  (excel-graph, 2026-11-01). All three are committed with T-3627's PR, so `make eval` lists their failing cells (327,
  all baselined as F lines).
- Code: `effective_digits` and `round_decimal` in `crates/rustsheet-engine/src/decimal_round.rs`. TRUNC shares the
  ROUNDDOWN path, so the change reaches it too, but TRUNC was not probed with a narrowed `num_digits` of 32,768 or
  more. Probe that (`excel-probe` skill) before changing TRUNC, or keep TRUNC on today's path.

## Acceptance criteria
- [ ] The probe cells of the three t3627 cases pass (`make eval-show CASE=probes/t3627-roundup-rounddown-huge-digits-place`).
- [ ] Unit tests in decimal_round.rs cover positive and negative k at 32,768 and 65,535, and 32,767.
- [ ] `make eval-baseline` records the fixed cells; `make check` green; reviewers approve.

## Out of scope
- ROUND, whose `num_digits` narrowing is different and matches every probed cell.
