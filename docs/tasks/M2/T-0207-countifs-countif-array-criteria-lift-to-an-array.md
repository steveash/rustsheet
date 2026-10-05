---
id: T-0207
title: "COUNTIFS/COUNTIF array criteria lift to an array of counts (4 known failures)"
milestone: M2
lane: functions-criteria
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
When a criteria argument of COUNTIF(S) is an array (or a multi-cell range in an array context),
the function returns one count per criteria element. So `SUM(COUNTIFS(r, {"a","b"}))` matches Excel.

## Context
Evidence: `poi/70005-countifs.xlsx` (excel-graph, T-0019), all NumericMismatch:
Sheet1!A3 and D3 `=SUM(COUNTIFS(A1:F1, {"bar","baz"}))` expected 3, actual 0; A4 and D4
`=SUM(COUNTIFS(A1:F1, {"foo","bar"}))` expected 4, actual 0.
`fn_countifs` (`crates/rustsheet-engine/src/functions/statistical/if_ifs.rs`) passes the whole
criteria value to `build_criteria`, so an array criterion matches nothing. The lift belongs in
one place shared by COUNTIF(S), SUMIF(S), AVERAGEIF(S) and MAXIFS/MINIFS (T-0201 single matcher), not per function.
Check how several array criteria broadcast against each other (shape rules as for other lifted
functions; see `docs/excel-semantics/arrays.md`). Use `excel-probe` where nothing decides it.
Related, excel-cached and not required here: `poi/FormulaEvalTestData_Copy.xlsx`
EverythingTests!E1324 `SUMIF(AA7:AA13,C10:D10, C7)` expected 1.1, actual #VALUE!.

## Acceptance criteria
- [ ] The 4 lines (Sheet1!A3, D3, A4, D4) disappear from eval/baseline/poi.txt with no new F lines.
- [ ] Unit tests: array criteria for COUNTIF, COUNTIFS and SUMIFS give arrays of the right shape.
- [ ] The lift is implemented once for the whole *IF(S) family.
- [ ] Broadcast rule recorded in `docs/excel-semantics/criteria.md` with evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
D-functions. The criteria grammar itself (T-0206, T-0011).
