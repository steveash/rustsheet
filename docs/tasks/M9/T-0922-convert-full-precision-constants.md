---
id: T-0922
title: "CONVERT: full-precision unit constants (67 known failures)"
milestone: M9
lane: functions-engineering
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`CONVERT` matches Excel's cached values bit-for-bit for the unit pairs exercised by CONVERT.xlsx.

## Context
Oracle-free: the evidence is the Excel-authored workbook `crates/rustsheet-xlsx/tests/calc_tests/ENGINEERING/CONVERT.xlsx`
(eval case `ironcalc-calc/ENGINEERING/CONVERT.xlsx`; `make eval-show CASE=ironcalc-calc/ENGINEERING/CONVERT.xlsx`).
67 failing cells (45 NumericUlp, 22 NumericClose), all on the same pattern: our unit table in
`crates/rustsheet-engine/src/functions/engineering/convert.rs` stores factors rounded to ~15 significant digits
(`("sg", 14593.9029372064)`, `("parsec", 30856775812815500.0)`), while Excel keeps the full double.
Examples:
- `Weight!C3` `=CONVERT(1,B2,$B$2)` expected 14593.902937206363, actual 14593.9029372064
- `distance!C11` expected 3.0856775812815532e16, actual 3.08567758128155e16
- `distance!C13` expected 0.0003527777777777778, actual 0.000352777777777778
- `Weight!G5` expected 8.30269391e-20, actual 8.302693910000001e-20 (operation order, via-base-unit divide vs multiply)

Derive each constant (and the multiply/divide order through the base unit) from the cached cells so that all
exercised pairs match exactly; prefer exact rational/derived forms (e.g. `0.0254 / 12`, `1852.0 / ...`) over decimal literals
and cite the workbook cell in a comment. Also sanity-check the Celsius branch of `convert_temperature`
(`from_t_kelvin - 273.5`) against any temperature cells in the workbook.

## Acceptance criteria
- [x] The 67 `F` lines for `ironcalc-calc/ENGINEERING/CONVERT.xlsx` disappear from `eval/baseline/ironcalc-calc.txt` (via `make eval-baseline`), with no new ones.
- [x] Unit tests for at least three unit families (mass, distance, time/area) assert exact bits.
- [x] `make check` green.

## Outcome
CONVERT.xlsx passes 509/509 (the 67 known failures are gone; 0 regressions in any suite). Fitting the cached cells showed
that Excel computes `value * F_from / F_to`, then applies the prefixes, with factors in class-internal bases: dynes for
force, ergs for energy, litres for volume and square angstroms for area. Every unit's factor is the double fitted to its
cells. `CONVERT(…,"F","K")` uses `(v-32)*5/9+273.15`. Bit-exact unit tests cover mass, distance, volume, energy, force,
area, speed and temperature. Rule, rejected models and open questions: engineering.md "CONVERT constants and evaluation
order". Follow-up: T-1297 (to-Celsius 273.5, prefixed and non-base targets).

## Out of scope
New units, prefix-handling changes, units not covered by cached evidence (use the oracle task for those).
