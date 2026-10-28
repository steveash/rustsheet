---
id: T-2617
title: "Probe D* computed criteria over a database on another sheet: which records match (gen-m9, T-2553 leftovers)"
milestone: M9
lane: research
status: in-progress
priority: P1
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
Excel evidence decides which records a D-function matches when a computed criterion's relative reference points at
another sheet than the database, so the 8 gen-m9 cells T-2553 left can be implemented.

## Context
T-2553 fixed the same-sheet case (a moved reference on the database's header row fails; criteria.md "D* computed
criteria and an unknown header in the gen-m9 database sweep (T-2258)"). With the criterion Db!O1:O2 (O1 "Calc", O2
`=B2>3`, so unqualified references are Db's) over a database on Data, the evidence does not fit one rule:
- the record-by-record shift ours does (record i reads Db!B(2+i)) gives Excel's value for
  `DCOUNTA(Data!A2:C10,3,Db!O1:O2)` = 3 (02b7cfabebd2cc0a A231) and `DVARP(Data!A4:D9,1,Db!O1:O2)` = 0
  (d193e8b4ab820f52 A114);
- but Excel matches no record in these 8 cells (ours finds some): 6d16afd40600302b A365 `DMAX(Data!B2:E4,4,..)` 0,
  88cabddc4935351c A255 `DGET(Data!A7:D11,2,..)` `#VALUE!`, c9d7659896d95368 A82 `DCOUNTA(Data!B4:B6,1,..)` 0,
  cb80ca09be0e12bc A52 `DAVERAGE(Data!A10:C12,2,..)` `#DIV/0!`, d193e8b4ab820f52 A260 `DPRODUCT(Data!B11:E12,3,..)` 0,
  d9d55972b50e55e8 A106 `DVAR(Data!D2:E12,1,..)` `#DIV/0!`, f066968376660329 A205 `DPRODUCT(Data!C6:D10,2,..)` 0 and
  A313 `DPRODUCT(Data!C10:E12,3,..)` 0.
No shift d (record i reads Db!B(2+i+d)) fits all ten. Probe, with one Db-like criteria sheet and a Data sheet, databases
starting at rows 1, 2, 4 and 10 and at columns A, B and D, a criterion with an unqualified reference, the same qualified
with the criteria sheet's name, and one qualified with the database sheet's name; read DCOUNTA and DSUM, plus ROW() and
CELL("address") of the reference inside the condition (`=AND(B2>3,ISNUMBER(B2))`) if that tells which cell is read.
Also probe the same-sheet choices T-2553 made without evidence (its `reads_records`): a reference two or more rows above
the first record, a reference in a column outside the database, and a range with one end on the header row
(`=SUM(B2:B3)>3`).

Owned gen-m9 cells (moved from T-2553, excel-graph, 2026-10-23): 8 known-failure lines in eval/baseline/gen-m9.txt.
Cells:
- gen-m9/6d16afd40600302b.case.json (1): Calc!A365
- gen-m9/88cabddc4935351c.case.json (1): Calc!A255
- gen-m9/c9d7659896d95368.case.json (1): Calc!A82
- gen-m9/cb80ca09be0e12bc.case.json (1): Calc!A52
- gen-m9/d193e8b4ab820f52.case.json (1): Calc!A260
- gen-m9/d9d55972b50e55e8.case.json (1): Calc!A106
- gen-m9/f066968376660329.case.json (2): Calc!A205, A313
Code: `reads_records` and `db_computed_condition` in crates/rustsheet-engine/src/functions/database.rs (TODO(T-2617)).

## Acceptance criteria
- [ ] A probe case (excel-probe skill) with oracle values for the placements under Context is committed under eval/suites/probes.
- [ ] criteria.md records the rule; a functions-criteria task (campaign: visible) is filed to implement it for the 8 cells,
      or this task implements it if it is small.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The same-sheet header-row rule (T-2553, done).

## Visible-parity stage
Target (decision 0114): every visibly-off cell this task owns is within half a unit of the 15th significant digit
of Excel's value, or has Excel's type, error and text. Bit-level criteria still unmet then move to a follow-up
filed with `defer: phase-1b`.

Phase 1 triage (T-0090, eval of 2026-10-27 at `main` 7f6b373a, task ownership as of `main` e5bc60c4): this task owns the visibly-off gen-m9 cells of DPRODUCT (3), DMAX, DVAR, DAVERAGE, DCOUNTA and DGET (1 each). Fixing them lifts all six D* functions to the Phase 1 rate (≥ 99.9% of attributed cells within 15 digits); no other open task owns enough of those cells.
