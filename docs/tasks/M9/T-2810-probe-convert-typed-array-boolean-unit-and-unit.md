---
id: T-2810
title: "Probe CONVERT typed/array boolean unit and unit error order"
milestone: M9
lane: functions-engineering
status: in-progress
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
- [ ] Probe cases with oracle values land, the entry's Status is updated, and ours matches or a fix task is filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
