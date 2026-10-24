---
id: T-2489
title: "CUMIPMT and CUMPRINC: a start period below 1 is #NUM! (gen-m9, 4 cells)"
milestone: M9
lane: functions-financial
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
CUMIPMT and CUMPRINC return `#NUM!` when the start period is below 1, also when it is fractional, as Excel does.

## Context
Gap triage of the T-2257 gen-m9 financial sweep. Excel: `CUMPRINC(0.05,249,329832,0.3,19,0)`,
`CUMIPMT(0.004166666666666667,267,43109,0.5,24,1)`, `CUMPRINC(0.004166666666666667,183,165876,1e-15,14,0)` and
`CUMIPMT(0.05,275,266391,8.261999899647271e-07,22,0)` are `#NUM!`; ours returns -2.67, -4009.89, -8719.76 and -293029.77
(it truncates or rounds the start before the check). Excel's evidence is these 4 cells; the check is a comparison of the
given start with 1 before any rounding. docs/excel-semantics/financial.md "Financial functions in the gen-m9 financial sweep (T-2257)".
Code: `fn_cumipmt`, `fn_cumprinc` in crates/rustsheet-engine/src/functions/financial/cumulative.rs.

Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 4 known-failure lines in eval/baseline/gen-m9.txt, 4 visibly off.
Examples:
- gen-m9/0a957119475760c5 Calc!A40 `=CUMPRINC(0.05,249,329832,0.3,19,0)`: Excel #NUM!, ours -2.6667634321786515
- gen-m9/3aa168259335be54 Calc!A10 `=CUMIPMT(0.05/12,267,43109,0.5,24,1)`: Excel #NUM!, ours -4009.8915549335015
Cells:
- gen-m9/0a957119475760c5.case.json (1): Calc!A40
- gen-m9/3aa168259335be54.case.json (1): Calc!A10
- gen-m9/6376e27be7a05871.case.json (1): Calc!A306
- gen-m9/bc04c1f478c9896c.case.json (1): Calc!A260

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2257): these 4 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Unit tests for a start of 0.3, 0.5, 1E-15 and 1 (1 still computes).
- [ ] `make check` green; reviewers approve.

## Out of scope
- CUMIPMT/CUMPRINC values (T-2363).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2257): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.
