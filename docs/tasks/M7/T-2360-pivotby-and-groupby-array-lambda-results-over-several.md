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

gen-m7 cells (T-2254 triage, excel-graph, 2026-10-22): the same GROUPBY shape with one LAMBDA over four value columns,
`_xlfn.GROUPBY(Data!B1:B12,Data!B1:E12,LAMBDA(acc,acc),3)`, used as REGEXEXTRACT's return_mode (21b4eef18424ce94 A1504, the 11x5 result lifts
the call). Where ours has a `#VALUE!` value cell (each group's `acc` is a multi-cell array), Excel's REGEXEXTRACT element is "" in 26 cells,
i.e. the GROUPBY cell there is a valid return_mode (0 or 1, or a value read as one), not an error: the same "not an error to ISERROR" as
P!C12/P!D12. Some other cells of that spill are `#VALUE!` in both (the rows whose group has one row and whose value is text, presumably),
and 11 are holes excel-graph did not report (T-2397). Probe `INDEX(GROUPBY(..,LAMBDA(acc,acc),3),r,c)` with ISERROR, TYPE and `+0` for a
one-row group and a several-row group to pin it down. Owned: 26 known-failure lines in eval/baseline/gen-m7.txt, 1 formulas.
- gen-m7/21b4eef18424ce94.case.json A1504 `=REGEXEXTRACT((("#N/A"&"Hello World")&(Data!B5&"#N/A")),"x*",_xlfn.GROUPBY(Data!B1:B12,Data!B1:E12,LAMBDA(acc,acc),3),1)`: B1505 Excel "", ours #VALUE!; cells (26): B1505:E1505, B1506, E1506, C1507:E1507, C1508, C1509:E1509, B1510:E1510, C1511:E1511, B1512, E1512, B1513, D1513:E1513, C1514

## Acceptance criteria
- [ ] The four probe lines above disappear from eval/baseline/probes.txt, no new known failures.
- [ ] Visible parity: the 26 gen-m7 lines listed under Context (T-2254 triage) disappear from eval/baseline/gen-m7.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The rules are recorded in docs/excel-semantics (arrays.md, dates.md) with probe cells.
- [ ] Unit tests cover them.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Pivot tables (SPEC-0015).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-22, T-2254): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.
