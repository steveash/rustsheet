---
id: T-1717
title: "A legacy formula puts @ on an array-valued call or array constant in a value argument and at the top level"
milestone: M2
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A legacy (non-array) formula reads an array-valued call or array constant in a value position through an implicit `@`, as Excel does: a legacy `=IF(1,{2;3})` is 2 with FORMULATEXT `=@IF(1,{2;3})`, not a 2, 3 spill.

## Context
probes/legacy-array-result (excel-graph, 2026-10-16, T-1713): legacy row 10, FORMULATEXT row 20.
Excel writes `=@IF(1,{2;3})` (2), `=ABS(@IF(1,{-2;3}))` (2), `=LEN(@IF(1,{"ab";"c"}))` (2),
`=N(@{2;3})` (2), `=ABS(@{-2;3})` (2), `=ISNUMBER(@IF(1,{2;3}))` (TRUE), `=-@IF(1,{2;3})` (-2) and
`=N(@TRANSPOSE({2,3}))` (2). Ours adds no `@` there (`add_implicit_intersection`,
expressions/parser/static_analysis.rs, and legacy.rs), so the legacy cells spill (row 11 filled) and
FORMULATEXT lacks the `@`. Exceptions to keep: N and T over `IF(...)` get no `@` (T-1713), and
SUM/SUMPRODUCT arguments are not intersected (probes/n-array-lift A3, A10). Check which other
calls Excel leaves without `@` (reference-returning ones such as IF, CHOOSE, INDEX, OFFSET) before
generalizing; probe if needed. The xlsx import uses the same rewrite, so a plain `<f>` changes too:
run the full eval (ironcalc-calc, poi) for regressions. docs/excel-semantics/references.md
"Legacy formulas intersect the ranges in value arguments".

## Acceptance criteria
- [ ] probes/legacy-array-result B10:F11, H10:H11, K10:L11 and their FORMULATEXT cells in row 20 leave eval/baseline/probes.txt; no regressions.
- [ ] references.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
