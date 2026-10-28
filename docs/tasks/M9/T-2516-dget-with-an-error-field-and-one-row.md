---
id: T-2516
title: "DGET with an error field and one-row criteria: which error wins (T-1390 follow-up)"
milestone: M9
lane: functions-stat
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
A D-function whose `field` is an error and whose criteria (or database) is one row returns the same
error Excel does.

## Context
T-1390 made one-row criteria and one-row databases `#VALUE!` (docs/excel-semantics/statistics.md
"Database criteria"). One owned gen-m9 cell still differs:
gen-m9/7bd219ec0c0c5e22 Calc!A324, whose inner call is
`DGET(Data!A2:A4,GAMMA.DIST(Data!$A$10,42,1e-15,"FALSE"),Data!A7:D7)` (Data!A10 is TRUE, A7:D7 is one
row): Excel `#VALUE!`, ours `#NUM!`. Either our GAMMA.DIST over a boolean cell gives the wrong error, or
Excel checks the criteria shape before the field (`db_query` in functions/database.rs resolves the
field first). Probe both with excel-probe: `GAMMA.DIST(<TRUE cell>,42,1E-15,"FALSE")` alone, and
`DSUM(<db>,NA(),<one-row criteria>)`, `DSUM(<one-row db>,NA(),<criteria>)`, `DSUM(<db>,"Nope",<one-row criteria>)`.

gen-m9 database sweep (gap triage 2026-10-23, T-2258): two more cells of the same question. `DPRODUCT(Data!B1:E7,VLOOKUP(..,NA())<#N/A>,Db!G1:H2)` is `#VALUE!` (ours the field's `#N/A`), and `DMAX(Data!E8:E12,REGEXEXTRACT(..,TINV(<"1.5">,7),0.1),Db!F1)` (a one-cell criteria range) is `#VALUE!` (ours the field's `#NUM!`): something about the database or criteria is checked before the field's error, or the inner VLOOKUP/REGEXEXTRACT gives `#VALUE!` in Excel. Probe both bare inner calls with the case's Data sheet before changing the order.
Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 2 known-failure lines in eval/baseline/gen-m9.txt, 2 formulas, 2 visibly off.
Examples:
- gen-m9/88cabddc4935351c Calc!A14 `=DPRODUCT(Data!B1:E7,VLOOKUP(SIGN(Data!B11),Data!A9:A11,ACCRINTM(Data!C10,Data!D9,10,3761),NA()),Db!G1:H2)`: Excel #VALUE!, ours #N/A
- gen-m9/c9d7659896d95368 Calc!A271 `=DMAX(Data!E8:E12,REGEXEXTRACT(LOWER(Data!D12),"[0-9]",TINV(Data!E12,7),0.1),Db!F1)`: Excel #VALUE!, ours #NUM!
Cells:
- gen-m9/88cabddc4935351c.case.json (1): Calc!A14
- gen-m9/c9d7659896d95368.case.json (1): Calc!A271

## Acceptance criteria
- [x] Visible parity (gen-m9, T-2258): these 2 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] A probe answers the order, recorded in docs/excel-semantics/statistics.md "Database criteria".
- [x] gen-m9/7bd219ec0c0c5e22 Calc!A324 leaves eval/baseline/gen-m9.txt (`make eval-baseline`).
- [x] `make check` green; reviewers approve.

## Out of scope
- Other D-function semantics (T-1390).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2258): every cell this task owns has the wrong type, error, text or value (none is a last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria states it.

## Outcome
- The question was answered by probes/db-field-error-order (13 questions, excel-graph): the field is resolved
  before the database or criteria shape is checked, which is what `db_query` already does. The three cells this
  task owned (gen-m9/88cabddc4935351c A14, c9d7659896d95368 A271, 7bd219ec0c0c5e22 A324) already pass on main and
  have no known-failure lines in eval/baseline/gen-m9.txt, so no engine change was needed.
- Recorded in docs/excel-semantics/statistics.md "Database criteria". Eval delta: none (probe adds 13 passing cells).
