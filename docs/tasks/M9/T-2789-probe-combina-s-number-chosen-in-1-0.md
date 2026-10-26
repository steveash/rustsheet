---
id: T-2789
title: "Probe COMBINA's number_chosen in (-1, 0) and below -1 against number 0, (0, 1) and 1 (T-2621 follow-up)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
COMBINA's rounding of a negative fractional number_chosen is settled by Excel values, and the engine matches them.

## Context
T-2621 found that `COMBINA(-0.5,-0.0001)` is 1 (probes/t2475-stat-subexpressions P!A35) while `COMBINA(7,1-1.005)` and
`COMBINA(24,-0.0001)` are `#NUM!` (gen-m5 c688ca172fe533dc A118, gen-m6 44194b2fefa30de2 A222). The engine now truncates
number_chosen only when number truncates to 0 and floors it otherwise (`fn_combina`, crates/rustsheet-engine/src/functions/
math_and_trigonometry/mathematical.rs). That rule is UNVERIFIED: statistics.md "Statistical argument-domain errors and
extreme values (T-2621)". Write a probe (excel-probe skill) with at least: COMBINA(0,-0.0001), COMBINA(0.5,-0.0001),
COMBINA(-0.5,-0.5), COMBINA(0,-1), COMBINA(0,-1.5), COMBINA(-0.5,-1.5), COMBINA(1,-0.5), COMBINA(1,-0.0001),
COMBINA(2,-0.5), and the same with COMBIN for comparison; then align the rule and its statistics.md entry.

## Acceptance criteria
- [ ] The probe is filled by the oracle and passes; the statistics.md entry states the rule from it (no UNVERIFIED left
      for COMBINA's number_chosen).
- [ ] `make check` green; reviewers approve.

## Out of scope
- COMBINA's number bound (numbers.md "COMBINA is COMBIN(n+k-1, k)").
