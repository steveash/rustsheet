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
SUM/SUMPRODUCT arguments are not intersected (probes/n-array-lift A3, A10). probes/legacy-n-t-reference-args
(T-1713) adds: under N, IFERROR gets the `@` (`=N(@IFERROR({2;3},0))`, D20) but SWITCH gets none and
the legacy cell spills 2, 3 (M10:M11, M20); `=IF(1,N(IF(1,{2;3})))` is `=@IF(...)` (P20, 2, no spill);
N/T over IF, CHOOSE, INDEX, IFS and ranges are T-1727/T-1713 and T-1728, not this task's. Check which other
calls Excel leaves without `@` (reference-returning ones such as IF, CHOOSE, INDEX, OFFSET) before
generalizing; probe if needed. The xlsx import uses the same rewrite, so a plain `<f>` changes too:
run the full eval (ironcalc-calc, poi) for regressions. docs/excel-semantics/references.md
"Legacy formulas intersect the ranges in value arguments". probes/t-n-range-top-left (T-0985) adds `=T(@{"a","b"})` (M10/M20, "a")
and `=N(@{5,6})` (O10/O20, 5), known failures until this task: ours spills with no `@`.

## Acceptance criteria
- [ ] probes/legacy-array-result B10:F11, H10:H11, K10:L11 and their FORMULATEXT cells in row 20 leave eval/baseline/probes.txt; no regressions.
- [ ] probes/legacy-n-t-reference-args S!D11, D20 (IFERROR under N gets the `@`), M11, M20 (SWITCH under N gets
      none: ours prints `=N(@SWITCH(1,1,{2;3}))` and does not spill) and P11, P20 (`=@IF(1,N(IF(1,{2;3})))`) leave
      eval/baseline/probes.txt. (Added by the 2026-10-17 planner: the Context assigns these six cells here, but no
      acceptance criterion named them, so no open task owned them.)
- [ ] references.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
