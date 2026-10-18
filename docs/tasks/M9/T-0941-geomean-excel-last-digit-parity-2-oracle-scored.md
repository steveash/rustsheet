---
id: T-0941
title: "GEOMEAN: Excel last-digit parity (2 oracle-scored ulp cells)"
milestone: M9
lane: functions-stat
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
GEOMEAN returns Excel's exact double, including the last digit, on the oracle-scored
`ironcalc-calc/STATISTICAL/GEOMEAN.xlsx` cells.

## Context
The steward's oracle fill (2026-10-05) scored GEOMEAN.xlsx with excel-graph; 31/33 cells pass and
two are NumericUlp, baselined as new coverage:
- `Sheet1!B27` `=GEOMEAN("47")`: Excel 47, ours 46.99999999999999 (a single value must round-trip,
  so Excel does not compute it as `exp(ln(x))` here).
- `Sheet1!G27` `=GEOMEAN(1,2,"5", TRUE)`: Excel 1.778279410038923, ours 1.7782794100389228.
Code: `crates/rustsheet-engine/src/functions/statistical/geomean.rs`. Excel's evaluation order is
unknown; probe candidate formulations (product then root, `pow(prod, 1/n)`, log-sum) against the
oracle with the `excel-probe` skill rather than fitting to two cells: `exp(ln(product)/n)` matches
G27 but `exp(ln(47))` still misses B27, so no single candidate fits both yet. Both values come from
Excel for the web: the goldens are the excel-graph sidecar `GEOMEAN.oracle.json`, because the workbook
is not Excel-authored. Fit to them. excel-graph is the oracle of record for Phases 1–3 (decision 0039),
and the desktop confirmation is item 3 of T-1075 (M14), not a gate. Record the rule in
`docs/excel-semantics/`.

## Acceptance criteria
- [x] Both cells pass (`make eval-show CASE=ironcalc-calc/STATISTICAL/GEOMEAN.xlsx`), 0 regressions, baseline updated.
- [x] The formulation is backed by oracle probe evidence and recorded in `docs/excel-semantics/`.
- [x] `make check` green; reviewers approve.

## Out of scope
- HARMEAN and other means unless the same probe shows they share the formulation.

## Outcome
Done by T-1113 (numbers.md: "matches both since T-1113"); closed by T-0059. Both cells pass on main:
eval/baseline/ironcalc-calc.txt:164 is the coverage line `ironcalc-calc/STATISTICAL/GEOMEAN.xlsx 33` and there is no known-failure line for
the case. The rule is in docs/excel-semantics/numbers.md, "GEOMEAN last digits differ from a log-sum
evaluation" and "GEOMEAN in-range root" (a single value is returned as is; otherwise r = exp(ln(P)/n)
with the 15-digit snap). The desktop confirmation stays item 3 of T-1075.
