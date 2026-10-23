---
id: T-2422
title: "D* criteria: an unknown header is ignored, computed criteria, a header-only range is #VALUE!, blank trailing records"
milestone: M9
lane: functions-criteria
status: todo
priority: P2
size: M
tier: sonnet
depends_on: []
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

## Acceptance criteria
- [ ] A probe case holds the cells above, plus a computed criterion with a relative and an absolute reference, a computed criterion next to a header criterion, and DGET and DAVERAGE over the same ranges.
- [ ] Ours matches every probe cell; criteria.md is updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other criteria grammar.
