---
id: T-2422
title: "D* criteria: an unknown header is ignored, computed criteria, a header-only range is #VALUE!, blank trailing records"
milestone: M9
lane: functions-criteria
status: done
priority: P2
size: M
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
The D* functions read a criteria range as Excel does in four cases where ours differs.

## Context
Found by the T-2249 smoke check: a scratch suite of 790 cells, one formula per entry of the M9 engineering and database pools, filled through excel-graph on 2026-10-22 and deleted afterwards (docs/excel-semantics/engineering.md and criteria.md, "edge-pool smoke check (T-2249)"). Commit a probe case with these cells first (excel-probe skill).
The data is Db!A1:D9 (Name, Qty, Price, Date over eight records; Qty 4, 10, 0, 2, 1, -4, 7, 5) in rustsheet_tools/gen/database.py. Excel:
- A criteria header the database does not have is ignored: `DSUM(Db!A1:D9,"Qty",Db!N1:N2)` (N1 "Nope", N2 1) is 25, every record, and `DCOUNTA(Db!A1:D9,1,Db!N1:N2)` is 7. Ours 0 for both.
- A computed criterion works: a label that is not a header (O1 "Calc") over a formula about the first record (O2 `=B2>3`) gives `DSUM(...,"Qty",Db!O1:O2)` 26 and `DCOUNTA(...,1,Db!O1:O2)` 4, the records with Qty > 3. Ours 0.
- A criteria range of the header row only (`Db!F1:G1`, `Db!F1`) is `#VALUE!` in DSUM and DCOUNTA. Ours matches every record (25, 7).
- Wholly blank rows at the end of the database are not records: `DCOUNTA(Db!A1:D12,,Db!K1:K2)` is 8, as for Db!A1:D9. Ours 11.
Code: functions/database.rs (criteria_cell_matches and the criteria reader). See docs/excel-semantics/statistics.md "Database function argument modes" and criteria.md.

gen-m9 database sweep (gap triage 2026-10-23, T-2258): the unknown header is ignored over a database on another sheet too, `DGET(Data!B6:E8,4,Db!N1:N2)` is -0.5 (ours `#NUM!`). The computed-criterion cells of the sweep (criterion Db!O1:O2 over Db!A2:D9 or a Data range) are T-2553's (criteria.md "D* computed criteria and an unknown header in the gen-m9 database sweep (T-2258)").
Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 1 known-failure lines in eval/baseline/gen-m9.txt, 1 formulas, 1 visibly off.
Examples:
- gen-m9/c9d7659896d95368 Calc!A1 `=DGET(Data!B6:E8,4,Db!N1:N2)`: Excel -0.5, ours #NUM!
Cells:
- gen-m9/c9d7659896d95368.case.json (1): Calc!A1

## Acceptance criteria
- [x] Visible parity (gen-m9, T-2258): these 1 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] A probe case holds the cells above, plus a computed criterion with a relative and an absolute reference, a computed criterion next to a header criterion, and DGET and DAVERAGE over the same ranges.
- [x] Ours matches every probe cell; criteria.md is updated.
- [x] `make check` green; reviewers approve.

## Out of scope
- Other criteria grammar.

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2258): every cell this task owns has the wrong type, error, text or value (none is a last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria states it.

## Outcome
The other probe items are covered by probes/db-computed-criteria-shapes and db-header-criteria-shapes (T-1390). Cases 1, 3 and 4 of the Goal (unknown header, computed criterion, header-only range, trailing blank rows) were already fixed by T-1390 (#1638). The remaining difference was DGET: it now counts only matching records whose field cell is non-empty (probes/db-get-blank-field, 11 Excel cells, all pass), which fixes gen-m9/c9d7659896d95368 Calc!A1. Eval delta: gen-m9 known failures 7719 -> 7718, probes +1 case, no regressions. criteria.md updated. gen-m9 A82 and A271 stay with T-2553.
