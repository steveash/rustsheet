---
id: T-1191
title: "SERIESSUM is #VALUE! for a boolean or numeric-text cell in the coefficient range"
milestone: M5
lane: functions-math
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
SERIESSUM is `#VALUE!` when its coefficient range holds a boolean or numeric text, as Excel is.

## Context
probes/math-argument-modes (excel-graph, 2026-10-08, T-1082): `SERIESSUM(1,0,1,H1:H3)` with
H1:H3 = 4, TRUE, 6 is `#VALUE!` in Excel; the engine returns 11 (it reads TRUE as 1). Excel is also
`#VALUE!` for text and numeric text cells in the range (C40, C48: the engine already matches) and
reads a blank cell as 0 (C36). Code: `fn_seriessum` in
crates/rustsheet-engine/src/functions/math_and_trigonometry/seriessum.rs. Semantics:
docs/excel-semantics/coercion.md "GCD / LCM / MULTINOMIAL range cells" (SERIESSUM paragraph).

## Acceptance criteria
- [x] probes/math-argument-modes C44 passes; C15, C17, C36, C40, C48 still pass.
- [x] `make check` green; reviewers approve.

## Out of scope
- Typed boolean coefficients (`SERIESSUM(1,0,1,TRUE)`): probe first if touched.

## Outcome
Probed typed/array/cell forms (probes/seriessum-boolean-coefficients, 14 cells, excel-graph): a boolean coefficient is `#VALUE!` everywhere; a single text value or one-cell reference is read as a number. seriessum.rs now matches; math-argument-modes C44 passes (56/56), new probe 14/14.
