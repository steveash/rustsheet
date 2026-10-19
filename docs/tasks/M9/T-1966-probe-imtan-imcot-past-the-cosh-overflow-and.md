---
id: T-1966
title: "Probe IMTAN/IMCOT past the cosh overflow and near pi/2"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
IMTAN and IMCOT match Excel where `cosh 2y` overflows (|y| above about 355) and for a real
argument at or near the double nearest pi/2, or the mismatch is recorded.

## Context
Found by T-0907's review. T-0907 moved IMTAN to the double-angle form
`(sin 2x + i sinh 2y)/(cos 2x + cosh 2y)` and IMCOT to its reciprocal
(docs/excel-semantics/engineering.md "IMTAN and IMCOT use the double-angle form"). Where the
denominator overflows, `tan_parts` (crates/rustsheet-engine/src/functions/engineering/complex.rs)
returns `±0 ± i`, and where it is 0 it returns `tan x`. Both are UNVERIFIED: Excel could give `#NUM!`
there. Probe (excel-probe skill, excel-graph) `IMTAN("1+400i")`, `IMTAN("1-400i")`,
`IMCOT("1+400i")`, IMTAN/IMCOT at y = 354, 355 and 356, and `IMTAN("1.5707963267948966")`,
`IMTAN("1.5707963267949")`.

## Acceptance criteria
- [ ] A probe case with oracle values for the shapes above; the engine matches them, or each
      mismatch is recorded.
- [ ] The UNVERIFIED bullet in engineering.md is resolved.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
