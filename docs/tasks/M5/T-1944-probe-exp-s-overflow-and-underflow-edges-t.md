---
id: T-1944
title: "Probe EXP's overflow and underflow edges (T-1931)"
milestone: M5
lane: functions-math
status: done
priority: P3
size: S
tier: opus
depends_on: [T-1931, T-2203]
---

## Goal
Establish with oracle evidence what Excel's EXP does at its range edges, and make the engine match.

## Context
- Evidence first (planner, 2026-10-20): the probe this task asks for is T-2203 (research), which fills `probes-numeric/t2203-exp-range-edges`.  Filled 2026-10-22: the engine matches all 780 checked cells (last finite 709.782712893384, first #NUM! 709.7827128933841, underflow switch between -745.1332191019412 and -745.1332191019411; numbers.md "Range edges"). Use that case; do not probe again. Its failing cells are this task's cells too.
- `EXP_SIGN.xlsx` B44 is no longer this task's cell. Excel keeps 2^-1074 there (probes/subnormal-function-results C4), and Excel writes a kept subnormal to the saved file as 0 (probes/subnormal-saved-as-zero, T-2204), so the cached 0 is that artifact. Decision 0089 names it an M5 exit exception owned by T-2210; it is out of this task's scope.
- Since T-1931 the worksheet EXP is model V6 (`x87_exp::excel_exp`; docs/excel-semantics/numbers.md "EXP is an
  x87 2^(a*log2 e) evaluation, not correctly rounded"). V6 has not been probed at the edges of its range.
- Overflow: a in (709.7827, 709.79), where libm's exp and V6 could place the last finite double and `#NUM!`
  differently. Also probe `EXP(710)`.
- Underflow: a near -745.13, where V6's single rounding to the subnormal grid decides between 2^-1074 and 0. Use
  `(x)=0` companions, because the saved file holds a kept subnormal as 0 (decisions 0042, 0089).
- Use the `excel-probe` skill. Put a dense grid of consecutive doubles, as stored inputs, around each edge.

## Acceptance criteria
- [x] A probes-numeric case with an excel-graph oracle covers both edges and `EXP(710)`, and every Excel quirk
  found is recorded in numbers.md.
- [x] The engine matches every cell of the case, or each miss is a listed residual.
- [x] `make check` green; reviewers approve.

## Out of scope
- EXP accuracy inside the range (T-1931, T-1940).

## Outcome
No engine change was needed: probes-numeric/t2203-exp-range-edges (T-2203, excel-graph 2026-10-22) already covers both
edges with dense grids of consecutive stored doubles, the 709.79-710.5 overflow tail with 710 itself, and `(EXP(a))=0`
companions for the underflow switch, and the engine (model V6) matches all 780 checked cells (527 rows: the inputs plus the `=0` companions) (`rustsheet-eval show
probes-numeric/t2203-exp-range-edges.case.json`: status=Pass, checked=780 passed=780, 2026-10-25). The quirks (last finite
argument 709.782712893384 with EXP = 1.7976931348622732E308, the underflow switch between -745.1332191019412 and
-745.1332191019411) are in docs/excel-semantics/numbers.md "Range edges". Eval: no change (no residuals).
