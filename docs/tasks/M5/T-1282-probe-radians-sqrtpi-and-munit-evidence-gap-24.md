---
id: T-1282
title: "Probe RADIANS, SQRTPI and MUNIT (evidence gap: 24, 13 and 11 Excel cells)"
milestone: M5
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
RADIANS, SQRTPI and MUNIT each have at least 25 Excel-valued cells in eval, covering their domains, boundaries and argument coercion, and any engine mismatch the probe finds is fixed (if local to the function and not a last-digit fit) or filed as a task.

## Context
Gap triage 2026-10-09 (decision 0031): report.md "Excel evidence per function" lists these M5 math functions below the 25-cell threshold: RADIANS 24 Excel cells (prior 0.3), SQRTPI 13 (0.1), MUNIT 11 (0.1). Current evidence is the availability probes, the ironcalc-calc workbooks and gen-m5 (`gen-m5/065c0542a4ce3d1c`). The engine computes `RADIANS` as `x * (PI / 180.0)` and `SQRTPI` as `sqrt(x * PI)` (other groupings round differently in the last digit), and `MUNIT` truncates the dimension and gives `#VALUE!` for anything below 1.

Write one probe case `eval/suites/probes/cases/radians-sqrtpi-munit.case.json` (excel-probe skill). Suggested cells:
- RADIANS: `RADIANS(180)`, `(90)`, `(45)`, `(30)`, `(1)`, `(-270)`, `(360)`, `(1E+300)`, `(1E-300)`, `(0.1)`, `(123.456)`, `RADIANS("90")`, `RADIANS(TRUE)`, `RADIANS("a")`.
- SQRTPI: `SQRTPI(1)`, `(2)`, `(0)`, `(-1)` (#NUM!), `(1E-310)`, `(1E+307)`, `(1E+308)` (does `x*PI` overflow?), `(0.5)`, `(100)`, `(1/PI())`, `SQRTPI("4")`, `SQRTPI(TRUE)`.
- MUNIT (a probe cell per element, or wrap: `SUM(MUNIT(n))`, `ROWS(MUNIT(n))`, `INDEX(MUNIT(n),i,j)`): `MUNIT(1)`, `MUNIT(3)` spilled, `SUM(MUNIT(4))`, `ROWS(MUNIT(2.9))` (truncation), `MUNIT(0)`, `MUNIT(-1)`, `MUNIT(0.5)`, `MUNIT("2")`, `MUNIT(TRUE)`, `MUNIT({1,2})` (lift or whole?), `MUNIT(1048577)` (over the sheet's row count). excel-graph refuses some workbooks with a huge MUNIT dimension (text.md "Text function argument modes", the `MUNIT(SUBTOTAL(9,..))` note): if the fill fails, drop that cell and note it.
Fill with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes`.

Code: `fn_radians` and `fn_sqrtpi` (crates/rustsheet-engine/src/functions/math_and_trigonometry/mathematical.rs), `fn_munit` (…/math_and_trigonometry/matrix.rs).

## Acceptance criteria
- [ ] `probes/radians-sqrtpi-munit` is committed with its excel-graph oracle sidecar, and report.md's evidence table shows RADIANS, SQRTPI and MUNIT with 25 or more Excel cells each.
- [ ] docs/excel-semantics/numbers.md records the results (VERIFIED, citing cells): MUNIT's dimension rule and error kinds, SQRTPI's domain and overflow, and, if RADIANS or SQRTPI differ in the last digit, which cells differ.
- [ ] Error-kind and coercion mismatches local to one function are fixed in this PR; last-digit (ulp) differences are not fitted here but filed as an M9 task naming the cells (they are numeric fitting, decision 0018). `make eval-baseline` shows no known failures outside the new case.
- [ ] `make check` green; reviewers approve.

## Out of scope
- DEGREES (27 Excel cells), MINVERSE/MDETERM (legacy CSE work in T-0950).

## Outcome
Added probes/radians-sqrtpi-munit (94 cells, excel-graph). It found three engine misses, now fixed: SQRTPI took booleans as numbers (Excel: `#VALUE!`), MUNIT gave `#N/IMPL!` for an array argument and `#VALUE!` for a range (Excel lifts), and SQRTPI was typed as a scalar result. No ulp differences in RADIANS or SQRTPI, so no M9 follow-up. Recorded in numbers.md. Excel cells now: RADIANS 62, SQRTPI 47, MUNIT 42 (report.md evidence table).
