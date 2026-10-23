---
id: T-2553
title: "D* computed criteria when the database starts below the criteria row or sits on another sheet: Excel's records (gen-m9, 16 cells)"
milestone: M9
lane: functions-criteria
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-2525, T-2422]
campaign: visible
---

## Goal
D* functions select Excel's records for a computed criterion when the database does not start on the row after the criteria's first-record reference, or sits on another sheet.

## Context
Gap triage of the T-2258 gen-m9 database sweep. docs/excel-semantics/criteria.md "D* computed criteria and an unknown header in the gen-m9 database sweep (T-2258)" records the cells. Criterion Db!O1:O2 (O1 "Calc", O2 `=B2>3`): over Db!A2:D9 Excel matches the records of rows 4 and 9 only (DVAR 12.5, DSTDEVP 2.5, DCOUNTA 2, DAVERAGE and DGET of the date 45291), ours rows 3, 4 and 9; over a database on the Data sheet Excel matches no record (0, `#DIV/0!`, DGET `#VALUE!`), ours some. `DGET(Db!A2:D9,4,Db!O1:O2)` 45291 with two matching records (one with an empty date) suggests DGET counts only records with a non-empty field: confirm in the probe. T-2525 (research) probes where a formula condition's references point for other placements; implement from its cells and these. The unknown-header cell (`DGET(Data!B6:E8,4,Db!N1:N2)`) is T-2422's.
Code: `db_computed_condition` and DGET's match count in crates/rustsheet-engine/src/functions/database.rs.

Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 16 known-failure lines in eval/baseline/gen-m9.txt, 15 formulas, 16 visibly off.
Examples:
- gen-m9/4dc761698ca23149 Calc!A129 `=DVAR(Db!A2:D9,2,Db!O1:O2)`: Excel 12.5, ours 25.0
- gen-m9/5bdef392d16a5efb Calc!A161 `=DVAR(Db!A2:D9,4,Db!O1:O2)`: Excel #DIV/0!, ours 42340.5
- gen-m9/6d16afd40600302b Calc!A365 `=DMAX(Data!B2:E4,4,Db!O1:O2)`: Excel 0.0, ours 1.0
- gen-m9/74779cff2810ccf4 Calc!A52 `=DSTDEVP(Db!A2:D9,2,Db!O1:O2)`: Excel 2.5, ours 4.08248290463863
- gen-m9/7a819ffbbeff8fcc Calc!A303 `=DSTDEVP(Db!A2:D9,4,Db!O1:O2)`: Excel 0.0, ours 145.5
- gen-m9/88cabddc4935351c Calc!A255 `=DGET(Data!A7:D11,2,Db!O1:O2)`: Excel #VALUE!, ours #NUM!
Cells:
- gen-m9/4dc761698ca23149.case.json (1): Calc!A129
- gen-m9/5bdef392d16a5efb.case.json (1): Calc!A161
- gen-m9/6d16afd40600302b.case.json (1): Calc!A365
- gen-m9/74779cff2810ccf4.case.json (1): Calc!A52
- gen-m9/7a819ffbbeff8fcc.case.json (1): Calc!A303
- gen-m9/88cabddc4935351c.case.json (1): Calc!A255
- gen-m9/c9d7659896d95368.case.json (1): Calc!A82
- gen-m9/cb80ca09be0e12bc.case.json (1): Calc!A52
- gen-m9/d193e8b4ab820f52.case.json (2): Calc!A260, A375
- gen-m9/d9d55972b50e55e8.case.json (1): Calc!A106
- gen-m9/f066968376660329.case.json (3): Calc!A205, A292, A313
- gen-m9/f61a73cd9af21e6d.case.json (1): Calc!A289
- gen-m9/fa8ee59f2f1be7e0.case.json (1): Calc!A332

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2258): these 16 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] criteria.md's T-2258 entry gets the confirmed rule and an "Ours" line.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2258): every cell this task owns has the wrong type, error, text or value (none is a last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria states it.
