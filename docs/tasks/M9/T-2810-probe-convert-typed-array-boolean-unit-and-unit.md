---
id: T-2810
title: "Probe CONVERT typed/array boolean unit and unit error order"
milestone: M9
lane: functions-engineering
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Establish with the Excel oracle what CONVERT does for a typed boolean unit (`CONVERT(1,TRUE,"m")`), a boolean array element as a unit, and which error wins between from_unit and to_unit (`CONVERT(1,NA(),<TRUE cell>)`).

## Context
Follow-up of T-2793: ours gives `#VALUE!` for every boolean unit, read in order, but only boolean cells are evidenced. See docs/excel-semantics/engineering.md, "CONVERT: a boolean unit is #VALUE!". Use the `excel-probe` skill; code: `convert_of` in crates/rustsheet-engine/src/functions/engineering/convert.rs.

## Acceptance criteria
- [x] Probe cases with oracle values land, the entry's Status is updated, and ours matches or a fix task is filed.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
probes/eng-convert-boolean-unit (31 formula cells, excel-graph 2026-10-26). A typed boolean and a boolean array
element as a unit are `#VALUE!` like a boolean cell; an array of units lifts element by element; from_unit's error or
boolean wins over to_unit's; a boolean in either unit beats an unknown or empty unit in the other; the number argument's
error comes first. Ours matches every error cell, so no fix task. The only misses are the three spilled
`CONVERT(1,"m","ft")` cells, one ulp low, already owned by T-2046/T-2047 (baselined as new coverage). The
engineering.md entry is now VERIFIED for typed and array-element booleans.
Eval delta: probes +34 checked cells, 31 pass, 3 new NumericUlp known failures; 0 regressions.
