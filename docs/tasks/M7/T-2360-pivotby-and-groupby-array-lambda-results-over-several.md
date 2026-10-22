---
id: T-2360
title: "PIVOTBY and GROUPBY: array LAMBDA results over several value columns and WORKDAY.INTL holiday errors (t2311 probe cells)"
milestone: M7
lane: functions-lookup
status: todo
priority: P3
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
PIVOTBY and GROUPBY match Excel on the four probe cells T-2311 left: an array LAMBDA result over several value columns (Excel's cells are not errors to ISERROR and MAP over the result fails) and WORKDAY.INTL with a holidays group that holds text or booleans.

## Context
Known failures added by T-2311 (excel-graph, 2026-10-22), new coverage in eval/baseline/probes.txt:
- probes/t2311-pivotby-value-columns-error P!C12, P!D12: `PIVOTBY(D!A1:A5,D!B1:B5,D!C1:D5,LAMBDA(v,v))`. Excel: `SUM(--ISERROR(..))` is 0 and the MAP/TEXTJOIN is an error; ours: 12 `#VALUE!` cells. arrays.md says a LAMBDA array result is `#VALUE!` per cell (one value column evidence); with two value columns it is something ISERROR does not see. Probe what the cells are (ISTEXT, TYPE, ERROR.TYPE, ROWS of INDEX).
- probes/t2311-pivotby-value-cause P!D46, P!D51: error counts of `PIVOTBY(D90!A1:D12,D90!A1:D12,D90!A1:A12,LAMBDA(z,WORKDAY.INTL(26624,-19,5,z)))` (Excel 13, ours 16) and the GROUPBY form (27 vs 34): WORKDAY.INTL's holidays from a group array (text, booleans, blanks) error in fewer groups in Excel. Probe WORKDAY.INTL with array holidays holding text/TRUE/blank directly first (docs/excel-semantics/dates.md).

## Acceptance criteria
- [ ] The four probe lines above disappear from eval/baseline/probes.txt, no new known failures.
- [ ] The rules are recorded in docs/excel-semantics (arrays.md, dates.md) with probe cells.
- [ ] Unit tests cover them.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Pivot tables (SPEC-0015).
