---
id: T-1290
title: "Statistical functions whose arguments are all scalar lift over arrays and ranges (probes/stat-argument-modes)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-1770]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
Every scalar argument of the M9 statistical functions lifts elementwise over an array and over a multi-cell range, as in Excel.

## Context
SPEC-0011 rollout step 2 (docs/specs/SPEC-0011-scalar-argument-lifting.md). Enter the functions whose every argument is scalar in `Function::arg_kinds` as `Lift`: the distributions, CONFIDENCE*, STANDARDIZE, FISHER*, PERMUT*, GAUSS, PHI and the rest. Then turn each one into a kernel over values and delete its old static-analysis lines. The scalar arguments next to a data array (LARGE k, PERCENTILE k, RANK number, TRIMMEAN, FORECAST, PROB, Z.TEST, T.TEST) are T-1772's. The general shape, error and range rules are in docs/excel-semantics/arrays.md "Scalar arguments lift: shapes, element errors, and the Analysis ToolPak range rule" (probes/scalar-lift-shapes H15, H16, H20, H21, H34, H35, H38, H40, J8, K8).

probes/stat-argument-modes (excel-graph, 2026-10-09, T-1080; legend in docs/excel-semantics/statistics.md "Statistical function argument modes"): all 302 lift cells are 2 (210 `ROWS(F(..{v;v}..))`, 92 `ROWS(F(..G1:G2..))`) in Excel (distribution
parameters, LARGE/SMALL k, PERCENTILE/QUARTILE k, PERCENTRANK and Z.TEST x, RANK number, T.TEST tails/type, TRIMMEAN
percent, FORECAST x, PROB lower_limit, the FORECAST.ETS target_date and statistic_type). The engine fails all 302: `#N/IMPL!`
for an array (207 cells) and `#VALUE!` for a range (89 cells), plus 6 FORECAST.ETS and BETA.DIST/Z.TEST cells. Reuse the lifting the math and date functions already use (search
before adding a helper); BETA.DIST, Z.TEST and ZTEST are `#VALUE!` even for an array. Split by family if larger than M (T-1772 is the first split).

More cells (T-1289's gen-m9 suite, excel-graph, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/55fc8c4ffbd7c7cc Calc!A5 `=ROUND(SWITCH(..),BETA.DIST(SUMIFS(..),MUNIT(4),..))`: Excel `#SPILL!` (BETA.DIST lifts over MUNIT(4) although its x is `#VALUE!`), engine `#VALUE!`.
- gen-m9/d5bd4272655c5211 Calc!A6 `=AVERAGE(NORMDIST(PRICEDISC(38107,43861,0.5,110),VSTACK("TRUE",..),HLOOKUP(..),AND(..)),0.5,Data!E8:E9)`: Excel `#VALUE!`, engine `#N/IMPL!` (NORMDIST over the VSTACK array).
- gen-m9/1466c66c61189a83 Calc!A1 `=EXPONDIST(INDEX(Data!A1:C3,3,-1e-15),..)`: Excel 0.027981694209681673 (EXPONDIST over the row A3:C3, anchor value), engine `#VALUE!`; also needs T-1407 (INDEX with -1E-15).

## Acceptance criteria
- [ ] The lift cells of probes/stat-argument-modes for the functions in scope pass (their baseline lines removed), as do probes/scalar-lift-shapes H15, H16, H20, H21, H34, H35, H38, H40, J8 and K8. Ulp misses in values are named in the Outcome with their owner task.
- [ ] gen-m9/55fc8c4ffbd7c7cc A5 and d5bd4272655c5211 A6 pass, and 1466c66c61189a83 A1 once T-1407 has landed, with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
