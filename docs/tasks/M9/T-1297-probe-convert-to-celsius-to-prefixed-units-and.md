---
id: T-1297
title: "Probe CONVERT to Celsius, to prefixed units and between non-base units"
milestone: M9
lane: functions-engineering
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
CONVERT matches Excel for conversions to Celsius, to prefixed units and between two non-base units, on oracle evidence.

## Context
Found by T-0922 (docs/excel-semantics/engineering.md "CONVERT constants and evaluation order"). CONVERT.xlsx only has
conversions to the class base unit (plus time to `yr`). `convert_temperature` in
crates/rustsheet-engine/src/functions/engineering/convert.rs subtracts 273.5 (not 273.15) for Celsius output, so
`CONVERT(68,"F","C")` gives 19.65 and the unit test `fn_convert` pins it; Excel presumably gives 20. Also unprobed:
the order for a prefixed target (`CONVERT(1,"m","km")`, `CONVERT(5,"cal","kJ")`) and two non-base units
(`CONVERT(5,"cal","eV")`, `CONVERT(1,"uk_pt","gal")`, `CONVERT(1,"lbf","dyn")`).

## Acceptance criteria
- [ ] Probe case (excel-probe) covering every temperature pair (including to C, Reau, Rank, F), prefixed targets and non-base pairs in force, energy, volume and area, plus extreme magnitudes (`CONVERT(1E290,"m2","m2")`, `CONVERT(1E300,"ha","m2")`, `CONVERT(1E302,"J","J")`); oracle values recorded.
- [ ] The engine matches them (fix the 273.5 constant and the unit test if Excel says so); findings in engineering.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
