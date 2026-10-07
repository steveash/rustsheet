---
id: T-1119
title: "MOD lifts over array arguments (engine gives #N/IMPL!)"
milestone: M1
lane: functions-math
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
MOD lifts element-wise over array arguments as Excel does: `=SUM(MOD(SEQUENCE(4),2))` is 2.

## Context
Found in T-1112: the engine gives `#N/IMPL!` for `MOD(SEQUENCE(4),2)` and anything built on it
(`IF(MOD(SEQUENCE(n),2),…)`), because `Model::fn_mod`
(crates/rustsheet-engine/src/functions/math_and_trigonometry/mathematical.rs) reads both arguments
with `get_number`. Excel: probes/geomean-overflow-cut-3 E4 `=SUM(MOD(SEQUENCE(4),2))` = 2 (excel-graph).
Reuse the existing lift helpers other math functions use (search `get_number_or_array`, T-0222 for
ROUNDDOWN); MOD's own quirks (sign of the divisor, `#DIV/0!`) must hold per element.

## Acceptance criteria
- [ ] probes/geomean-overflow-cut-3 E4 passes; `make eval-baseline` records it.
- [ ] Unit tests: MOD over an array and over a range in an array formula, with an error element and a 0 divisor element.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
