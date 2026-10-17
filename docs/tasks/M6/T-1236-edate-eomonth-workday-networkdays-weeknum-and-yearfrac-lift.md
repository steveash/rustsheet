---
id: T-1236
title: "EDATE, EOMONTH, WORKDAY, NETWORKDAYS, WEEKNUM and YEARFRAC lift over arrays but are #VALUE! for a range"
milestone: M6
lane: functions-date
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
EDATE, EOMONTH, WEEKNUM, WORKDAY, WORKDAY.INTL, NETWORKDAYS, NETWORKDAYS.INTL and YEARFRAC lift over an array argument and are `#VALUE!` for a multi-cell range argument, as Excel does.

## Context
probes/date-argument-modes (excel-graph, 2026-10-09, T-1090): these Analysis ToolPak-era functions return `#VALUE!`
when a date, month count, day count, return type or basis argument is a multi-cell range (the engine matches that),
but lift over an array literal in the same place (`INDEX(EDATE({45000.25;45031.5;45100.75},1),2)` = 45061), where the
engine gives `#N/IMPL!` (F20, F22, F24, F26, F36, F38, F40, F54, F56, F58, F60, F62, F66, F68 and their G cells).
Holidays stay read whole (dates.md "WORKDAY holidays take the whole range"); the weekend argument is T-1235's.
Semantics: docs/excel-semantics/dates.md "Date function argument modes".

More cells (T-1289's gen-m9 suite, excel-graph, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/4df1b22d3ed63f58 Calc!A24 `=EOMONTH((IF(Data!E11,Data!$E1)+WRAPROWS(Data!D3:D7,1,Data!$E9)),-14)`: Excel `#SPILL!`, engine `#N/IMPL!` (an array start date lifts).
- gen-m9/b58b6d7cd39d195f Calc!A9 `=EOMONTH((1*HYPGEOMDIST(0,3,1,13)),TAKE(Data!B9:E12,-2,-1))`: Excel `#VALUE!`, engine `#N/IMPL!`. TAKE of a range is a reference (references.md, ISREF of it is TRUE), so this is the range form (`#VALUE!`), not an array lift: keep TAKE/DROP results on the range path when lifting.
- corpus-bench/45738 (26 cells, T-0427 run, excel-graph agreeing with the cached values): `Sheet1!L48` `=$B6*ISNUMBER(MATCH(L$4,EOMONTH($D6,-ROW($AI$1:$AI$8)*12/$C6),))+(EOMONTH($D6,0)=L$4)*($A6+$B6)` expected 1000, engine 0: the months argument is a computed array, EOMONTH gives `#N/IMPL!`, so MATCH never matches (also O49, M50, F51, I52, E53, J54, ...).

## Acceptance criteria
- [ ] The listed probes/date-argument-modes cells pass, and the range forms stay `#VALUE!`; baseline updated.
- [ ] gen-m9/4df1b22d3ed63f58 A24 and b58b6d7cd39d195f A9 pass, with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [ ] corpus-bench/45738's `F` line leaves eval/baseline/corpus-bench.txt (no new ones).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
