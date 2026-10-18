---
id: T-1791
title: "Three corpus-bench array/legacy cells: CSE IF-lift over FIND/LEFT, legacy MATCH intersection, INDEX(SMALL(IF)) of a whole column"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Three small corpus-bench array/legacy cells match Excel; each distinct cause gets a fix or its own task.

## Context
All values are the Excel cached values, which Excel for the web reproduced (T-0427 run, 2026-10-17).
Inspect with `./target/release/rustsheet-eval show corpus-bench/<id>/1_<id>_input.xlsx`.
- corpus-bench/42181 `Sheet1!B14`, CSE array formula `{=SUM(IFERROR(IF(ISNUMBER(FIND(A13,$I$4:$I$10)),VALUE(LEFT($I$4:$I$10,FIND(A13,$I$4:$I$10)-1)),0),0))}`, A13 = `"Note"`, I4:I10 = `"Note (blah blah blah)"`, `"1 Note"`, blank, `"Note"`, `"2 Notes"`, blank, blank: expected 3 (`VALUE("1 ")` + `VALUE("2 ")`), engine 0. Suspect VALUE of text with a trailing space, or the IF lift inside IFERROR.
- corpus-bench/49300 `Sheet1!C2`, legacy (no `cm`) cell `=INDEX(Data!C3:AY4,MATCH(1,(Sheet1!A2=Data!B3:B4)*(Sheet1!B2=Data!C2:AY2),0))`: expected `#VALUE!`, engine `#N/A`. Suspect implicit intersection of `Data!B3:B4` in row 2 (no overlap: `#VALUE!`) in a legacy cell's MATCH argument.
- corpus-bench/530-28 `Sheet1!P8`, Q8, CSE `{=INDEX($A:$A,SMALL(IF(($B$2:$B$176&$C$2:$C$176=$E2&$F2)*MATCH($A$2:$A$176&"",$A$2:$A$176&"",)=ROW($1:$175),ROW($2:$176),4^8),COLUMN(J$1)))&""}`: expected `""`, engine `#VALUE!`. Suspect the omitted MATCH match_type, `ROW($1:$175)` whole rows, or INDEX of row 65536 of a whole column.

## Acceptance criteria
- [x] The `F` lines of corpus-bench/42181, 49300 and 530-28 leave eval/baseline/corpus-bench.txt with no new ones; or each unfixed cause is filed as its own task with these cells and the line stays.
- [x] Each fixed cause has a unit or spec test reproducing it.
- [x] Any quirk found goes into docs/excel-semantics/; `make check` green; reviewers approve.

## Out of scope
- Other legacy-parse work (T-1096).

## Outcome
corpus-bench/42181 B14 was VALUE not lifting over an array (`#N/IMPL!` inside IFERROR, so 0). VALUE now lifts element-wise with its scalar rules (probes/value-argument-lift, 22 cells; text.md "VALUE lifts over a range or an array"), and B14 is 3. The other two cells have separate causes, filed as T-1861 (530-28 P8:Q8: MATCH over `range&""` arrays is `#N/A`) and T-1862 (49300 C2: legacy MATCH intersection); their baseline lines stay.
