---
id: T-2046
title: "Oracle sweep: CONVERT from each class's base unit to every unit (CONVERT(1,\"m\",\"ft\") is one ulp off)"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
An oracle-filled probes-numeric case gives T-2047 Excel's CONVERT values from each class's base unit
(and SI unit, where they differ) to every unit, so the factor or order behind the one-ulp miss of
`CONVERT(1,"m","ft")` can be fitted instead of guessed.

## Context
- Known failures (gap triage 2026-10-19): probes/scalar-lift-open-a Sheet1!Z8, Z9, Z10
  `CONVERT(1,"m","ft")`: Excel (excel-graph) 3.2808398950131235, ours 3.280839895013123
  (= 1 × 1 / 0.3048, the rule in docs/excel-semantics/engineering.md "CONVERT constants and
  evaluation order"). `CONVERT(2.5,"m","ft")` (eng-argument-modes) passes at 8.202099737532809, and
  ironcalc-calc CONVERT.xlsx (Mac cache) has `CONVERT(1,"ft","m")` 0.3048 and `CONVERT(4,"ft","m")`
  1.2192. A reciprocal factor for ft (3.2808398950131235, multiply to go m→ft, divide to go ft→m)
  reproduces all four; engineering.md says reciprocal tables fit 143–156 of 159 units overall, so
  the choice may be per unit. CONVERT.xlsx only has unit → base cells, and convert-targets a few
  non-base targets, so the base → unit direction is nearly unevidenced.
- Excel evidence for CONVERT is otherwise ample (>= 20 cells), so this is a fitting sweep, not an
  evidence-floor gap.

## Acceptance criteria
- [ ] A probes-numeric case (excel-probe skill, `rustsheet-oracle fill`, excel-graph) with, for every
      non-temperature unit in the engine's unit table (convert.rs; at least every length, mass, time,
      volume, area, energy and force unit),
      `CONVERT(v, base, unit)` and `CONVERT(v, unit, base)` for v in {1, 2.5, 4, 7.3, 1E-3}; no pending cells.
- [ ] The case is cited in T-2047's Context; any new quirk is recorded in
      docs/excel-semantics/engineering.md with cells.
- [ ] No gated baseline changes; `make check` green; reviewers approve.

## Out of scope
- Fitting or changing the engine (T-2047). Temperature (already fitted per pair, "CONVERT temperature").
